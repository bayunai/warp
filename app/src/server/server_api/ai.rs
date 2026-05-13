// OpenWarp(本地化,Wave 2-2):`AIClient` trait 的云端方法已本地化为 stub。
// 历史职责:通过 warp.dev 后端的 GraphQL/HTTP RPC 完成 AI 对话、命令生成、
// ambient agent 远程调度、conversation 同步、artifact 上传/下载、orchestration v2 消息等。
// BYOP(Bring-Your-Own-Provider)链路完全不经过 `AIClient` trait —— 走
// `genai::Client::exec_chat_stream`,所以这里全部直接返回 Err。
// trait 签名保留(Wave 3 再决定是否物理删 trait),impl 一律 stub 报错。
// 调用方都用 `?` 传播 Err / log::warn / fallback / 静默 placeholder,无 panic 风险。

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
#[cfg(test)]
use mockall::automock;
use warp_core::report_error;

use crate::ai::ambient_agents::AmbientAgentTaskId;
use crate::ai::generate_code_review_content::api::{
    GenerateCodeReviewContentRequest, GenerateCodeReviewContentResponse,
};
use crate::{
    ai::llms::{
        AvailableLLMs, DisableReason, LLMContextWindow, LLMInfo, LLMProvider, LLMSpec,
        LLMUsageMetadata, ModelsByFeature, RoutingHostConfig,
    },
    ai_assistant::{
        execution_context::WarpAiExecutionContext, requests::GenerateDialogueResult,
        utils::TranscriptPart, AIGeneratedCommand, GenerateCommandsFromNaturalLanguageError,
    },
};
use warp_graphql::ai::{AgentTaskState, PlatformErrorCode};

// Re-export ambient agent types for backwards compatibility
pub use crate::ai::ambient_agents::{
    task::AttachmentInput, AgentConfigSnapshot, AgentSource, AmbientAgentTask,
    AmbientAgentTaskState, TaskStatusMessage,
};

/// A status update for a task, optionally including a platform error code.
pub struct TaskStatusUpdate {
    pub message: String,
    pub error_code: Option<PlatformErrorCode>,
}

impl TaskStatusUpdate {
    /// Create a status update with just a message (no error code).
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: None,
        }
    }

    /// Create a status update with a message and error code.
    pub fn with_error_code(message: impl Into<String>, error_code: PlatformErrorCode) -> Self {
        Self {
            message: message.into(),
            error_code: Some(error_code),
        }
    }
}

/// JSON payload sent to the public `POST /agent/run` API.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SpawnAgentRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<AgentConfigSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<bool>,
    /// Use a Claude-compatible skill as the base prompt.
    /// Format: "repo:skill_name" or just "skill_name".
    /// The skill is resolved at runtime in the agent environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactive: Option<bool>,
    /// Populated when a cloud agent spawns a child run via the public API.
    /// Not yet wired through the local start_agent flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    /// Base64-encoded `warp.multi_agent.v1.Skill` payloads to restore as runtime skills.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub runtime_skills: Vec<String>,
    /// Base64-encoded `warp.multi_agent.v1.Attachment` payloads to restore as referenced attachments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referenced_attachments: Vec<String>,
}

// --- Orchestrations V2 messaging types ---

#[derive(Debug, Clone, serde::Serialize)]
pub struct SendAgentMessageRequest {
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub sender_run_id: String,
}

