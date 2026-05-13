pub mod ai;
// OpenWarp Wave 3-1:`server_api/auth.rs`(AuthClient trait + impl)整文件物理删,
// `AuthManager` 改为本地 stub。两个 HTTP header 常量直接迁入本文件,供 ambient agent
// 路径继续使用(实际运行时永远不命中,因 OpenWarp 已无云端 ambient workload)。
// OpenWarp Wave 6-8:`server_api/block.rs`(BlockClient trait + impl)与
// `server_api/referral.rs`(ReferralsClient trait + impl)整文件物理删 —— 两个
// trait 全部 stub Err / 空列表,对应的 `ShowBlocksView` / `ReferralsPageView`
// 设置页一并移除。
pub mod harness_support;
pub mod integrations;
pub mod managed_secrets;
pub(crate) mod presigned_upload;
// OpenWarp(Wave 3-2):`team` / `workspace` 两个 client trait 与 impl 已物理删,
// 在 app/ 外 0 消费,UserWorkspaces / TeamUpdateManager 已在 Phase 5 本地化为 no-op。

use crate::ai::ambient_agents::AmbientAgentTaskId;
use crate::auth::AuthState;
use crate::server::graphql::default_request_options;
use ai::{AIClient, LocalAIClient};
use channel_versions::{ChannelChangelogs, ChannelVersion, ChannelVersions, VersionInfo};
use futures::StreamExt;
use warpui::{r#async::BoxFuture, ModelContext};

// OpenWarp Wave 5-3:原 `AMBIENT_WORKLOAD_TOKEN_HEADER` 随 `generate_multi_agent_output` 云端
// SSE 路径 stub 化后在全仓库 0 消费,物理删。`get_or_create_ambient_workload_token`
// 在 W3-1 后永返 `None`,代码中不再有 header 注入点。

/// Header key for the cloud agent task ID attached to requests from ambient agents.
pub const CLOUD_AGENT_ID_HEADER: &str = "X-Warp-Cloud-Agent-ID";

use crate::settings::PrivacySettingsSnapshot;
use crate::settings_view;

use crate::ChannelState;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use instant::Instant;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use warp_core::errors::{AnyhowErrorExt, ErrorExt};
use warp_core::register_error;
use warp_core::telemetry::TelemetryEvent;
use warpui::Entity;
use warpui::SingletonEntity;

use super::experiments::ServerExperiment;
use super::experiments::ServerExperiments;
use super::graphql::GraphQLError;

pub const FETCH_CHANNEL_VERSIONS_TIMEOUT: std::time::Duration = Duration::from_secs(60);

// openWarp 闭源遥测剥离 P4b:`X-Warp-Experiment-Id` HTTP header 原本携带 anonymous_id
// 注入到 /client/login、fetch_channel_versions、GraphQL 等请求,服务端用于实验分组 +
// 跨会话追踪。P0 后值已是 nil-UUID,P4b 直接删 header 注入,服务端见到的请求里就
// 不再有这个字段。注入点(共 3 处)同步删除。

/// We use a special error code header `X-Warp-Error-Code` to allow the server to send
/// more specific error code information, so that the client can discern between different
/// errors with the same error code.
/// See errors/http_error_codes.go on the server for possible values.
const WARP_ERROR_CODE_HEADER: &str = "X-Warp-Error-Code";

/// An error indicating the user is out of credits. The server sends 429s to communicate this
/// state, but if Cloud Run is overloaded, it can also send 429s that aren't credit-related.
/// So we use this to distinguish between the two cases.
const WARP_ERROR_CODE_OUT_OF_CREDITS: &str = "OUT_OF_CREDITS";

/// Error code indicating the user has reached their cloud agent concurrency limit.
const WARP_ERROR_CODE_AT_CAPACITY: &str = "AT_CLOUD_AGENT_CAPACITY";

/// Header used to communicate the source of an agent run (e.g. "CLI", "GITHUB_ACTION").
pub(crate) const AGENT_SOURCE_HEADER: &str = "X-Oz-Api-Source";

#[cfg(feature = "agent_mode_evals")]
pub const EVAL_USER_ID_HEADER: &str = "X-Eval-User-ID";

/// IDs in the staging database that were created specifically for evals.
/// These users have a clean state where they haven't been referred nor have referred anyone (which causes a popup in the client).
/// DO NOT REMOVE OR CHANGE THESE USERS!
///
/// Keep this list in sync with `script/populate_agent_mode_eval_user.sql`
/// in warp-server. Those rows need to exist in the DB so the authz user loader
/// can resolve these IDs during task creation; otherwise the server will 500
/// on every eval request with a nil-deref in `UserIDFromUser`.
#[cfg(feature = "agent_mode_evals")]
const EVAL_USER_IDS: [i32; 11] = [
    2162, 2164, 2165, 2166, 2167, 2168, 2169, 2172, 2173, 2174, 2175,
];

/// ResponseType received by Client
#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
#[error("{error}")]
pub struct ClientError {
    pub error: String,
    // We unconditionally check for GitHub auth errors in any public API response. It'd be much better
    // to have the server return error codes that we can parse, but this isn't yet supported.
    // See REMOTE-666
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
}

/// Error when the user is at their cloud agent concurrency limit.
#[derive(thiserror::Error, Debug, Clone, Deserialize)]
#[error("{error} (running agents: {running_agents})")]
pub struct CloudAgentCapacityError {
    pub error: String,
    pub running_agents: i32,
}

// OpenWarp Wave 5-3:`TimeResponse` 随云端 `/current_time` GET 接口 stub 化后 0 消费,物理删。

#[derive(Debug, Clone)]
pub struct ServerTime {
    time_at_fetch: DateTime<FixedOffset>,
    fetched_at: Instant,
}

impl ServerTime {
    pub fn current_time(&self) -> DateTime<FixedOffset> {
        let elapsed = chrono::Duration::from_std(self.fetched_at.elapsed())
            .expect("duration should not be bigger than limit");
        self.time_at_fetch + elapsed
    }
}

/// Wrapper for deserialization errors. This covers both:
/// * Using `serde` directly
/// * Using `reqwest` decoding utilities
#[derive(thiserror::Error, Debug)]
pub enum DeserializationError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum AIApiError {
    #[error("Request failed due to lack of AI quota.")]
    QuotaLimit,

    #[error("Warp is currently overloaded. Please try again later.")]
    ServerOverloaded,

    #[error("Internal error occurred at transport layer.")]
    Transport(#[source] reqwest::Error),

    #[error("Failed to deserialize API response.")]
    Deserialization(#[source] DeserializationError),

    #[error("No context found on context search.")]
    NoContextFound,

    #[error("Failed with status code {0}: {1}")]
    ErrorStatus(http::StatusCode, String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("Got error when streaming {stream_type}: {source:#}")]
    Stream {
        stream_type: &'static str,
        #[source]
        source: anyhow::Error,
    },
}

impl From<http_client::ResponseError> for AIApiError {
    fn from(err: http_client::ResponseError) -> Self {
        Self::from_response_error(err.source, &err.headers)
    }
}

impl From<reqwest::Error> for AIApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::from_transport_error(err)
    }
}

impl From<serde_json::Error> for AIApiError {
    fn from(err: serde_json::Error) -> Self {
        AIApiError::Deserialization(err.into())
    }
}

impl AIApiError {
    /// Converts a reqwest error to an AIApiError, using response headers to distinguish
    /// between different types of 429 errors.
    fn from_response_error(err: reqwest::Error, headers: &::http::HeaderMap) -> Self {
        // For HTTP 429 errors, check the X-Warp-Error-Code header to distinguish
        // between out-of-credits and server-overload.
        if err.status() == Some(http::StatusCode::TOO_MANY_REQUESTS) {
            return Self::error_for_429(headers);
        }

        Self::from_transport_error(err)
    }

    /// Converts a transport-level reqwest error (no HTTP response) to an AIApiError.
    fn from_transport_error(err: reqwest::Error) -> Self {
        // Unfortunately, `reqwest` reports some non-decoding errors as decoding errors (e.g.
        // unexpected disconnects or timeouts while deserializing a response body). Since we
        // render deserialization and transport errors differently, we try to detect those cases
        // here.
        if err.is_timeout() {
            return AIApiError::Transport(err);
        }
        if err.is_decode() {
            #[cfg(not(target_family = "wasm"))]
            {
                use std::error::Error as _;
                let mut source = err.source();
                while let Some(underlying) = source {
                    if underlying.is::<hyper::Error>() {
                        return AIApiError::Transport(err);
                    }

                    source = underlying.source();
                }
            }

            return AIApiError::Deserialization(DeserializationError::Transport(err));
        }

        AIApiError::Transport(err)
    }

    /// Returns the appropriate error for a 429 response by checking the X-Warp-Error-Code header.
    fn error_for_429(headers: &::http::HeaderMap) -> Self {
        if headers
            .get(WARP_ERROR_CODE_HEADER)
            .and_then(|v| v.to_str().ok())
            == Some(WARP_ERROR_CODE_OUT_OF_CREDITS)
        {
            AIApiError::QuotaLimit
        } else {
            AIApiError::ServerOverloaded
        }
    }

    /// Format a stream error into a human-readable error message. This will read the response
    /// body if there is one.
    async fn from_stream_error(stream_type: &'static str, err: reqwest_eventsource::Error) -> Self {
        match err {
            reqwest_eventsource::Error::InvalidStatusCode(
                http::StatusCode::TOO_MANY_REQUESTS,
                ref res,
            ) => Self::error_for_429(res.headers()),
            reqwest_eventsource::Error::InvalidStatusCode(status, res) => Self::ErrorStatus(
                status,
                res.text()
                    .await
                    .unwrap_or_else(|e| format!("(no response body: {e:#})")),
            ),
            reqwest_eventsource::Error::Transport(err) => Self::from_transport_error(err),
            err => AIApiError::Stream {
                stream_type,
                // On WASM, `reqwest_eventsource::Error` doesn't implement `Into<anyhow::Error>` or
                // `Send` because it may contain a `wasm_bindgen` JS value.
                #[cfg(target_family = "wasm")]
                source: anyhow!("{err:#?}"),
                #[cfg(not(target_family = "wasm"))]
                source: anyhow!(err),
            },
        }
    }

    /// Returns whether or not the error can be retried.
    pub fn is_retryable(&self) -> bool {
        // Don't retry client errors, except for timeouts and quota limits.
        fn is_retryable_status(status: http::StatusCode) -> bool {
            !status.is_client_error()
                || status == http::StatusCode::REQUEST_TIMEOUT
                || status == http::StatusCode::TOO_MANY_REQUESTS
        }

        match self {
            AIApiError::ErrorStatus(status, _) => is_retryable_status(*status),
            AIApiError::Transport(e) => {
                if let Some(status) = e.status() {
                    return is_retryable_status(status);
                }
                true
            }
            // By default, retry on error.
            _ => true,
        }
    }
}

impl ErrorExt for AIApiError {
    fn is_actionable(&self) -> bool {
        match self {
            AIApiError::Deserialization(_) => true,
            AIApiError::Transport(error) => error.is_actionable(),
            AIApiError::Other(error) => error.is_actionable(),
            AIApiError::Stream { source, .. } => source.is_actionable(),
            AIApiError::ErrorStatus(_, _) => self.is_retryable(),
            AIApiError::QuotaLimit | AIApiError::ServerOverloaded | AIApiError::NoContextFound => {
                false
            }
        }
    }
}
register_error!(AIApiError);

#[derive(thiserror::Error, Debug)]
pub enum TranscribeError {
    #[error("Request failed due to lack of Voice quota.")]
    QuotaLimit,

    #[error("Warp is currently overloaded. Please try again later.")]
    ServerOverloaded,

    #[error("Internal error occurred at transport layer.")]
    Transport,

    #[error("Failed to deserialize JSON.")]
    Deserialization,

    /// OpenWarp 已禁用语音转写(BYOP genai 协议无法承载音频)。
    #[error("Voice transcription is unavailable in OpenWarp.")]
    Disabled,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        // The WASM version of this type has no bound on `Send`, which is not implemented on
        // `wasm_bindgen::JsValue`, which is ultimately used in reqwest_eventsource::Error. Furthermore,
        // `Send` is an unnecessary bound when targeting wasm because the browser is single-threaded (and
        // we don't leverage WebWorkers for async execution in WoW).
        pub type AIOutputStream<T> = futures::stream::LocalBoxStream<'static, Result<T, Arc<AIApiError>>>;
    } else {
        pub type AIOutputStream<T> = futures::stream::BoxStream<'static, Result<T, Arc<AIApiError>>>;
    }
}

