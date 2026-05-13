// OpenWarp:presigned_upload 模块已本地化为 stub。
// 历史职责:把 attachment / artifact 字节流上传到 warp.dev 后端通过 GraphQL
// 颁发的 S3-style presigned URL(走 PUT/POST + content-length + CRC32C
// checksum 校验)。OpenWarp 走 BYOP 本地路径,不需要这条云端上传腿。
//
// 保留:
//   - `HttpStatusError`:被 retry_strategies / agent_sdk/retry / agent_sdk/driver/attachments
//     等多处用于"瞬时 vs 永久"HTTP 错误分类,与 stub 化无关,纯通用类型。
//   - `upload_to_target`:函数签名保留,所有调用一律返回
//     "Presigned upload disabled in OpenWarp" 错误,让上层 caller 走 `?` 自然失败。
// 删除:
//   - CRC32C checksum 流式计算、build_upload_request / send_upload_request /
//     ensure_upload_succeeded / NormalizedUploadTarget 等所有 HTTP 上传内部辅助。
//   - 对 `super::harness_support::UploadTarget` 的 `From` 适配(stub 后不再需要规整化)。

use anyhow::{anyhow, Result};
use thiserror::Error;

use super::harness_support::UploadTarget;

/// Typed error for HTTP-backed operations so downstream classifiers (e.g. the agent-SDK
/// retry helper) can decide transient vs permanent failures without string-parsing the
/// anyhow Display.
///
/// OpenWarp 已下线云端 presigned upload,但该类型仍被本地 HTTP 失败分类逻辑
/// (retry_strategies、agent_sdk 的瞬时/永久判定)复用,故保留。
#[derive(Debug, Error)]
#[error("HTTP request failed with status {status}: {body}")]
pub struct HttpStatusError {
    pub status: u16,
    pub body: String,
}

/// OpenWarp:云端 presigned URL 上传已下线,签名保留以兼容现有 caller,直接返回错误。
pub(crate) async fn upload_to_target(
    _http_client: &http_client::Client,
    _target: &UploadTarget,
    _body: impl Into<reqwest::Body>,
) -> Result<()> {
    Err(anyhow!("Presigned upload disabled in OpenWarp"))
}
