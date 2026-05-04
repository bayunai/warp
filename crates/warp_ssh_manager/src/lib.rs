//! SSH 管理器数据层 — 持久化的服务器 / 文件夹树 + OS keychain 凭据存储 +
//! 命令拼装。UI 与 PTY 注入逻辑放在 `app/src/ssh_manager/` 与 `secret_injector`
//! 模块,这里保持纯 Rust、无 warpui 依赖、可单独 `cargo test` 跑。

pub mod repository;
pub mod secrets;
pub mod ssh_command;
pub mod types;

pub use repository::{SshRepository, SshRepositoryError};
pub use secrets::{SecretKind, SshSecretStore, SshSecretStoreError};
pub use ssh_command::{build_ssh_args, build_ssh_command_line};
pub use types::{AuthType, NodeKind, SshNode, SshServerInfo};