/// An event related to the server API itself (and not a particular API call).
/// Most errors should be handled in callbacks to individual APIs, rather than sent over the
/// server API channel.
//
// OpenWarp Wave 6-1:`NeedsReauth` 与 `AccessTokenRefreshed` 两个 variant 在 Wave 3-1
// 删 auth 子系统后已无任何 emit 点(全仓 0 处 `try_send`),订阅链(`wire_auth_token_rotation`
// + `ServerApiProvider::new` 内 match 分支)随之物理删。
#[derive(Clone, Debug)]
pub enum ServerApiEvent {
    /// We made a staging API call that was blocked, which may indicate a firewall misconfiguration.
    StagingAccessBlocked,
    /// The user's account has been disabled.
    UserAccountDisabled,
}

/// An API wrapper struct with methods to requests to warp-server.
///
/// Prefer NOT adding new methods directly on this struct; instead, add to one of the existing
/// client trait objects, or create your own. This helps keep `ServerApi` from being overloaded
/// with disparate types of calls, and allows you to mock methods in tests.
pub struct ServerApi {
    client: Arc<http_client::Client>,
    auth_state: Arc<AuthState>,
    event_sender: async_channel::Sender<ServerApiEvent>,
    // OpenWarp Wave 5-2:`telemetry_api: TelemetryApi` 字段物理删 — TelemetryApi
    // 以全 no-op 实现存在于 `server/telemetry/mod.rs`，但 `flush_telemetry_events` /
    // `flush_and_persist_events` / `flush_persisted_events_to_rudder` 均 0 外部消费，
    // `send_telemetry_event` 仅 view.rs:25525 一处调用，直接折 no-op 到本 struct。
    last_server_time: Arc<Mutex<Option<ServerTime>>>,
    // OpenWarp Wave 3-1:原 `oauth_client: self::auth::OAuth2Client` 随 auth.rs 一同
    // 物理删。CLI headless device auth 路径在 OpenWarp 下线。
    // OpenWarp Wave 6-1:`ambient_workload_token: Arc<Mutex<Option<WorkloadToken>>>` 字段
    // 物理删 — 仅在 `new` / `new_for_test` 初始化为 `None`,永无 get/set;
    // `get_or_create_ambient_workload_token` 实现已是 `Ok(None)` 不读字段。
    /// The ambient agent task ID for requests from cloud agents.
    ambient_agent_task_id: Arc<RwLock<Option<AmbientAgentTaskId>>>,
    /// The source of agent runs (e.g. CLI, GitHub Action). Set once at startup and immutable.
    agent_source: Option<ai::AgentSource>,