#[derive(Debug, Clone)]
pub struct ListAgentMessagesRequest {
    pub unread_only: bool,
    pub since: Option<String>,
    pub limit: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SendAgentMessageResponse {
    pub message_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentMessageHeader {
    pub message_id: String,
    pub sender_run_id: String,
    pub subject: String,
    pub sent_at: String,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentRunEvent {
    pub event_type: String,
    pub run_id: String,
    pub ref_id: Option<String>,
    pub execution_id: Option<String>,
    pub occurred_at: String,
    pub sequence: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadAgentMessageResponse {
    pub message_id: String,
    pub sender_run_id: String,
    pub subject: String,
    pub body: String,
    pub sent_at: String,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SpawnAgentResponse {
    pub task_id: AmbientAgentTaskId,
    pub run_id: String,
    #[serde(default)]
    pub at_capacity: bool,
}

/// Filter parameters for listing ambient agent tasks.
#[derive(Clone, Debug, Default)]
pub struct TaskListFilter {
    pub creator_uid: Option<String>,
    pub updated_after: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub states: Option<Vec<AmbientAgentTaskState>>,
    pub source: Option<AgentSource>,
    pub execution_location: Option<ExecutionLocation>,
    pub environment_id: Option<String>,
    pub skill_spec: Option<String>,
    pub schedule_id: Option<String>,
    pub ancestor_run_id: Option<String>,
    pub config_name: Option<String>,
    pub model_id: Option<String>,
    pub artifact_type: Option<ArtifactType>,
    pub search_query: Option<String>,
    pub sort_by: Option<RunSortBy>,
    pub sort_order: Option<RunSortOrder>,
    pub cursor: Option<String>,
}

/// Execution location filter values accepted by the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionLocation {
    Local,
    Remote,
}

impl ExecutionLocation {
    pub fn as_query_param(&self) -> &'static str {
        match self {
            ExecutionLocation::Local => "LOCAL",
            ExecutionLocation::Remote => "REMOTE",
        }
    }
}

/// Artifact type filter values accepted by the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactType {
    Plan,
    PullRequest,
    Screenshot,
    File,
}

impl ArtifactType {
    pub fn as_query_param(&self) -> &'static str {
        match self {
            ArtifactType::Plan => "PLAN",
            ArtifactType::PullRequest => "PULL_REQUEST",
            ArtifactType::Screenshot => "SCREENSHOT",
            ArtifactType::File => "FILE",
        }
    }
}

/// Sort-by values accepted by the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunSortBy {
    UpdatedAt,
    CreatedAt,
    Title,
    Agent,
}

impl RunSortBy {
    pub fn as_query_param(&self) -> &'static str {
        match self {
            RunSortBy::UpdatedAt => "updated_at",
            RunSortBy::CreatedAt => "created_at",
            RunSortBy::Title => "title",
            RunSortBy::Agent => "agent",
        }
    }
}

/// Sort-order values accepted by the public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunSortOrder {
    Asc,
    Desc,
}

impl RunSortOrder {
    pub fn as_query_param(&self) -> &'static str {
        match self {
            RunSortOrder::Asc => "asc",
            RunSortOrder::Desc => "desc",
        }
    }
}

/// Build the path + query string for `GET /api/v1/agent/runs` from a filter.
pub(crate) fn build_list_agent_runs_url(limit: i32, filter: &TaskListFilter) -> String {
    let mut url = format!("agent/runs?limit={limit}");

    let mut push = |key: &str, value: &str| {
        url.push('&');
        url.push_str(key);
        url.push('=');
        url.push_str(urlencoding::encode(value).as_ref());
    };

    if let Some(creator_uid) = filter.creator_uid.as_deref() {
        push("creator", creator_uid);
    }
    if let Some(updated_after) = filter.updated_after {
        push("updated_after", &updated_after.to_rfc3339());
    }
    if let Some(created_after) = filter.created_after {
        push("created_after", &created_after.to_rfc3339());
    }
    if let Some(created_before) = filter.created_before {
        push("created_before", &created_before.to_rfc3339());
    }
    if let Some(states) = filter.states.as_ref() {
        for state in states {
            if let Some(value) = state.as_query_param() {
                push("state", value);
            }
        }
    }
    if let Some(source) = filter.source.as_ref() {
        push("source", source.as_str());
    }
    if let Some(execution_location) = filter.execution_location {
        push("execution_location", execution_location.as_query_param());
    }
    if let Some(environment_id) = filter.environment_id.as_deref() {
        push("environment_id", environment_id);
    }
    if let Some(skill_spec) = filter.skill_spec.as_deref() {
        push("skill_spec", skill_spec);
    }
    if let Some(schedule_id) = filter.schedule_id.as_deref() {
        push("schedule_id", schedule_id);
    }
    if let Some(ancestor_run_id) = filter.ancestor_run_id.as_deref() {
        push("ancestor_run_id", ancestor_run_id);
    }
    if let Some(config_name) = filter.config_name.as_deref() {
        push("name", config_name);
    }
    if let Some(model_id) = filter.model_id.as_deref() {
        push("model_id", model_id);
    }
    if let Some(artifact_type) = filter.artifact_type {
        push("artifact_type", artifact_type.as_query_param());
    }
    if let Some(search_query) = filter.search_query.as_deref() {
        push("q", search_query);
    }
    if let Some(sort_by) = filter.sort_by {
        push("sort_by", sort_by.as_query_param());
    }
    if let Some(sort_order) = filter.sort_order {
        push("sort_order", sort_order.as_query_param());
    }
    if let Some(cursor) = filter.cursor.as_deref() {
        push("cursor", cursor);
    }

    url
}

