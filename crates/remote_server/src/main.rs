//! Minimal remote-server daemon binary.
//!
//! Binds a Unix domain socket, accepts proxy connections, and handles
//! file-system operations via the `remote_server` protobuf protocol.
//! Does not use the WarpUI app framework — owns its own event loop and
//! thread-per-connection model.

use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use futures::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use remote_server::proto::{
    client_message, create_directory_response, list_directory_response,
    read_file_chunk_response, resolve_path_response, run_command_response,
    server_message, write_file_chunk_response, Authenticate, ClientMessage, CreateDirectory,
    CreateDirectoryResponse, CreateDirectorySuccess, DirEntry, ErrorCode, ErrorResponse,
    FileOperationError, FileSystemEntryKind, Initialize, InitializeResponse, ListDirectory,
    ListDirectoryResponse, ListDirectorySuccess, ReadFileChunk, ReadFileChunkResponse,
    ReadFileChunkSuccess, ResolvePath, ResolvePathResponse, ResolvePathSuccess, RunCommandError,
    RunCommandRequest, RunCommandResponse, RunCommandSuccess, ServerMessage, WriteFileChunk,
    WriteFileChunkResponse, WriteFileChunkSuccess,
};
use remote_server::protocol::{self, ProtocolError, RequestId};

const GRACE_PERIOD: Duration = Duration::from_secs(10 * 60);
const MAX_CHUNK: u64 = 8 * 1024 * 1024;

fn main() -> anyhow::Result<()> {
    let identity_key = std::env::args()
        .nth(1)
        .context("usage: remote_server_daemon <identity_key>")?;

    let daemon_dir = daemon_dir_path(&identity_key);
    let socket_path = daemon_dir.join("server.sock");
    let pid_path = daemon_dir.join("server.pid");

    std::fs::create_dir_all(&daemon_dir)?;
    #[cfg(unix)]
    {
        let mut perm = std::fs::metadata(&daemon_dir)?.permissions();
        perm.set_mode(0o700);
        std::fs::set_permissions(&daemon_dir, perm)?;
    }
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    std::fs::set_permissions(&socket_path, Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    std::fs::write(&pid_path, std::process::id().to_string())?;
    log::info!("Daemon bound to {}", socket_path.display());

    let running = Arc::new(AtomicBool::new(true));
    let last_active = Arc::new(Mutex::new(SystemTime::now()));

    // Grace-period watcher thread.
    {
        let running = running.clone();
        let last_active = last_active.clone();
        let socket_path = socket_path.clone();
        let pid_path = pid_path.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_secs(10));
                let idle = last_active
                    .lock()
                    .unwrap()
                    .elapsed()
                    .unwrap_or_default();
                if idle >= GRACE_PERIOD {
                    log::info!("Grace period expired, shutting down");
                    running.store(false, Ordering::Relaxed);
                    let _ = std::os::unix::net::UnixStream::connect(&socket_path);
                    break;
                }
            }
            let _ = std::fs::remove_file(&socket_path);
            let _ = std::fs::remove_file(&pid_path);
        });
    }

    // Accept loop.
    let async_listener = async_io::Async::new(listener)?;
    async_io::block_on(async {
        while running.load(Ordering::Relaxed) {
            match async_listener.accept().await {
                Ok((stream, _)) => {
                    *last_active.lock().unwrap() = SystemTime::now();
                    let last = last_active.clone();
                    std::thread::spawn(move || {
                        async_io::block_on(async {
                            if let Err(e) = handle_connection(stream).await {
                                log::error!("Connection error: {e}");
                            }
                        });
                        *last.lock().unwrap() = SystemTime::now();
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => log::error!("Accept error: {e}"),
            }
        }
    });
    log::info!("Daemon exiting");
    Ok(())
}

async fn handle_connection(
    stream: async_io::Async<std::os::unix::net::UnixStream>,
) -> anyhow::Result<()> {
    let (read_half, write_half) = stream.split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    let (tx, mut rx) = async_channel::unbounded::<ServerMessage>();

    // Writer task.
    let writer_handle = std::thread::spawn(move || {
        async_io::block_on(async {
            while let Ok(msg) = rx.recv().await {
                if let Err(e) = protocol::write_server_message(&mut writer, &msg).await {
                    log::error!("Write error: {e}");
                    break;
                }
                if let Err(e) = writer.flush().await {
                    log::error!("Flush error: {e}");
                    break;
                }
            }
            let _ = writer.flush().await;
        });
    });

    // Reader loop.
    let mut shell_info: Option<(String, Option<String>)> = None;

    loop {
        match protocol::read_client_message(&mut reader).await {
            Ok(msg) => {
                let request_id = RequestId::from(msg.request_id.clone());
                let Some(message) = msg.message else { continue };
                match message {
                    client_message::Message::Authenticate(_auth) => {
                        // Accept any authentication for the minimal daemon.
                    }
                    client_message::Message::Initialize(init) => {
                        handle_initialize(init, &request_id, &tx).await;
                    }
                    client_message::Message::SessionBootstrapped(sb) => {
                        shell_info = Some((sb.shell_type.clone(), sb.shell_path.clone()));
                    }
                    client_message::Message::NavigatedToDirectory(_) => {}
                    client_message::Message::ListDirectory(msg) => {
                        handle_list_directory(msg, &request_id, &tx).await;
                    }
                    client_message::Message::ResolvePath(msg) => {
                        handle_resolve_path(msg, &request_id, &tx).await;
                    }
                    client_message::Message::CreateDirectory(msg) => {
                        handle_create_directory(msg, &request_id, &tx).await;
                    }
                    client_message::Message::ReadFileChunk(msg) => {
                        handle_read_file_chunk(msg, &request_id, &tx).await;
                    }
                    client_message::Message::WriteFileChunk(msg) => {
                        handle_write_file_chunk(msg, &request_id, &tx).await;
                    }
                    client_message::Message::RunCommand(msg) => {
                        let (shell_type, shell_path) = shell_info
                            .as_ref()
                            .map(|(t, p)| (t.as_str(), p.as_deref()))
                            .unwrap_or(("sh", None));
                        handle_run_command(msg, &request_id, shell_type, shell_path, &tx).await;
                    }
                    client_message::Message::Abort(abort) => {
                        log::info!(
                            "Abort for request {} (abort id {})",
                            abort.request_id_to_abort,
                            request_id
                        );
                    }
                    _ => {
                        let _ = tx
                            .send(ServerMessage {
                                request_id: request_id.to_string(),
                                message: Some(server_message::Message::Error(ErrorResponse {
                                    code: ErrorCode::InvalidRequest as i32,
                                    message: "unsupported message type".into(),
                                })),
                            })
                            .await;
                    }
                }
            }
            Err(ProtocolError::UnexpectedEof) => break,
            Err(e) if e.is_read_recoverable() => {
                log::warn!("Skipping malformed message: {e}");
            }
            Err(e) => {
                log::error!("Fatal read error: {e}");
                break;
            }
        }
    }

    drop(tx);
    writer_handle.join().ok();
    Ok(())
}

async fn handle_initialize(
    _init: Initialize,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::InitializeResponse(
                InitializeResponse {
                    server_version: env!("CARGO_PKG_VERSION").into(),
                    host_id: format!("host-{}", uuid::Uuid::new_v4()),
                },
            )),
        })
        .await;
}