    #[cfg(feature = "agent_mode_evals")]
    eval_user_id: Option<i32>,
}

/// OpenWarp 仍保留的本地 `ServerApi` 窄接口:
/// - 复用长生命周期 HTTP client
/// - 管理 ambient task header 上下文
///
/// 这两项被 BYOP/本地 harness 继续使用,但不再向调用方暴露整颗 `ServerApi` 壳。
pub trait LocalServerApiClient: 'static + Send + Sync {
    fn set_ambient_agent_task_id(&self, task_id: Option<AmbientAgentTaskId>);
    fn http_client(&self) -> &http_client::Client;
}

impl<T> LocalServerApiClient for Arc<T>
where
    T: LocalServerApiClient + ?Sized,
{
    fn set_ambient_agent_task_id(&self, task_id: Option<AmbientAgentTaskId>) {
        self.as_ref().set_ambient_agent_task_id(task_id);
    }

    fn http_client(&self) -> &http_client::Client {
        self.as_ref().http_client()
    }
}

/// OpenWarp 下仍需保留的本地 agent 事件流入口。
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
pub trait AgentEventStreamClient: 'static + Send + Sync {
    async fn stream_agent_events(
        &self,
        run_ids: &[String],
        since_sequence: i64,
    ) -> Result<http_client::EventSourceStream>;
}