struct ListRunsResponse {
    runs: Vec<AmbientAgentTask>,
}

impl<'de> serde::Deserialize<'de> for ListRunsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RawResponse {
            runs: Vec<serde_json::Value>,
        }

        let raw = RawResponse::deserialize(deserializer)?;
        let mut runs = Vec::with_capacity(raw.runs.len());

        for task_value in raw.runs.into_iter() {
            match serde_json::from_value::<AmbientAgentTask>(task_value) {
                Ok(task) => runs.push(task),
                Err(e) => {
                    // Log the error and skip this task instead of failing the entire request
                    report_error!(anyhow!("Failed to deserialize ambient agent task: {}", e));
                }
            }
        }

        Ok(ListRunsResponse { runs })
    }
}

/// Source information for an agent skill.
#[derive(Clone, serde::Deserialize, Debug, PartialEq)]
pub struct AgentListSource {
    pub owner: String,
    pub name: String,
    pub skill_path: String,
}

/// Environment information for an agent skill.
#[derive(Clone, serde::Deserialize, Debug, PartialEq)]
pub struct AgentListEnvironment {
    pub uid: String,
    pub name: String,
}

/// A variant of an agent skill.
#[derive(Clone, serde::Deserialize, Debug, PartialEq)]
pub struct AgentListVariant {
    pub id: String,
    pub description: String,
    pub base_prompt: String,
    pub source: AgentListSource,
    pub environments: Vec<AgentListEnvironment>,
}

/// An agent skill item with its variants.
#[derive(Clone, serde::Deserialize, Debug, PartialEq)]
pub struct AgentListItem {
    pub name: String,
    pub variants: Vec<AgentListVariant>,
}

#[derive(serde::Deserialize)]
struct ListAgentsResponse {
    agents: Vec<AgentListItem>,
}

fn disabled_ai_client_method(method_name: &str) -> anyhow::Error {
    anyhow!("AI client `{method_name}` is disabled in OpenWarp")
}

#[derive(Default)]
pub struct LocalAIClient;