fn expand_path(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    }
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

fn entry_kind(
    file_type: Option<&std::fs::FileType>,
    metadata: Option<&std::fs::Metadata>,
) -> i32 {
    if file_type.is_some_and(|ft| ft.is_symlink()) {
        return FileSystemEntryKind::Symlink as i32;
    }
    if metadata.is_some_and(|m| m.is_dir()) {
        return FileSystemEntryKind::Directory as i32;
    }
    if metadata.is_some_and(|m| m.is_file()) {
        return FileSystemEntryKind::File as i32;
    }
    FileSystemEntryKind::Other as i32
}

fn system_time_to_epoch_millis(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

async fn handle_list_directory(
    msg: ListDirectory,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    let path = expand_path(&msg.path);
    let result = match std::fs::read_dir(&path) {
        Ok(read_dir) => {
            let mut entries = Vec::new();
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let ft = entry.file_type().ok();
                let meta = entry.metadata().ok();
                let kind = entry_kind(ft.as_ref(), meta.as_ref());
                let is_dir = kind == FileSystemEntryKind::Directory as i32;
                let size_bytes = meta.as_ref().filter(|m| m.is_file()).map(|m| m.len());
                let modified_epoch_millis = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(system_time_to_epoch_millis);
                entries.push(DirEntry {
                    name,
                    is_dir,
                    kind,
                    size_bytes,
                    modified_epoch_millis,
                });
            }
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            let canonical = path.canonicalize().unwrap_or(path);
            list_directory_response::Result::Success(ListDirectorySuccess {
                entries,
                canonical_path: canonical.to_string_lossy().into_owned(),
            })
        }
        Err(e) => list_directory_response::Result::Error(FileOperationError {
            message: format!("Failed to list {}: {e}", msg.path),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::ListDirectoryResponse(
                ListDirectoryResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

async fn handle_resolve_path(
    msg: ResolvePath,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    let path = expand_path(&msg.path);
    let result = match std::fs::symlink_metadata(&path) {
        Ok(meta) => {
            let ft = meta.file_type();
            let kind = entry_kind(Some(&ft), Some(&meta));
            let canonical = path.canonicalize().unwrap_or(path);
            resolve_path_response::Result::Success(ResolvePathSuccess {
                canonical_path: canonical.to_string_lossy().into_owned(),
                kind,
                size_bytes: meta.is_file().then_some(meta.len()),
            })
        }
        Err(e) => resolve_path_response::Result::Error(FileOperationError {
            message: format!("Failed to resolve {}: {e}", msg.path),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::ResolvePathResponse(
                ResolvePathResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

async fn handle_create_directory(
    msg: CreateDirectory,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    let path = expand_path(&msg.path);
    let result = match std::fs::create_dir_all(&path) {
        Ok(()) => {
            let canonical = path.canonicalize().unwrap_or(path);
            create_directory_response::Result::Success(CreateDirectorySuccess {
                canonical_path: canonical.to_string_lossy().into_owned(),
            })
        }
        Err(e) => create_directory_response::Result::Error(FileOperationError {
            message: format!("Failed to create {}: {e}", msg.path),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::CreateDirectoryResponse(
                CreateDirectoryResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

async fn handle_read_file_chunk(
    msg: ReadFileChunk,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    use std::io::{Read, Seek, SeekFrom};

    let path = expand_path(&msg.path);
    let result = (|| -> std::io::Result<ReadFileChunkSuccess> {
        let mut file = std::fs::File::open(&path)?;
        let total_size = file.metadata().ok().map(|m| m.len());
        file.seek(SeekFrom::Start(msg.offset))?;
        let max_bytes = msg.max_bytes.min(MAX_CHUNK) as usize;
        let mut buf = vec![0u8; max_bytes];
        let read = file.read(&mut buf)?;
        buf.truncate(read);
        let next_offset = msg.offset + read as u64;
        let eof = total_size.is_some_and(|s| next_offset >= s) || read == 0;
        Ok(ReadFileChunkSuccess {
            bytes: buf,
            next_offset,
            total_size,
            eof,
        })
    })();

    let result = match result {
        Ok(success) => read_file_chunk_response::Result::Success(success),
        Err(e) => read_file_chunk_response::Result::Error(FileOperationError {
            message: format!("Failed to read {}: {e}", msg.path),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::ReadFileChunkResponse(
                ReadFileChunkResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

async fn handle_write_file_chunk(
    msg: WriteFileChunk,
    request_id: &RequestId,
    tx: &async_channel::Sender<ServerMessage>,
) {
    use std::io::{Seek, SeekFrom, Write};

    let path = expand_path(&msg.path);
    let result = (|| -> std::io::Result<WriteFileChunkSuccess> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut opts = std::fs::OpenOptions::new();
        opts.create(true).write(true);
        if msg.truncate {
            opts.truncate(true);
        }
        let mut file = opts.open(&path)?;
        file.seek(SeekFrom::Start(msg.offset))?;
        file.write_all(&msg.bytes)?;
        #[cfg(unix)]
        if let Some(executable) = msg.executable {
            let mode = if executable { 0o755 } else { 0o644 };
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
        }
        Ok(WriteFileChunkSuccess {
            next_offset: msg.offset + msg.bytes.len() as u64,
        })
    })();

    let result = match result {
        Ok(success) => write_file_chunk_response::Result::Success(success),
        Err(e) => write_file_chunk_response::Result::Error(FileOperationError {
            message: format!("Failed to write {}: {e}", msg.path),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::WriteFileChunkResponse(
                WriteFileChunkResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

async fn handle_run_command(
    msg: RunCommandRequest,
    request_id: &RequestId,
    shell_type: &str,
    shell_path: Option<&str>,
    tx: &async_channel::Sender<ServerMessage>,
) {
    use std::process::Command;

    let shell = shell_path.unwrap_or(shell_type);
    let shell = if shell.is_empty() { "sh" } else { shell };

    let mut cmd = Command::new(shell);
    cmd.arg("-c").arg(&msg.command);
    if let Some(cwd) = &msg.working_directory {
        cmd.current_dir(expand_path(cwd));
    }
    for (k, v) in &msg.environment_variables {
        cmd.env(k, v);
    }

    let result = match cmd.output() {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            run_command_response::Result::Success(RunCommandSuccess {
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: Some(exit_code),
            })
        }
        Err(e) => run_command_response::Result::Error(RunCommandError {
            code: 1,
            message: format!("Failed to execute: {e}"),
        }),
    };

    let _ = tx
        .send(ServerMessage {
            request_id: request_id.to_string(),
            message: Some(server_message::Message::RunCommandResponse(
                RunCommandResponse {
                    result: Some(result),
                },
            )),
        })
        .await;
}

fn daemon_dir_path(identity_key: &str) -> PathBuf {
    // Use ~/.openwarp/remote-server/{identity_key}/ — matches what the
    // ssh_transport expects on the remote host for the oss channel.
    let warp_dir = std::env::var("WARP_REMOTE_SERVER_DIR").unwrap_or_else(|_| ".openwarp".into());
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    if warp_dir.starts_with('/') {
        PathBuf::from(&warp_dir).join("remote-server").join(identity_key)
    } else {
        home.join(&warp_dir).join("remote-server").join(identity_key)
    }
}