impl ServerApi {
    fn new(
        auth_state: Arc<AuthState>,
        event_sender: async_channel::Sender<ServerApiEvent>,
        agent_source: Option<ai::AgentSource>,
    ) -> Self {
        // We generate a random user ID for evals so we can run evals in parallel.
        #[cfg(feature = "agent_mode_evals")]
        let oauth_user_id = {
            use rand::Rng;
            Some(EVAL_USER_IDS[rand::thread_rng().gen_range(0..EVAL_USER_IDS.len())])
        };

        Self {
            client: Arc::new(http_client::Client::new()),
            auth_state,
            event_sender,
            last_server_time: Arc::new(Mutex::new(None)),
            ambient_agent_task_id: Arc::new(RwLock::new(None)),
            agent_source,
            #[cfg(feature = "agent_mode_evals")]
            eval_user_id: oauth_user_id,
        }
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        let (tx, _) = async_channel::unbounded();

        Self {
            client: Arc::new(http_client::Client::new_for_test()),
            auth_state: Arc::new(AuthState::new_for_test()),
            event_sender: tx,
            last_server_time: Arc::new(Mutex::new(None)),
            ambient_agent_task_id: Arc::new(RwLock::new(None)),
            agent_source: None,
            #[cfg(feature = "agent_mode_evals")]
            eval_user_id: None,
        }
    }

