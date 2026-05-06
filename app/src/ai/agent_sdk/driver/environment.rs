use std::{
    future::Future,
    path::{Path, PathBuf},
};

use crate::ai::cloud_environments::{AmbientAgentEnvironment, GithubRepo};
use crate::terminal::model::session::command_executor::shell_escape_single_quotes;
use crate::terminal::shell::ShellType;

use repo_metadata::repositories::{DetectedRepositories, RepoDetectionSource};
use warp_completer::completer::CommandExitStatus;
use warp_core::{command::ExitCode, safe_info, safe_warn};
use warpui::{ModelContext, ModelSpawner, SingletonEntity};

use super::{terminal::TerminalDriver, AgentDriverError};
use warp_cli::agent::Harness;

#[derive(Debug, thiserror::Error)]
pub enum PrepareEnvironmentError {
    #[error("Invalid runtime state - please file a bug report.")]
    InvalidRuntimeState,
    #[error("Failed to clone {repo_name}")]
    CloneRepo { repo_name: String },
    #[error("Failed to run setup command: {command}")]
    SetupCommand { command: String },
    #[error("Failed to change directory into {repo_name}")]
    ChangeDirectory { repo_name: String },
    #[error("Terminal driver error while preparing environment: {source}")]
    TerminalDriver { source: AgentDriverError },
}

/// Prepare a cloud agent environment within a terminal session. This will:
/// 1. Clone all repositories, skipping any that are already cloned.
/// 2. Begin codebase indexing for all repositories (Oz harness only).
/// 3. Run any setup commands.
/// 4. If there is only one repository, navigate into it.
///
/// `is_sandbox` tells the preparer that `working_dir` only exists inside a
/// Docker sandbox container and therefore the host filesystem can't be used
/// for repo detection or indexing. This is an explicit signal from the
/// caller rather than a path-prefix inference, so non-sandbox callers that
/// happen to pass a path like `/home/agent/...` don't silently flip into
/// sandbox-only mode.
pub fn prepare_environment(
    environment: AmbientAgentEnvironment,
    working_dir: PathBuf,
    is_sandbox: bool,
    _harness: Harness,
    ctx: &mut ModelContext<TerminalDriver>,
) -> impl Future<Output = Result<(), PrepareEnvironmentError>> {
    let spawner = ctx.spawner();
    async move {
        let AmbientAgentEnvironment {
            github_repos,
            setup_commands,
            ..
        } = environment;

        let result = prepare_environment_impl(
            &spawner,
            working_dir.as_path(),
            is_sandbox,
            &github_repos,
            setup_commands,
        )
        .await;

        result
    }
}