impl LocalAIClient {
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(test, automock)]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
pub trait AIClient: 'static + Send + Sync {
    async fn generate_commands_from_natural_language(
        &self,
        prompt: String,
        ai_execution_context: Option<WarpAiExecutionContext>,
    ) -> Result<Vec<AIGeneratedCommand>, GenerateCommandsFromNaturalLanguageError>;

    async fn generate_dialogue_answer(
        &self,
        transcript: Vec<TranscriptPart>,
        prompt: String,
        ai_execution_context: Option<WarpAiExecutionContext>,
    ) -> anyhow::Result<GenerateDialogueResult>;

    async fn update_agent_task(
        &self,
        task_id: AmbientAgentTaskId,
        task_state: Option<AgentTaskState>,
        session_id: Option<session_sharing_protocol::common::SessionId>,
        conversation_id: Option<String>,
        status_message: Option<TaskStatusUpdate>,
    ) -> anyhow::Result<(), anyhow::Error>;

    async fn spawn_agent(
        &self,
        request: SpawnAgentRequest,
    ) -> anyhow::Result<SpawnAgentResponse, anyhow::Error>;

    async fn list_ambient_agent_tasks(
        &self,
        limit: i32,
        filter: TaskListFilter,
    ) -> anyhow::Result<Vec<AmbientAgentTask>, anyhow::Error>;

    /// List agent runs and return the raw server JSON response.
    async fn list_agent_runs_raw(
        &self,
        limit: i32,
        filter: TaskListFilter,
    ) -> anyhow::Result<serde_json::Value, anyhow::Error>;

    async fn get_ambient_agent_task(
        &self,
        task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<AmbientAgentTask, anyhow::Error>;

    /// Fetch a single agent run and return the raw server JSON response.
    async fn get_agent_run_raw(
        &self,
        task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<serde_json::Value, anyhow::Error>;

    async fn list_agents(
        &self,
        repo: Option<String>,
    ) -> anyhow::Result<Vec<AgentListItem>, anyhow::Error>;

    async fn cancel_ambient_agent_task(
        &self,
        task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<(), anyhow::Error>;

    // --- Orchestrations V2 messaging ---

    async fn send_agent_message(
        &self,
        request: SendAgentMessageRequest,
    ) -> anyhow::Result<SendAgentMessageResponse, anyhow::Error>;

    async fn list_agent_messages(
        &self,
        run_id: &str,
        request: ListAgentMessagesRequest,
    ) -> anyhow::Result<Vec<AgentMessageHeader>, anyhow::Error>;

    /// Persists the latest observed event sequence number for a run on the
    /// server. Used to keep the server-side cursor in sync with the client so
    /// that driver/cloud restores can resume without replaying events the
    /// parent has already acted on.
    async fn update_event_sequence_on_server(
        &self,
        run_id: &str,
        sequence: i64,
    ) -> anyhow::Result<(), anyhow::Error>;

    async fn mark_message_delivered(&self, message_id: &str) -> anyhow::Result<(), anyhow::Error>;

    async fn read_agent_message(
        &self,
        message_id: &str,
    ) -> anyhow::Result<ReadAgentMessageResponse, anyhow::Error>;

    /// Generates AI copy for code-review flows: commit messages at dialog-open
    /// time and PR titles / bodies at confirm time. `output_type` in the
    /// request picks which of the three the server returns.
    async fn generate_code_review_content(
        &self,
        request: GenerateCodeReviewContentRequest,
    ) -> Result<GenerateCodeReviewContentResponse, anyhow::Error>;
}

// OpenWarp:`AIClient` 由本地 `LocalAIClient` 承担。
// 调用方都用 `?` 传播 Err / log::warn / fallback,UI 拿到 Err 后只 toast 错误,不会 panic。
// 其中真正依赖云端数据/副作用的方法继续返回 Err;仅承担"回执/同步"语义的方法改为 no-op
// `Ok(())`,避免 OpenWarp 本地链路在后台持续打印无意义告警。
#[cfg_attr(not(target_family = "wasm"), async_trait)]
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
impl AIClient for LocalAIClient {
    async fn generate_commands_from_natural_language(
        &self,
        _prompt: String,
        _ai_execution_context: Option<WarpAiExecutionContext>,
    ) -> Result<Vec<AIGeneratedCommand>, GenerateCommandsFromNaturalLanguageError> {
        // OpenWarp:Warp AI 命令面板"自然语言 → 命令"已下线(BYOP 不走此 trait)。
        Err(GenerateCommandsFromNaturalLanguageError::Other)
    }

    async fn generate_dialogue_answer(
        &self,
        _transcript: Vec<TranscriptPart>,
        _prompt: String,
        _ai_execution_context: Option<WarpAiExecutionContext>,
    ) -> anyhow::Result<GenerateDialogueResult> {
        Err(disabled_ai_client_method("generate_dialogue_answer"))
    }

    async fn update_agent_task(
        &self,
        _task_id: AmbientAgentTaskId,
        _task_state: Option<AgentTaskState>,
        _session_id: Option<session_sharing_protocol::common::SessionId>,
        _conversation_id: Option<String>,
        _status_message: Option<TaskStatusUpdate>,
    ) -> anyhow::Result<(), anyhow::Error> {
        Ok(())
    }

    async fn spawn_agent(
        &self,
        _request: SpawnAgentRequest,
    ) -> anyhow::Result<SpawnAgentResponse, anyhow::Error> {
        Err(disabled_ai_client_method("spawn_agent"))
    }

    async fn list_ambient_agent_tasks(
        &self,
        _limit: i32,
        _filter: TaskListFilter,
    ) -> anyhow::Result<Vec<AmbientAgentTask>, anyhow::Error> {
        Err(disabled_ai_client_method("list_ambient_agent_tasks"))
    }

    async fn list_agent_runs_raw(
        &self,
        _limit: i32,
        _filter: TaskListFilter,
    ) -> anyhow::Result<serde_json::Value, anyhow::Error> {
        Err(disabled_ai_client_method("list_agent_runs_raw"))
    }

    async fn get_ambient_agent_task(
        &self,
        _task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<AmbientAgentTask, anyhow::Error> {
        Err(disabled_ai_client_method("get_ambient_agent_task"))
    }

    async fn get_agent_run_raw(
        &self,
        _task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<serde_json::Value, anyhow::Error> {
        Err(disabled_ai_client_method("get_agent_run_raw"))
    }

    async fn list_agents(
        &self,
        _repo: Option<String>,
    ) -> anyhow::Result<Vec<AgentListItem>, anyhow::Error> {
        Err(disabled_ai_client_method("list_agents"))
    }

    async fn cancel_ambient_agent_task(
        &self,
        _task_id: &AmbientAgentTaskId,
    ) -> anyhow::Result<(), anyhow::Error> {
        Err(disabled_ai_client_method("cancel_ambient_agent_task"))
    }

    async fn send_agent_message(
        &self,
        _request: SendAgentMessageRequest,
    ) -> anyhow::Result<SendAgentMessageResponse, anyhow::Error> {
        Err(disabled_ai_client_method("send_agent_message"))
    }

    async fn list_agent_messages(
        &self,
        _run_id: &str,
        _request: ListAgentMessagesRequest,
    ) -> anyhow::Result<Vec<AgentMessageHeader>, anyhow::Error> {
        Err(disabled_ai_client_method("list_agent_messages"))
    }

    async fn update_event_sequence_on_server(
        &self,
        _run_id: &str,
        _sequence: i64,
    ) -> anyhow::Result<(), anyhow::Error> {
        Ok(())
    }

    async fn mark_message_delivered(&self, _message_id: &str) -> anyhow::Result<(), anyhow::Error> {
        Ok(())
    }

    async fn read_agent_message(
        &self,
        _message_id: &str,
    ) -> anyhow::Result<ReadAgentMessageResponse, anyhow::Error> {
        Err(disabled_ai_client_method("read_agent_message"))
    }

    async fn generate_code_review_content(
        &self,
        _request: GenerateCodeReviewContentRequest,
    ) -> Result<GenerateCodeReviewContentResponse, anyhow::Error> {
        Err(disabled_ai_client_method("generate_code_review_content"))
    }
}

// ---------------------------------------------------------------------------
// OpenWarp:`workspace::*` 系列的 GraphQL → 本地 LLM 类型转换保留。
//
// 这条链 **被 `super::auth::AuthClient` 处理 user_properties 时直接消费**:
//   `user_properties.llms.try_into() -> ModelsByFeature`
//
// 触达深度:`FeatureModelChoice` → `AvailableLlms` × 4 (agent_mode/coding/cli_agent/computer_use)
//   → `LlmInfo` (多个 model) → `LlmProvider` / `LlmSpec` / `LlmUsageMetadata`
//   / `DisableReason` / `RoutingHostConfig` / `LlmModelHost`
//
// 此链不属 AIClient,所以 Wave 2-2 不动。Wave 3 处理 auth.rs 时一并裁掉。
// 与之并存的 `queries::get_feature_model_choices::*` 一族(独立 RootQuery)已删除,
// 因为它仅被 AIClient::get_feature_model_choices / get_free_available_models 调用。
// ---------------------------------------------------------------------------

impl TryFrom<warp_graphql::workspace::FeatureModelChoice> for ModelsByFeature {
    type Error = anyhow::Error;

    fn try_from(value: warp_graphql::workspace::FeatureModelChoice) -> Result<Self, Self::Error> {
        Ok(Self {
            agent_mode: value.agent_mode.try_into()?,
            coding: value.coding.try_into()?,
            cli_agent: Some(value.cli_agent.try_into()?),
            computer_use: Some(value.computer_use_agent.try_into()?),
        })
    }
}

impl TryFrom<warp_graphql::workspace::AvailableLlms> for AvailableLLMs {
    type Error = anyhow::Error;

    fn try_from(value: warp_graphql::workspace::AvailableLlms) -> Result<Self, Self::Error> {
        Self::new(
            value.default_id.into(),
            value.choices.into_iter().map(LLMInfo::from),
            value.preferred_codex_model_id.map(Into::into),
        )
    }
}

impl From<warp_graphql::workspace::LlmInfo> for LLMInfo {
    fn from(value: warp_graphql::workspace::LlmInfo) -> Self {
        let host_configs = {
            let mut map = std::collections::HashMap::new();
            for config in value.host_configs {
                let config: RoutingHostConfig = config.into();
                let host = config.model_routing_host.clone();
                if map.insert(host.clone(), config).is_some() {
                    log::warn!("Duplicate LlmModelHost entry for {host:?}, using latest value");
                }
            }
            map
        };
        Self {
            id: value.id.into(),
            display_name: value.display_name,
            base_model_name: value.base_model_name,
            reasoning_level: value.reasoning_level,
            usage_metadata: value.usage_metadata.into(),
            description: value.description,
            disable_reason: value.disable_reason.map(DisableReason::from),
            vision_supported: value.vision_supported,
            spec: value.spec.map(Into::into),
            provider: value.provider.into(),
            host_configs,
            discount_percentage: value.pricing.discount_percentage.map(|v| v as f32),
            context_window: LLMContextWindow {
                is_configurable: value.context_window.is_configurable,
                min: value.context_window.min.into(),
                max: value.context_window.max.into(),
                default_max: value.context_window.default.into(),
            },
        }
    }
}

impl From<warp_graphql::workspace::RoutingHostConfig> for RoutingHostConfig {
    fn from(value: warp_graphql::workspace::RoutingHostConfig) -> Self {
        Self {
            enabled: value.enabled,
            model_routing_host: value.model_routing_host.into(),
        }
    }
}

// OpenWarp:`From<warp_graphql::workspace::LlmModelHost> for LLMModelHost` 已由
// `app/src/workspaces/gql_convert.rs` 提供,这里不重复。

impl From<warp_graphql::workspace::LlmProvider> for LLMProvider {
    fn from(value: warp_graphql::workspace::LlmProvider) -> Self {
        match value {
            warp_graphql::workspace::LlmProvider::Openai => LLMProvider::OpenAI,
            warp_graphql::workspace::LlmProvider::Anthropic => LLMProvider::Anthropic,
            warp_graphql::workspace::LlmProvider::Google => LLMProvider::Google,
            warp_graphql::workspace::LlmProvider::Xai => LLMProvider::Xai,
            warp_graphql::workspace::LlmProvider::Unknown => LLMProvider::Unknown,
            warp_graphql::workspace::LlmProvider::Other(value) => {
                report_error!(anyhow!(
                    "Invalid LlmProvider '{value}'. Make sure to update client GraphQL types!"
                ));
                LLMProvider::Unknown
            }
        }
    }
}

impl From<warp_graphql::workspace::LlmSpec> for LLMSpec {
    fn from(value: warp_graphql::workspace::LlmSpec) -> Self {
        Self {
            cost: value.cost as f32,
            quality: value.quality as f32,
            speed: value.speed as f32,
        }
    }
}

impl From<warp_graphql::workspace::LlmUsageMetadata> for LLMUsageMetadata {
    fn from(value: warp_graphql::workspace::LlmUsageMetadata) -> Self {
        Self {
            request_multiplier: value.request_multiplier.max(1) as usize,
            credit_multiplier: value.credit_multiplier.map(|v| v as f32),
        }
    }
}

impl From<warp_graphql::workspace::DisableReason> for DisableReason {
    fn from(value: warp_graphql::workspace::DisableReason) -> Self {
        match value {
            warp_graphql::workspace::DisableReason::AdminDisabled => DisableReason::AdminDisabled,
            warp_graphql::workspace::DisableReason::OutOfRequests => DisableReason::OutOfRequests,
            warp_graphql::workspace::DisableReason::ProviderOutage => DisableReason::ProviderOutage,
            warp_graphql::workspace::DisableReason::RequiresUpgrade => {
                DisableReason::RequiresUpgrade
            }
            warp_graphql::workspace::DisableReason::Other(_) => DisableReason::Unavailable,
        }
    }
}