    /// Sets the ambient agent task ID to be sent with all subsequent requests.
    pub fn set_ambient_agent_task_id(&self, task_id: Option<AmbientAgentTaskId>) {
        *self.ambient_agent_task_id.write() = task_id;
    }

    /// Returns ambient agent headers to attach to requests.
    ///
    /// OpenWarp Wave 4-1:原调用 `get_or_create_ambient_workload_token().await`
    /// 获取 `X-Warp-Ambient-Workload-Token` header,W3-1 后该 token 代库永返 None
    /// (企业号 federation 路径随 auth.rs 下线),直接删掉该分支。`task_id`
    /// + `agent_source` header 仍按设置动态附加,运行时 BYOP 路径可能仍会
    /// `set_ambient_agent_task_id(Some(_))`。
    fn ambient_agent_headers(&self) -> Vec<(&'static str, String)> {
        let task_id = self
            .ambient_agent_task_id
            .read()
            .as_ref()
            .map(|id| id.to_string());

        let agent_source = self.agent_source.as_ref().map(|s| s.as_str().to_string());

        task_id
            .map(|id| (CLOUD_AGENT_ID_HEADER, id))
            .into_iter()
            .chain(agent_source.map(|s| (AGENT_SOURCE_HEADER, s)))
            .collect()
    }

    // OpenWarp Wave 3-1:`create_oauth_client` 随 `OAuth2Client` 类型与
    // `request_device_code` / `exchange_device_access_token` RPC 一同物理删。
    // CLI headless device auth 路径在 OpenWarp 下线。

    // OpenWarp Wave 3-1:`get_or_refresh_access_token()` 原是 `AuthClient` trait method。
    // trait 随 auth.rs 一同物理删,但 ServerApi 内部仍有 ~9 处调用。
    // 这里提供本地 stub:bearer token 取 AuthState 本地缓存(使用 `crate::auth::AuthToken`
    // 作为返回类型以兼容原 trait 签名)。
    //
    // OpenWarp Wave 6-1:`get_or_create_ambient_workload_token` 全仓 0 外部消费 + 实现
    // 已是 `Ok(None)`,物理删。

    pub async fn get_or_refresh_access_token(&self) -> Result<crate::auth::AuthToken> {
        Ok(self
            .auth_state
            .credentials()
            .map(|c| c.bearer_token())
            .unwrap_or(crate::auth::AuthToken::NoAuth))
    }

    pub fn send_graphql_request<'a, QF, O: warp_graphql::client::Operation<QF> + Send + 'a>(
        &'a self,
        operation: O,
        timeout: Option<Duration>,
    ) -> BoxFuture<'a, Result<QF>> {
        let client = self.client.clone();
        let event_sender = self.event_sender.clone();

        #[cfg(feature = "agent_mode_evals")]
        let headers = if let Some(eval_user_id) = self.eval_user_id {
            std::collections::HashMap::from([(
                EVAL_USER_ID_HEADER.to_string(),
                eval_user_id.to_string(),
            )])
        } else {
            Default::default()
        };

