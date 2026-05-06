use std::path::Path;
use std::sync::Arc;

use crate::language_server_candidate::{LanguageServerCandidate, LanguageServerMetadata};
#[cfg(feature = "local_fs")]
use crate::supported_servers::CustomBinaryConfig;
use crate::CommandBuilder;
use async_trait::async_trait;

#[cfg(feature = "local_fs")]
const SERVER_NAME: &str = "gopls";

#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
pub struct GoPlsCandidate {
    client: Arc<http_client::Client>,
}

impl GoPlsCandidate {
    pub fn new(client: Arc<http_client::Client>) -> Self {
        Self { client }
    }

    #[cfg(feature = "local_fs")]
    pub async fn find_installed_binary_config() -> Option<CustomBinaryConfig> {
        let binary_path = warp_core::paths::data_dir()
            .join(SERVER_NAME)
            .join(if cfg!(windows) { "gopls.exe" } else { "gopls" });
        if !binary_path.is_file() {
            return None;
        }

        let output = command::r#async::Command::new(&binary_path)
            .arg("version")
            .output()
            .await
            .ok()?;
        if !output.status.success() {
            return None;
        }

        Some(CustomBinaryConfig {
            binary_path,
            prepend_args: vec![],
        })
    }
}

#[async_trait]
#[cfg(feature = "local_fs")]
impl LanguageServerCandidate for GoPlsCandidate {
    async fn should_suggest_for_repo(&self, path: &Path, executor: &CommandBuilder) -> bool {
        if !path.join("go.mod").exists() {
            return false;
        }

        // Check if Go is installed
        executor
            .command("go")
            .arg("version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn is_installed_in_data_dir(&self, _executor: &CommandBuilder) -> bool {
        Self::find_installed_binary_config().await.is_some()
    }

    async fn is_installed_on_path(&self, executor: &CommandBuilder) -> bool {
        executor
            .command("gopls")
            .arg("version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn install(
        &self,
        _metadata: LanguageServerMetadata,
        executor: &CommandBuilder,
    ) -> anyhow::Result<()> {
        let install_dir = warp_core::paths::data_dir().join(SERVER_NAME);
        async_fs::create_dir_all(&install_dir).await?;

        let output = executor
            .command("go")
            .args(["install", "golang.org/x/tools/gopls@latest"])
            .env("GOBIN", &install_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install gopls: {}", stderr);
        }

        Ok(())
    }

    async fn fetch_latest_server_metadata(&self) -> anyhow::Result<LanguageServerMetadata> {
        // gopls 通过 `go install ...@latest` 安装，不需要预先访问 GitHub 获取 metadata。
        // 这样与 opencode 一致，也避免 GitHub API rate limit 阻断安装链路。
        let _ = &self.client;
        Ok(LanguageServerMetadata {
            version: "latest".to_string(),
            url: None,
            digest: None,
        })
    }
}

#[async_trait]
#[cfg(not(feature = "local_fs"))]
impl LanguageServerCandidate for GoPlsCandidate {
    async fn should_suggest_for_repo(&self, _path: &Path, _executor: &CommandBuilder) -> bool {
        false
    }

    async fn is_installed_in_data_dir(&self, _executor: &CommandBuilder) -> bool {
        false
    }

    async fn is_installed_on_path(&self, _executor: &CommandBuilder) -> bool {
        false
    }

    async fn install(
        &self,
        _metadata: LanguageServerMetadata,
        _executor: &CommandBuilder,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn fetch_latest_server_metadata(&self) -> anyhow::Result<LanguageServerMetadata> {
        todo!()
    }
}