async fn prepare_environment_impl(
    spawner: &ModelSpawner<TerminalDriver>,
    working_dir: &Path,
    is_sandbox: bool,
    github_repos: &[GithubRepo],
    setup_commands: Vec<String>,
) -> Result<(), PrepareEnvironmentError> {
    let working_dir_string = working_dir.to_string_lossy().to_string();

    // Position the session in `working_dir` before running any probes / clones.
    // Routed through the silent executor so we don't add a user-visible `cd`
    // block to the blocklist — in the common case (cloud agents) the session
    // is already cd'd here by its startup dir, so this is a no-op re-cd and
    // shouldn't appear in the user's terminal history.
    if !cd_in_terminal_silent(working_dir_string.clone(), spawner).await? {
        return Err(PrepareEnvironmentError::ChangeDirectory {
            repo_name: working_dir_string,
        });
    }

    for repo in github_repos {
        let repo_name = format!("{}/{}", repo.owner, repo.repo);
        let repo_url = format!("https://github.com/{repo_name}.git");
        // We do a partial clone here to speed up environment setup time.
        let command = format!("git clone --filter=tree:0 {repo_url}");

        let repo_dir = working_dir.join(&repo.repo);
        // Always ask the session whether the repo dir already exists, rather
        // than stat'ing from the host. The session knows about sandbox-only
        // paths, and this goes through the silent executor so `test -d` is
        // not added to the user-visible blocklist. Pass the absolute path
        // explicitly so the probe doesn't rely on the session's CWD.
        let dir_exists = terminal_directory_exists(&repo_dir.to_string_lossy(), spawner).await?;

        if dir_exists {
            safe_warn!(
                safe: ("We already have a directory with the same repository name in the terminal working directory, skipping clone..."),
                full: (
                "We already have a directory with the name {} in the terminal working directory, skipping clone...",
                repo.repo)
            );
        } else {
            safe_info!(
                safe: ("Cloning repository via terminal"),
                full: ("Cloning repository via terminal: {repo_name}")
            );

            let exit_code = execute_command(command, spawner).await?;
            if exit_code != 0.into() {
                return Err(PrepareEnvironmentError::CloneRepo {
                    repo_name: repo_name.clone(),
                });
            }

            safe_info!(
                safe: ("Successfully cloned repository"),
                full: ("Successfully cloned: {repo_name}")
            );
        }

        // Register the repo with DetectedRepositories so that the skill watcher
        // and other repo-aware subsystems can discover it before the first query.
        //
        // TODO(advait): When the remote code server lands for Docker sandboxes,
        // sandbox-only working directories will be reachable from the host and
        // we should register + index them here too (likely via a remote-aware
        // path instead of `detect_possible_git_repo`/`index_directory`, which
        // both assume a local filesystem). For now, skip so we don't try to
        // stat paths that only exist inside the sandbox.
        if is_sandbox {
            safe_info!(
                safe: ("Skipping local repo detection for sandbox-only working directory"),
                full: (
                    "Skipping local repo detection and indexing for sandbox-only working directory {}",
                    working_dir.display()
                )
            );
        } else {
            let repo_dir_str = repo_dir.to_string_lossy().to_string();
            let detect_future = spawner
                .spawn(move |_, ctx| {
                    DetectedRepositories::handle(ctx).update(ctx, |repos, ctx| {
                        repos.detect_possible_git_repo(
                            &repo_dir_str,
                            RepoDetectionSource::CloudEnvironmentPrep,
                            ctx,
                        )
                    })
                })
                .await
                .map_err(|_| PrepareEnvironmentError::InvalidRuntimeState)?;
            // Await detection so the repo is registered in DirectoryWatcher
            // before the agent's first query.
            if detect_future.await.is_none() {
                safe_warn!(
                    safe: ("Repository detection returned no path"),
                    full: ("Repository detection returned no path for {}", repo_dir.display())
                );
            }
        }
    }

    let has_setup_commands = !setup_commands.is_empty();
    if has_setup_commands {
        // Set CI=true so setup commands run in a CI-like environment. This should help us run
        // non-interactive versions of setup commands, as many command line tools recognize the CI
        // environment variable.
        execute_command("export CI=true".to_string(), spawner).await?;
    }

    for command in setup_commands {
        let command_for_error = command.clone();
        safe_info!(
            safe: ("Running setup command"),
            full: ("Running setup command: {command}")
        );

        let exit_code = execute_command(command, spawner).await?;
        if exit_code != 0.into() {
            return Err(PrepareEnvironmentError::SetupCommand {
                command: command_for_error,
            });
        }

        let working_dir_string = working_dir.to_string_lossy().to_string();
        if let Err(error) = cd_in_terminal(working_dir_string, spawner).await {
            log::warn!("Failed to reset working directory after setup command: {error}");
        }

        safe_info!(
            safe: ("Successfully completed setup command"),
            full: ("Successfully completed setup command: {command_for_error}")
        );
    }

    if has_setup_commands {
        // Unset CI after setup commands complete so the agent session
        // does not run with CI=true.
        execute_command("unset CI".to_string(), spawner).await?;
    }

    // If there's only one repo in the environment, start the agent in that repo.
    // This way, it doesn't have to locate the correct repo to work on.
    if let Some(repo_name) = single_repo_name(github_repos) {
        safe_info!(
            safe: ("Changing directory into single repository"),
            full: ("Changing directory into single repository: {repo_name}")
        );
        let exit_code = cd_in_terminal(repo_name.clone(), spawner).await?;
        if exit_code != 0.into() {
            return Err(PrepareEnvironmentError::ChangeDirectory { repo_name });
        }
    }

    Ok(())
}