        Box::pin(async move {
            let operation_name = operation.operation_name().map(Cow::into_owned);
            // OpenWarp Wave 3-1:原 `self.get_or_refresh_access_token().await` (AuthClient method)
            // 随 auth.rs 一同物理删。本地化后 bearer token 直接读 AuthState 缓存,
            // OpenWarp 路径下绝大多数为 `None`。
            let auth_token = self.auth_state.get_access_token_ignoring_validity();

            #[cfg(feature = "agent_mode_evals")]
            let mut headers = headers;
            #[cfg(not(feature = "agent_mode_evals"))]
            let mut headers = std::collections::HashMap::new();

            for (name, value) in self.ambient_agent_headers() {
                headers.insert(name.to_string(), value);
            }

            let options = warp_graphql::client::RequestOptions {
                auth_token,
                timeout,
                headers,
                ..default_request_options()
            };

            let response = match operation.send_request(client, options).await {
                Ok(response) => response,
                Err(GraphQLError::StagingAccessBlocked) => {
                    let _ = event_sender.try_send(ServerApiEvent::StagingAccessBlocked);
                    anyhow::bail!(GraphQLError::StagingAccessBlocked)
                }
                Err(err) => anyhow::bail!(err),
            };

            if let Some(errors) = response.errors.as_ref() {
                crate::safe_error!(
                    safe: ("graphql response for {:?} had errors", operation_name),
                    full: ("graphql response for {:?} had errors {:?}", operation_name, errors)
                );

                // "User not in context: Not found" comes from warp-server as an error when attempting
                // to get a required user for some gql field. If we see that, since we have already
                // successfully refreshed the user's access token earlier in this function, we know
                // that this error is the result of the user's account being disabled/deleted.
                if errors
                    .iter()
                    .any(|error| error.message.contains("User not in context: Not found"))
                {
                    log::error!("GraphQL request failed due to unauthenticated user");
                    let _ = event_sender.try_send(ServerApiEvent::UserAccountDisabled);
                }
            }

            response.data.ok_or_else(|| {
                let operation_label = operation_name
                    .as_deref()
                    .unwrap_or("unknown GraphQL operation");
                let error_messages = response
                    .errors
                    .as_ref()
                    .map(|errors| {
                        errors
                            .iter()
                            .filter_map(|error| {
                                let message = error.message.trim();
                                (!message.is_empty()).then(|| message.to_string())
                            })
                            .collect::<Vec<_>>()
                            .join("; ")
                    })
                    .filter(|messages| !messages.is_empty());

                match error_messages {
                    Some(messages) => {
                        anyhow!("missing response data for {operation_label}: {messages}")
                    }
                    None => anyhow!("missing response data for {operation_label}"),
                }
            })
        })
    }

    /// Opens an SSE stream to the agent event-push endpoint.
    ///
    /// OpenWarp Wave 5-3:本方法原走 RTC 云端 SSE 端点 (云端 RTC `/api/v1/agent/events/stream`)
    /// 用于接收 cloud agent run 的事件推送。OpenWarp 已剩离云端 RTC pool,该 URL
    /// 不可达 — 直接 stub 返回错误。两个消费方都 graceful 处理 Err:
    /// - `ai/agent_sdk/ambient.rs:949`: 重连 retry 退避到最大重试后上抓,initial 连接
    ///   失败直接传播 Err
    /// - `ai/agent_events/driver.rs:126`: `.await?` 立即向上传播
    pub async fn stream_agent_events(
        &self,
        _run_ids: &[String],
        _since_sequence: i64,
    ) -> Result<http_client::EventSourceStream> {
        Err(anyhow!(
            "Cloud agent event stream disabled in OpenWarp — RTC endpoint is removed"
        ))
    }

    // OpenWarp Wave 5-3:`get_public_api` / `get_public_api_response` /
    // `post_public_api` / `post_public_api_response` / `post_public_api_unit` /
    // `patch_public_api_unit` / `error_from_response` 七个 private helper 原为
    // /api/v1/* HTTP REST 调用准备,server_api/* 8 个文件 stub 化后 0 外部消费,
    // 整体物理删。需要权限代理 / X-Warp 错误码 / HttpStatusError 包装的 BYOP
    // 路径依然会走 `send_graphql_request`(BYOP OAuth + cloud env image fetch),同路径与
    // public-api 拆开。本次顺手删除 `HttpStatusError` 在 server_api.rs 内唯一
    // 消费点后,外部 `presigned_upload.rs` 仅作为错误类型保留本体,import 需同步删。

    // OpenWarp Wave 4-1:`notify_login` (原向 /client/login 发生命令心跳) 0 消费方，物理删。

    /// 向 远端 Rudderstack 发送 [`TelemetryEvent`]。
    ///
    /// OpenWarp Wave 5-2：Rudder 网络层在 Phase 4-2(commit 60e37e160)中已全部物理删。
    /// 本方法仅为保留 `terminal/view.rs:25525` 一处现有调用点的调用签名兼容 —
    /// 方法体仅做轻量类型检查 + drain，返回 `Ok(())`。后续如果以后只剩
    /// 零 个调用点可连同该方法一同删。隔层的 `TelemetryApi` struct 与
    /// `flush_telemetry_events` / `flush_and_persist_events` / `flush_persisted_events_to_rudder`
    /// 均 0 外部消费 — 随本 PR 一起物理删。
    pub async fn send_telemetry_event(
        &self,
        event: impl TelemetryEvent,
        _settings_snapshot: PrivacySettingsSnapshot,
    ) -> Result<()> {
        let _ = event.name();
        Ok(())
    }

    // OpenWarp Wave 5-2：`flush_telemetry_events` 及 `flush_persisted_events_to_rudder` /
    // `persist_telemetry_events` 等均 0 外部消费，随 `TelemetryApi` 一同物理删。
    // 历史语义：本地落盘 telemetry batch 回放 → Rudderstack。

    // OpenWarp Wave 6-1:`pub async fn transcribe` 全仓 0 外部消费(语音 UI 走
    // `Transcriber` trait,实现在 `voice/transcriber.rs`),物理删 + 同步清
    // `TranscribeRequest` / `TranscribeResponse` import。`TranscribeError` enum 本身
    // 保留,继续被 `voice/transcriber.rs` 消费。

    pub async fn generate_multi_agent_output(
        &self,
        _request: &warp_multi_agent_api::Request,
    ) -> std::result::Result<AIOutputStream<warp_multi_agent_api::ResponseEvent>, Arc<AIApiError>>
    {
        // OpenWarp Wave 5-3:`generate_multi_agent_output` 是云端 agent SSE 端点
        // (`/ai/multi-agent` 与 `/ai/passive-suggestions`) 的唯一入口。OpenWarp 主走
        // BYOP 路径(`crate::ai::agent_providers::chat_stream::generate_byop_output`),
        // 本方法仅在 `byop_dispatch_info` 返回 `None` 时被调用作为 fallback,
        // 但云端已剩离,fallback 会被 dns 拒绝 / 404 拒绝 — 直接 stub
        // 为返回 `Disabled` 错误流。
        //
        // 所有消费点都通过 `take_until(cancellation_rx)` 或 channel 包装
        // graceful 处理 stream Err:
        // - `ai/agent/api/impl.rs:139` -> err event 走 channel
        // - `blocklist/controller/response_stream.rs:269/375` -> response_stream_result
        //   会走 retry/error handling
        // - `blocklist/passive_suggestions/maa.rs:177` -> passive suggestion 提取到 None
        //   后静默退出
        //
        // BYOP 主路径已配 provider 用户 → 不受影响(走不到本方法)。
        // 未配 BYOP 用户 → 立即报错而非超时拒绝,UX 改进。
        log::debug!("generate_multi_agent_output disabled in OpenWarp (BYOP-only)");
        let error_stream = futures::stream::once(async {
            Err(Arc::new(AIApiError::Other(anyhow!(
                "Cloud multi-agent endpoint disabled in OpenWarp — configure a BYOP provider in Settings"
            ))))
        });
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                Ok(error_stream.boxed_local())
            } else {
                Ok(error_stream.boxed())
            }
        }
    }

    fn set_server_time(&self, server_time: ServerTime) {
        let mut last_server_time = self.last_server_time.lock();
        *last_server_time = Some(server_time);
    }

    fn cached_server_time(&self) -> Option<ServerTime> {
        let last_server_time = self.last_server_time.lock();
        last_server_time.as_ref().cloned()
    }

    /// Returns the inner `http_client::Client` used by the `ServerApi`. Callers can use this long-lived
    /// client to make requests without having to create a new client.
    pub fn http_client(&self) -> &http_client::Client {
        &self.client
    }

    /// 返回用于计算 autoupdate update-by 截止时间的「服务器时间」。
    ///
    /// OpenWarp Wave 5-3:原实现 GET `云端/current_time` 端点进行时钟同步,
    /// OpenWarp 剩离云端 → 该端点不可达。唯一消费方 `root_view.rs::server_time_updated`
    /// 是在 autoupdate ready + 有 `update_by` 时以服务器时间为准决定是否马上重启,
    /// 不依赖时钟「权威」ⓓⓒⓓ。1987·仅为防本地时钟被用户手动拨后。OpenWarp
    /// 环境下允许使用本地时钟 → 返回本地 [`Utc::now()`] 包装的 [`ServerTime`],
    /// autoupdate 逻辑不变,且不再产生云端 HTTP 请求。
    pub async fn server_time(&self) -> Result<ServerTime> {
        if let Some(cached) = self.cached_server_time() {
            return Ok(cached);
        }

        let server_time = ServerTime {
            time_at_fetch: chrono::Utc::now().into(),
            fetched_at: Instant::now(),
        };
        self.set_server_time(server_time.clone());
        Ok(server_time)
    }

    /// Fetches updated Warp Channel Versions from Warp Server. If it is the first such request of
    /// the current calendar day, first attempts to call the '/client_version/daily'. If that call
    /// fails or if it not the first request of the calendar day, returns the result of a call to
    /// `/client_version'. The caller can specify whether or not changelog information should be
    /// included in the response based on whether or not it will be used.
    pub async fn fetch_channel_versions(
        &self,
        include_changelogs: bool,
        is_daily: bool,
    ) -> Result<ChannelVersions> {
        let _ = is_daily;
        let version = ChannelState::app_version()
            .unwrap_or("v0.local.testing.string_00")
            .to_string();
        let channel_version = ChannelVersion::new(VersionInfo::new(version));
        let changelogs = include_changelogs.then(|| ChannelChangelogs {
            dev: std::collections::HashMap::new(),
            preview: std::collections::HashMap::new(),
            stable: std::collections::HashMap::new(),
        });
        let versions = ChannelVersions {
            dev: channel_version.clone(),
            preview: channel_version.clone(),
            stable: channel_version,
            changelogs,
        };
        log::info!("OpenWarp 本地版跳过远端 channel versions 请求，返回本地版本快照");
        Ok(versions)
    }
}

impl LocalServerApiClient for ServerApi {
    fn set_ambient_agent_task_id(&self, task_id: Option<AmbientAgentTaskId>) {
        ServerApi::set_ambient_agent_task_id(self, task_id);
    }

    fn http_client(&self) -> &http_client::Client {
        ServerApi::http_client(self)
    }
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
impl AgentEventStreamClient for ServerApi {
    async fn stream_agent_events(
        &self,
        run_ids: &[String],
        since_sequence: i64,
    ) -> Result<http_client::EventSourceStream> {
        ServerApi::stream_agent_events(self, run_ids, since_sequence).await
    }
}

/// A singleton entity that provides access to the global [`ServerApi`] instance
/// and the few trait-object facades still retained in OpenWarp.
pub struct ServerApiProvider {
    server_api: Arc<ServerApi>,
    ai_client: Arc<dyn AIClient>,
}

impl ServerApiProvider {
    /// Constructs a new ServerApiProvider.
    pub fn new(
        auth_state: Arc<AuthState>,
        agent_source: Option<ai::AgentSource>,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        let (event_sender, event_receiver) = async_channel::bounded(10);
        let server_api = Arc::new(ServerApi::new(
            auth_state.clone(),
            event_sender,
            agent_source,
        ));
        let ai_client: Arc<dyn AIClient> = Arc::new(LocalAIClient::new());

        // OpenWarp Wave 6-1:原 `NeedsReauth` 分支调 `AuthManager::set_needs_reauth(true)`,
        // Wave 6-1 删 `ServerApiEvent::NeedsReauth` variant 后,剩余 variant 全部走
        // re-emit 路径,match 简化为直传。`AuthManager::set_needs_reauth` 函数本体保留
        // (`root_view.rs` web handoff 路径仍调,但已是 no-op)。
        ctx.spawn_stream_local(
            event_receiver,
            move |_, event, ctx| ctx.emit(event),
            |_, _| {},
        );
        Self {
            server_api,
            ai_client,
        }
    }