/// Execute a command in the context of a terminal session.
async fn execute_command(
    command: String,
    spawner: &ModelSpawner<TerminalDriver>,
) -> Result<ExitCode, PrepareEnvironmentError> {
    spawner
        .spawn(move |terminal_driver, ctx| terminal_driver.execute_command(&command, ctx))
        .await
        .map_err(|_| PrepareEnvironmentError::InvalidRuntimeState)?
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })
}

/// Change the current directory in the context of a terminal session (using `cd {dir}`).
async fn cd_in_terminal(
    target: String,
    spawner: &ModelSpawner<TerminalDriver>,
) -> Result<ExitCode, PrepareEnvironmentError> {
    spawner
        .spawn(move |terminal_driver, ctx| terminal_driver.cd(&target, ctx))
        .await
        .map_err(|_| PrepareEnvironmentError::InvalidRuntimeState)?
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })
}

fn single_repo_name(repos: &[GithubRepo]) -> Option<String> {
    if repos.len() != 1 {
        return None;
    }
    Some(repos[0].repo.clone())
}

/// Change the active terminal session's working directory via `cd <target>`,
/// silently.
///
/// Thin wrapper around [`TerminalDriver::cd_silent`] so the call stays
/// consistent with the other `*_in_terminal` / `terminal_*` helpers in this
/// module. Uses the same [`ShellFamily::shell_escape`] logic as the visible
/// [`TerminalDriver::cd`] path, so it's safe across bash/zsh/fish/pwsh host
/// shells.
///
/// Returns `true` if the `cd` exited successfully.
async fn cd_in_terminal_silent(
    target: String,
    spawner: &ModelSpawner<TerminalDriver>,
) -> Result<bool, PrepareEnvironmentError> {
    let output = spawner
        .spawn(move |driver, ctx| driver.cd_silent(&target, ctx))
        .await
        .map_err(|_| PrepareEnvironmentError::InvalidRuntimeState)?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?;
    Ok(output.status == CommandExitStatus::Success)
}

/// Returns whether the given path resolves to an existing directory from the
/// perspective of the active terminal session.
///
/// Runs `test -d <path>` through the session's in-band command executor, so
/// the check is invisible in the user-facing blocklist and works for paths
/// that only exist inside a remote/sandbox filesystem. The path is escaped
/// using the *session's* actual shell type (bash/zsh use the `'"'"'` trick,
/// fish uses a backslash, PowerShell doubles the quote) rather than assuming
/// bash.
///
/// Prefer passing an absolute path: relative paths resolve against the
/// session's current working directory, which couples the caller to
/// whatever `cd` state the session happens to be in.
///
/// TODO(advait): `test -d ...` itself is POSIX-only. When we support
/// environment prep on Windows host shells (PowerShell / cmd.exe), also
/// branch on `ShellType` to emit the appropriate probe (e.g.
/// `Test-Path -PathType Container <path>` for PowerShell).
async fn terminal_directory_exists(
    path: &str,
    spawner: &ModelSpawner<TerminalDriver>,
) -> Result<bool, PrepareEnvironmentError> {
    let path = path.to_owned();
    let output = spawner
        .spawn(move |driver, ctx| {
            // Fall back to Bash if the session's shell type isn't known yet
            // (e.g. pre-bootstrap). Bash-style escaping is a safe default for
            // every POSIX shell we currently support.
            let shell_type = driver
                .active_session_shell_type(ctx)
                .unwrap_or(ShellType::Bash);
            let escaped = shell_escape_single_quotes(&path, shell_type);
            let command = format!("test -d '{escaped}'");
            driver.execute_silent_command(command, ctx)
        })
        .await
        .map_err(|_| PrepareEnvironmentError::InvalidRuntimeState)?
        .await
        .map_err(|error| match error {
            AgentDriverError::InvalidRuntimeState => PrepareEnvironmentError::InvalidRuntimeState,
            source => PrepareEnvironmentError::TerminalDriver { source },
        })?;
    Ok(output.status == CommandExitStatus::Success)
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