    /// Handles fetching server-side experiments by updating the appropriate app state.
    pub fn handle_experiments_fetched(
        &self,
        experiments: Vec<ServerExperiment>,
        ctx: &mut ModelContext<Self>,
    ) {
        ServerExperiments::handle(ctx).update(ctx, |state, ctx| {
            state.apply_latest_state(experiments, ctx);
        });

        settings_view::handle_experiment_change(ctx);
    }

    /// Constructs a new SeverApiProvider for tests.
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            server_api: Arc::new(ServerApi::new_for_test()),
            ai_client: Arc::new(LocalAIClient::new()),
        }
    }

    /// 返回 BYOP/本地 harness 仍需的最小本地接口:
    /// ambient task header 上下文 + 共享 HTTP client。
    pub fn get_local_client(&self) -> Arc<dyn LocalServerApiClient> {
        self.server_api.clone()
    }

    /// 兼容仍未迁出的本地 transport 调用点。新增代码应优先使用窄接口。
    pub fn get(&self) -> Arc<ServerApi> {
        self.server_api.clone()
    }

    // OpenWarp Wave 3-1:`get_auth_client()` 随 `AuthClient` trait 一同物理删,
    // 所有外部原调用方改为本地 stub (返回 `AuthToken::NoAuth` / `Ok(())`)。
    // OpenWarp Wave 6-8:`get_referrals_client()` / `get_block_client()` 随对应
    // trait 与设置页 UI 一同物理删。C5 再删通用 `get()`:
    // `agent_sdk` 仅能按用途获取 `ai_client` / `harness_support` / `local_client`
    // / `agent_event_stream` 四个最小切面。

    pub fn get_ai_client(&self) -> Arc<dyn AIClient> {
        self.ai_client.clone()
    }

    pub fn get_harness_support_client(&self) -> Arc<dyn harness_support::HarnessSupportClient> {
        self.server_api.clone()
    }

    pub fn get_agent_event_stream_client(&self) -> Arc<dyn AgentEventStreamClient> {
        self.server_api.clone()
    }
}

impl Entity for ServerApiProvider {
    type Event = ServerApiEvent;
}

impl SingletonEntity for ServerApiProvider {}
