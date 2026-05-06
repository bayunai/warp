## User (checkpoint)

开始实现压缩功能:[@tmp.md (1:252)](file:///C:/Users/zero/Desktop/code/zerx-lab/warp/docs/tmp.md#L1:252)

**Tool Call: Run MCP tool `mem_context`**
Status: Completed

## Memory from Previous Sessions

### Recent Sessions
- **warp** (2026-04-29 06:42:56) [309 observations]
- **warp** (2026-05-05 10:24:16) [3 observations]
- **warp** (2026-05-05 10:51:28) [0 observations]
- **warp** (2026-05-05 10:25:24) [1 observations]
- **warp** (2026-05-05 10:06:41) [0 observations]

### Recent User Prompts
- 2026-05-05 11:05:06: 用户要求去掉 `init_project` tool capability，并且将“旧 LSP 初始化向导路线可以去掉或停用”相关功能函数全部去掉。
- 2026-05-05 10:51:28: 目前 /init 已经不走原先的初始化路径了；检查当前内置 /init 路线，确认原先需要 LSP 的 /init 能力是否可以去掉。
- 2026-05-05 10:49:34: /init回车无法直接选择输入,/plan可以直接回车执行
- 2026-05-05 10:25:24: 设置中的 LSP 管理应该显示“卸载”和“查看日志”按钮，而不是当前的开关；并且在非项目目录下启动不应闪退。
- 2026-05-05 10:24:16: - 索引与项目  这个菜单项修改为LSP管理
- agent命令增加/init 参考"C:\Users\zero\Desktop\code\github\opencode"的实现方式
- 2026-05-05 10:06:41: - 设置页名字需要改下,应该就是 lsp 管理,并且不需要按照项目进行区分,lsp 是全局的,任何项目都可以用
- /init 命令被删掉了,应该保留,参考 "C:\Users\zero\Desktop\code\github\opencode" 下的 /init 命令进行实现
- 2026-05-05 08:35:50: 查看记忆,继续实现 mem_search Phase 4

### Recent Observations
- [discovery] **Plan tool 保存/持久化機制调研**: **What**: 完整调研 Plan tool 的保存/持久化机制,明确红框按钮、存储路径、与 Notebook 差异。

**核心链路**:
1. AI Agent 执行 `/plan` → `CreateDocuments` tool → `CreateDocumentsExecutor::execute` → `AIDocumentModel::create_document` → 文档存入内存 + `Artifact::Plan{notebook_uid: None}` 加入对话
2. Plan 内容 **自动** 写入本地 SQLite(`ai_document_panes` 表)...
- [architecture] **移除 init_project tool 和旧 LSP 初始化向导**: **What**: 移除 `init_project` tool/capability 与旧 `/init` 初始化向导路线。BYOP 工具注册表不再暴露 `init_project`; 服务端 supported tools 不再声明 `ToolType::InitProject`; 如果旧会话/旧服务端消息仍带 `Tool::InitProject`,客户端转为 `NoClientRepresentation`。同时删除 `app/src/terminal/view/init_project/` 整个旧向导目录,移除 `TerminalAction::InitProject`、旧 work...
- [bugfix] **修复 /init 回车选择行为**: **What**: 修正 `/init` slash command 的选择/回车行为:移除 `Argument::optional().with_execute_on_selection()`,改为 `Argument::optional()`。
**Why**: 用户反馈 `/init` 不能像 `/plan` 一样回车选择后直接进入可执行输入;根因是 `should_execute_on_selection=true` 会在菜单选择时走 `execute_slash_command`,而内置 agent 命令应像 `/plan` 一样选择时插入 `/init ` 前缀,再由输入链路发送。...
- [bugfix] **LSP 管理改卸载和日志按钮**: **What**: 将设置页的 LSP 管理行从全局开关改为“查看日志”+“卸载”操作；查看日志在无项目/未运行时打开该 server 的日志目录，卸载会关闭所有已启动实例、从全局 enabled_lsp_servers 移除，并删除 Warp 托管安装目录后刷新安装状态。
**Why**: 用户反馈设置中的 LSP 不应显示开关，应该是卸载和查看日志按钮；并且非项目目录启动/打开设置不应因为空 workspace 路径触发异常。
**Where**: `app/src/settings_view/code_page.rs`, `app/src/code/lsp_logs.rs`, `app/...
- [architecture] **/init 内置命令模板化实现**: **What**: 将 `/init` 调整为类似 `/plan` 的内置 agent slash command: slash command 菜单选择后不再触发旧 `TerminalAction::InitProject`,而是保留 `/init` 前缀进入 AI 输入链路;`SlashCommandRequest::from_query` 识别 `/init` 和 `/init <args>` 后渲染命令模板并作为 `UserQuery` 发送。
**Why**: 用户纠正 `/init` 应是内置 agent 命令,且 prompt 必须写在 j2 模板文件中,不能硬编码在 Rust ...
- [pattern] **/init prompt 使用 j2 模板**: **What**: 用户明确纠正 `/init` 的 prompt 不应写死在 Rust 代码中,需要放到 j2 提示词模板文件里。
**Why**: 保持 agent 命令提示词与项目现有 prompt 模板体系一致,避免硬编码长提示词。
**Where**: 待调整 `app/src/ai/blocklist/controller/slash_command.rs` 及 `app/src/ai/agent_providers/prompts/` 下模板文件。
**Learned**: 后续新增/修改 agent 命令 prompt 时优先使用 j2 模板文件,不要把长 prompt 直接写...
- [manual] **提交 Remove codebase indexing runtime**: **What**: 将当前工作区改动提交为 Git commit `14ab30b`，提交信息为 `Remove codebase indexing runtime`。
**Why**: 用户要求“提交commit”。
**Where**: 仓库根目录 `warp`；提交包含 64 个文件变更，主要是删除 `crates/ai/src/index/full_source_code_embedding/` 下 codebase indexing 运行时文件，并更新 app 侧相关引用与 Linux 构建依赖。
**Learned**: 提交前执行了 `git diff --check`，仅出现 ...
- [architecture] **Phase 4 codebase indexing runtime peel**: **What**: 继续 Phase 4 剥离 codebase indexing 运行路径,并保持主二进制可编译。主要改动:删除 `CodeSettings` 里的 codebase context/auto-index setting;移除 app 注册 `CodebaseIndexManager` 和 `SyncQueue<SyncTask>`;`GetRelevantFilesController` 改为只走 outline/BYOP relevant files;`PersistedWorkspace` 不再订阅/驱动 CodebaseIndexManager;`RepoOutlin...
- [config] **添加 Linux libdbus 构建依赖**: **What**: 将 `libdbus-1-dev` 加入 Linux 通用构建依赖脚本。
**Why**: PR Check 的 `cargo check --bin warp-oss --locked` 在 GitHub Actions run 25366029739/job 74381844196 中因 `libdbus-sys` 找不到 `dbus-1.pc` 失败；release workflow 之前单独安装了该包,但 PR Check 走 `script/linux/install_build_deps` 时未安装。
**Where**: `script/linux/insta...
- [session_summary] **Session summary: warp**: ## Goal
设计 + 落实 openWarp"删 codebase indexing + LSP 启用全局化"重构,5 phase 计划。

## Instructions
- 用户偏好:**LSP 启用应是全局开关,不要按 workspace 区分**(详见 `feedback_lsp_global_not_per_workspace.md`)。装好 + 全局开 → 任何项目检测到对应语言自动启动。
- Phase 4 路线选项里用户明确选"彻底删模块"(不要 stub)+"硬切不迁移"(LSP per-workspace → global 不写 migration)。
- /init ...
- [architecture] **openWarp 删 codebase indexing + LSP 全局化 实施进度**: **What**: openWarp 删代码库索引 + LSP per-workspace → 全局化 重构,5 phase 计划。

**Why**: 用户要求"代码库索引不需要,LSP 装好后任何 agent/项目都能用,只是装没装"。/init 命令也需要清掉 CodebaseContext step + /index 命令。

**Where**: 计划见 task #1-#5。
- Phase 1 ✅ `app/src/settings/code.rs` 加 `enabled_lsp_servers: Vec<LSPServerType>` 全局 setting,`crates/ls...
- [architecture] **openWarp BYOP 真复刻 /plan Plan Mode(per-turn)**: **What**: 把 openWarp 之前在 BYOP 路径下完全无效的 `/plan` 命令真正复刻为只读 Plan Mode,语义对齐 Claude Code EnterPlanMode + opencode plan mode。

**改动文件**:
1. `app/src/ai/agent_providers/prompts/partials/plan_mode.j2` 新建 — `{% if plan_mode %}` 守卫的只读模式 prompt:Investigate(只读工具)→ Plan(markdown 结构化)→ Stop and wait;明确禁用工具列表;鼓励用 ...
- [bugfix] **openWarp BYOP 压缩摘要路径只叠加不裁剪 — 完整链路确认 bug**: **What**: 完整读完 BYOP 压缩链路(chat_stream 主循环+输入循环、algorithm::select、commit_summarization、tests.rs)后确认:摘要路径只在历史末尾"叠加"摘要,不会"替代"head 区,下一轮普通请求体反而更长。prune 路径(tool output 占位)是正确的。

**Why**: 用户反复确认要完整代码链路调研后再下结论。

**Where**:
- `app/src/ai/byop_compaction/state.rs:137-142` — hidden_message_ids 只返回 (user_msg_id...
- [discovery] **openWarp BYOP /plan 命令在 BYOP 路径下完全无效**: **What**: 调研 openWarp `/plan <prompt>` 在 BYOP 模式下的行为 —— 结论:**完全无效**,只是把 `/plan ` 前 6 字符吞掉,模型行为与裸输入一致。

**数据流**:
1. `app/src/ai/blocklist/controller.rs:645-653` strip `/plan ` 前缀,设 `UserQueryMode::Plan`
2. `app/src/ai/agent/api/convert_to.rs:485-494` 写进 proto `warp_multi_agent_api::UserQueryMode::Pla...
- [discovery] **openWarp BYOP 压缩 hidden_message_ids 疑似缺口**: **What**: 调研 openWarp BYOP 本地会话压缩(`app/src/ai/byop_compaction/`)是否真正清理上下文,发现 `state.rs::hidden_message_ids` 只返回 (user_msg_id, assistant_msg_id) 一对,而 `commit_summarization` 选的 user_msg_id 仅是「触发 /compact 时最近一条真实 UserQuery」。

**Why**: 用户问"压缩会话是否会正确清理上下文"。

**Where**:
- `app/src/ai/byop_compaction/state...
- [architecture] **openWarp 对话持久化第三轮 - 全语义边界 flush**: **What**: 在 openWarp BYOP 写盘链路完成"对齐 opencode 语义边界写盘"的最后一轮:补全所有应该 flush 的 Action 分支 + 异常路径归零计数。

**新增的 part-边界 flush 点(`app/src/ai/agent/conversation.rs`)**:
- `Action::CommitTransaction` 末尾 — transaction 提交是一组语义相关变更落定的边界
- `Action::AddMessagesToTask` 末尾 — 完整 message 落定(text/reasoning/tool_call/tool_...
- [architecture] **openWarp 对话流式持久化优化 - 借鉴 opencode part 边界**: **What**: 在 openWarp BYOP 写盘链路加入两层早写,缓解 turn 中途强退导致整轮丢失。

**A. 早写 user query**: `app/src/ai/agent/conversation.rs::update_for_new_request_input` 末尾加 `self.write_updated_conversation_state(ctx)` —— turn 一启动就把 user query 落盘。即便 stream 1 chunk 没收到就强退,/conversations 也能看见这次提问。

**B. 流式节流 flush**: 在 `AICon...
- [decision] **openWarp auto-approve 不再吞 ask_user_question**: **What**: openWarp 改 ctrl+shift+i (auto-approve) 行为 — 仅对 shell/编辑等执行类工具自动通过,`ask_user_question` 永远弹给用户,不再静默 `SkippedByAutoApprove`。

**Why**: 用户反馈 "auto-execute 后 AI 询问问题被自动跳过,这是不对的"。上游 Warp 默认 `AskExceptInAutoApprove` 在 `autoexecute_any_action()` 时直接返回 `SkippedByAutoApprove`,模型收到"已跳过"反馈,根本不弹给用户。

*...
- [bugfix] **openWarp 历史不落盘真正根因 - restore_session 同时控制持久化**: **What**: openWarp(及上游 Warp)把"启动恢复会话"和"对话历史落盘"耦合到同一个 `[general] restore_session` 开关。用户关掉自动恢复 → 顺手把所有 turn finish 后的 SQLite upsert 也跳过 → 历史完全不写。

**关键位置**: `app/src/ai/agent/conversation.rs:2780-2784`
```rust
pub(crate) fn write_updated_conversation_state(...) {
    if self.is_viewing_shared_session ...
- [bugfix] **openWarp BYOP 持久化"丢失"真正根因 - stream 中途强退**: **What**: 修正之前结论 — openWarp BYOP 写盘机制本身没坏,**根因是用户每次在 stream 还在跑时就强退/Ctrl+C**。

**证据链**:
- 日志: `STATUS_CONTROL_C_EXIT (0xc000013a)` + 启动时 `SQLite error 283: recovered 9 frames from WAL` 证实强杀
- BYOP 发出请求后无任何 stream finish 日志 → turn 中途断
- `WARN convert_conversation: No result present for tool call ID: ...


---
Memory stats: 12 sessions, 355 observations across projects: warp, fluxdown, openwarp, openwarp byop, warp-agent-tools, warp-byop-research, .claude

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\state.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\commit.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\agent_providers\chat_stream.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\blocklist\controller.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\tests.rs

**Tool Call: Read file `app\src\ai\byop_compaction\state.rs` (lines 1-220)**
Status: Completed

```warp/app/src/ai/byop_compaction/state.rs
//! 压缩 sidecar 状态 — 挂在 `AIConversation` 上,与 warp `api::Message` 协议解耦。
//!
//! 因为 warp 的 `api::Message` 来自外部 protobuf 依赖 (`warp_multi_agent_api`),
//! 无法新增字段标记 `is_summary` / `compacted` 等;本 sidecar 用 message_id 索引
//! 把这些"压缩元数据"挂在 conversation 这一侧。
//!
//! 序列化版本号 [`CompactionState::VERSION`] 在 schema 演进时手动 bump,
//! 反序列化失败的旧 conversation 退化为 `Default`(等价"从未被压缩")。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// 触发压缩的来源。`Auto` 仅由 token-overflow 自动触发,`Manual` 是 /compact /compact-and。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompactionTrigger {
    Manual,
    Auto,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMarker {
    /// 这条 assistant message 是一份摘要,内容用于在请求拼装时替换前面的历史。
    #[serde(default)]
    pub is_summary: bool,
    /// 这条 user message 是一次 compaction 触发占位(opencode `parts.some(p => p.type === "compaction")`)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_trigger: Option<CompactionTrigger>,
    /// 这条 ToolCallResult 的 output 已被 prune,投影时替换为占位符。Unix epoch ms。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_output_compacted_at: Option<u64>,
    /// 自动续跑时合成的 user "Continue..." synthetic message 标记
    /// (对齐 opencode `metadata.compaction_continue`)。
    #[serde(default)]
    pub synthetic_continue: bool,
}

/// 一个已完成的压缩区间(对齐 opencode `completedCompactions()` 返回项)。
///
/// `user_msg_id` 是触发摘要的 user message(带 compaction_trigger marker),
/// `assistant_msg_id` 是合成的摘要 AgentOutput message。两者在 [`CompactionState::hidden_message_ids`]
/// 中视为已被覆盖,投影时跳过 — 但摘要文本本身会被取出代填到 head 区。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedCompaction {
    pub user_msg_id: String,
    pub assistant_msg_id: String,
    /// tail 起点 message id,用于 split 验证 / debug。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_start_id: Option<String>,
    /// 摘要内容(从 assistant message 直接取也可,但缓存到 state 方便 build_prompt 拿 previous_summary)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_text: Option<String>,
    pub auto: bool,
    pub overflow: bool,
}

/// 与 `AIConversation` 一同持久化的 sidecar 表。
///
/// 默认值 = 空表 = 未压缩状态,完全无侵入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionState {
    /// schema 版本,演进时 bump。
    #[serde(default = "CompactionState::current_version")]
    pub version: u32,
    #[serde(default)]
    markers: HashMap<String, MessageMarker>,
    #[serde(default)]
    completed: Vec<CompletedCompaction>,
}

impl Default for CompactionState {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            markers: HashMap::new(),
            completed: Vec::new(),
        }
    }
}

impl CompactionState {
    pub const VERSION: u32 = 1;
    fn current_version() -> u32 {
        Self::VERSION
    }

    pub fn marker(&self, msg_id: &str) -> Option<&MessageMarker> {
        self.markers.get(msg_id)
    }

    /// 写一个 marker(merge 到已有 marker 上,而不是覆盖整个 marker)。
    pub fn upsert_marker(&mut self, msg_id: impl Into<String>, f: impl FnOnce(&mut MessageMarker)) {
        let entry = self.markers.entry(msg_id.into()).or_default();
        f(entry);
    }

    /// 标记一条 ToolCallResult 的 output 已 prune。
    pub fn mark_tool_compacted(&mut self, msg_id: impl Into<String>, now_ms: u64) {
        self.upsert_marker(msg_id, |m| m.tool_output_compacted_at = Some(now_ms));
    }

    /// 推一次完成的压缩。
    pub fn push_completed(&mut self, c: CompletedCompaction) {
        // 同步把 user 与 assistant 各自打上 marker(便于投影时单独识别)。
        self.upsert_marker(c.user_msg_id.clone(), |m| {
            m.compaction_trigger = Some(if c.auto {
                CompactionTrigger::Auto
            } else {
                CompactionTrigger::Manual
            });
        });
        self.upsert_marker(c.assistant_msg_id.clone(), |m| m.is_summary = true);
        self.completed.push(c);
    }

    /// 标记一条 synthetic "Continue..." user message(auto+overflow 路径合成)。
    pub fn mark_synthetic_continue(&mut self, msg_id: impl Into<String>) {
        self.upsert_marker(msg_id, |m| m.synthetic_continue = true);
    }

    /// 取最后一次完成的压缩(用于 [`super::prompt::build_prompt`] 的增量摘要锚点)。
    pub fn previous_summary(&self) -> Option<&str> {
        self.completed
            .last()
            .and_then(|c| c.summary_text.as_deref())
    }

    pub fn completed(&self) -> &[CompletedCompaction] {
        &self.completed
    }

    /// 所有应在拼请求时跳过的 message id(对齐 opencode `hidden`):
    /// 已完成压缩的每个区间的 user_msg_id + assistant_msg_id。
    ///
    /// 注:这只是"原本要从历史里隐去的 message id 集",**不**包含摘要本身 —
    /// 摘要文本由 `project_for_request` 在 head 第一个 hidden 位置插入合成消息覆盖。
    pub fn hidden_message_ids(&self) -> HashSet<String> {
        self.completed
            .iter()
            .flat_map(|c| [c.user_msg_id.clone(), c.assistant_msg_id.clone()])
            .collect()
    }

    /// 调试 / 测试入口:看一条 marker 是否存在。
    #[cfg(test)]
    pub(crate) fn marker_count(&self) -> usize {
        self.markers.len()
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn cc(uid: &str, aid: &str, auto: bool) -> CompletedCompaction {
        CompletedCompaction {
            user_msg_id: uid.to_string(),
            assistant_msg_id: aid.to_string(),
            tail_start_id: None,
            summary_text: Some(format!("summary-{aid}")),
            auto,
            overflow: false,
        }
    }

    #[test]
    fn push_completed_marks_both_messages() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", true));
        assert!(s.marker("u1").unwrap().compaction_trigger == Some(CompactionTrigger::Auto));
        assert!(s.marker("a1").unwrap().is_summary);
    }

    #[test]
    fn previous_summary_returns_last() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        assert_eq!(s.previous_summary(), Some("summary-a2"));
    }

    #[test]
    fn hidden_message_ids_covers_all_completed() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        let h = s.hidden_message_ids();
        assert!(h.contains("u1"));
        assert!(h.contains("a1"));
        assert!(h.contains("u2"));
        assert!(h.contains("a2"));
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn upsert_marker_merges() {
        let mut s = CompactionState::default();
        s.upsert_marker("m1", |m| m.is_summary = true);
        s.upsert_marker("m1", |m| m.synthetic_continue = true);
        let m = s.marker("m1").unwrap();
        assert!(m.is_summary);
        assert!(m.synthetic_continue);
        assert_eq!(s.marker_count(), 1);
    }

    #[test]
    fn default_serializable_roundtrip() {
        let s = CompactionState::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: CompactionState = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, CompactionState::VERSION);
        assert!(back.completed.is_empty());
    }
}

```


**Tool Call: Read file `app\src\ai\byop_compaction\commit.rs` (lines 1-220)**
Status: Completed

```warp/app/src/ai/byop_compaction/commit.rs
//! 把刚刚完成的 SummarizeConversation 流的产出写回 conversation.compaction_state —
//! 对齐 opencode `compaction.ts processCompaction` 末尾的状态变更 + bus.publish(Compacted)。
//!
//! 本模块独立于 controller,作为可单元测试的 helper(虽然真实调用站点在 controller.rs)。

use warp_multi_agent_api as api;

use crate::ai::agent::conversation::AIConversation;

use super::algorithm::{prune_decisions, MessageRef};
use super::config::CompactionConfig;
use super::message_view::{build_tool_name_lookup, project};
use super::state::CompletedCompaction;

/// 从 conversation 的 root task 倒序找最后一条 `Message::AgentOutput` —
/// 它就是模型刚 emit 的摘要文本。
///
/// `user_msg_id` 选最后一条 AgentOutput 之前最近一条真实 UserQuery 的 id;
/// 没有时合成一个独立 uuid(只用作 marker key,build_chat_request 的 hidden
/// 投影不会命中真实 message)。
pub fn commit_summarization(conversation: &mut AIConversation, overflow: bool) -> bool {
    // 用 conversation 已有的 linearized messages accessor — 跨所有 task 已按时间序合并
    let mut all_msgs: Vec<&api::Message> = conversation.all_linearized_messages();
    all_msgs.sort_by_key(|m| {
        m.timestamp
            .as_ref()
            .map(|ts| (ts.seconds, ts.nanos))
            .unwrap_or((0, 0))
    });

    let last_agent_output: Option<(String, String)> = all_msgs.iter().rev().find_map(|m| {
        let inner = m.message.as_ref()?;
        match inner {
            api::message::Message::AgentOutput(a) => Some((m.id.clone(), a.text.clone())),
            _ => None,
        }
    });

    let Some((assistant_id, summary_text)) = last_agent_output else {
        log::warn!("[byop-compaction] commit: no AgentOutput found — nothing to commit");
        return false;
    };

    let assistant_id_str: &str = &assistant_id;
    let assistant_pos = all_msgs
        .iter()
        .position(|m| m.id.as_str() == assistant_id_str);
    let user_msg_id: String = assistant_pos
        .and_then(|pos| {
            all_msgs[..pos]
                .iter()
                .rev()
                .find_map(|m| match m.message.as_ref() {
                    Some(api::message::Message::UserQuery(_)) => Some(m.id.clone()),
                    _ => None,
                })
        })
        .unwrap_or_else(|| format!("compaction-trigger-{}", uuid::Uuid::new_v4()));

    let auto = overflow;
    let summary_len = summary_text.len();
    let completed = CompletedCompaction {
        user_msg_id: user_msg_id.clone(),
        assistant_msg_id: assistant_id.clone(),
        tail_start_id: None,
        summary_text: Some(summary_text),
        auto,
        overflow,
    };
    log::info!(
        "[byop-compaction] commit: assistant_msg={} user_msg={} summary_len={} auto={} overflow={}",
        assistant_id,
        user_msg_id,
        summary_len,
        auto,
        overflow,
    );
    conversation.compaction_state.push_completed(completed);
    true
}

/// 在每次 LLM 请求前自动跑 prune — 1:1 对齐 opencode `compaction.ts:297-341`。
///
/// 计算决策(哪些 ToolCallResult 的 output 应被替换为占位)然后写入
/// `conversation.compaction_state.markers.tool_output_compacted_at`。
/// 实际替换发生在 `chat_stream::build_chat_request` 投影时(读 marker)。
///
/// `cfg.prune == false` 时 no-op。
pub fn prune_now(conversation: &mut AIConversation, cfg: &CompactionConfig) -> usize {
    if !cfg.prune {
        return 0;
    }
    let all_msgs: Vec<&api::Message> = conversation.all_linearized_messages();
    if all_msgs.is_empty() {
        return 0;
    }
    let tool_names = build_tool_name_lookup(all_msgs.iter().copied());
    let state_snapshot = conversation.compaction_state.clone();
    let views = project(&all_msgs, &state_snapshot, &tool_names);
    // 用 trait 引用避免泛型推导歧义
    let views_ref: &[_] = &views;
    let decisions = prune_decisions::<super::message_view::WarpMessageView<'_>>(views_ref);
    if decisions.is_empty() {
        return 0;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let count = decisions.len();
    for (msg_id, _call_id) in decisions {
        // msg_id 是 ToolCallResult 的 message id;mark_tool_compacted 会在 marker 上写时间戳
        conversation
            .compaction_state
            .mark_tool_compacted(msg_id, now_ms);
    }
    log::info!("[byop-compaction] pruned {count} tool output(s)");
    count
}

// Reference traits for type inference
#[allow(unused_imports)]
use super::algorithm::Role as _Role;
#[allow(unused_imports)]
use super::algorithm::ToolOutputRef as _ToolOutputRef;
// Mention MessageRef so that the import isn't dropped
#[allow(dead_code)]
fn _ensure_message_ref_imported<M: MessageRef>(_m: &M) {}

```


**Tool Call: Read file `app\src\ai\agent_providers\chat_stream.rs` (lines 450-640)**
Status: Completed

```warp/app/src/ai/agent_providers/chat_stream.rs
    all_msgs.sort_by_key(|m| {
        m.timestamp
            .as_ref()
            .map(|ts| (ts.seconds, ts.nanos))
            .unwrap_or((0, 0))
    });

    // OpenWarp BYOP 本地会话压缩:把 conversation.compaction_state 应用到 message 序列。
    //   1. 过滤已被某次压缩覆盖的 (user, assistant) 对(`hidden_message_ids`)
    //   2. 在被隐去区间的位置插入一对合成的 (user "已压缩,以下为摘要" + assistant 摘要文本) message —
    //      这一步通过 `summary_inserts` 索引在主 loop 里就近 emit
    //   3. ToolCallResult 的 marker.tool_output_compacted_at 不为空时,后面分支替换 content 为占位符
    //
    // 当前 input 是 `AIAgentInput::SummarizeConversation` 时:进一步用 select 算法把 messages
    // 切到 head(去掉 tail),最后 input loop 末尾会追加 `build_prompt(...)` 作为 user message
    // (走完整的 SUMMARY_TEMPLATE),让上游 LLM 输出结构化摘要。
    let is_summarization_request = params
        .input
        .iter()
        .any(|i| matches!(i, AIAgentInput::SummarizeConversation { .. }));
    let summarization_overflow = params.input.iter().any(|i| {
        matches!(
            i,
            AIAgentInput::SummarizeConversation { overflow: true, .. }
        )
    });
    let _ = summarization_overflow; // 当前在 input loop 内的 follow-up 文案分支会用,目前先 silence dead

    let summary_inserts: std::collections::HashMap<String, String> =
        if let Some(state) = params.compaction_state.as_ref() {
            // user_msg_id → summary_text;遇到该 user_msg_id 时(它本来要被 hidden)替换为合成的摘要对
            state
                .completed()
                .iter()
                .filter_map(|c| {
                    c.summary_text
                        .as_ref()
                        .map(|s| (c.user_msg_id.clone(), s.clone()))
                })
                .collect()
        } else {
            std::collections::HashMap::new()
        };
    let hidden_msg_ids: std::collections::HashSet<String> = params
        .compaction_state
        .as_ref()
        .map(|s| s.hidden_message_ids())
        .unwrap_or_default();
    let compacted_tool_msg_ids: std::collections::HashSet<String> = params
        .compaction_state
        .as_ref()
        .map(|s| {
            // 收集所有标记了 tool_output_compacted_at 的 ToolCallResult message_ids
            // 通过遍历 all_msgs 并查 marker 实现
            let mut out = std::collections::HashSet::new();
            for msg in &all_msgs {
                if let Some(api::message::Message::ToolCallResult(_)) = &msg.message {
                    if s.marker(&msg.id)
                        .and_then(|m| m.tool_output_compacted_at)
                        .is_some()
                    {
                        out.insert(msg.id.clone());
                    }
                }
            }
            out
        })
        .unwrap_or_default();

    // 摘要请求路径:用 byop_compaction::algorithm::select 切 head;tail 不送上游
    let summarize_head_end: Option<usize> = if is_summarization_request {
        // 临时投影成 WarpMessageView 算 select
        let state_for_select = params.compaction_state.clone().unwrap_or_default();
        let tool_names =
            byop_compaction::message_view::build_tool_name_lookup(all_msgs.iter().copied());
        let views =
            byop_compaction::message_view::project(&all_msgs, &state_for_select, &tool_names);
        let cfg = byop_compaction::CompactionConfig::default();
        let model_limit = byop_compaction::overflow::ModelLimit::FALLBACK;
        let result = byop_compaction::algorithm::select(&views, &cfg, model_limit, |slice| {
            slice
                .iter()
                .map(byop_compaction::algorithm::MessageRef::estimate_size)
                .sum()
        });
        // head_end 是 views 里"head 区间"上界,与 all_msgs 同序
        Some(result.head_end)
    } else {
        None
    };

    let mut buf = AssistantBuffer::new(force_echo_reasoning);
    // OpenWarp:历史里被 skip 掉的 subagent ToolCall 对应的 call_id —— 它们的
    // ToolCallResult 也必须 skip,否则会成为孤儿 tool_response,Anthropic 直接 400
    // `unexpected tool_use_id ... no corresponding tool_use block`。
    let mut skipped_subagent_call_ids: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (idx, msg) in all_msgs.iter().enumerate() {
        // 摘要请求:tail 区间不送上游(只送 head + 末尾追加 SUMMARY_TEMPLATE)
        if let Some(head_end) = summarize_head_end {
            if idx >= head_end {
                continue;
            }
        }
        let Some(inner) = &msg.message else {
            continue;
        };
        match inner {
            api::message::Message::UserQuery(u) => {
                // 压缩投影:hidden 区间的 user message 替换为合成的"以下为已压缩历史的摘要"对
                if hidden_msg_ids.contains(&msg.id) {
                    if let Some(summary_text) = summary_inserts.get(&msg.id) {
                        buf.flush_into(&mut messages);
                        messages.push(ChatMessage::user(
                            "Conversation history was compacted. Below is the structured summary of all prior turns.".to_string(),
                        ));
                        messages.push(ChatMessage::assistant(summary_text.clone()));
                    }
                    // 没有 summary_text 的 hidden user 直接 skip(不应该发生,防御性)
                    continue;
                }
                buf.flush_into(&mut messages);
                // OpenWarp:历史轮多模态保活。warp 自家路径靠云端 server 重注入 InputContext,
                // BYOP 直连没有那层,所以 `make_user_query_message` 持久化时把所有 binary
                // (image / pdf / audio)塞进了 `UserQuery.context.images`,这里反向恢复成
                // UserBinary 走 `build_user_message_with_binaries`,使后续轮模型仍能看到先前
                // 粘贴的多模态附件。模型 caps 不支持的 mime 由 build_user_message_with_binaries
                // 替换为 ERROR 文本(opencode unsupportedParts 风格),不会静默 drop。
                // 没有 binary → 退回老路 `ChatMessage::user(text)`,与修复前等价。
                let history_binaries: Vec<user_context::UserBinary> = u
                    .context
                    .as_ref()
                    .map(|ctx| {
                        ctx.images
                            .iter()
                            .filter(|b| !b.data.is_empty())
                            .enumerate()
                            .map(|(idx, b)| {
                                use base64::Engine;
                                user_context::UserBinary {
                                    name: format!("history-attachment-{}-{idx}", &msg.id),
                                    content_type: if b.mime_type.is_empty() {
                                        "application/octet-stream".to_string()
                                    } else {
                                        b.mime_type.clone()
                                    },
                                    data: base64::engine::general_purpose::STANDARD.encode(&b.data),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if history_binaries.is_empty() {
                    messages.push(ChatMessage::user(u.query.clone()));
                } else {
                    messages.push(build_user_message_with_binaries(
                        u.query.clone(),
                        history_binaries,
                        api_type,
                        model_id,
                    ));
                }
            }
            // hidden assistant message 直接 skip(它是某次压缩对的 assistant_msg_id,
            // 摘要文本已经在对应 user 分支注入)
            api::message::Message::AgentReasoning(_) | api::message::Message::AgentOutput(_)
                if hidden_msg_ids.contains(&msg.id) =>
            {
                continue;
            }
            api::message::Message::AgentReasoning(r) => {
                // 把上一轮的 reasoning 挂到下一个要 flush 的 assistant message 上。
                // genai 0.6 的 with_reasoning_content 会按当前 adapter 序列化:
                // DeepSeek/Kimi → reasoning_content 字段;Anthropic → thinking blocks。
                // 多段 AgentReasoning 累加(同一 turn 可能 stream 出多个 reasoning chunk
                // 落地为多条 AgentReasoning)。
                let next = r.reasoning.clone();
                if !next.is_empty() {
                    match buf.reasoning.as_mut() {
                        Some(existing) => existing.push_str(&next),
                        None => buf.reasoning = Some(next),
                    }
                }
            }
            api::message::Message::AgentOutput(a) => {
                if buf.text.is_some() || !buf.tool_calls.is_empty() {
                    buf.flush_into(&mut messages);
                }
                buf.text = Some(a.text.clone());
            }

```


**Tool Call: Read file `app\src\ai\blocklist\controller.rs` (lines 2000-2940)**
Status: Completed

```warp/app/src/ai/blocklist/controller.rs
        let conversation_data = api::ConversationData {
            id: conversation_id,
            tasks: active_tasks,
            server_conversation_token: conversation_server_token,
            forked_from_conversation_token: conversation_forked_from_token,
            ambient_agent_task_id: self.ambient_agent_task_id,
            existing_suggestions: history_model
                .as_ref(ctx)
                .existing_suggestions_for_conversation(conversation_id)
                .cloned(),
        };

        // Log an error if tool call results do not have corresponding tool calls in task context
        validate_tool_call_results(
            request_input.all_inputs(),
            &conversation_data.tasks,
            &conversation_data.server_conversation_token,
        );

        let mut request_params = api::RequestParams::new(
            Some(self.terminal_view_id),
            SessionContext::from_session(self.active_session.as_ref(ctx), ctx),
            &request_input,
            conversation_data.clone(),
            query_metadata,
            ctx,
        );
        request_params.parent_agent_id = parent_agent_id;
        request_params.agent_name = agent_name;

        // OpenWarp BYOP 本地会话压缩:
        //   1. 自动 prune 旧 tool output(对齐 opencode `compaction.ts:297-341` prune)
        //   2. 把 conversation.compaction_state.clone() 注入 request_params
        //
        // chat_stream::build_chat_request 会据此投影 messages(隐去已压缩区间 + 替换 compacted tool output);
        // SummarizeConversation input 路径还会切 head + 拼 SUMMARY_TEMPLATE。
        // 非 BYOP 路径(走 server protobuf)不读这个字段,无副作用。
        let compaction_cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
        history_model.update(ctx, |history_model, _ctx| {
            if let Some(convo) = history_model.conversation_mut(&conversation_id) {
                crate::ai::byop_compaction::commit::prune_now(convo, &compaction_cfg);
            }
        });
        if let Some(convo) = history_model.as_ref(ctx).conversation(&conversation_id) {
            request_params.compaction_state = Some(convo.compaction_state.clone());
        }

        // OpenWarp BYOP:检测当前请求是否绑定 LRC(alt-screen 长命令)。
        // - tag-in 首轮:注入 command_id + running_command,并让 chat_stream 合成 subagent
        //   CreateTask 事件来升级 master 路径已经创建的 optimistic CLI subtask。
        // - 已进入 agent control 的后续轮:auto-resume / tool result 仍要注入 command_id
        //   与最新 PTY 快照,但不能重复 spawn subagent。
        {
            let terminal_model = self.terminal_model.lock();
            let active_block = terminal_model.block_list().active_block();
            let is_lrc_tagged_in = active_block.is_agent_tagged_in();
            let is_matching_lrc_agent = active_block.is_agent_in_control()
                && active_block
                    .agent_interaction_metadata()
                    .is_some_and(|metadata| metadata.conversation_id() == &conversation_id);
            if is_lrc_tagged_in || is_matching_lrc_agent {
                request_params.lrc_command_id = Some(active_block.id().to_string());
                request_params.lrc_should_spawn_subagent = is_lrc_tagged_in;

                // OpenWarp A3:把完整 RunningCommand 注入到本轮 UserQuery 中,
                // 严格对齐上游 `get_running_command` 的 grid_contents 提取逻辑
                // (alt-screen 走 alt_screen.grid_handler,非 alt-screen 走 output_grid)。
                // 之前用 `output_to_string_force_full_grid_contents()` 在 nvim 等
                // alt-screen TUI 下取到空字符串,导致 prefix 块为空,模型说看不到 command_id。
                if let Some(running_command) = byop_get_running_command_for_lrc(&terminal_model) {
                    request_params.lrc_running_command = Some(running_command.clone());
                    let total_inputs = request_params.input.len();
                    let mut filled_count = 0usize;
                    for input in request_params.input.iter_mut() {
                        if let crate::ai::agent::AIAgentInput::UserQuery {
                            running_command: rc_slot @ None,
                            ..
                        } = input
                        {
                            *rc_slot = Some(running_command.clone());
                            filled_count += 1;
                        }
                    }
                    log::info!(
                        "[byop-diag] LRC running_command filled: {filled_count}/{total_inputs} \
                         UserQuery slot(s); should_spawn={} grid_contents_len={} command={:?} is_alt_screen={}",
                        request_params.lrc_should_spawn_subagent,
                        running_command.grid_contents.len(),
                        running_command.command,
                        running_command.is_alt_screen_active
                    );
                } else {
                    log::warn!(
                        "[byop-diag] LRC detected but byop_get_running_command_for_lrc \
                         returned None (active_block 状态不符)"
                    );
                }
            }
        }

        let server_conversation_token_for_identifiers =
            conversation_data.server_conversation_token.clone();

        let response_stream = ctx.add_model(|ctx| {
            // Create AIIdentifiers for the response stream
            let ai_identifiers = AIIdentifiers {
                server_output_id: None, // Will be populated by the successful response
                server_conversation_id: server_conversation_token_for_identifiers.map(Into::into),
                client_conversation_id: Some(conversation_data.id),
                client_exchange_id: None,
                model_id: Some(request_params.model.clone()),
            };
            ResponseStream::new(
                request_params.clone(),
                ai_identifiers,
                can_attempt_resume_on_error,
                ctx,
            )
        });
        let response_stream_id = response_stream.as_ref(ctx).id().clone();
        let response_stream_clone = response_stream.clone();
        let input_contains_user_query = request_input
            .all_inputs()
            .any(|input| input.is_user_query());
        ctx.subscribe_to_model(&response_stream, move |me, event, ctx| {
            me.handle_response_stream_event(
                input_contains_user_query,
                event,
                &response_stream_clone,
                ctx,
            );
        });

        let is_passive_request = request_input
            .all_inputs()
            .any(|input| input.is_passive_request());

        for input in request_input.all_inputs() {
            if let AIAgentInput::UserQuery {
                referenced_attachments,
                ..
            } = input
            {
                self.maybe_populate_plans_for_ai_document_model(
                    referenced_attachments,
                    conversation_data.id,
                    ctx,
                );
            }
        }

        history_model.update(ctx, |history_model, ctx| {
            match history_model.update_conversation_for_new_request_input(
                request_input,
                response_stream_id.clone(),
                self.terminal_view_id,
                ctx,
            ) {
                Ok(_) => {
                    history_model.update_conversation_status(
                        self.terminal_view_id,
                        conversation_data.id,
                        ConversationStatus::InProgress,
                        ctx,
                    );
                }
                Err(e) => {
                    log::warn!("Failed to push new exchange to AI conversation: {e:?}");
                }
            }
        });

        self.in_flight_response_streams.register_new_stream(
            response_stream_id.clone(),
            conversation_data.id,
            response_stream,
            CancellationReason::FollowUpSubmitted {
                is_for_same_conversation: true,
            },
            ctx,
        );

        if input_contains_user_query {
            // Get the pending document ID before clearing context
            let pending_document_id = self.context_model.as_ref(ctx).pending_document_id();

            // Reset the context state to the default.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.reset_context_to_default(ctx);
            });

            // Update the document status to UpToDate after query submission
            if let Some(doc_id) = pending_document_id {
                AIDocumentModel::handle(ctx).update(ctx, |model, mctx| {
                    model.set_user_edit_status(&doc_id, AIDocumentUserEditStatus::UpToDate, mctx);
                });
            }
        }

        ctx.emit(BlocklistAIControllerEvent::SentRequest {
            contains_user_query: input_contains_user_query,
            is_queued_prompt,
            model_id: request_params.model.clone(),
            stream_id: response_stream_id.clone(),
        });
        if !is_passive_request {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history_model, ctx| {
                history_model.set_active_conversation_id(
                    conversation_data.id,
                    self.terminal_view_id,
                    ctx,
                )
            });
        }

        // Trigger a snapshot save to persist the agent view state when a user query is sent.
        // This ensures the agent view is restored if the app restarts.
        if input_contains_user_query {
            ctx.dispatch_global_action("workspace:save_app", ());
        }

        // If `AgentView` is enabled, the agent view is guaranteed to be active when the agent
        // input is sent, so logic to ensure follow-ups is redundant.
        if !FeatureFlag::AgentView.is_enabled() && default_to_follow_up_on_success {
            // Set the input mode to AI but allow autodetection to run
            self.input_model.update(ctx, |input_model, ctx| {
                input_model.set_input_config_for_classic_mode(
                    InputConfig {
                        input_type: InputType::AI,
                        is_locked: false,
                    },
                    ctx,
                );
            });
            // After making an AI query, default to asking a follow up.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.set_pending_query_state_for_existing_conversation(
                    conversation_data.id,
                    AgentViewEntryOrigin::AutoFollowUp,
                    ctx,
                )
            });
        }

        Ok((conversation_data.id, response_stream_id))
    }

    /// Cancels a pending AI request response stream, given the exchange ID, if it exists.
    /// Returns true if a pending stream was found and canceled, false otherwise.
    pub fn try_cancel_pending_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        self.in_flight_response_streams
            .try_cancel_stream(stream_id, reason, ctx)
    }

    /// Cancels 'progress' for the active conversation if there is one:
    ///  * If there is an in-flight request, cancels it.
    ///  * Else, if the request finished, but actions from the response are pending or mid-execution, cancels all of them.
    pub fn cancel_conversation_progress(
        &mut self,
        conversation_id: AIConversationId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) {
        // Cancel any pending auto-resume for this conversation.
        if let Some(handle) = self.pending_auto_resume_handles.remove(&conversation_id) {
            handle.abort();
        }

        // Discard any queued passive suggestion results for this conversation.
        self.pending_passive_suggestion_results
            .remove(&conversation_id);

        if !self
            .in_flight_response_streams
            .try_cancel_streams_for_conversation(conversation_id, reason, ctx)
        {
            // Otherwise, cancel pending actions and update the input state.
            self.action_model.update(ctx, |action_model, ctx| {
                action_model.cancel_all_pending_actions(conversation_id, Some(reason), ctx);
            });
            self.set_input_mode_for_cancellation(ctx);
        }
    }

    /// Clears finished action results for a conversation. Used when reverting.
    pub fn clear_finished_action_results(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.action_model.update(ctx, |action_model, _| {
            action_model.clear_finished_action_results(conversation_id);
        });
    }

    /// Cancels the in-flight request for the given conversation, if there is one.
    ///
    /// Returns `true` if a request was actually cancelled.
    pub fn cancel_request(
        &mut self,
        response_stream_id: &ResponseStreamId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        self.in_flight_response_streams
            .try_cancel_stream(response_stream_id, reason, ctx)
    }

    fn start_title_generation(
        &mut self,
        pending_title_generation: PendingTitleGeneration,
        stream_id: ResponseStreamId,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        let terminal_view_id = self.terminal_view_id;
        let _ = ctx.spawn(
            async move {
                let result = crate::ai::agent_providers::chat_stream::generate_title_via_byop(
                    &pending_title_generation.input,
                    &pending_title_generation.user_query,
                )
                .await;
                (pending_title_generation.task_id, result)
            },
            move |me, (task_id, result), ctx| match result {
                Ok(Some(title)) => {
                    log::info!("[byop] title generated: {title:?}");
                    let client_actions = vec![ClientAction {
                        action: Some(Action::UpdateTaskDescription(UpdateTaskDescription {
                            task_id,
                            description: title,
                        })),
                    }];
                    let response_event = warp_multi_agent_api::ResponseEvent {
                        r#type: Some(warp_multi_agent_api::response_event::Type::ClientActions(
                            warp_multi_agent_api::response_event::ClientActions {
                                actions: client_actions.clone(),
                            },
                        )),
                    };
                    if FeatureFlag::AgentSharedSessions.is_enabled() {
                        let participant_id = me
                            .get_current_response_initiator()
                            .or_else(|| me.get_sharer_participant_id());
                        let mut model = me.terminal_model.lock();
                        if model.shared_session_status().is_sharer() {
                            model.send_agent_response_for_shared_session(
                                &response_event,
                                participant_id,
                                None,
                            );
                        }
                    }
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history_model, ctx| {
                        match history_model.apply_client_actions(
                            &stream_id,
                            client_actions,
                            conversation_id,
                            terminal_view_id,
                            ctx,
                        ) {
                            Ok(()) => {
                                ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationMetadata {
                                    terminal_view_id: Some(terminal_view_id),
                                    conversation_id,
                                });
                            }
                            Err(e) => {
                                log::warn!("[byop] title update failed: {e:#}");
                            }
                        }
                    });
                }
                Ok(None) => {
                    log::warn!("[byop] title gen returned empty content; skip");
                }
                Err(e) => {
                    log::warn!("[byop] title gen failed: {e:#}; skip");
                }
            },
        );
    }

    fn handle_response_stream_event(
        &mut self,
        did_input_contain_user_query: bool,
        event: &ResponseStreamEvent,
        response_stream: &ModelHandle<ResponseStream>,
        ctx: &mut ModelContext<Self>,
    ) {
        let stream_id = response_stream.as_ref(ctx).id().clone();

        match event {
            ResponseStreamEvent::ReceivedEvent(event) => {
                // Dynamic lookup handles conversation splits mid-stream.
                let Some(conversation_id) = BlocklistAIHistoryModel::as_ref(ctx)
                    .conversation_for_response_stream(&stream_id)
                else {
                    log::warn!("Could not find conversation for response stream: {stream_id:?}");
                    return;
                };
                let Some(event) = event.consume() else {
                    debug_assert!(
                        false,
                        "This model should only have a single subscriber that takes ownership over the event."
                    );
                    return;
                };
                let history_model = BlocklistAIHistoryModel::handle(ctx);
                match event {
                    Ok(event) => {
                        // If this controller is part of a shared session, forward the entire response event to viewers first.
                        if FeatureFlag::AgentSharedSessions.is_enabled() {
                            let mut model = self.terminal_model.lock();
                            if model.shared_session_status().is_sharer() {
                                // Get the participant who initiated this response, falling back to the sharer if needed.
                                let participant_id = self
                                    .get_current_response_initiator()
                                    .or_else(|| self.get_sharer_participant_id());

                                // For forked conversations (e.g. when loading from cloud), include
                                // the original conversation token so viewers can link the new
                                // server-assigned token to their existing conversation.
                                //
                                // This token is cleared after the first Init event (see below),
                                // so it's only sent once per forked conversation.
                                let forked_from_token = history_model
                                    .as_ref(ctx)
                                    .conversation(&conversation_id)
                                    .and_then(|conv| {
                                        conv.forked_from_server_conversation_token()
                                            .map(|t| t.as_str().to_string())
                                    });

                                model.send_agent_response_for_shared_session(
                                    &event,
                                    participant_id,
                                    forked_from_token,
                                );
                            }
                        }
                        let Some(event) = event.r#type else {
                            return;
                        };
                        match event {
                            warp_multi_agent_api::response_event::Type::Init(init_event) => {
                                history_model.update(ctx, |history_model, ctx| {
                                    history_model.initialize_output_for_response_stream(
                                        &stream_id,
                                        conversation_id,
                                        self.terminal_view_id,
                                        init_event,
                                        ctx,
                                    );

                                    // Clear the forked_from token after the first Init event.
                                    // For forked conversations, we only need to send this once so
                                    // viewers can update their conversation's server token. After
                                    // that, the viewer's conversation uses the new token directly.
                                    if let Some(conversation) =
                                        history_model.conversation_mut(&conversation_id)
                                    {
                                        conversation.clear_forked_from_server_conversation_token();
                                    }
                                });
                            }
                            warp_multi_agent_api::response_event::Type::Finished(
                                finished_event,
                            ) => {
                                let completed_successfully = matches!(
                                    finished_event.reason.as_ref(),
                                    Some(
                                        warp_multi_agent_api::response_event::stream_finished::Reason::Done(_)
                                    ) | None
                                );
                                if completed_successfully {
                                    if let Some(pending_title_generation) =
                                        response_stream.update(ctx, |response_stream, _| {
                                            response_stream.take_pending_title_generation()
                                        })
                                    {
                                        self.start_title_generation(
                                            pending_title_generation,
                                            stream_id.clone(),
                                            conversation_id,
                                            ctx,
                                        );
                                    }
                                }
                                // OpenWarp BYOP 本地会话压缩:在 stream finished 前拿 summarization 标志
                                let summarize_overflow =
                                    response_stream.as_ref(ctx).summarization_overflow();
                                self.handle_response_stream_finished(
                                    &stream_id,
                                    finished_event,
                                    conversation_id,
                                    did_input_contain_user_query,
                                    summarize_overflow,
                                    ctx,
                                );
                            }
                            warp_multi_agent_api::response_event::Type::ClientActions(actions) => {
                                let client_actions = actions.actions;
                                let apply_result =
                                    history_model.update(ctx, |history_model, ctx| {
                                        history_model.apply_client_actions(
                                            &stream_id,
                                            client_actions,
                                            conversation_id,
                                            self.terminal_view_id,
                                            ctx,
                                        )
                                    });
                                if let Err(e) = apply_result {
                                    log::error!(
                                        "Failed to apply client actions to conversation: {e:?}"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if matches!(e.as_ref(), AIApiError::QuotaLimit) {
                            // If the error is a quota limit, we want to refresh workspace metadata
                            // So the current state of AI overages is immediately up to date.
                            TeamUpdateManager::handle(ctx).update(
                                ctx,
                                |team_update_manager, ctx| {
                                    std::mem::drop(
                                        team_update_manager.refresh_workspace_metadata(ctx),
                                    );
                                },
                            );
                            AIRequestUsageModel::handle(ctx).update(ctx, |model, ctx| {
                                model.enable_buy_credits_banner(ctx);
                            });
                        }

                        let mut renderable_error: RenderableAIError = e.as_ref().into();
                        if let RenderableAIError::Other {
                            will_attempt_resume,
                            waiting_for_network,
                            ..
                        } = &mut renderable_error
                        {
                            let should_attempt_resume = response_stream
                                .as_ref(ctx)
                                .should_resume_conversation_after_stream_finished();
                            *will_attempt_resume |= should_attempt_resume;
                            if should_attempt_resume {
                                let network_status = NetworkStatus::as_ref(ctx);
                                *waiting_for_network = !network_status.is_online();
                            }
                        }

                        history_model.update(ctx, |history_model, ctx| {
                            history_model.mark_response_stream_completed_with_error(
                                renderable_error,
                                &stream_id,
                                conversation_id,
                                self.terminal_view_id,
                                ctx,
                            );
                        });
                    }
                }
            }
            ResponseStreamEvent::AfterStreamFinished { cancellation } => {
                // Cancellations provide conversation_id (survives truncation); otherwise use dynamic lookup.
                let conversation_id = match &cancellation {
                    Some(stream_cancellation) => stream_cancellation.conversation_id,
                    None => {
                        let Some(id) = BlocklistAIHistoryModel::as_ref(ctx)
                            .conversation_for_response_stream(&stream_id)
                        else {
                            log::warn!(
                                "Could not find conversation for response stream: {stream_id:?}"
                            );
                            return;
                        };
                        id
                    }
                };

                let history_model = BlocklistAIHistoryModel::handle(ctx);
                let Some(conversation) = history_model.as_ref(ctx).conversation(&conversation_id)
                else {
                    log::warn!("Conversation not found.");
                    return;
                };
                let new_exchange_ids = conversation.new_exchange_ids_for_response(&stream_id);
                let mut was_passive_request = false;
                let mut is_any_exchange_unfinished = false;
                let mut actions_to_queue = vec![];
                // OpenWarp BYOP:收集本轮新加 message id,稍后用于在 EMPTY 分支检测
                // synthetic invalid_arguments 错误标记。**只看本轮 added** 才能避免
                // 在历史里反复命中导致 auto-resume 死循环(标记一旦持久化就永远在)。
                let mut newly_added_message_ids: std::collections::HashSet<MessageId> =
                    std::collections::HashSet::new();

                for new_exchange_id in new_exchange_ids {
                    let Some(exchange) = conversation.exchange_with_id(new_exchange_id) else {
                        log::warn!("Exchange not found.");
                        return;
                    };
                    was_passive_request |= exchange.has_passive_request();
                    is_any_exchange_unfinished |= !exchange.output_status.is_finished();
                    newly_added_message_ids.extend(exchange.added_message_ids.iter().cloned());

                    if let AIAgentOutputStatus::Finished {
                        finished_output: FinishedAIAgentOutput::Success { output },
                        ..
                    } = &exchange.output_status
                    {
                        actions_to_queue.extend(output.get().actions().cloned());
                    }
                }

                if let Some(stream_cancellation) = &cancellation {
                    // If this is a shared session, send a synthetic StreamFinished event to notify viewers
                    // of any user-initiated cancellation. We skip FollowUpSubmitted because that's an internal
                    // cancellation for continuing the conversation.
                    if FeatureFlag::AgentSharedSessions.is_enabled()
                        && !stream_cancellation
                            .reason
                            .is_follow_up_for_same_conversation()
                    {
                        self.send_cancellation_to_viewers(ctx);
                    }

                    history_model.update(ctx, |history_model, ctx| {
                        history_model.mark_response_stream_cancelled(
                            &stream_id,
                            conversation_id,
                            self.terminal_view_id,
                            stream_cancellation.reason,
                            ctx,
                        );
                    });

                    if !was_passive_request {
                        self.set_input_mode_for_cancellation(ctx);
                    }
                } else if is_any_exchange_unfinished {
                    log::warn!(
                        "generate_multi_agent_output stream ended without emitting StreamFinished event."
                    );

                    let error_message = "Request did not successfully complete";
                    history_model.update(ctx, |history_model, ctx| {
                        history_model.mark_response_stream_completed_with_error(
                            RenderableAIError::Other {
                                error_message: error_message.to_string(),
                                will_attempt_resume: false,
                                waiting_for_network: false,
                            },
                            &stream_id,
                            conversation_id,
                            self.terminal_view_id,
                            ctx,
                        );
                    });
                } else if !actions_to_queue.is_empty() {
                    log::info!(
                        "[byop-diag] queue_actions: count={} ids=[{}] conversation_id={:?}",
                        actions_to_queue.len(),
                        actions_to_queue
                            .iter()
                            .map(|a| format!("{}", a.id))
                            .collect::<Vec<_>>()
                            .join(", "),
                        conversation_id,
                    );
                    // OpenWarp:LRC tag-in 首轮自动授权 agent 工具执行。
                    //
                    // 触发条件:发起本轮请求时 active_block 处于
                    // InteractionMode::User { did_user_tag_in_agent: true }。不能用当前
                    // active_block 的 monitored metadata 兜底,否则同一 CLI subagent 会话里的
                    // 后续普通请求也会被自动确认,导致确认 UI 不显示。
                    let auto_accept_for_lrc_tag_in =
                        response_stream.as_ref(ctx).is_lrc_tag_in_request();
                    if auto_accept_for_lrc_tag_in {
                        log::info!(
                            "[byop] LRC tag-in: queue with auto-accept ({} action(s))",
                            actions_to_queue.len()
                        );
                    }
                    self.action_model.update(ctx, |action_model, ctx| {
                        action_model.queue_actions_with_options(
                            actions_to_queue,
                            conversation_id,
                            auto_accept_for_lrc_tag_in,
                            ctx,
                        );
                    });
                } else {
                    // OpenWarp BYOP:from_args 解析失败时,chat_stream 走 fallback emit
                    // carrier ToolCall(tool=None) + synthetic error ToolCallResult(result=None,
                    // server_message_data 是 invalid_arguments JSON)。两者都走 NoClientRepresentation,
                    // 不入 actions_to_queue,exchange 静默结束 → 模型永远收不到错误反馈,
                    // 用户必须手动再发消息才能让模型重试。
                    //
                    // 检测最近 ~16 条 messages 是否含 BYOP synthetic 错误标记;有的话复用
                    // line 2695+ 的 auto-resume 路径触发重发,让模型立即基于 error tool_result
                    // 修正参数重试。`can_attempt_resume_on_error=false` 防 LLM 持续输出坏 args 导致死循环。
                    // 只在本轮新加的 messages 里查找 synthetic 错误标记,避免历史持久化的
                    // 同标记反复命中导致死循环。
                    // OpenWarp BYOP:两类 synthetic ToolCallResult 需要 auto-resume
                    // (二者都不入 actions_to_queue,exchange 静默结束 → 模型卡死等结果)。
                    // 1. invalid_arguments — from_args 解析失败兜底(原始)
                    // 2. _byop_intercepted — webfetch / websearch 等本地拦截工具结果
                    //    (chat_stream::dispatch_byop_web_tool 不走 protobuf executor,
                    //     直接合成 result,没有 AIAgentAction 入队)
                    let needs_byop_local_resume = conversation.all_tasks().any(|task| {
                        task.messages().any(|msg| {
                            newly_added_message_ids.contains(&MessageId::new(msg.id.clone()))
                                && matches!(
                                    msg.message,
                                    Some(message::Message::ToolCallResult(
                                        message::ToolCallResult { result: None, .. },
                                    )),
                                )
                                && (msg
                                    .server_message_data
                                    .contains(r#""error":"invalid_arguments""#)
                                    || msg
                                        .server_message_data
                                        .contains(r#""_byop_intercepted":true"#))
                        })
                    });
                    if needs_byop_local_resume {
                        log::info!(
                            "[byop] detected synthetic local tool_result (invalid_arguments \
                             or _byop_intercepted) without queued action → schedule auto-resume. \
                             conversation_id={conversation_id:?}"
                        );
                        let network_status = NetworkStatus::handle(ctx);
                        let wait_for_online = network_status.as_ref(ctx).wait_until_online();
                        let handle = ctx.spawn(wait_for_online, move |me, _, ctx| {
                            me.pending_auto_resume_handles.remove(&conversation_id);
                            me.resume_conversation(
                                conversation_id,
                                /*can_attempt_resume_on_error*/
                                false,
                                /*is_auto_resume_after_error*/
                                true,
                                vec![],
                                ctx,
                            );
                        });
                        self.pending_auto_resume_handles
                            .insert(conversation_id, handle);
                    }
                }

                // Cancelled streams will handle pending_response_stream updates synchronously.
                if cancellation.is_none() {
                    self.in_flight_response_streams.cleanup_stream(&stream_id);

                    // Now that the stream is cleaned up, re-check for pending
                    // orchestration events that couldn't be drained earlier.
                    if FeatureFlag::Orchestration.is_enabled() {
                        self.handle_pending_events_ready(conversation_id, ctx);
                    }
                }

                // Before cleaning up the response stream, check if we should attempt to resume.
                if response_stream
                    .as_ref(ctx)
                    .should_resume_conversation_after_stream_finished()
                {
                    let network_status = NetworkStatus::handle(ctx);
                    let wait_for_online = network_status.as_ref(ctx).wait_until_online();
                    let handle = ctx.spawn(wait_for_online, move |me, _, ctx| {
                        // Clean up the pending handle now that the resume is executing.
                        me.pending_auto_resume_handles.remove(&conversation_id);
                        me.resume_conversation(
                            conversation_id,
                            // Don't allow a second resume-on-error to prevent a persistent
                            // loop.
                            /*can_attempt_resume_on_error*/
                            false,
                            /*is_auto_resume_after_error*/
                            true,
                            vec![],
                            ctx,
                        );
                    });
                    self.pending_auto_resume_handles
                        .insert(conversation_id, handle);
                }

                // Clean up the response stream tracking entry now that the stream is complete.
                history_model.update(ctx, |history_model, _| {
                    if let Some(conversation) = history_model.conversation_mut(&conversation_id) {
                        conversation.cleanup_completed_response_stream(&stream_id);
                    }
                });
                ctx.unsubscribe_from_model(response_stream);

                if self.should_refresh_available_llms_on_stream_finish {
                    self.should_refresh_available_llms_on_stream_finish = false;
                    LLMPreferences::handle(ctx).update(ctx, |llm_preferences, ctx| {
                        llm_preferences.refresh_authed_models(ctx);
                    });
                }
                ctx.emit(BlocklistAIControllerEvent::FinishedReceivingOutput {
                    stream_id,
                    conversation_id,
                });
                AIRequestUsageModel::handle(ctx).update(ctx, |request_usage_model, ctx| {
                    request_usage_model.refresh_request_usage_async(ctx);
                });

                self.maybe_refresh_ai_overages(ctx);
            }
        }
    }

    /// Sets the terminal input state after an AI request is cancelled.
    /// From the user perspective, we downgrade the level of autonomy so:
    /// * Executing a task automatically -> interactive AI input
    /// * Interactive AI input -> interactive shell input
    fn set_input_mode_for_cancellation(&mut self, ctx: &mut ModelContext<Self>) {
        // If the request was cancelled, default to shell mode with autodetection
        // enabled.
        self.input_model.update(ctx, |input_model, ctx| {
            input_model.set_input_config_for_classic_mode(
                input_model
                    .input_config()
                    .with_shell_type()
                    .unlocked_if_autodetection_enabled(false, ctx),
                ctx,
            );
        });
    }

    /// Checks if we should refresh AI overage information after an AI request completes.
    /// This is used to ensure the UI matches the state of the workspace,
    /// especially because overages are not real-time communicated to clients.
    fn maybe_refresh_ai_overages(&mut self, ctx: &mut ModelContext<Self>) {
        let workspace = UserWorkspaces::as_ref(ctx).current_workspace();
        let Some(workspace) = workspace else {
            return;
        };

        // We want to minimize the number of times we ping our backend for updated usage information;
        // doing it after every AI query finishes would be very expensive.

        // If a user is below their personal limits, then we know that they won't eat into overages,
        // so we don't need to refresh.
        let has_no_requests_remaining = !AIRequestUsageModel::as_ref(ctx).has_requests_remaining();
        // If overages aren't enabled, we're not going to reap the benefit of refreshing at all anyway.
        let are_overages_enabled = workspace.are_overages_enabled();

        if are_overages_enabled && has_no_requests_remaining {
            // Give a one second delay to ensure that Stripe has been charged and the database is completely updated,
            // before syncing new AI overages data.
            ctx.spawn(
                async move { Timer::after(Duration::from_secs(1)).await },
                |_, _, ctx| {
                    UserWorkspaces::handle(ctx).update(ctx, |user_workspaces, ctx| {
                        user_workspaces.refresh_ai_overages(ctx);
                    });
                },
            );
        }
    }

    pub(super) fn handle_response_stream_finished(
        &mut self,
        stream_id: &ResponseStreamId,
        mut finished_event: warp_multi_agent_api::response_event::StreamFinished,
        conversation_id: AIConversationId,
        did_request_contain_user_query: bool,
        summarize_overflow: Option<bool>,
        ctx: &mut ModelContext<Self>,
    ) {
        // OpenWarp BYOP 本地会话压缩:在 token_usage move 进下面 closure 前先聚合,
        // 用于 auto overflow 检查(后面 Done 分支用)。
        let aggregate_token_count: usize = finished_event
            .token_usage
            .iter()
            .map(|u| (u.total_input + u.output + u.input_cache_read + u.input_cache_write) as usize)
            .max()
            .unwrap_or(0);

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        history_model.update(ctx, |history_model, _| {
            // Update conversation cost and usage information before updating and
            // persisting the conversation.
            history_model.update_conversation_cost_and_usage_for_request(
                conversation_id,
                finished_event
                    .request_cost
                    .map(|cost| RequestCost::new(cost.exact.into())),
                finished_event.token_usage,
                finished_event.conversation_usage_metadata.take(),
                did_request_contain_user_query,
            );
        });

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        match finished_event.reason {
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::Done(_)) | None => {
                // OpenWarp BYOP 本地会话压缩 - 写回 summary
                if let Some(overflow) = summarize_overflow {
                    history_model.update(ctx, |history_model, _ctx| {
                        if let Some(convo) = history_model.conversation_mut(&conversation_id) {
                            crate::ai::byop_compaction::commit::commit_summarization(
                                convo, overflow,
                            );
                        }
                    });
                }
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_successfully(
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });

                // OpenWarp BYOP 本地会话压缩 - auto overflow 触发(对齐 opencode `processor.ts:395-403`)
                // 仅在本流不是摘要本身时检查,防止递归。
                if summarize_overflow.is_none() {
                    let aggregate_count = aggregate_token_count;
                    if aggregate_count > 0 {
                        let cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
                        let model_limit =
                            crate::ai::byop_compaction::overflow::ModelLimit::FALLBACK;
                        let counts = crate::ai::byop_compaction::overflow::TokenCounts {
                            total: aggregate_count,
                            ..Default::default()

```


**Tool Call: Read file `app\src\ai\byop_compaction\tests.rs` (lines 1-260)**
Status: Completed

```warp/app/src/ai/byop_compaction/tests.rs
//! Phase 1 单元测试 — 覆盖纯函数(token / overflow / prompt / config / algorithm)。
//!
//! Phase 3 (state + message_view) 落地后再补 e2e 集成测试。

use super::algorithm::{prune_decisions, select, turns, MessageRef, Role, ToolOutputRef};
use super::config::CompactionConfig;
use super::consts::*;
use super::overflow::{is_overflow, usable, ModelLimit, TokenCounts};
use super::prompt::{build_continue_message, build_prompt, SUMMARY_TEMPLATE};
use super::token::estimate;

// -- token ---------------------------------------------------------------

#[test]
fn token_estimate_empty() {
    assert_eq!(estimate(""), 0);
}

#[test]
fn token_estimate_short() {
    // "hello world" = 11 chars → round(11/4) = 3
    assert_eq!(estimate("hello world"), 3);
}

#[test]
fn token_estimate_aligned() {
    assert_eq!(estimate(&"a".repeat(40)), 10);
    assert_eq!(estimate(&"a".repeat(41)), 10); // 41/4 = 10.25 → 10 (banker's rounding 不影响)
    assert_eq!(estimate(&"a".repeat(42)), 11); // 42/4 = 10.5 → 11
}

// -- overflow ------------------------------------------------------------

fn cfg_default() -> CompactionConfig {
    CompactionConfig::default()
}

#[test]
fn usable_with_input_limit() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    // reserved = min(20_000, 8_000) = 8_000
    // usable = max(0, 180_000 - 8_000) = 172_000
    assert_eq!(usable(&cfg, model), 172_000);
}

#[test]
fn usable_without_input_limit() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 0,
        max_output: 8_000,
    };
    // 走第二分支:context - max_output = 192_000
    assert_eq!(usable(&cfg, model), 192_000);
}

#[test]
fn usable_zero_context() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 0,
        input: 0,
        max_output: 0,
    };
    assert_eq!(usable(&cfg, model), 0);
}

#[test]
fn usable_respects_cfg_reserved_override() {
    let mut cfg = cfg_default();
    cfg.reserved = Some(50_000);
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    // reserved 覆盖为 50_000 → 180_000 - 50_000 = 130_000
    assert_eq!(usable(&cfg, model), 130_000);
}

#[test]
fn is_overflow_auto_off() {
    let mut cfg = cfg_default();
    cfg.auto = false;
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    let tokens = TokenCounts {
        total: 999_999,
        ..Default::default()
    };
    assert!(!is_overflow(&cfg, tokens, model));
}

#[test]
fn is_overflow_at_threshold() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    let usable_n = usable(&cfg, model);
    let tokens = TokenCounts {
        total: usable_n,
        ..Default::default()
    };
    assert!(is_overflow(&cfg, tokens, model));
    let tokens_below = TokenCounts {
        total: usable_n - 1,
        ..Default::default()
    };
    assert!(!is_overflow(&cfg, tokens_below, model));
}

#[test]
fn token_counts_count_uses_total_when_present() {
    let t = TokenCounts {
        total: 100,
        input: 50,
        output: 60,
        cache_read: 10,
        cache_write: 5,
    };
    assert_eq!(t.count(), 100); // total 优先
}

#[test]
fn token_counts_count_sums_when_total_zero() {
    let t = TokenCounts {
        total: 0,
        input: 50,
        output: 60,
        cache_read: 10,
        cache_write: 5,
    };
    assert_eq!(t.count(), 125);
}

// -- preserve_recent_budget ----------------------------------------------

#[test]
fn preserve_recent_budget_default_formula() {
    let cfg = cfg_default();
    // usable=80_000 → 80_000/4 = 20_000 → max(2_000, 20_000)=20_000 → min(8_000, 20_000) = 8_000
    assert_eq!(
        cfg.preserve_recent_budget(80_000),
        MAX_PRESERVE_RECENT_TOKENS
    );
    // usable=4_000 → 1_000 → max(2_000, 1_000)=2_000 → min(8_000, 2_000)=2_000
    assert_eq!(
        cfg.preserve_recent_budget(4_000),
        MIN_PRESERVE_RECENT_TOKENS
    );
    // usable=20_000 → 5_000 → max(2_000, 5_000)=5_000 → min(8_000, 5_000)=5_000
    assert_eq!(cfg.preserve_recent_budget(20_000), 5_000);
}

#[test]
fn preserve_recent_budget_override() {
    let mut cfg = cfg_default();
    cfg.preserve_recent_tokens = Some(12_345);
    assert_eq!(cfg.preserve_recent_budget(80_000), 12_345);
}

// -- prompt --------------------------------------------------------------

#[test]
fn summary_template_contains_all_sections() {
    let must = [
        "## Goal",
        "## Constraints & Preferences",
        "## Progress",
        "### Done",
        "### In Progress",
        "### Blocked",
        "## Key Decisions",
        "## Next Steps",
        "## Critical Context",
        "## Relevant Files",
        "Rules:",
        "<template>",
        "</template>",
    ];
    for m in must {
        assert!(SUMMARY_TEMPLATE.contains(m), "missing section: {m}");
    }
}

#[test]
fn build_prompt_no_previous() {
    let s = build_prompt(None, &[]);
    assert!(s.starts_with("Create a new anchored summary from the conversation history above."));
    assert!(s.contains(SUMMARY_TEMPLATE));
}

#[test]
fn build_prompt_with_previous() {
    let s = build_prompt(Some("OLD-SUMMARY"), &[]);
    assert!(s.starts_with("Update the anchored summary below"));
    assert!(s.contains("<previous-summary>\nOLD-SUMMARY\n</previous-summary>"));
    assert!(s.contains(SUMMARY_TEMPLATE));
}

#[test]
fn build_prompt_with_plugin_context() {
    let ctx = vec!["EXTRA1".to_string(), "EXTRA2".to_string()];
    let s = build_prompt(None, &ctx);
    assert!(s.contains("EXTRA1"));
    assert!(s.contains("EXTRA2"));
}

#[test]
fn continue_message_overflow_branch() {
    let s = build_continue_message(true);
    assert!(s.contains("previous request exceeded"));
    assert!(s.contains("Continue if you have next steps"));
}

#[test]
fn continue_message_normal_branch() {
    let s = build_continue_message(false);
    assert!(!s.contains("previous request exceeded"));
    assert!(s.starts_with("Continue if you have next steps"));
}

// -- algorithm: turns / select / prune ----------------------------------

/// 测试用 mock message 实现。
#[derive(Debug, Clone)]
struct M {
    id: u32,
    role: Role,
    /// user 消息是否带 compaction 标记
    is_compaction: bool,
    /// assistant 消息是否是摘要
    is_summary: bool,
    size: usize,
    tools: Vec<ToolOutputRef<u32>>,
}

impl M {
    fn user(id: u32, size: usize) -> Self {
        Self {
            id,
            role: Role::User,
            is_compaction: false,
            is_summary: false,
            size,
            tools: vec![],
        }
    }

```


**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\algorithm.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\message_view.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\byop_compaction\mod.rs

**Tool Call: Read file `app\src\ai\byop_compaction\algorithm.rs` (lines 1-240)**
Status: Completed

```warp/app/src/ai/byop_compaction/algorithm.rs
//! 压缩核心算法 — 1:1 移植 opencode `compaction.ts:141-341`(turns / select / splitTurn / prune)。
//!
//! 与 warp 的具体消息类型解耦:对外通过 [`MessageRef`] trait 抽象,
//! 真实实现见 `super::message_view`。
use std::hash::Hash;

use super::consts::{PRUNE_MINIMUM, PRUNE_PROTECT, PRUNE_PROTECTED_TOOLS};
use super::overflow::{usable, ModelLimit};
use super::CompactionConfig;

/// 消息的角色 — 用于 turn 检测与 select。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

/// 单条 tool 输出的元信息(prune 决策需要)。
#[derive(Debug, Clone)]
pub struct ToolOutputRef<CallId> {
    pub call_id: CallId,
    pub tool_name: String,
    /// 估算 token 数(对齐 opencode `Token.estimate(part.state.output)`)。
    pub output_size: usize,
    pub completed: bool,
    /// 已被 prune 标记 `compacted`,继续遍历时遇到要 break。
    pub already_compacted: bool,
}

/// 抽象的消息引用 — algorithm 只与本 trait 交互,与 warp 类型解耦。
pub trait MessageRef {
    type Id: Clone + Eq + Hash;
    type CallId: Clone + Eq + Hash;

    fn id(&self) -> Self::Id;
    fn role(&self) -> Role;

    /// user message 是否承载了一次 compaction 触发标记(opencode `parts.some(p => p.type === "compaction")`)。
    fn is_compaction_marker(&self) -> bool;

    /// assistant message 是否是摘要本身(opencode `info.summary === true`)。
    fn is_summary(&self) -> bool;

    /// 单条消息的 token 估算 — 实现可用 `serde_json` + `super::token::estimate`。
    fn estimate_size(&self) -> usize;

    /// 这条消息内的所有 tool outputs(prune 用)。assistant message 才会有。
    fn tool_outputs(&self) -> Vec<ToolOutputRef<Self::CallId>>;
}

/// `compaction.ts:76-80` 类型对应。
#[derive(Debug, Clone)]
pub struct Turn<Id> {
    pub start: usize,
    pub end: usize,
    pub id: Id,
}

/// `compaction.ts:82-85`。
#[derive(Debug, Clone)]
pub struct Tail<Id> {
    pub start: usize,
    pub id: Id,
}

/// `select` 返回值:`head` 是要送给摘要 LLM 的范围,`tail_start_id` 是保留段起点。
#[derive(Debug, Clone)]
pub struct SelectResult<Id> {
    pub head_end: usize,
    pub tail_start_id: Option<Id>,
}

/// `compaction.ts:141-157`。
pub fn turns<M: MessageRef>(messages: &[M]) -> Vec<Turn<M::Id>> {
    let mut result: Vec<Turn<M::Id>> = Vec::new();
    let n = messages.len();
    for (i, msg) in messages.iter().enumerate() {
        if msg.role() != Role::User {
            continue;
        }
        if msg.is_compaction_marker() {
            continue;
        }
        result.push(Turn {
            start: i,
            end: n,
            id: msg.id(),
        });
    }
    let len = result.len();
    if len > 1 {
        for i in 0..len - 1 {
            result[i].end = result[i + 1].start;
        }
    }
    result
}

/// `compaction.ts:159-182` splitTurn — 在 turn 内部找第一个能塞进 budget 的切点。
fn split_turn<M, EstFn>(
    messages: &[M],
    turn: &Turn<M::Id>,
    budget: usize,
    estimate: &EstFn,
) -> Option<Tail<M::Id>>
where
    M: MessageRef,
    EstFn: Fn(&[M]) -> usize,
{
    if budget == 0 {
        return None;
    }
    if turn.end.saturating_sub(turn.start) <= 1 {
        return None;
    }
    let mut start = turn.start + 1;
    while start < turn.end {
        let size = estimate(&messages[start..turn.end]);
        if size > budget {
            start += 1;
            continue;
        }
        return Some(Tail {
            start,
            id: messages[start].id(),
        });
    }
    None
}

/// `compaction.ts:244-293` select — 切出 head/tail。
///
/// `estimate_slice` 对应 opencode `estimate({ messages: slice, model })`。
/// 调用方传入因为它要决定如何把 message 列表序列化(JSON)再用 `Token.estimate`。
pub fn select<M, EstFn>(
    messages: &[M],
    cfg: &CompactionConfig,
    model: ModelLimit,
    estimate_slice: EstFn,
) -> SelectResult<M::Id>
where
    M: MessageRef,
    EstFn: Fn(&[M]) -> usize,
{
    let limit = cfg.tail_turns;
    if limit == 0 {
        return SelectResult {
            head_end: messages.len(),
            tail_start_id: None,
        };
    }
    let usable_tokens = usable(cfg, model);
    let budget = cfg.preserve_recent_budget(usable_tokens);
    let all = turns(messages);
    if all.is_empty() {
        return SelectResult {
            head_end: messages.len(),
            tail_start_id: None,
        };
    }
    let recent_start = all.len().saturating_sub(limit);
    let recent: Vec<&Turn<M::Id>> = all[recent_start..].iter().collect();
    let sizes: Vec<usize> = recent
        .iter()
        .map(|t| estimate_slice(&messages[t.start..t.end]))
        .collect();

    let mut total: usize = 0;
    let mut keep: Option<Tail<M::Id>> = None;
    for i in (0..recent.len()).rev() {
        let turn = recent[i];
        let size = sizes[i];
        if total + size <= budget {
            total += size;
            keep = Some(Tail {
                start: turn.start,
                id: turn.id.clone(),
            });
            continue;
        }
        let remaining = budget.saturating_sub(total);
        let split = split_turn(messages, turn, remaining, &estimate_slice);
        if split.is_some() {
            keep = split;
        }
        // 注意 opencode 的实现:首次 size 超 budget 就 break,无论 splitTurn 是否找到都不再尝试更早 turn。
        break;
    }

    match keep {
        None => SelectResult {
            head_end: messages.len(),
            tail_start_id: None,
        },
        Some(t) if t.start == 0 => SelectResult {
            head_end: messages.len(),
            tail_start_id: None,
        },
        Some(t) => SelectResult {
            head_end: t.start,
            tail_start_id: Some(t.id),
        },
    }
}

/// `compaction.ts:297-341` prune 决策 — 返回应被标记 `compacted` 的 (message_id, tool_call_id) 对。
///
/// 调用方据此写入 `CompactionState.markers`(实际 protobuf message 不动)。
pub fn prune_decisions<M: MessageRef>(messages: &[M]) -> Vec<(M::Id, M::CallId)> {
    let mut total: usize = 0;
    let mut pruned: usize = 0;
    let mut to_prune: Vec<(M::Id, M::CallId)> = Vec::new();
    let mut user_turns_seen: usize = 0;

    'outer: for msg in messages.iter().rev() {
        if msg.role() == Role::User {
            user_turns_seen += 1;
        }
        // 至少保留最近 2 个 user turn 不动(opencode `if (turns < 2) continue`)。
        if user_turns_seen < 2 {
            continue;
        }
        // 已是摘要边界 — 不再往前看。
        if msg.role() == Role::Assistant && msg.is_summary() {
            break 'outer;
        }
        let outputs = msg.tool_outputs();
        for tp in outputs.into_iter().rev() {
            if !tp.completed {
                continue;
            }
            if PRUNE_PROTECTED_TOOLS.contains(&tp.tool_name.as_str()) {
                continue;
            }
            if tp.already_compacted {
                break 'outer;
            }
            let estimate = tp.output_size;
            total += estimate;

```


**Tool Call: Read file `app\src\ai\byop_compaction\message_view.rs` (lines 1-240)**
Status: Completed

```warp/app/src/ai/byop_compaction/message_view.rs
//! 把 warp `api::Message` 序列适配为 [`MessageRef`] trait,供 [`super::algorithm`] 操作。
//!
//! ## 与 opencode `MessageV2.WithParts` 的语义映射
//!
//! opencode:一条 user/assistant message 含多个 parts(text/tool/file/...);
//! warp:一条 protobuf `api::Message` 是细粒度的(UserQuery / AgentReasoning / AgentOutput / ToolCall / ToolCallResult 各自独立)。
//!
//! 本投影**一对一**把 warp 的每条 `api::Message` 视为一个 `MessageRef`,
//! turn 检测仍按 user message 边界切 — 一个 user message 后跟连续的非 user message 就是一个 turn。
//! 这不影响 [`super::algorithm::turns`] / [`super::algorithm::select`] 算法的正确性。
//!
//! prune 决策针对 `Role::Tool`(ToolCallResult)— 每条 ToolCallResult 自己是一个候选。
//! 调用方需提前把 conversation 内所有 ToolCall 的 `tool_call_id → tool_name` 索引到 [`ToolNameLookup`]。

use std::collections::HashMap;

use warp_multi_agent_api as api;

use super::algorithm::{MessageRef, Role, ToolOutputRef};
use super::state::CompactionState;

/// `tool_call_id → tool_name` 索引,投影时用于:
/// 1. 给 ToolCallResult 标注 tool_name(用于 PRUNE_PROTECTED_TOOLS 判断)
/// 2. 让 prune 决策跳过 protected 工具(如 `skill`)
pub type ToolNameLookup = HashMap<String, String>;

/// 给定一组 tasks,提取所有 ToolCall 的 `(tool_call_id, tool_name)` 对。
pub fn build_tool_name_lookup<'a, I>(messages: I) -> ToolNameLookup
where
    I: IntoIterator<Item = &'a api::Message>,
{
    let mut out = ToolNameLookup::new();
    for msg in messages {
        if let Some(api::message::Message::ToolCall(tc)) = &msg.message {
            // 直接用 protobuf tool_call.tool 的 enum variant 名
            let name = tool_name_for(tc).unwrap_or_default();
            out.insert(tc.tool_call_id.clone(), name);
        }
    }
    out
}

/// 从 protobuf ToolCall 拿"工具名"。
///
/// 本投影只需要识别 [`PRUNE_PROTECTED_TOOLS`](`super::consts::PRUNE_PROTECTED_TOOLS`) 里的工具
/// (目前只有 "skill",对应 warp 的 `Tool::ReadSkill`),其他工具返回空串 — 在 prune 决策里
/// 空串不匹配任何 protected entry,行为正确(允许被 prune)。
fn tool_name_for(tc: &api::message::ToolCall) -> Option<String> {
    use api::message::tool_call::Tool;
    let t = tc.tool.as_ref()?;
    let s = match t {
        Tool::ReadSkill(_) => "skill",
        _ => "",
    };
    Some(s.to_string())
}

/// 单条 `api::Message` 的视图。
#[derive(Clone, Copy)]
pub struct WarpMessageView<'a> {
    pub msg: &'a api::Message,
    pub state: &'a CompactionState,
    pub tool_names: &'a ToolNameLookup,
}

/// 估算单条 message 的 token 占用 — 累加可见文本字符数 / 4。
fn estimate_message(msg: &api::Message) -> usize {
    use super::token::estimate;
    use api::message::Message as M;
    let chars = msg
        .message
        .as_ref()
        .map(|inner| match inner {
            M::UserQuery(u) => u.query.chars().count(),
            M::AgentOutput(a) => a.text.chars().count(),
            M::AgentReasoning(r) => r.reasoning.chars().count(),
            M::ToolCall(_) => msg.server_message_data.chars().count().max(64),
            M::ToolCallResult(tcr) => {
                // 优先用 result oneof 的 estimate;fallback 用 server_message_data。
                // 简化:都按字符数算,result.estimate 走 Debug repr。
                let from_oneof = tcr
                    .result
                    .as_ref()
                    .map(|r| format!("{r:?}").chars().count())
                    .unwrap_or(0);
                from_oneof
                    .max(msg.server_message_data.chars().count())
                    .max(32)
            }
            _ => 0,
        })
        .unwrap_or(0);
    // 与 opencode 同算法:chars / 4 round。
    estimate(&" ".repeat(chars))
}

impl<'a> MessageRef for WarpMessageView<'a> {
    type Id = String;
    type CallId = String;

    fn id(&self) -> String {
        self.msg.id.clone()
    }

    fn role(&self) -> Role {
        use api::message::Message as M;
        match &self.msg.message {
            Some(M::UserQuery(_)) => Role::User,
            Some(M::ToolCallResult(_)) => Role::Tool,
            // AgentOutput / AgentReasoning / ToolCall / 其他 → Assistant
            _ => Role::Assistant,
        }
    }

    fn is_compaction_marker(&self) -> bool {
        // 只有 user 消息且带 compaction_trigger marker 才算
        if self.role() != Role::User {
            return false;
        }
        self.state
            .marker(&self.msg.id)
            .map(|m| m.compaction_trigger.is_some())
            .unwrap_or(false)
    }

    fn is_summary(&self) -> bool {
        // 只有 assistant message 才能是摘要
        if self.role() != Role::Assistant {
            return false;
        }
        self.state
            .marker(&self.msg.id)
            .map(|m| m.is_summary)
            .unwrap_or(false)
    }

    fn estimate_size(&self) -> usize {
        estimate_message(self.msg)
    }

    fn tool_outputs(&self) -> Vec<ToolOutputRef<String>> {
        let Some(api::message::Message::ToolCallResult(tcr)) = &self.msg.message else {
            return Vec::new();
        };
        let tool_name = self
            .tool_names
            .get(&tcr.tool_call_id)
            .cloned()
            .unwrap_or_default();
        let already_compacted = self
            .state
            .marker(&self.msg.id)
            .and_then(|m| m.tool_output_compacted_at)
            .is_some();
        // output_size 复用 estimate_message — ToolCallResult 路径会走 result/server_message_data 的字符数
        let output_size = estimate_message(self.msg);
        vec![ToolOutputRef {
            call_id: tcr.tool_call_id.clone(),
            tool_name,
            output_size,
            completed: tcr.result.is_some() || !self.msg.server_message_data.is_empty(),
            already_compacted,
        }]
    }
}

/// 把一组 messages 投影成 `Vec<WarpMessageView>`,按 timestamp 升序排序 —
/// 与 [`crate::ai::agent_providers::chat_stream::build_chat_request`] 的排序保持一致。
pub fn project<'a>(
    messages: &'a [&'a api::Message],
    state: &'a CompactionState,
    tool_names: &'a ToolNameLookup,
) -> Vec<WarpMessageView<'a>> {
    let mut sorted: Vec<&api::Message> = messages.iter().copied().collect();
    sorted.sort_by_key(|m| {
        m.timestamp
            .as_ref()
            .map(|ts| (ts.seconds, ts.nanos))
            .unwrap_or((0, 0))
    });
    sorted
        .into_iter()
        .map(|msg| WarpMessageView {
            msg,
            state,
            tool_names,
        })
        .collect()
}

```


**Tool Call: Read file `app\src\ai\byop_compaction\mod.rs` (lines 1-120)**
Status: Completed

```warp/app/src/ai/byop_compaction/mod.rs
//! BYOP 本地会话压缩 — 1:1 复刻 opencode `packages/opencode/src/session/{compaction,overflow,summary}.ts`。
//!
//! 入口 API:
//! - [`overflow::is_overflow`] — 自动触发判断(基于 LLM response usage)
//! - [`algorithm::select`] — 切分 head(送摘要 LLM) + tail(原样保留)
//! - [`algorithm::prune`] — 只清旧 tool output(不删消息)
//! - [`prompt::build_prompt`] — 拼摘要请求文本
//!
//! 与 warp 服务端 protobuf `SummarizeConversation` 解耦,只在 BYOP 路径生效。
pub mod algorithm;
pub mod commit;
pub mod config;
pub mod message_view;
pub mod overflow;
pub mod prompt;
pub mod state;
pub mod token;

pub use config::CompactionConfig;
pub use overflow::{is_overflow, usable};

/// 字节级对齐 opencode `compaction.ts` 顶部常数(行 33-39, overflow.ts:6, util/token.ts:1)。
pub mod consts {
    pub const PRUNE_MINIMUM: usize = 20_000;
    pub const PRUNE_PROTECT: usize = 40_000;
    pub const TOOL_OUTPUT_MAX_CHARS: usize = 2_000;
    pub const DEFAULT_TAIL_TURNS: usize = 2;
    pub const MIN_PRESERVE_RECENT_TOKENS: usize = 2_000;
    pub const MAX_PRESERVE_RECENT_TOKENS: usize = 8_000;
    pub const COMPACTION_BUFFER: usize = 20_000;
    pub const CHARS_PER_TOKEN: usize = 4;
    pub const PRUNE_PROTECTED_TOOLS: &[&str] = &["skill"];
}

#[cfg(test)]
mod tests;

```


**Tool Call: Read file `app\src\ai\blocklist\controller.rs` (lines 2880-2940)**
Status: Completed

```warp/app/src/ai/blocklist/controller.rs
        did_request_contain_user_query: bool,
        summarize_overflow: Option<bool>,
        ctx: &mut ModelContext<Self>,
    ) {
        // OpenWarp BYOP 本地会话压缩:在 token_usage move 进下面 closure 前先聚合,
        // 用于 auto overflow 检查(后面 Done 分支用)。
        let aggregate_token_count: usize = finished_event
            .token_usage
            .iter()
            .map(|u| (u.total_input + u.output + u.input_cache_read + u.input_cache_write) as usize)
            .max()
            .unwrap_or(0);

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        history_model.update(ctx, |history_model, _| {
            // Update conversation cost and usage information before updating and
            // persisting the conversation.
            history_model.update_conversation_cost_and_usage_for_request(
                conversation_id,
                finished_event
                    .request_cost
                    .map(|cost| RequestCost::new(cost.exact.into())),
                finished_event.token_usage,
                finished_event.conversation_usage_metadata.take(),
                did_request_contain_user_query,
            );
        });

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        match finished_event.reason {
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::Done(_)) | None => {
                // OpenWarp BYOP 本地会话压缩 - 写回 summary
                if let Some(overflow) = summarize_overflow {
                    history_model.update(ctx, |history_model, _ctx| {
                        if let Some(convo) = history_model.conversation_mut(&conversation_id) {
                            crate::ai::byop_compaction::commit::commit_summarization(
                                convo, overflow,
                            );
                        }
                    });
                }
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_successfully(
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });

                // OpenWarp BYOP 本地会话压缩 - auto overflow 触发(对齐 opencode `processor.ts:395-403`)
                // 仅在本流不是摘要本身时检查,防止递归。
                if summarize_overflow.is_none() {
                    let aggregate_count = aggregate_token_count;
                    if aggregate_count > 0 {
                        let cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
                        let model_limit =
                            crate::ai::byop_compaction::overflow::ModelLimit::FALLBACK;
                        let counts = crate::ai::byop_compaction::overflow::TokenCounts {
                            total: aggregate_count,
                            ..Default::default()

```


**Tool Call: Read file `app\src\ai\agent_providers\chat_stream.rs` (lines 540-630)**
Status: Completed

```warp/app/src/ai/agent_providers/chat_stream.rs

    let mut buf = AssistantBuffer::new(force_echo_reasoning);
    // OpenWarp:历史里被 skip 掉的 subagent ToolCall 对应的 call_id —— 它们的
    // ToolCallResult 也必须 skip,否则会成为孤儿 tool_response,Anthropic 直接 400
    // `unexpected tool_use_id ... no corresponding tool_use block`。
    let mut skipped_subagent_call_ids: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (idx, msg) in all_msgs.iter().enumerate() {
        // 摘要请求:tail 区间不送上游(只送 head + 末尾追加 SUMMARY_TEMPLATE)
        if let Some(head_end) = summarize_head_end {
            if idx >= head_end {
                continue;
            }
        }
        let Some(inner) = &msg.message else {
            continue;
        };
        match inner {
            api::message::Message::UserQuery(u) => {
                // 压缩投影:hidden 区间的 user message 替换为合成的"以下为已压缩历史的摘要"对
                if hidden_msg_ids.contains(&msg.id) {
                    if let Some(summary_text) = summary_inserts.get(&msg.id) {
                        buf.flush_into(&mut messages);
                        messages.push(ChatMessage::user(
                            "Conversation history was compacted. Below is the structured summary of all prior turns.".to_string(),
                        ));
                        messages.push(ChatMessage::assistant(summary_text.clone()));
                    }
                    // 没有 summary_text 的 hidden user 直接 skip(不应该发生,防御性)
                    continue;
                }
                buf.flush_into(&mut messages);
                // OpenWarp:历史轮多模态保活。warp 自家路径靠云端 server 重注入 InputContext,
                // BYOP 直连没有那层,所以 `make_user_query_message` 持久化时把所有 binary
                // (image / pdf / audio)塞进了 `UserQuery.context.images`,这里反向恢复成
                // UserBinary 走 `build_user_message_with_binaries`,使后续轮模型仍能看到先前
                // 粘贴的多模态附件。模型 caps 不支持的 mime 由 build_user_message_with_binaries
                // 替换为 ERROR 文本(opencode unsupportedParts 风格),不会静默 drop。
                // 没有 binary → 退回老路 `ChatMessage::user(text)`,与修复前等价。
                let history_binaries: Vec<user_context::UserBinary> = u
                    .context
                    .as_ref()
                    .map(|ctx| {
                        ctx.images
                            .iter()
                            .filter(|b| !b.data.is_empty())
                            .enumerate()
                            .map(|(idx, b)| {
                                use base64::Engine;
                                user_context::UserBinary {
                                    name: format!("history-attachment-{}-{idx}", &msg.id),
                                    content_type: if b.mime_type.is_empty() {
                                        "application/octet-stream".to_string()
                                    } else {
                                        b.mime_type.clone()
                                    },
                                    data: base64::engine::general_purpose::STANDARD.encode(&b.data),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if history_binaries.is_empty() {
                    messages.push(ChatMessage::user(u.query.clone()));
                } else {
                    messages.push(build_user_message_with_binaries(
                        u.query.clone(),
                        history_binaries,
                        api_type,
                        model_id,
                    ));
                }
            }
            // hidden assistant message 直接 skip(它是某次压缩对的 assistant_msg_id,
            // 摘要文本已经在对应 user 分支注入)
            api::message::Message::AgentReasoning(_) | api::message::Message::AgentOutput(_)
                if hidden_msg_ids.contains(&msg.id) =>
            {
                continue;
            }
            api::message::Message::AgentReasoning(r) => {
                // 把上一轮的 reasoning 挂到下一个要 flush 的 assistant message 上。
                // genai 0.6 的 with_reasoning_content 会按当前 adapter 序列化:
                // DeepSeek/Kimi → reasoning_content 字段;Anthropic → thinking blocks。
                // 多段 AgentReasoning 累加(同一 turn 可能 stream 出多个 reasoning chunk
                // 落地为多条 AgentReasoning)。
                let next = r.reasoning.clone();
                if !next.is_empty() {
                    match buf.reasoning.as_mut() {
                        Some(existing) => existing.push_str(&next),

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

app\src\server\server_api\ai.rs
 1519: let operation = DeleteAIConversation::build(variables);
app\src\ai\agent\api\convert_conversation.rs
 106: match AIConversation::new_restored(
app\src\terminal\view\pane_impl.rs
 1005: .and_then(AIConversation::latest_user_query)
app\src\ai\blocklist\orchestration_event_streamer_tests.rs
 150: AIConversation::new_restored(AIConversationId::new(), vec![task], Some(data))
 209: let conversation = AIConversation::new(false);
 262: let mut conversation = AIConversation::new(false);
 359: let mut conversation = AIConversation::new(false);
 447: let mut conversation = AIConversation::new(false);
 520: let mut conversation = AIConversation::new(false);
specs\APP-3630\TECH.md
 11: …tionArtifacts` (history_model.rs:2003) is emitted by `AIConversation::add_artifact` and `update_plan_notebook_uid`. It carries `terminal_view_id` and `conversation_id` but not the …

**Tool Call: Run MCP tool `grep`**
Status: Completed

0 exact matches. 7 approximate:
app\src\ai\blocklist\controller.rs
 2915:                             crate::ai::byop_compaction::commit::commit_summarization(
 2949:                             // commit_summarization 写回时也以 overflow=true 落 state(便于 UI 区分)。
app\src\ai\blocklist\controller\response_stream.rs
 303:     /// Done 分支据此调 commit_summarization 把摘要落到 conversation.compaction_state。

**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent_providers\chat_stream.rs (best match)
app\src\ai\agent_providers\chat_stream.rs
 457: // OpenWarp BYOP 本地会话压缩:把 conversation.compaction_state 应用到 message 序列。
 479: if let Some(state) = params.compaction_state.as_ref() {
 494: .compaction_state
 499: .compaction_state
 522: let state_for_select = params.compaction_state.clone().unwrap_or_default();
 813: // 后把它写回 conversation.compaction_state(参见 Phase 6 controller 改动)。
 815: .compaction_state
app\src\ai\agent\conversation.rs
 235: pub(crate) compaction_state: crate::ai::byop_compaction::state::CompactionState,
 286: compaction_state: Default::default(),
 468: compaction_state: Default::default(),
app\src\ai\agent\api.rs
 140: /// OpenWarp BYOP 本地会话压缩 sidecar 快照(controller 把 conversation.compaction_state.clone() 塞进来)。
 148: pub compaction_state: Option<crate::ai::byop_compaction::state::CompactionState>,
 372: compaction_state: None,

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\agent\conversation.rs

**Tool Call: 1 match**
Status: Completed

file://C:\Users\zero\Desktop\code\zerx-lab\warp\app\src\ai\blocklist\orchestration_event_streamer_tests.rs

**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 220-330)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
    /// True when this conversation is a placeholder for a child agent executing
    /// on a remote worker. The parent's client does not drive execution for
    /// these conversations — the remote worker's own client handles status
    /// reporting. TaskStatusSyncModel skips status updates for these.
    is_remote_child: bool,

    /// The last event sequence number observed from the v2 orchestration
    /// event log. Used on restore to resume event delivery without
    /// re-delivering already-processed events.
    last_event_sequence: Option<i64>,

    /// OpenWarp BYOP 本地会话压缩 sidecar — 与 warp protobuf message 解耦,
    /// 通过 message_id 索引挂"is_summary / tool_output_compacted_at / synthetic_continue"等元数据。
    /// 默认空表 = 未压缩状态,完全无侵入。
    /// 详见 [`crate::ai::byop_compaction`]。
    pub(crate) compaction_state: crate::ai::byop_compaction::state::CompactionState,
}

pub(crate) fn artifact_from_fork_proto(
    proto_artifact: &api::message::artifact_event::ConversationArtifact,
) -> Option<Artifact> {
    use api::message::artifact_event::conversation_artifact::Artifact as ProtoArtifact;

    match &proto_artifact.artifact {
        Some(ProtoArtifact::PullRequest(pr)) => Some(Artifact::from(pr.clone())),
        Some(ProtoArtifact::Screenshot(ss)) => Some(Artifact::from(ss.clone())),
        Some(ProtoArtifact::Plan(plan)) => Some(Artifact::from(plan.clone())),
        Some(ProtoArtifact::File(file)) => Some(Artifact::from(file.clone())),
        None => None,
    }
}

impl AIConversation {
    pub fn new(is_viewing_shared_session: bool) -> Self {
        let root_task = Task::new_optimistic_root();
        Self {
            id: AIConversationId::new(),
            task_store: TaskStore::with_root_task(root_task),
            optimistic_cli_subagent_subtask_id: None,
            code_review: None,
            is_viewing_shared_session,
            todo_lists: vec![],
            status: ConversationStatus::InProgress,
            status_error_message: None,
            has_opened_code_review: false,
            conversation_usage_metadata: ConversationUsageMetadata::default(),
            server_conversation_token: None,
            task_id: None,
            forked_from_server_conversation_token: None,
            server_metadata: None,
            transaction: None,
            autoexecute_override: Default::default(),
            added_exchanges_by_response: Default::default(),
            hidden_exchanges: Default::default(),
            reverted_action_ids: Default::default(),
            existing_suggestions: None,
            dismissed_suggestion_ids: Default::default(),
            total_request_cost: RequestCost::new(0.),
            total_token_usage_by_model: Default::default(),
            fallback_display_title: None,
            artifacts: Vec::new(),
            parent_agent_id: None,
            agent_name: None,
            parent_conversation_id: None,
            is_remote_child: false,
            last_event_sequence: None,
            compaction_state: Default::default(),
        }
    }

    // TODO: derive todo list state from tasks instead of taking args.
    //
    // This would make it possible to fully restore a convo from tasks, instead of having to persist this additional data.
    pub fn new_restored(
        id: AIConversationId,
        tasks: Vec<api::Task>,
        conversation_data: Option<AgentConversationData>,
    ) -> Result<Self, RestoreConversationError> {
        let api_tasks_by_id: HashMap<String, api::Task> =
            tasks.into_iter().map(|t| (t.id.clone(), t)).collect();

        // To process a task, we need to reference some of the data in its parent task.  To
        // avoid cloning, we process the task tree from deepest tasks to shallowest tasks.  This
        // ensures that children are always processed before their parents, avoiding any need to
        // clone task data to ensure the parent is available when processing the child.
        let depths = compute_task_depths(&api_tasks_by_id);
        let mut task_ids: Vec<String> = api_tasks_by_id.keys().cloned().collect();
        task_ids.sort_by(|a, b| {
            depths
                .get(b.as_str())
                .unwrap_or(&0)
                .cmp(depths.get(a.as_str()).unwrap_or(&0))
        });

        let mut api_tasks_and_exchanges_by_id: HashMap<_, _> = api_tasks_by_id
            .into_iter()
            .map(|(id, task)| {
                let exchanges = task.into_exchanges();
                (id, (task, exchanges))
            })
            .collect();

        let mut tasks_by_id = HashMap::new();
        let mut root_task = None;
        for task_id in task_ids {
            let Some((task, exchanges)) = api_tasks_and_exchanges_by_id.remove(&task_id) else {
                continue;
            };

            if let Some(parent_id) = task.parent_id() {
                if let Some((parent_task, _)) = api_tasks_and_exchanges_by_id.get(parent_id) {

```


**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 430-490)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs

        // Determine the correct status based on the exchanges before constructing
        let status = Self::derive_status_from_root_task(&tasks_by_id.get(&root_task_id));

        let task_store = TaskStore::from_tasks(tasks_by_id, root_task_id);

        Ok(Self {
            id,
            is_viewing_shared_session: false,
            task_store,
            status,
            status_error_message: None,
            todo_lists,
            // TODO(alokedesai): Support session restoration for code review comments.
            code_review: None,
            has_opened_code_review: false,
            conversation_usage_metadata,
            server_conversation_token,
            task_id: run_id.as_deref().and_then(|id| id.parse().ok()),
            forked_from_server_conversation_token,
            server_metadata: None,
            transaction: None,
            autoexecute_override,
            added_exchanges_by_response: Default::default(),
            existing_suggestions: None,
            hidden_exchanges: Default::default(),
            reverted_action_ids,
            dismissed_suggestion_ids: Default::default(),
            total_request_cost: RequestCost::new(0.),
            total_token_usage_by_model: Default::default(),
            optimistic_cli_subagent_subtask_id: None,
            fallback_display_title: None,
            artifacts,
            parent_agent_id,
            agent_name,
            parent_conversation_id,
            is_remote_child: false,
            last_event_sequence,
            compaction_state: Default::default(),
        })
    }

    pub fn id(&self) -> AIConversationId {
        self.id
    }

    /// Assigns fresh exchange IDs to all exchanges in this conversation.
    /// Used when forking conversations to avoid ID collisions with persisted blocks.
    pub fn reassign_exchange_ids(&mut self) {
        let task_ids: Vec<TaskId> = self.task_store.tasks().map(|t| t.id().clone()).collect();
        for task_id in task_ids {
            self.task_store.modify_task(&task_id, |task| {
                task.reassign_exchange_ids();
            });
        }
    }

    pub fn is_viewing_shared_session(&self) -> bool {
        self.is_viewing_shared_session
    }


```


**Tool Call: Read file `app\src\ai\blocklist\orchestration_event_streamer_tests.rs` (lines 190-280)**
Status: Completed

```warp/app/src/ai/blocklist/orchestration_event_streamer_tests.rs
    }
}

#[test]
fn finish_restore_fetch_uses_server_cursor_when_sqlite_is_absent() {
    use crate::ai::agent::conversation::AIConversation;
    use crate::server::server_api::ai::MockAIClient;
    use crate::server::server_api::ServerApiProvider;
    use std::sync::Arc;
    use warpui::App;

    App::test((), |mut app| async move {
        let _v2_guard = FeatureFlag::OrchestrationV2.override_enabled(true);

        let history_model = app.add_singleton_model(|_| BlocklistAIHistoryModel::new(vec![], &[]));

        // Restore a conversation with no SQLite cursor (`last_event_sequence:
        // None`). After the server fetch completes with `Some(42)` we expect
        // the in-memory cursor to be 42 (max(0, 42)).
        let conversation = AIConversation::new(false);
        let conversation_id = conversation.id();
        let terminal_view_id = warpui::EntityId::new();
        history_model.update(&mut app, |model, ctx| {
            model.restore_conversations(terminal_view_id, vec![conversation], ctx);
        });

        let mock = MockAIClient::new();
        let ai_client: Arc<dyn AIClient> = Arc::new(mock);
        let server_api = ServerApiProvider::new_for_test().get();

        let poller = app.add_singleton_model(|ctx| {
            OrchestrationEventStreamer::new_with_clients_for_test(ai_client, server_api, ctx)
        });

        // Seed event_cursor as on_restored_conversations would before spawning
        // the async fetch. Without this the guard that detects mid-flight
        // conversation deletion would fire and return early.
        poller.update(&mut app, |me, _| {
            me.event_cursor.insert(conversation_id, 0);
        });

        let task_id: crate::ai::ambient_agents::AmbientAgentTaskId =
            "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
        poller.update(&mut app, |me, ctx| {
            me.finish_restore_fetch(
                conversation_id,
                task_id,
                /* sqlite_cursor */ 0,
                Ok(make_ambient_task_with_event_seq(Some(42))),
                ctx,
            );
        });

        poller.read(&app, |me, _| {
            assert_eq!(me.event_cursor.get(&conversation_id).copied(), Some(42));
        });
    });
}

#[test]
fn restored_inprogress_parent_defers_delivery_until_success() {
    use crate::ai::agent::conversation::{AIConversation, AIConversationId, ConversationStatus};
    use crate::server::server_api::ai::MockAIClient;
    use crate::server::server_api::ServerApiProvider;
    use std::sync::Arc;
    use warpui::App;

    App::test((), |mut app| async move {
        let _v2_guard = FeatureFlag::OrchestrationV2.override_enabled(true);

        let history_model = app.add_singleton_model(|_| BlocklistAIHistoryModel::new(vec![], &[]));

        let mut conversation = AIConversation::new(false);
        // Use a parsable UUID-shaped run_id so the poller can construct
        // an `AmbientAgentTaskId` for the (mocked) server fetch.
        conversation.set_run_id("550e8400-e29b-41d4-a716-446655440100".to_string());
        let conversation_id: AIConversationId = conversation.id();
        let terminal_view_id = warpui::EntityId::new();
        history_model.update(&mut app, |model, ctx| {
            model.restore_conversations(terminal_view_id, vec![conversation], ctx);
            // The default status after restore is `InProgress` for live
            // conversations, but assert it explicitly to make the test
            // self-documenting.
            model.update_conversation_status(
                terminal_view_id,
                conversation_id,
                ConversationStatus::InProgress,
                ctx,
            );
        });


```


**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 700-780)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
        self.added_exchanges_by_response
            .get(stream_id)
            .into_iter()
            .flat_map(|added_exchanges| {
                added_exchanges
                    .iter()
                    .map(|new_exchange| new_exchange.exchange_id)
            })
    }

    pub fn server_conversation_token(&self) -> Option<&ServerConversationToken> {
        self.server_conversation_token.as_ref()
    }

    /// Returns the server-assigned run identifier as a string.
    pub fn run_id(&self) -> Option<String> {
        self.task_id.map(|id| id.to_string())
    }

    /// Sets the task ID by parsing a run_id string.
    pub fn set_run_id(&mut self, id: String) {
        self.task_id = id.parse().ok();
    }

    /// Returns the server-assigned task ID, if available.
    pub fn task_id(&self) -> Option<AmbientAgentTaskId> {
        self.task_id
    }

    /// Sets the task ID directly (used for child agents spawned via `SpawnAgentResponse`).
    pub fn set_task_id(&mut self, id: AmbientAgentTaskId) {
        self.task_id = Some(id);
    }

    /// Returns the server-side agent identifier appropriate for the active
    /// orchestration version: `task_id` (as string) under v2,
    /// `server_conversation_token` under v1.
    pub fn orchestration_agent_id(&self) -> Option<String> {
        if FeatureFlag::OrchestrationV2.is_enabled() {
            self.run_id()
        } else {
            self.server_conversation_token
                .as_ref()
                .map(|t| t.as_str().to_string())
        }
    }

    /// Updates the server conversation token for this conversation.
    ///
    /// This is used internally for session sharing when a forked conversation receives
    /// its new server-assigned token. The viewer needs to update the conversation's token
    /// from the original (forked-from) token to the new token so subsequent messages can
    /// be matched to the correct conversation.
    ///
    /// This should only be called by session sharing viewer logic when linking forked conversations.
    pub(crate) fn set_server_conversation_token(&mut self, token: String) {
        self.server_conversation_token = Some(ServerConversationToken::new(token));
    }

    pub fn forked_from_server_conversation_token(&self) -> Option<&ServerConversationToken> {
        self.forked_from_server_conversation_token.as_ref()
    }

    /// Clears the forked_from token after the first Init event has been sent to viewers.
    /// This ensures we only send the forked_from token once during session sharing.
    pub(crate) fn clear_forked_from_server_conversation_token(&mut self) {
        self.forked_from_server_conversation_token = None;
    }

    pub fn server_id(&self) -> Option<ServerId> {
        self.server_metadata
            .as_ref()
            .map(|metadata| metadata.metadata.uid)
    }

    pub fn server_metadata(&self) -> Option<&ServerAIConversationMetadata> {
        self.server_metadata.as_ref()
    }

    pub fn set_server_metadata(&mut self, metadata: ServerAIConversationMetadata) {
        self.server_metadata = Some(metadata);

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent\conversation.rs (only match)
app\src\ai\agent\conversation.rs
 837: pub fn all_linearized_messages(&self) -> Vec<&api::Message> {
  838| self.task_store.all_linearized_messages()
  839| }
 838: self.task_store.all_linearized_messages()

**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 820-900)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
    }

    /// Returns true if this is a placeholder for a child agent executing on a
    /// remote worker. The parent's client should not report task status for
    /// these — the remote worker handles it.
    pub fn is_remote_child(&self) -> bool {
        self.is_remote_child
    }

    /// Marks this conversation as a remote child placeholder.
    pub fn mark_as_remote_child(&mut self) {
        self.is_remote_child = true;
    }

    /// Returns a flat list of linearized messages across all tasks, interpolating subtask messages
    /// in between subagent tool calls and results, effectively corresponding to the order in which
    /// the messages were created and added to the conversation.
    pub fn all_linearized_messages(&self) -> Vec<&api::Message> {
        self.task_store.all_linearized_messages()
    }

    /// Returns all the tasks in this conversation.
    ///
    /// Note that until we've fully migrated to the multi-agent endpoint, in reality, each
    /// conversation is comprised of a single task (the legacy endpoint `GenerateAIAgentOutput` does
    /// not support multiple tasks within a conversation).
    pub fn all_tasks(&self) -> impl Iterator<Item = &Task> {
        self.task_store.tasks()
    }

    /// Returns the set of tasks that are still active (relevant to the agent).
    ///
    /// This filters the full task list using DFS linearization to determine
    /// which tasks have open subagent tool calls without corresponding results.
    pub fn compute_active_tasks(&self) -> Vec<warp_multi_agent_api::Task> {
        use std::collections::HashMap;

        let root_task_id = self.get_root_task_id().to_string();
        let all_tasks: HashMap<&str, &warp_multi_agent_api::Task> = self
            .all_tasks()
            .filter_map(|task| {
                let source = task.source()?;
                Some((source.id.as_str(), source))
            })
            .collect();
        let active_task_ids =
            crate::ai::agent::linearization::compute_active_task_ids(&root_task_id, &all_tasks);
        all_tasks
            .into_values()
            .filter(|task| active_task_ids.contains(task.id.as_str()))
            .cloned()
            .collect()
    }

    /// Returns the titles from the CreateDocuments request corresponding to the given action ID (if any).
    /// This is used by shared-session viewers to use the correct document titles from the original CreateDocuments action.
    pub fn get_document_titles_for_action(
        &self,
        action_id: &AIAgentActionId,
    ) -> Option<Vec<String>> {
        for exchange in self.all_exchanges() {
            let Some(output) = exchange.output_status.output() else {
                continue;
            };

            for message in &output.get().messages {
                if let AIAgentOutputMessage {
                    message: AIAgentOutputMessageType::Action(action),
                    ..
                } = message
                {
                    if &action.id == action_id {
                        if let super::AIAgentActionType::CreateDocuments(
                            super::CreateDocumentsRequest { documents },
                        ) = &action.action
                        {
                            let titles = documents
                                .iter()
                                .map(|doc| doc.title.clone())
                                .collect::<Vec<_>>();
                            return Some(titles);

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent_providers\chat_stream.rs (best match)
app\src\ai\agent_providers\chat_stream.rs
 1828: //   1. AddMessagesToTask{UserQuery}    ← 当前轮所有 UserQuery input
 1829: //   2. AddMessagesToTask{ToolCallResult} ← 当前轮所有 ActionResult input
 2009: //   a) AddMessagesToTask(root, [<虚拟 subagent tool_call>])
 2456: // 第二帧不能再用 AddMessagesToTask —— 那会往 task.messages 追加第二条
 2760: action: Some(api::client_action::Action::AddMessagesToTask(
 2761: api::client_action::AddMessagesToTask {
app\src\ai\agent\conversation.rs
 2201: Action::AddMessagesToTask(AddMessagesToTask { task_id, messages }) => {

**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 2160-2235)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
                    let root_task_id = self.task_store.root_task_id().clone();
                    if let Some(mut root_task) = self.task_store.remove(&root_task_id) {
                        let old_id = root_task.id().clone();
                        root_task = root_task.into_server_created_task(
                            task,
                            None,
                            self.todo_lists.last(),
                            self.code_review.as_ref(),
                        )?;
                        ctx.emit(BlocklistAIHistoryEvent::UpgradedTask {
                            optimistic_id: old_id,
                            server_id: root_task.id().clone(),
                            terminal_view_id,
                        });

                        for AddedExchange {
                            ref mut task_id, ..
                        } in self
                            .added_exchanges_by_response
                            .get_mut(response_stream_id)
                            .ok_or(UpdateConversationError::NoPendingRequest)?
                            .iter_mut()
                        {
                            if *task_id == root_task_id {
                                *task_id = root_task.id().clone();
                            }
                        }
                        self.task_store.set_root_task(root_task);
                    }
                }
            }
            Action::UpdateTaskDescription(UpdateTaskDescription {
                task_id,
                description,
            }) => {
                let task_id = TaskId::new(task_id);
                self.checkpoint_task(&task_id);
                self.task_store
                    .modify_task(&task_id, |task| task.update_description(description))
                    .ok_or(UpdateConversationError::TaskNotFound)?;
            }
            Action::AddMessagesToTask(AddMessagesToTask { task_id, messages }) => {
                for message in messages.iter() {
                    match message.message.as_ref() {
                        Some(api::message::Message::UpdateTodos(update)) => {
                            if let Some(todos_op) = update.operation.as_ref() {
                                update_todo_list_from_todo_op(
                                    &mut self.todo_lists,
                                    todos_op.clone(),
                                );
                                ctx.emit(BlocklistAIHistoryEvent::UpdatedTodoList {
                                    terminal_view_id,
                                });
                            }
                        }
                        Some(api::message::Message::UpdateReviewComments(comments)) => {
                            if let Some(comments_op) = comments.operation.as_ref() {
                                if let Some(active_code_review) = self.code_review.as_mut() {
                                    let resolved_count = update_comment_from_comment_operation(
                                        active_code_review,
                                        comments_op.clone(),
                                    );
                                    if resolved_count > 0 {
                                        send_telemetry_from_ctx!(
                                            CodeReviewTelemetryEvent::CommentResolved {
                                                resolved_count
                                            },
                                            ctx
                                        );
                                    }
                                } else {
                                    log::error!(
                                        "Received an UpdateReviewComments message but there's no active code review state"
                                    );
                                }
                            }

```


**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 2235-2295)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
                            }
                        }
                        Some(api::message::Message::ArtifactEvent(artifact_event)) => {
                            match &artifact_event.event {
                                Some(api::message::artifact_event::Event::Created(
                                    artifact_created,
                                )) => {
                                    match &artifact_created.artifact {
                                        Some(
                                            api::message::artifact_event::artifact_created::Artifact::PullRequest(pr),
                                        ) => {
                                            self.add_artifact(
                                                Artifact::from(pr.clone()),
                                                terminal_view_id,
                                                ctx,
                                            );
                                        }
                                        Some(
                                            api::message::artifact_event::artifact_created::Artifact::Screenshot(screenshot),
                                        ) => {
                                            self.add_artifact(
                                                Artifact::from(screenshot.clone()),
                                                terminal_view_id,
                                                ctx,
                                            );
                                        }
                                        Some(
                                            api::message::artifact_event::artifact_created::Artifact::File(file),
                                        ) => {
                                            self.add_artifact(
                                                Artifact::from(file.clone()),
                                                terminal_view_id,
                                                ctx,
                                            );
                                        }
                                        None => {}
                                    }
                                }
                                Some(api::message::artifact_event::Event::ForkArtifacts(
                                    fork_artifacts,
                                )) => {
                                    for proto_artifact in &fork_artifacts.artifacts {
                                        let Some(artifact) =
                                            artifact_from_fork_proto(proto_artifact)
                                        else {
                                            continue;
                                        };
                                        self.add_artifact(artifact, terminal_view_id, ctx);
                                    }
                                }
                                None => {}
                            }
                        }
                        Some(api::message::Message::ToolCallResult(tcr)) => {
                            // Clean up temp directories from conversation search subagents.
                            if let Some(api::message::tool_call_result::Result::Subagent(_)) =
                                &tcr.result
                            {
                                cleanup_conversation_search_temp_dir(
                                    &tcr.tool_call_id,
                                    &task_id,

```


**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 2295-2345)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
                                    &task_id,
                                    &self.task_store,
                                );
                            }
                        }
                        Some(api::message::Message::ModelUsed(model_used)) => {
                            let exchange_id = self
                                .added_exchanges_by_response
                                .get(response_stream_id)
                                .ok_or(UpdateConversationError::NoPendingRequest)?
                                .last()
                                .exchange_id;
                            let exchange = self.get_exchange_to_update(exchange_id)?;
                            if let Some(output) = exchange.output_status.output() {
                                let mut output = output.get_mut();
                                output.model_info = Some(OutputModelInfo {
                                    model_id: model_used.model_id.clone().into(),
                                    display_name: model_used.model_display_name.clone(),
                                    is_fallback: model_used.is_fallback,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                let task_id = TaskId::new(task_id);
                self.checkpoint_task(&task_id);
                let current_todo_list = self.todo_lists.last().cloned();

                // Remove the task to relinquish mutable borrow on self, we add it back later.
                let mut task = self
                    .task_store
                    .remove(&task_id)
                    .ok_or(UpdateConversationError::TaskNotFound)?;
                let added_exchanges = self
                    .added_exchanges_by_response
                    .get(response_stream_id)
                    .ok_or(UpdateConversationError::NoPendingRequest)?;
                let exchange_id = if let Some(info) =
                    added_exchanges.iter().find(|info| info.task_id == task_id)
                {
                    info.exchange_id
                } else {
                    let existing_exchange = self
                        .get_task(&added_exchanges.last().task_id)
                        .ok_or(UpdateConversationError::TaskNotFound)?
                        .exchange(added_exchanges.last().exchange_id)
                        .ok_or(UpdateConversationError::ExchangeNotFound)?;
                    let new_exchange_id = task.append_new_exchange(existing_exchange);
                    if self.optimistic_cli_subagent_subtask_id.is_some() && task.is_root_task() {

```


**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 2345-2395)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
                    if self.optimistic_cli_subagent_subtask_id.is_some() && task.is_root_task() {
                        // If we are lazily creating a new exchange at this point, this means we are updating
                        // a new task for the first time in this response stream.
                        //
                        // This is a bit of a hack, but if the optimistic CLI Subagent task is some and this is
                        // the root task, then this exchange corresponds to "setup" messages in the root task
                        // for bootstrapping the CLI subagent. In these cases, we don't care about
                        // surfacing the new root task messages in the UI (e.g. the blocklist) - there would basically
                        // be an empty AI Block corresponding to the CLI subagent tool call message added to the root
                        // task, with not user rendered output.
                        //
                        // The real fix here is to lazily create exchanges only when there are real messages to be
                        // rendered, or at the very least, lazily create AI blocks for an exchange only once the exchange
                        // actually has renderable content.
                        self.hidden_exchanges.insert(new_exchange_id);
                    }
                    new_exchange_id
                };

                let current_comment_state = self.code_review.as_ref().cloned();
                task.add_messages(
                    messages,
                    exchange_id,
                    current_todo_list.as_ref(),
                    current_comment_state.as_ref(),
                    // In shared-session viewers, we have to reconstruct what the original user input
                    // was using subsequent conversation messages (as the original input was not
                    // sent on this client). Once we reconstruct these inputs, we will insert them
                    // to mimic the normal conversation flow. (If this is not a shared session, the
                    // exchange inputs will already be populated).
                    self.is_viewing_shared_session,
                )?;

                self.task_store.insert(task);
                if !added_exchanges
                    .iter()
                    .any(|new_exchange_info| new_exchange_info.exchange_id == exchange_id)
                {
                    self.added_exchanges_by_response
                        .get_mut(response_stream_id)
                        .ok_or(UpdateConversationError::NoPendingRequest)?
                        .push(AddedExchange {
                            task_id: task_id.clone(),
                            exchange_id,
                        });
                    let is_hidden = self.hidden_exchanges.contains(&exchange_id);
                    ctx.emit(BlocklistAIHistoryEvent::AppendedExchange {
                        response_stream_id: Some(response_stream_id.clone()),
                        exchange_id,
                        task_id: task_id.clone(),
                        terminal_view_id,

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent\conversation.rs (best match)
app\src\ai\agent\conversation.rs
 352: let root_task_id = root_task.id().clone();
 432: let status = Self::derive_status_from_root_task(&tasks_by_id.get(&root_task_id));
 434: let task_store = TaskStore::from_tasks(tasks_by_id, root_task_id);
 857: let root_task_id = self.get_root_task_id().to_string();
 866: crate::ai::agent::linearization::compute_active_task_ids(&root_task_id, &all_tasks);
 1385: let root_task_id = self.task_store.root_task_id().clone();
 1392: task_id: root_task_id.clone(),
 1399: task_id: root_task_id.clone(),
 1406: self.append_exchange_to_task(&root_task_id, exchange)?;
 1411: new_task_id: root_task_id,
app\src\ai\blocklist\controller\slash_command.rs
 140: conversation.get_root_task_id().clone(),
app\src\ai\blocklist\orchestration_events.rs
 919: Some((inputs, conversation.get_root_task_id().clone()))

**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 1360-1420)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
            if should_hide {
                self.hidden_exchanges.insert(new_exchange_id);
            }

            ctx.emit(BlocklistAIHistoryEvent::AppendedExchange {
                exchange_id: new_exchange_id,
                task_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden: should_hide,
                response_stream_id: Some(stream_id.clone()),
            });
        }
        // turn 启动即落盘:user query 提交时先写一次,stream 中途强退也能保留提问记录。
        self.write_updated_conversation_state(ctx);
        Ok(())
    }

    pub fn append_reassigned_exchange(
        &mut self,
        response_stream_id: &ResponseStreamId,
        exchange: AIAgentExchange,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let root_task_id = self.task_store.root_task_id().clone();
        let exchange_id = exchange.id;
        if exchange.output_status.is_streaming() {
            if let Some(added_exchanges) =
                self.added_exchanges_by_response.get_mut(response_stream_id)
            {
                added_exchanges.push(AddedExchange {
                    task_id: root_task_id.clone(),
                    exchange_id,
                });
            } else {
                self.added_exchanges_by_response.insert(
                    response_stream_id.clone(),
                    Vec1::new(AddedExchange {
                        task_id: root_task_id.clone(),
                        exchange_id,
                    }),
                );
            }
        }

        self.append_exchange_to_task(&root_task_id, exchange)?;

        ctx.emit(BlocklistAIHistoryEvent::ReassignedExchange {
            exchange_id,
            terminal_view_id,
            new_task_id: root_task_id,
            new_conversation_id: self.id,
        });
        Ok(())
    }

    fn append_exchange_to_task(
        &mut self,
        task_id: &TaskId,
        exchange: AIAgentExchange,

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent\conversation.rs (best match)
app\src\ai\agent\conversation.rs
 2626: pub fn get_root_task_id(&self) -> &TaskId {
  2627| self.task_store.root_task_id()
  2628| }
 857: let root_task_id = self.get_root_task_id().to_string();
app\src\ai\blocklist\controller\slash_command.rs
 140: conversation.get_root_task_id().clone(),
app\src\ai\blocklist\orchestration_events.rs
 919: Some((inputs, conversation.get_root_task_id().clone()))

**Tool Call: Read file `app\src\ai\agent\conversation.rs` (lines 1320-1365)**
Status: Completed

```warp/app/src/ai/agent/conversation.rs
            shared_session_response_initiator,
            request_start_ts,
            ..
        } = request_input;

        for (task_id, inputs) in input_messages.into_iter() {
            let should_hide = inputs
                .iter()
                .any(|input| input.is_passive_suggestion_trigger());

            let new_exchange = AIAgentExchange {
                id: AIAgentExchangeId::new(),
                input: inputs,
                output_status: AIAgentOutputStatus::Streaming { output: None },
                added_message_ids: HashSet::new(),
                start_time: request_start_ts,
                finish_time: None,
                time_to_first_token_ms: None,
                working_directory: working_directory.clone(),
                // TODO(CORE-3546): fetch shell launch data from active session
                model_id: model_id.clone(),
                coding_model_id: coding_model_id.clone(),
                cli_agent_model_id: cli_agent_model_id.clone(),
                computer_use_model_id: computer_use_model_id.clone(),
                request_cost: None,
                // This will be None for non-shared sessions
                response_initiator: shared_session_response_initiator.clone(),
            };

            let new_exchange_id = new_exchange.id;
            self.append_exchange_to_task(&task_id, new_exchange)?;

            self.added_exchanges_by_response.insert(
                stream_id.clone(),
                Vec1::new(AddedExchange {
                    task_id: task_id.clone(),
                    exchange_id: new_exchange_id,
                }),
            );

            if should_hide {
                self.hidden_exchanges.insert(new_exchange_id);
            }

            ctx.emit(BlocklistAIHistoryEvent::AppendedExchange {
                exchange_id: new_exchange_id,

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

! regex failed: regex parse error:
    Task {
         ^
error: unclosed counted repetition, using literal match
→ Read app\src\ai\agent\api\convert_conversation.rs [def]
20/37 matches shown
app\src\ai\agent\api\convert_conversation.rs
 294: impl ConvertToExchanges for &api::Task {
  295| /// Converts a list of tasks into AIAgentExchange objects.
  296| ///
  297| /// Note: for now, we only restore messages from the root task (task with no parent).
  298| fn into_exchanges(self) -> Vec<AIAgentExchange> {
  299| let mut exchanges = Vec::new();
  300| let mut todo_lists: Vec<AIAgentTodoList> = Vec::new();
app\src\ai\persisted_workspace.rs
 62: pub enum LspTask {
  63| /// Install and enable an LSP server for a file path.
  64| Install {
  65| file_path: PathBuf,
  66| repo_root: PathBuf,
  67| server_type: LSPServerType,
.git\rr-cache\3aec5ec3b07b7c32023883404cd6df025645ed21\preimage
 384: enum WhichTask {
  385| NewConversation,
  386| Task {
  387| conversation_id: AIConversationId,
  388| task_id: TaskId,
  389| },
app\src\ai\agent_providers\chat_stream.rs
 2014: //   b) CreateTask(api::Task { id=<新 subtask>, dependencies.parent_task_id=root })
 2761: api::client_action::AddMessagesToTask {
 3314: api::client_action::CreateTask {
 3315: task: Some(api::Task {
 3342: api::client_action::CreateTask {
 3343: task: Some(api::Task {
app\src\code_review\code_review_view.rs
 2617: let task = FileInvalidationTask {
app\src\ai\agent\conversation.rs
 2011: Action::CreateTask(CreateTask { task: Some(task) }) => {
 2046: ctx.emit(BlocklistAIHistoryEvent::UpgradedTask {
 2169: ctx.emit(BlocklistAIHistoryEvent::UpgradedTask {
 2201: Action::AddMessagesToTask(AddMessagesToTask { task_id, messages }) => {
 2547: Action::MoveMessagesToNewTask(MoveMessagesToNewTask {
app\src\ai\ambient_agents\spawn_tests.rs
 17: ) -> AmbientAgentTask {
 18: AmbientAgentTask {
app\src\ai\blocklist\history_model_test.rs
 1137: let tasks = vec![warp_multi_agent_api::Task {
app\src\ai\agent_conversations_model.rs
 1384: | BlocklistAIHistoryEvent::UpgradedTask { .. }
.git\rr-cache\3aec5ec3b07b7c32023883404cd6df025645ed21\preimage
 386: Task {

**Tool Call: Read file `app\src\ai\blocklist\history_model_test.rs` (lines 1120-1165)**
Status: Completed

```warp/app/src/ai/blocklist/history_model_test.rs
    App::test((), |mut app| async move {
        let history_model = app.add_singleton_model(|_| BlocklistAIHistoryModel::new(vec![], &[]));

        let forked_conversation_id = AIConversationId::new();
        let conversation_data = AgentConversationData {
            server_conversation_token: Some("forked-token".to_string()),
            conversation_usage_metadata: None,
            reverted_action_ids: None,
            forked_from_server_conversation_token: None,
            artifacts_json: None,
            parent_agent_id: None,
            agent_name: None,
            parent_conversation_id: None,
            run_id: None,
            autoexecute_override: None,
            last_event_sequence: None,
        };
        let tasks = vec![warp_multi_agent_api::Task {
            id: "root-task".to_string(),
            messages: vec![],
            dependencies: None,
            description: String::new(),
            summary: String::new(),
            server_data: String::new(),
        }];

        history_model.update(&mut app, |model, _| {
            model
                .insert_forked_conversation_from_tasks(
                    forked_conversation_id,
                    tasks,
                    conversation_data,
                )
                .expect("forked conversation should insert");
        });

        let token = ServerConversationToken::new("forked-token".to_string());
        history_model.read(&app, |model, _| {
            assert_eq!(
                model.find_conversation_id_by_server_token(&token),
                Some(forked_conversation_id),
            );
        });
    });
}


```


**Tool Call: Run MCP tool `grep`**
Status: Completed

! regex failed: regex parse error:
    api::Message {
                 ^
error: unclosed counted repetition, using literal match
→ Read app\src\ai\agent\api\convert_from.rs [def]
app\src\ai\agent_providers\chat_stream.rs
 2948: fn make_reasoning_message(task_id: &str, request_id: &str, reasoning: String) -> api::Message {
  2949| api::Message {
  2950| id: Uuid::new_v4().to_string(),
  2951| task_id: task_id.to_owned(),
  2952| server_message_data: String::new(),
  2953| citations: vec![],
  2954| message: Some(api::message::Message::AgentReasoning(
  2955| api::message::AgentReasoning {
  2956| reasoning,
 2965: fn make_agent_output_message(task_id: &str, request_id: &str, text: String) -> api::Message {
app\src\ai\agent\api\convert_from.rs
 110: impl ConvertAPIMessageToClientOutputMessage for api::Message {
  111| fn to_client_output_message(
  112| self,
  113| params: ConversionParams,
  114| ) -> Result<MaybeAIAgentOutputMessage, MessageToAIAgentOutputMessageError> {
  115| let Some(message) = self.message else {
app\src\ai\agent\conversation_yaml_tests.rs
 23: fn make_user_query_message(id: &str, task_id: &str, query: &str) -> api::Message {
  24| api::Message {
  25| id: id.to_string(),
  26| task_id: task_id.to_string(),
  27| server_message_data: String::new(),
  28| citations: vec![],
app\src\ai\agent_providers\chat_stream.rs
 2812: let message = api::Message {
 2949: api::Message {
 2966: api::Message {
 2984: ) -> api::Message {
 3011: api::Message {
 3032: ) -> api::Message {
 3033: api::Message {
 3122: ) -> api::Message {
app\src\ai\blocklist\orchestration_event_streamer_tests.rs
 118: messages: vec![api::Message {
app\src\ai\agent\conversation_yaml_tests.rs
 24: api::Message {
 46: ) -> api::Message {
 47: api::Message {
 66: ) -> api::Message {
 67: api::Message {

**Tool Call: Read file `app\src\ai\agent\conversation_yaml_tests.rs` (lines 1-95)**
Status: Completed

```warp/app/src/ai/agent/conversation_yaml_tests.rs
use std::fs;
use std::path::Path;

use warp_multi_agent_api as api;

use crate::test_util::ai_agent_tasks::{
    create_api_subtask, create_api_task, create_message, create_subagent_tool_call_message,
};

use super::{base_dir, materialize_tasks_to_yaml};

/// Lists filenames (not full paths) in a directory, sorted.
fn list_dir_sorted(dir: &Path) -> Vec<String> {
    let mut entries: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    entries
}

fn make_user_query_message(id: &str, task_id: &str, query: &str) -> api::Message {
    api::Message {
        id: id.to_string(),
        task_id: task_id.to_string(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::UserQuery(api::message::UserQuery {
            query: query.to_string(),
            context: None,
            mode: None,
            referenced_attachments: Default::default(),
            intended_agent: Default::default(),
        })),
        request_id: String::new(),
        timestamp: None,
    }
}

fn make_tool_call_message(
    id: &str,
    task_id: &str,
    tool_call_id: &str,
    tool: api::message::tool_call::Tool,
) -> api::Message {
    api::Message {
        id: id.to_string(),
        task_id: task_id.to_string(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::ToolCall(api::message::ToolCall {
            tool_call_id: tool_call_id.to_string(),
            tool: Some(tool),
        })),
        request_id: String::new(),
        timestamp: None,
    }
}

fn make_tool_call_result_message(
    id: &str,
    task_id: &str,
    tool_call_id: &str,
    result: api::message::tool_call_result::Result,
) -> api::Message {
    api::Message {
        id: id.to_string(),
        task_id: task_id.to_string(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::ToolCallResult(
            api::message::ToolCallResult {
                tool_call_id: tool_call_id.to_string(),
                result: Some(result),
                context: None,
            },
        )),
        request_id: String::new(),
        timestamp: None,
    }
}

fn cleanup_dir(path: &str) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn mixed_message_types_produce_sequentially_indexed_files() {
    let task_id = "root";
    let tasks = vec![create_api_task(
        task_id,
        vec![
            make_user_query_message("m1", task_id, "hello"),
            // AgentOutput via create_message helper

```


**Tool Call: Read file `app\src\ai\agent_providers\chat_stream.rs` (lines 2960-2980)**
Status: Completed

```warp/app/src/ai/agent_providers/chat_stream.rs
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_agent_output_message(task_id: &str, request_id: &str, text: String) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::AgentOutput(
            api::message::AgentOutput { text },
        )),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_user_query_message(
    task_id: &str,

```


**Tool Call: Read file `app\src\ai\agent_providers\chat_stream.rs` (lines 630-720)**
Status: Completed

```warp/app/src/ai/agent_providers/chat_stream.rs
                        Some(existing) => existing.push_str(&next),
                        None => buf.reasoning = Some(next),
                    }
                }
            }
            api::message::Message::AgentOutput(a) => {
                if buf.text.is_some() || !buf.tool_calls.is_empty() {
                    buf.flush_into(&mut messages);
                }
                buf.text = Some(a.text.clone());
            }
            api::message::Message::ToolCall(tc) => {
                // OpenWarp BYOP:**虚拟 subagent tool_call 不发给上游模型**。
                // LRC tag-in 场景下,我们在 chat_stream 流头合成 `Tool::Subagent { metadata: Cli }`
                // 写入 root.task.messages,只用于触发 conversation 创建 cli subtask + spawn 浮窗,
                // 它不是模型实际产出的工具调用,模型看到会 confused(多余 tool call + 没法回应)。
                // 同样它对应的 placeholder ToolResponse(由 sanitize_tool_call_pairs 补的)
                // 也要由下面 ToolCallResult 分支的 skip 逻辑配合过滤,避免出现
                // "tool_response 找不到匹配的 tool_call" 的不平衡。
                use crate::ai::agent::task::helper::ToolCallExt;
                if tc.subagent().is_some() {
                    skipped_subagent_call_ids.insert(tc.tool_call_id.clone());
                    continue;
                }
                let (name, args_json) = serialize_outgoing_tool_call(
                    tc,
                    params.mcp_context.as_ref(),
                    &msg.server_message_data,
                );
                buf.tool_calls.push(ToolCall {
                    call_id: tc.tool_call_id.clone(),
                    fn_name: name,
                    fn_arguments: args_json,
                    thought_signatures: None,
                });
            }
            api::message::Message::ToolCallResult(tcr) => {
                buf.flush_into(&mut messages);
                // OpenWarp:对应 ToolCall 已被 skip(subagent 虚拟 call)→ result 也 skip,
                // 否则留下孤儿 tool_response 导致上游 400。
                if skipped_subagent_call_ids.contains(&tcr.tool_call_id) {
                    continue;
                }
                // BYOP 持久化的 ToolCallResult 走 server_message_data(content 已是 JSON 字符串);
                // server 端 emit 走 result oneof 结构化 variant — 兼容两路。
                let content = if compacted_tool_msg_ids.contains(&msg.id) {
                    // 压缩投影:被 prune 的 tool output 替换为占位符,不送实际内容上游
                    r#"{"status":"compacted","note":"tool output was pruned by local compaction"}"#
                        .to_string()
                } else if tcr.result.is_some() {
                    tools::serialize_result(tcr)
                } else if !msg.server_message_data.is_empty() {
                    msg.server_message_data.clone()
                } else {
                    r#"{"status":"empty"}"#.to_owned()
                };
                messages.push(ChatMessage::from(ToolResponse::new(
                    tcr.tool_call_id.clone(),
                    content,
                )));
            }
            _ => {
                // 其他 message 类型(SystemQuery/UpdateTodos/...)BYOP 暂不送上游。
            }
        }
    }
    buf.flush_into(&mut messages);

    // 当前轮新输入 → 追加。
    for input in &params.input {
        match input {
            AIAgentInput::UserQuery {
                query,
                context,
                running_command,
                ..
            } => {
                // 当前轮 UserQuery 自带的附件类 context(Block / SelectedText / File / Image)
                // 严格对齐 warp 自家路径走 `api::InputContext.executed_shell_commands` 等字段
                // 上行后由后端注入 prompt 的语义。BYOP 没有后端这层,直接 prepend 到 user message。
                // 环境型 context(env / git / skills / ...)由 prompt_renderer 渲染进 system,
                // 与本路径不重叠。
                //
                // OpenWarp:LRC tag-in 场景下,`running_command: Some(...)` 含完整 PTY 上下文
                // (alt-screen grid_contents + command + is_alt_screen_active 标志),用
                // `render_running_command_context` 渲成 `<attached_running_command>` XML 块
                // prepend 到 user message,模型据此决定调 write_to_long_running_shell_command。
                // 没填(普通对话或 controller 没注入)时回退到 `lrc_command_id` 简短上下文。
                let mut prefixes: Vec<String> = Vec::new();
                let request_running_command = running_command
                    .as_ref()

```


**Tool Call: app\\src\\ai\\byop\_compaction\\state.rs**
Status: Completed

Diff: app\src\ai\byop_compaction\state.rs
```
//! 压缩 sidecar 状态 — 挂在 `AIConversation` 上,与 warp `api::Message` 协议解耦。
//!
//! 因为 warp 的 `api::Message` 来自外部 protobuf 依赖 (`warp_multi_agent_api`),
//! 无法新增字段标记 `is_summary` / `compacted` 等;本 sidecar 用 message_id 索引
//! 把这些"压缩元数据"挂在 conversation 这一侧。
//!
//! 序列化版本号 [`CompactionState::VERSION`] 在 schema 演进时手动 bump,
//! 反序列化失败的旧 conversation 退化为 `Default`(等价"从未被压缩")。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// 触发压缩的来源。`Auto` 仅由 token-overflow 自动触发,`Manual` 是 /compact /compact-and。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompactionTrigger {
    Manual,
    Auto,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMarker {
    /// 这条 assistant message 是一份摘要,内容用于在请求拼装时替换前面的历史。
    #[serde(default)]
    pub is_summary: bool,
    /// 这条 user message 是一次 compaction 触发占位(opencode `parts.some(p => p.type === "compaction")`)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_trigger: Option<CompactionTrigger>,
    /// 这条 ToolCallResult 的 output 已被 prune,投影时替换为占位符。Unix epoch ms。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_output_compacted_at: Option<u64>,
    /// 自动续跑时合成的 user "Continue..." synthetic message 标记
    /// (对齐 opencode `metadata.compaction_continue`)。
    #[serde(default)]
    pub synthetic_continue: bool,
}

/// 一个已完成的压缩区间(对齐 opencode `completedCompactions()` 返回项)。
///
/// `user_msg_id` 是触发摘要的 user message(带 compaction_trigger marker),
/// `assistant_msg_id` 是合成的摘要 AgentOutput message。两者在 [`CompactionState::hidden_message_ids`]
/// 中视为已被覆盖,投影时跳过 — 但摘要文本本身会被取出代填到 head 区。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedCompaction {
    pub user_msg_id: String,
    pub assistant_msg_id: String,
    /// 本次摘要覆盖的 head 区 message ids,投影普通请求时全部隐藏。
    #[serde(default)]
    pub head_message_ids: Vec<String>,
    /// tail 起点 message id,用于 split 验证 / debug。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_start_id: Option<String>,
    /// 摘要内容(从 assistant message 直接取也可,但缓存到 state 方便 build_prompt 拿 previous_summary)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_text: Option<String>,
    pub auto: bool,
    pub overflow: bool,
}

/// 与 `AIConversation` 一同持久化的 sidecar 表。
///
/// 默认值 = 空表 = 未压缩状态,完全无侵入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionState {
    /// schema 版本,演进时 bump。
    #[serde(default = "CompactionState::current_version")]
    pub version: u32,
    #[serde(default)]
    markers: HashMap<String, MessageMarker>,
    #[serde(default)]
    completed: Vec<CompletedCompaction>,
}

impl Default for CompactionState {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            markers: HashMap::new(),
            completed: Vec::new(),
        }
    }
}

impl CompactionState {
    pub const VERSION: u32 = 2;
    fn current_version() -> u32 {
        Self::VERSION
    }

    pub fn marker(&self, msg_id: &str) -> Option<&MessageMarker> {
        self.markers.get(msg_id)
    }

    /// 写一个 marker(merge 到已有 marker 上,而不是覆盖整个 marker)。
    pub fn upsert_marker(&mut self, msg_id: impl Into<String>, f: impl FnOnce(&mut MessageMarker)) {
        let entry = self.markers.entry(msg_id.into()).or_default();
        f(entry);
    }

    /// 标记一条 ToolCallResult 的 output 已 prune。
    pub fn mark_tool_compacted(&mut self, msg_id: impl Into<String>, now_ms: u64) {
        self.upsert_marker(msg_id, |m| m.tool_output_compacted_at = Some(now_ms));
    }

    /// 推一次完成的压缩。
    pub fn push_completed(&mut self, c: CompletedCompaction) {
        // 同步把 user 与 assistant 各自打上 marker(便于投影时单独识别)。
        self.upsert_marker(c.user_msg_id.clone(), |m| {
            m.compaction_trigger = Some(if c.auto {
                CompactionTrigger::Auto
            } else {
                CompactionTrigger::Manual
            });
        });
        self.upsert_marker(c.assistant_msg_id.clone(), |m| m.is_summary = true);
        self.completed.push(c);
    }

    /// 标记一条 synthetic "Continue..." user message(auto+overflow 路径合成)。
    pub fn mark_synthetic_continue(&mut self, msg_id: impl Into<String>) {
        self.upsert_marker(msg_id, |m| m.synthetic_continue = true);
    }

    /// 取最后一次完成的压缩(用于 [`super::prompt::build_prompt`] 的增量摘要锚点)。
    pub fn previous_summary(&self) -> Option<&str> {
        self.completed
            .last()
            .and_then(|c| c.summary_text.as_deref())
    }

    pub fn completed(&self) -> &[CompletedCompaction] {
        &self.completed
    }

    /// 所有应在拼请求时跳过的 message id(对齐 opencode `hidden`):
    /// 已完成压缩的每个区间的 head_message_ids + user_msg_id + assistant_msg_id。
    ///
    /// 注:这只是"原本要从历史里隐去的 message id 集",**不**包含摘要本身 —
    /// 摘要文本由请求投影在 compaction trigger user_msg_id 位置插入合成消息覆盖。
    pub fn hidden_message_ids(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        for c in &self.completed {
            out.extend(c.head_message_ids.iter().cloned());
            out.insert(c.user_msg_id.clone());
            out.insert(c.assistant_msg_id.clone());
        }
        out
    }

    /// 调试 / 测试入口:看一条 marker 是否存在。
    #[cfg(test)]
    pub(crate) fn marker_count(&self) -> usize {
        self.markers.len()
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn cc(uid: &str, aid: &str, auto: bool) -> CompletedCompaction {
        CompletedCompaction {
            user_msg_id: uid.to_string(),
            assistant_msg_id: aid.to_string(),
            head_message_ids: Vec::new(),
            tail_start_id: None,
            summary_text: Some(format!("summary-{aid}")),
            auto,
            overflow: false,
        }
    }

    #[test]
    fn push_completed_marks_both_messages() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", true));
        assert!(s.marker("u1").unwrap().compaction_trigger == Some(CompactionTrigger::Auto));
        assert!(s.marker("a1").unwrap().is_summary);
    }

    #[test]
    fn previous_summary_returns_last() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        assert_eq!(s.previous_summary(), Some("summary-a2"));
    }

    #[test]
    fn hidden_message_ids_covers_all_completed() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        let h = s.hidden_message_ids();
        assert!(h.contains("u1"));
        assert!(h.contains("a1"));
        assert!(h.contains("u2"));
        assert!(h.contains("a2"));
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn hidden_message_ids_includes_head_message_ids() {
        let mut s = CompactionState::default();
        let mut c = cc("u1", "a1", false);
        c.head_message_ids = vec!["h1".to_string(), "h2".to_string(), "u1".to_string()];
        s.push_completed(c);
        let h = s.hidden_message_ids();
        assert!(h.contains("h1"));
        assert!(h.contains("h2"));
        assert!(h.contains("u1"));
        assert!(h.contains("a1"));
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn v1_completed_compaction_deserializes_to_empty_head_message_ids() {
        let json = r#"{
            "user_msg_id":"u1",
            "assistant_msg_id":"a1",
            "tail_start_id":null,
            "summary_text":"summary",
            "auto":false,
            "overflow":false
        }"#;
        let c: CompletedCompaction = serde_json::from_str(json).unwrap();
        assert!(c.head_message_ids.is_empty());
    }

    #[test]
    fn upsert_marker_merges() {
        let mut s = CompactionState::default();
        s.upsert_marker("m1", |m| m.is_summary = true);
        s.upsert_marker("m1", |m| m.synthetic_continue = true);
        let m = s.marker("m1").unwrap();
        assert!(m.is_summary);
        assert!(m.synthetic_continue);
        assert_eq!(s.marker_count(), 1);
    }

    #[test]
    fn default_serializable_roundtrip() {
        let s = CompactionState::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: CompactionState = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, CompactionState::VERSION);
        assert!(back.completed.is_empty());
    }
}

```


**Tool Call: app\\src\\ai\\byop\_compaction\\commit.rs**
Status: Completed

Diff: app\src\ai\byop_compaction\commit.rs
```
//! 把刚刚完成的 SummarizeConversation 流的产出写回 conversation.compaction_state —
//! 对齐 opencode `compaction.ts processCompaction` 末尾的状态变更 + bus.publish(Compacted)。
//!
//! 本模块独立于 controller,作为可单元测试的 helper(虽然真实调用站点在 controller.rs)。

use warp_multi_agent_api as api;

use crate::ai::agent::conversation::AIConversation;

use super::algorithm::{prune_decisions, select, MessageRef};
use super::config::CompactionConfig;
use super::overflow::ModelLimit;
use super::message_view::{build_tool_name_lookup, project};
use super::state::CompletedCompaction;

/// 从 conversation 的 root task 倒序找最后一条 `Message::AgentOutput` —
/// 它就是模型刚 emit 的摘要文本。
///
/// `user_msg_id` 选最后一条 AgentOutput 之前最近一条真实 UserQuery 的 id;
/// 没有时合成一个独立 uuid(只用作 marker key,build_chat_request 的 hidden
/// 投影不会命中真实 message)。
pub fn commit_summarization(
    conversation: &mut AIConversation,
    overflow: bool,
    cfg: &CompactionConfig,
) -> bool {
    // 用 conversation 已有的 linearized messages accessor — 跨所有 task 已按时间序合并
    let mut all_msgs: Vec<&api::Message> = conversation.all_linearized_messages();
    all_msgs.sort_by_key(|m| {
        m.timestamp
            .as_ref()
            .map(|ts| (ts.seconds, ts.nanos))
            .unwrap_or((0, 0))
    });

    let last_agent_output: Option<(String, String)> = all_msgs.iter().rev().find_map(|m| {
        let inner = m.message.as_ref()?;
        match inner {
            api::message::Message::AgentOutput(a) => Some((m.id.clone(), a.text.clone())),
            _ => None,
        }
    });

    let Some((assistant_id, summary_text)) = last_agent_output else {
        log::warn!("[byop-compaction] commit: no AgentOutput found — nothing to commit");
        return false;
    };

    let assistant_id_str: &str = &assistant_id;
    let assistant_pos = all_msgs
        .iter()
        .position(|m| m.id.as_str() == assistant_id_str);
    let user_msg_id: String = assistant_pos
        .and_then(|pos| {
            all_msgs[..pos]
                .iter()
                .rev()
                .find_map(|m| match m.message.as_ref() {
                    Some(api::message::Message::UserQuery(_)) => Some(m.id.clone()),
                    _ => None,
                })
        })
        .unwrap_or_else(|| format!("compaction-trigger-{}", uuid::Uuid::new_v4()));

    let tool_names = build_tool_name_lookup(all_msgs.iter().copied());
    let state_snapshot = conversation.compaction_state.clone();
    let views = project(&all_msgs, &state_snapshot, &tool_names);
    let select_result = select(&views, cfg, ModelLimit::FALLBACK, |slice| {
        slice.iter().map(MessageRef::estimate_size).sum()
    });
    let head_message_ids = all_msgs[..select_result.head_end]
        .iter()
        .map(|m| m.id.clone())
        .collect::<Vec<_>>();
    let auto = overflow;
    let summary_len = summary_text.len();
    let completed = CompletedCompaction {
        user_msg_id: user_msg_id.clone(),
        assistant_msg_id: assistant_id.clone(),
        head_message_ids,
        tail_start_id: select_result.tail_start_id,
        summary_text: Some(summary_text),
        auto,
        overflow,
    };
    log::info!(
        "[byop-compaction] commit: assistant_msg={} user_msg={} summary_len={} auto={} overflow={} head_count={} tail_start={:?}",
        assistant_id,
        user_msg_id,
        summary_len,
        auto,
        overflow,
        completed.head_message_ids.len(),
        completed.tail_start_id,
    );
    conversation.compaction_state.push_completed(completed);
    true
}

/// 在每次 LLM 请求前自动跑 prune — 1:1 对齐 opencode `compaction.ts:297-341`。
///
/// 计算决策(哪些 ToolCallResult 的 output 应被替换为占位)然后写入
/// `conversation.compaction_state.markers.tool_output_compacted_at`。
/// 实际替换发生在 `chat_stream::build_chat_request` 投影时(读 marker)。
///
/// `cfg.prune == false` 时 no-op。
pub fn prune_now(conversation: &mut AIConversation, cfg: &CompactionConfig) -> usize {
    if !cfg.prune {
        return 0;
    }
    let all_msgs: Vec<&api::Message> = conversation.all_linearized_messages();
    if all_msgs.is_empty() {
        return 0;
    }
    let tool_names = build_tool_name_lookup(all_msgs.iter().copied());
    let state_snapshot = conversation.compaction_state.clone();
    let views = project(&all_msgs, &state_snapshot, &tool_names);
    // 用 trait 引用避免泛型推导歧义
    let views_ref: &[_] = &views;
    let decisions = prune_decisions::<super::message_view::WarpMessageView<'_>>(views_ref);
    if decisions.is_empty() {
        return 0;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let count = decisions.len();
    for (msg_id, _call_id) in decisions {
        // msg_id 是 ToolCallResult 的 message id;mark_tool_compacted 会在 marker 上写时间戳
        conversation
            .compaction_state
            .mark_tool_compacted(msg_id, now_ms);
    }
    log::info!("[byop-compaction] pruned {count} tool output(s)");
    count
}

// Reference traits for type inference
#[allow(unused_imports)]
use super::algorithm::Role as _Role;
#[allow(unused_imports)]
use super::algorithm::ToolOutputRef as _ToolOutputRef;
// Mention MessageRef so that the import isn't dropped
#[allow(dead_code)]
fn _ensure_message_ref_imported<M: MessageRef>(_m: &M) {}

```


**Tool Call: app\\src\\ai\\agent\_providers\\chat\_stream.rs**
Status: Completed

Diff: app\src\ai\agent_providers\chat_stream.rs
```
//! BYOP 模式下 chat completion + tool calling 适配层(基于 genai 0.5.3)。
//!
//! 把 `RequestParams` 翻译为 genai `ChatRequest`,通过 `Client::exec_chat_stream`
//! 调用用户配置的 provider,响应翻译回 `warp_multi_agent_api::ResponseEvent`,
//! controller 自家逻辑(权限/弹窗/执行/result 回写/触发下一轮)接管闭环。
//!
//! ## 5 种 API 协议显式路由
//!
//! 不再把所有 provider 当作 OpenAI 兼容硬塞,通过 `ServiceTargetResolver` 把
//! 用户在 settings UI 选的 `AgentProviderApiType` 一对一映射到 genai 的 `AdapterKind`:
//!
//! | ApiType        | AdapterKind  | 默认 endpoint                                  |
//! |----------------|--------------|------------------------------------------------|
//! | OpenAi         | OpenAI       | https://api.openai.com/v1                      |
//! | OpenAiResp     | OpenAIResp   | https://api.openai.com/v1 (走 /v1/responses)   |
//! | Gemini         | Gemini       | https://generativelanguage.googleapis.com/v1beta |
//! | Anthropic      | Anthropic    | https://api.anthropic.com                      |
//! | Ollama         | Ollama       | http://localhost:11434                         |
//!
//! 用户填的 `base_url` 始终覆盖默认。这样:
//! - DeepSeek / SiliconFlow / OpenRouter 等 OpenAI 兼容 provider 选 `OpenAi`,自定义 base_url
//! - 显式选定 adapter 完全绕过 genai 的"按模型名识别"默认行为,避免误识别
//!
//! ## 多轮 message 转换
//!
//! - system prompt: `ChatRequest::with_system()`(不进 messages 数组)
//! - user query: `ChatMessage::user(text)`
//! - assistant text: `ChatMessage::assistant(text)`
//! - assistant tool_calls: `ChatMessage::from(Vec<ToolCall>)`(自动 assistant role)
//! - tool result: `ChatMessage::from(ToolResponse::new(call_id, content))`(自动 tool role)
//!
//! ## 流式实现
//!
//! `Client::exec_chat_stream` 返回 `ChatStreamResponse`,其 `stream` 字段实现了
//! `futures_core::Stream<Item = Result<ChatStreamEvent>>`。事件:
//! - `Start` / `Chunk(text)` / `ReasoningChunk(text)` / `ToolCallChunk(tool_call)` / `End(StreamEnd)`
//!
//! 我们对 Chunk/ReasoningChunk 立即 emit `AppendToMessageContent`(打字机效果),
//! 对 ToolCallChunk 累积 buffer(按 call_id),流末统一 emit `Message::ToolCall`,
//! controller 自动接管。

use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;
use serde_json::{json, Value};
use uuid::Uuid;
use warp_multi_agent_api as api;

use genai::adapter::AdapterKind;
use genai::chat::{
    Binary, ChatMessage, ChatOptions, ChatRequest, ChatRole, ChatStreamEvent, ContentPart,
    MessageContent, Tool as GenaiTool, ToolCall, ToolResponse,
};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget, WebConfig};

use crate::ai::agent::api::{RequestParams, ResponseStream};
use crate::ai::agent::{AIAgentInput, RunningCommand, UserQueryMode};
use crate::ai::byop_compaction;
use crate::server::server_api::AIApiError;
use crate::settings::AgentProviderApiType;
use ai::agent::convert::ConvertToAPITypeError;

use super::openai_compatible::OpenAiCompatibleError;
use super::tools;

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------
// system prompt 由 `prompt_renderer::render_system` 通过 minijinja 模板生成,
// 按 LLMId 模型族选 system/{anthropic,gpt,beast,gemini,kimi,codex,trinity,default}.j2,
// 并把 warp 客户端已经收集好的 AIAgentContext(env / git / skills / project_rules / codebase / current_time)
// 渲染进 system,让 BYOP 路径也能拥有跟 warp 自家路径相当的环境信息。

use super::attachment_caps;
use super::prompt_renderer;
use super::user_context;
use crate::ai::agent::AIAgentContext;

/// 从 input 中抽出最近一条 `UserQuery.context`(等价 warp `convert_to.rs::convert_input` 取的那条)。
fn latest_input_context(input: &[AIAgentInput]) -> &[AIAgentContext] {
    for i in input.iter().rev() {
        if let Some(ctx) = i.context() {
            return ctx;
        }
    }
    &[]
}

/// LRC tag-in 场景下渲染 `<attached_running_command>` XML 块,prepend 到 user message,
/// 让模型看到当前 PTY 的实际状态(命令、grid 内容、是否 alt-screen),从而正确选择
/// `write_to_long_running_shell_command` 工具发送对应键序列。
fn render_running_command_context(rc: &RunningCommand) -> String {
    format!(
        "<attached_running_command command_id=\"{}\" is_alt_screen_active=\"{}\">\n  \
         <command>{}</command>\n  \
         <snapshot>\n{}\n  </snapshot>\n  \
         <instructions>This command is already running in the user's terminal. \
         Use `read_shell_command_output` with this command_id to inspect it, and \
         `write_to_long_running_shell_command` with this command_id to operate the program \
         through its PTY (in raw mode, use tokens like `<ESC>` and `<ENTER>` for control \
         keys). This command_id is valid even if the process was started by the user \
         rather than by run_shell_command. Do NOT spawn a new shell to control the same TUI.\
         </instructions>\n\
         </attached_running_command>",
        xml_attr(rc.block_id.as_str()),
        rc.is_alt_screen_active,
        xml_text(&rc.command),
        xml_text(&rc.grid_contents),
    )
}

/// 简短回退版本:仅有 command_id(没拿到 RunningCommand 完整快照时),
/// 让模型至少知道目标 PTY 的 id,可以用 `read_shell_command_output` 自己取最新内容。
fn render_running_command_id_context(command_id: &str) -> String {
    format!(
        "<attached_running_command command_id=\"{}\">\n  \
         <instructions>This command is already running in the user's terminal. \
         Use `read_shell_command_output` with this command_id to inspect it, and \
         `write_to_long_running_shell_command` with this command_id to operate the program \
         through its PTY. Do NOT spawn a new shell to control the same TUI.</instructions>\n\
         </attached_running_command>",
        xml_attr(command_id),
    )
}

fn render_lrc_request_context(params: &RequestParams) -> Option<String> {
    params
        .lrc_running_command
        .as_ref()
        .map(render_running_command_context)
        .or_else(|| {
            params
                .lrc_command_id
                .as_deref()
                .map(render_running_command_id_context)
        })
}

/// OpenWarp:渲染 SSH 会话状态块,append 到 system prompt 末尾。
///
/// 触发条件:`SessionContext.is_legacy_ssh()` 为 true(用户在本地 PTY 手敲
/// `ssh xx@xx` 进入远端,远端没装 warp shell hook)。这种会话:
/// - `session_type` 仍是 `Local`
/// - 整段 system prompt 的 [Environment] 区块描述的是**本地客户端** OS / shell,
///   而 PTY 当前实际跑在**远端**
///
/// 不主动告知模型这一点,LLM 会按 system prompt 里的本地 OS 推断"目标在远端,
/// 我得先 ssh 过去",于是输出 `ssh xx@xx uname -a` 这种二次嵌套命令。
///
/// 注:warpified SSH(`SessionType::WarpifiedRemote`)不在这里处理 — 那条路径
/// 远端 shell hook 已重新 bootstrap,host_info / shell 都是远端真值,prompt 本身就对。
fn render_ssh_session_block(
    session_context: &crate::ai::blocklist::SessionContext,
) -> Option<String> {
    if !session_context.is_legacy_ssh() {
        return None;
    }
    let info = session_context.ssh_connection_info();
    let host = info
        .and_then(|i| i.host.as_deref())
        .map(xml_attr)
        .unwrap_or_else(|| "unknown".to_owned());
    let port = info
        .and_then(|i| i.port.as_deref())
        .map(xml_attr)
        .unwrap_or_else(|| "22".to_owned());

    Some(format!(
        "\n\n<ssh_session host=\"{host}\" port=\"{port}\">\n  \
         <fact>The active terminal PTY is currently inside an SSH session opened by the user from their local machine. \
         All shell commands you run via `run_shell_command` execute on the REMOTE host, not on the local client.</fact>\n  \
         <warning>The [Environment] block (OS / shell / working directory) above describes the LOCAL client and may not match the remote host. \
         If you need precise remote info, probe it directly (e.g. `uname -a`, `cat /etc/os-release`, `pwd`).</warning>\n  \
         <rules>\n    \
         - Run commands DIRECTLY (e.g. `uname -a`, `ls /`). Do NOT prepend `ssh {host} ...` — that opens a NESTED ssh session inside the current one.\n    \
         - Treat the working directory and home directory shown above with skepticism; they may reflect the local client.\n    \
         - When LRC tag-in mode is active (an `<attached_running_command>` block is present), prefer `write_to_long_running_shell_command` with that command_id to inject keystrokes into this same remote PTY. Spawning a new shell would create a separate local-side ssh client, not interact with the remote process the user is watching.\n  \
         </rules>\n\
         </ssh_session>"
    ))
}

/// XML 转义,同时 strip 所有非法/有问题的控制字符,避免 JSON 序列化失败。
///
/// `grid_contents`(从 `formatted_terminal_contents_for_input` 提取的 alt-screen 内容)
/// 可能含原始 ANSI escape 序列(0x1b)、CSI sequences、SGR codes、box-drawing chars 等。
/// 其中 `< 0x20` 的控制字符会让 Anthropic 解析 JSON 报 "invalid escaped character in string",
/// 因为 JSON RFC 7159 只接受 `\b \f \n \r \t \" \\ \/ \uXXXX` 这几种合法转义,
/// 其他 `\v` `\a` `\x1b` 之类直接 reject。
///
/// 处理:
/// - `\n` `\r` `\t` 保留(JSON 合法)
/// - 其它 `< 0x20` 控制字符替换成空格(纯展示给模型,不需要保留 ANSI 颜色等)
/// - `&` `<` `>` 转 XML entity
fn xml_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\n' | '\r' | '\t' => out.push(c),
            c if (c as u32) < 0x20 => out.push(' '),
            // DEL(0x7f)单独处理 — 也是控制字符
            '\u{7f}' => out.push(' '),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn xml_attr(s: &str) -> String {
    xml_text(s).replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// Multi-turn message 转换
// ---------------------------------------------------------------------------

/// 累积同一 assistant turn 的 text + tool_calls + reasoning,然后 flush 成一个或两个
/// `ChatMessage`(text 一个,tool_calls 一个 — genai 把它们建模为分开的 message)。
///
/// **thinking-mode reasoning_content 回传 gate**(双向)。
///
/// `force_echo_reasoning` 同时控制两件事,语义统一为「这个 endpoint 接受/需要
/// `reasoning_content` 顶层字段」:
///
/// - `true`(DeepSeek api_type / OpenAI+kimi|moonshot):每条 assistant 必挂
///   `reasoning_content`(有真实 reasoning 用之,无则挂非空占位)— 满足
///   DeepSeek-v4-flash / Kimi 等 thinking-mode 服务端「字段必须存在」校验。
/// - `false`(其他):**即便 stream 收到了真实 reasoning_content,回放时也丢弃**,
///   不在历史 assistant 上挂 `with_reasoning_content`。
///
/// 为什么 `false` 时也要主动丢弃真实 reasoning:许多 OpenAI-strict provider 把
/// `messages[].reasoning_content` 视为非法字段并 400(`code: wrong_api_format`):
///
/// - **Cerebras**(zerx-lab/warp #25 元凶,zai-glm-4.7 第二轮 400)
/// - **Groq**(协议侧用 `reasoning_format` / `include_reasoning`,不接受 message 字段)
/// - **OpenRouter / Together AI / SambaNova / Anyscale / Replicate** 等中转/inference 厂商
/// - **OpenAI 官方**(GPT-5/o-series 走 OpenAIResp,o-series 用 `reasoning.encrypted_content`,不收 `reasoning_content`)
///
/// genai 0.6 OpenAI adapter `adapter_shared.rs:367,385-387` 见到
/// `ContentPart::ReasoningContent` 就**无条件** hoist 出顶层 `reasoning_content`
/// 字段,所以 gate 必须前移到 client 侧 — 即「不挂 ContentPart 就不会被序列化」。
///
/// Anthropic / Gemini adapter 序列化层会忽略 `ContentPart::ReasoningContent`
/// (各自走 thinking blocks / thought signature),不受这个 gate 影响,但保持一致仍走 `false` 分支不挂。
const REASONING_ECHO_PLACEHOLDER: &str = " ";

#[derive(Default)]
struct AssistantBuffer {
    text: Option<String>,
    tool_calls: Vec<ToolCall>,
    /// 上一轮 AgentReasoning(thinking 链)。flush 时挂到对应 assistant message
    /// 的 reasoning_content 字段(genai 内部按 adapter 序列化:DeepSeek/Kimi 走 reasoning_content,
    /// Anthropic 走 thinking blocks)。
    reasoning: Option<String>,
    /// thinking-mode adapter 强制回传 reasoning_content(非空占位)。由
    /// `super::reasoning::model_requires_reasoning_echo` 决定。
    force_echo_reasoning: bool,
}

impl AssistantBuffer {
    fn new(force_echo_reasoning: bool) -> Self {
        Self {
            force_echo_reasoning,
            ..Default::default()
        }
    }

    fn flush_into(&mut self, messages: &mut Vec<ChatMessage>) {
        let reasoning = self.reasoning.take();
        let has_tool_calls = !self.tool_calls.is_empty();
        // 决定本次 flush 要挂到 assistant message 上的 reasoning 字符串。
        //
        // **gate 反转**:`force_echo_reasoning = false` 时**一律不挂**,即使本 turn
        // stream 收到了真实 reasoning(zai-glm / qwen3-thinking 这类 thinking 模型走
        // OpenAI 兼容路径会 emit reasoning_content chunk)— 因为 Cerebras / Groq /
        // OpenRouter 等 OpenAI-strict provider 见到 `messages[].reasoning_content` 直接
        // 400 `wrong_api_format`(zerx-lab/warp #25)。
        //
        // `force_echo_reasoning = true` 时(DeepSeek api_type / OpenAI+kimi/moonshot):
        // - 有真实 reasoning → 用之
        // - 没有 → 非空占位(满足"字段必须存在"校验)
        let echo_reasoning: Option<String> = if self.force_echo_reasoning {
            match reasoning {
                Some(r) if !r.is_empty() => Some(r),
                _ => Some(REASONING_ECHO_PLACEHOLDER.to_owned()),
            }
        } else {
            // 注:即便 `reasoning` 是 Some(非空),也丢弃 — 见上方 gate 反转说明。
            None
        };
        if let Some(t) = self.text.take() {
            let mut msg = ChatMessage::assistant(t);
            if has_tool_calls {
                // DeepSeek thinking mode 要求每条 assistant message 都带
                // reasoning_content。text + tool_calls 被 genai 建模成两条
                // assistant 时,text 这条也必须补占位。
                if self.force_echo_reasoning {
                    msg = msg.with_reasoning_content(Some(REASONING_ECHO_PLACEHOLDER.to_owned()));
                }
            } else if let Some(r) = echo_reasoning.clone() {
                msg = msg.with_reasoning_content(Some(r));
            }
            messages.push(msg);
        }
        if has_tool_calls {
            // genai `From<Vec<ToolCall>> for ChatMessage` 自动产 assistant role +
            // MessageContent::from_tool_calls。
            let mut msg = ChatMessage::from(std::mem::take(&mut self.tool_calls));
            if let Some(r) = echo_reasoning {
                msg = msg.with_reasoning_content(Some(r));
            }
            messages.push(msg);
        }
    }
}

/// 构造一条 user `ChatMessage`,按 model capability 决定要不要切到
/// `MessageContent::Parts(Text + Binary[])` 多模态形态。
///
/// - 没有 binaries → 走老路 `ChatMessage::user(text)` 纯文本,与 P0 行为一致
/// - 有 binaries 且 model 支持对应 mime → `Parts(vec![Text(text), Binary(...), ...])`,
///   genai adapter 自动按线协议适配(OpenAI image_url/file、Anthropic image/document、
///   Gemini inline_data 等)
/// - binaries 但 model 不支持 → log warn 跳过该 part,降级为纯文本(prefix XML 里的
///   `<image .../>` / `<file binary=true .../>` 占位仍然在,LLM 至少知道用户附了什么)
fn build_user_message_with_binaries(
    text: String,
    binaries: Vec<user_context::UserBinary>,
    api_type: AgentProviderApiType,
    model_id: &str,
) -> ChatMessage {
    if binaries.is_empty() {
        return ChatMessage::user(text);
    }
    let caps = attachment_caps::caps_for(api_type, model_id);

    let mut parts: Vec<ContentPart> = Vec::with_capacity(1 + binaries.len());
    parts.push(ContentPart::Text(text));

    let mut error_replacements: Vec<(String, String)> = Vec::new();
    for bin in binaries {
        if !caps.supports_mime(&bin.content_type) {
            // OpenWarp 对齐 opencode `unsupportedParts`(packages/opencode/src/provider/transform.ts:305-341):
            // 模型不支持的 mime 不静默 drop,改成插入一条 ERROR 文本 part,让 LLM 自己告诉用户。
            // 文案严格照抄 opencode 的 `ERROR: Cannot read {name} (this model does not support
            // {modality} input). Inform the user.`,modality 由 mime 前缀映射,name 优先用文件名。
            let modality = mime_to_modality(&bin.content_type);
            let name = if bin.name.is_empty() {
                modality.to_string()
            } else {
                format!("\"{}\"", bin.name)
            };
            let err_text = format!(
                "ERROR: Cannot read {name} (this model does not support {modality} input). Inform the user."
            );
            error_replacements.push((bin.name.clone(), bin.content_type.clone()));
            parts.push(ContentPart::Text(err_text));
            continue;
        }
        parts.push(ContentPart::Binary(Binary::from_base64(
            bin.content_type,
            bin.data,
            Some(bin.name),
        )));
    }

    if !error_replacements.is_empty() {
        log::info!(
            "[byop] {} attachment(s) replaced with ERROR text — model {api_type:?}/{model_id} \
             does not support: {error_replacements:?}",
            error_replacements.len()
        );
    }

    // 若 binaries 全是被替换的 ERROR 文本(没有真正的 Binary part),仍保留 ERROR 文本 part
    // 让模型看到。退化情况(例如 text 为空 + 没有任何 part 加进来)兜底纯文本。
    if parts.len() == 1 {
        if let Some(ContentPart::Text(t)) = parts.into_iter().next() {
            return ChatMessage::user(t);
        }
        return ChatMessage::user("");
    }

    ChatMessage {
        role: ChatRole::User,
        content: MessageContent::from_parts(parts),
        options: None,
    }
}

/// MIME → modality 字符串映射。对齐 opencode `mimeToModality`
/// (packages/opencode/src/provider/transform.ts:12-18)。
fn mime_to_modality(mime: &str) -> &'static str {
    let lower = mime.trim().to_ascii_lowercase();
    if lower.starts_with("image/") {
        "image"
    } else if lower.starts_with("audio/") {
        "audio"
    } else if lower.starts_with("video/") {
        "video"
    } else if lower == "application/pdf" {
        "pdf"
    } else {
        "file"
    }
}

/// 把 RequestParams 翻译为 genai `ChatRequest`(含 system + messages + tools)。
///
/// `force_echo_reasoning`:由 `super::reasoning::model_requires_reasoning_echo`
/// 决定。true 时所有 assistant message 强制挂 reasoning_content(空串占位),
/// 修复 DeepSeek-v4-flash / Kimi 等收紧校验的 thinking-mode endpoint。
fn build_chat_request(
    params: &RequestParams,
    force_echo_reasoning: bool,
    api_type: AgentProviderApiType,
    model_id: &str,
) -> ChatRequest {
    let agent_ctx = latest_input_context(&params.input);
    let plan_mode = is_plan_mode_turn(&params.input);
    let tool_names = available_tool_names(params);
    let mut system_text =
        prompt_renderer::render_system(&params.model, agent_ctx, &tool_names, plan_mode);
    // OpenWarp:legacy SSH 会话画像补丁。`render_system` 走 AIAgentContext,
    // 拿到的 OS/shell 是本地客户端;legacy SSH 下 PTY 实际在远端,
    // 追加一段 SSH 状态块矫正 LLM 推断。
    if let Some(ssh_block) = render_ssh_session_block(&params.session_context) {
        system_text.push_str(&ssh_block);
    }
    // 注:LRC / 长命令的工具用法引导(write_to_long_running_shell_command + command_id +
    // 各种 mode 与 raw 字节序列)已经在 `prompts/system/default.j2:69-79` 完整覆盖。
    // 用户当前所处的具体 PTY 上下文(命令名 / alt-screen 标志 / grid 内容)通过
    // user message 前缀的 `<attached_running_command>` XML 块单独注入(见
    // `render_running_command_context` 与 build_chat_request 中的 UserQuery 分支)。
    // 不在 system 这层重复硬编码 TUI 退出键之类,避免与 default.j2 的标准引导冲突或冗余。

    let mut messages: Vec<ChatMessage> = Vec::new();

    // 收集所有 task 的 messages,按时间戳排序。
    let mut all_msgs: Vec<&api::Message> = params
        .tasks
        .iter()
        .flat_map(|t| t.messages.iter())
        .collect();
    all_msgs.sort_by_key(|m| {
        m.timestamp
            .as_ref()
            .map(|ts| (ts.seconds, ts.nanos))
            .unwrap_or((0, 0))
    });

    // OpenWarp BYOP 本地会话压缩:把 conversation.compaction_state 应用到 message 序列。
    //   1. 过滤已被某次压缩覆盖的 (user, assistant) 对(`hidden_message_ids`)
    //   2. 在被隐去区间的位置插入一对合成的 (user "已压缩,以下为摘要" + assistant 摘要文本) message —
    //      这一步通过 `summary_inserts` 索引在主 loop 里就近 emit
    //   3. ToolCallResult 的 marker.tool_output_compacted_at 不为空时,后面分支替换 content 为占位符
    //
    // 当前 input 是 `AIAgentInput::SummarizeConversation` 时:进一步用 select 算法把 messages
    // 切到 head(去掉 tail),最后 input loop 末尾会追加 `build_prompt(...)` 作为 user message
    // (走完整的 SUMMARY_TEMPLATE),让上游 LLM 输出结构化摘要。
    let is_summarization_request = params
        .input
        .iter()
        .any(|i| matches!(i, AIAgentInput::SummarizeConversation { .. }));
    let summarization_overflow = params.input.iter().any(|i| {
        matches!(
            i,
            AIAgentInput::SummarizeConversation { overflow: true, .. }
        )
    });
    let _ = summarization_overflow; // 当前在 input loop 内的 follow-up 文案分支会用,目前先 silence dead

    let summary_inserts: std::collections::HashMap<String, String> =
        if let Some(state) = params.compaction_state.as_ref() {
            // user_msg_id → summary_text;遇到该 user_msg_id 时(它本来要被 hidden)替换为合成的摘要对
            state
                .completed()
                .iter()
                .filter_map(|c| {
                    c.summary_text
                        .as_ref()
                        .map(|s| (c.user_msg_id.clone(), s.clone()))
                })
                .collect()
        } else {
            std::collections::HashMap::new()
        };
    let hidden_msg_ids: std::collections::HashSet<String> = params
        .compaction_state
        .as_ref()
        .map(|s| s.hidden_message_ids())
        .unwrap_or_default();
    let compacted_tool_msg_ids: std::collections::HashSet<String> = params
        .compaction_state
        .as_ref()
        .map(|s| {
            // 收集所有标记了 tool_output_compacted_at 的 ToolCallResult message_ids
            // 通过遍历 all_msgs 并查 marker 实现
            let mut out = std::collections::HashSet::new();
            for msg in &all_msgs {
                if let Some(api::message::Message::ToolCallResult(_)) = &msg.message {
                    if s.marker(&msg.id)
                        .and_then(|m| m.tool_output_compacted_at)
                        .is_some()
                    {
                        out.insert(msg.id.clone());
                    }
                }
            }
            out
        })
        .unwrap_or_default();

    // 摘要请求路径:用 byop_compaction::algorithm::select 切 head;tail 不送上游
    let summarize_head_end: Option<usize> = if is_summarization_request {
        // 临时投影成 WarpMessageView 算 select
        let state_for_select = params.compaction_state.clone().unwrap_or_default();
        let tool_names =
            byop_compaction::message_view::build_tool_name_lookup(all_msgs.iter().copied());
        let views =
            byop_compaction::message_view::project(&all_msgs, &state_for_select, &tool_names);
        let cfg = byop_compaction::CompactionConfig::default();
        let model_limit = byop_compaction::overflow::ModelLimit::FALLBACK;
        let result = byop_compaction::algorithm::select(&views, &cfg, model_limit, |slice| {
            slice
                .iter()
                .map(byop_compaction::algorithm::MessageRef::estimate_size)
                .sum()
        });
        // head_end 是 views 里"head 区间"上界,与 all_msgs 同序
        Some(result.head_end)
    } else {
        None
    };

    let mut buf = AssistantBuffer::new(force_echo_reasoning);
    // OpenWarp:历史里被 skip 掉的 subagent ToolCall 对应的 call_id —— 它们的
    // ToolCallResult 也必须 skip,否则会成为孤儿 tool_response,Anthropic 直接 400
    // `unexpected tool_use_id ... no corresponding tool_use block`。
    let mut skipped_subagent_call_ids: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (idx, msg) in all_msgs.iter().enumerate() {
        // 摘要请求:tail 区间不送上游(只送 head + 末尾追加 SUMMARY_TEMPLATE)
        if let Some(head_end) = summarize_head_end {
            if idx >= head_end {
                continue;
            }
        }
        if hidden_msg_ids.contains(&msg.id) {
            if let Some(summary_text) = summary_inserts.get(&msg.id) {
                buf.flush_into(&mut messages);
                messages.push(ChatMessage::user(
                    "Conversation history was compacted. Below is the structured summary of all prior turns.".to_string(),
                ));
                messages.push(ChatMessage::assistant(summary_text.clone()));
            }
            continue;
        }
        let Some(inner) = &msg.message else {
            continue;
        };
        match inner {
            api::message::Message::UserQuery(u) => {
                buf.flush_into(&mut messages);
                // OpenWarp:历史轮多模态保活。warp 自家路径靠云端 server 重注入 InputContext,
                // BYOP 直连没有那层,所以 `make_user_query_message` 持久化时把所有 binary
                // (image / pdf / audio)塞进了 `UserQuery.context.images`,这里反向恢复成
                // UserBinary 走 `build_user_message_with_binaries`,使后续轮模型仍能看到先前
                // 粘贴的多模态附件。模型 caps 不支持的 mime 由 build_user_message_with_binaries
                // 替换为 ERROR 文本(opencode unsupportedParts 风格),不会静默 drop。
                // 没有 binary → 退回老路 `ChatMessage::user(text)`,与修复前等价。
                let history_binaries: Vec<user_context::UserBinary> = u
                    .context
                    .as_ref()
                    .map(|ctx| {
                        ctx.images
                            .iter()
                            .filter(|b| !b.data.is_empty())
                            .enumerate()
                            .map(|(idx, b)| {
                                use base64::Engine;
                                user_context::UserBinary {
                                    name: format!("history-attachment-{}-{idx}", &msg.id),
                                    content_type: if b.mime_type.is_empty() {
                                        "application/octet-stream".to_string()
                                    } else {
                                        b.mime_type.clone()
                                    },
                                    data: base64::engine::general_purpose::STANDARD.encode(&b.data),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if history_binaries.is_empty() {
                    messages.push(ChatMessage::user(u.query.clone()));
                } else {
                    messages.push(build_user_message_with_binaries(
                        u.query.clone(),
                        history_binaries,
                        api_type,
                        model_id,
                    ));
                }
            }
            api::message::Message::AgentReasoning(r) => {
                // 把上一轮的 reasoning 挂到下一个要 flush 的 assistant message 上。
                // genai 0.6 的 with_reasoning_content 会按当前 adapter 序列化:
                // DeepSeek/Kimi → reasoning_content 字段;Anthropic → thinking blocks。
                // 多段 AgentReasoning 累加(同一 turn 可能 stream 出多个 reasoning chunk
                // 落地为多条 AgentReasoning)。
                let next = r.reasoning.clone();
                if !next.is_empty() {
                    match buf.reasoning.as_mut() {
                        Some(existing) => existing.push_str(&next),
                        None => buf.reasoning = Some(next),
                    }
                }
            }
            api::message::Message::AgentOutput(a) => {
                if buf.text.is_some() || !buf.tool_calls.is_empty() {
                    buf.flush_into(&mut messages);
                }
                buf.text = Some(a.text.clone());
            }
            api::message::Message::ToolCall(tc) => {
                // OpenWarp BYOP:**虚拟 subagent tool_call 不发给上游模型**。
                // LRC tag-in 场景下,我们在 chat_stream 流头合成 `Tool::Subagent { metadata: Cli }`
                // 写入 root.task.messages,只用于触发 conversation 创建 cli subtask + spawn 浮窗,
                // 它不是模型实际产出的工具调用,模型看到会 confused(多余 tool call + 没法回应)。
                // 同样它对应的 placeholder ToolResponse(由 sanitize_tool_call_pairs 补的)
                // 也要由下面 ToolCallResult 分支的 skip 逻辑配合过滤,避免出现
                // "tool_response 找不到匹配的 tool_call" 的不平衡。
                use crate::ai::agent::task::helper::ToolCallExt;
                if tc.subagent().is_some() {
                    skipped_subagent_call_ids.insert(tc.tool_call_id.clone());
                    continue;
                }
                let (name, args_json) = serialize_outgoing_tool_call(
                    tc,
                    params.mcp_context.as_ref(),
                    &msg.server_message_data,
                );
                buf.tool_calls.push(ToolCall {
                    call_id: tc.tool_call_id.clone(),
                    fn_name: name,
                    fn_arguments: args_json,
                    thought_signatures: None,
                });
            }
            api::message::Message::ToolCallResult(tcr) => {
                buf.flush_into(&mut messages);
                // OpenWarp:对应 ToolCall 已被 skip(subagent 虚拟 call)→ result 也 skip,
                // 否则留下孤儿 tool_response 导致上游 400。
                if skipped_subagent_call_ids.contains(&tcr.tool_call_id) {
                    continue;
                }
                // BYOP 持久化的 ToolCallResult 走 server_message_data(content 已是 JSON 字符串);
                // server 端 emit 走 result oneof 结构化 variant — 兼容两路。
                let content = if compacted_tool_msg_ids.contains(&msg.id) {
                    // 压缩投影:被 prune 的 tool output 替换为占位符,不送实际内容上游
                    r#"{"status":"compacted","note":"tool output was pruned by local compaction"}"#
                        .to_string()
                } else if tcr.result.is_some() {
                    tools::serialize_result(tcr)
                } else if !msg.server_message_data.is_empty() {
                    msg.server_message_data.clone()
                } else {
                    r#"{"status":"empty"}"#.to_owned()
                };
                messages.push(ChatMessage::from(ToolResponse::new(
                    tcr.tool_call_id.clone(),
                    content,
                )));
            }
            _ => {
                // 其他 message 类型(SystemQuery/UpdateTodos/...)BYOP 暂不送上游。
            }
        }
    }
    buf.flush_into(&mut messages);

    // 当前轮新输入 → 追加。
    for input in &params.input {
        match input {
            AIAgentInput::UserQuery {
                query,
                context,
                running_command,
                ..
            } => {
                // 当前轮 UserQuery 自带的附件类 context(Block / SelectedText / File / Image)
                // 严格对齐 warp 自家路径走 `api::InputContext.executed_shell_commands` 等字段
                // 上行后由后端注入 prompt 的语义。BYOP 没有后端这层,直接 prepend 到 user message。
                // 环境型 context(env / git / skills / ...)由 prompt_renderer 渲染进 system,
                // 与本路径不重叠。
                //
                // OpenWarp:LRC tag-in 场景下,`running_command: Some(...)` 含完整 PTY 上下文
                // (alt-screen grid_contents + command + is_alt_screen_active 标志),用
                // `render_running_command_context` 渲成 `<attached_running_command>` XML 块
                // prepend 到 user message,模型据此决定调 write_to_long_running_shell_command。
                // 没填(普通对话或 controller 没注入)时回退到 `lrc_command_id` 简短上下文。
                let mut prefixes: Vec<String> = Vec::new();
                let request_running_command = running_command
                    .as_ref()
                    .or(params.lrc_running_command.as_ref());
                if let Some(rc) = request_running_command {
                    prefixes.push(render_running_command_context(rc));
                } else if let Some(command_id) = params.lrc_command_id.as_deref() {
                    prefixes.push(render_running_command_id_context(command_id));
                }
                let user_attachments = user_context::collect_user_attachments(context);
                if let Some(p) = &user_attachments.prefix {
                    prefixes.push(p.clone());
                }
                let full_text = if prefixes.is_empty() {
                    query.clone()
                } else {
                    format!("{}\n\n{query}", prefixes.join("\n\n"))
                };
                log::info!(
                    "[byop-diag] build_chat_request UserQuery: query_len={} \
                     running_command={} prefixes={} full_text_len={} binaries={}",
                    query.len(),
                    match request_running_command {
                        Some(rc) => format!(
                            "Some(grid_len={} alt={})",
                            rc.grid_contents.len(),
                            rc.is_alt_screen_active
                        ),
                        None => "None".to_owned(),
                    },
                    prefixes.len(),
                    full_text.len(),
                    user_attachments.binaries.len(),
                );
                messages.push(build_user_message_with_binaries(
                    full_text,
                    user_attachments.binaries,
                    api_type,
                    model_id,
                ));
            }
            AIAgentInput::ActionResult { result, .. } => {
                // 上一轮模型回了 tool_calls,client 端执行完后 result 走 `params.input`
                // 而不是 `params.tasks` 历史。必须在这里序列化为 ToolResponse,否则
                // genai/上游会因 tool_call_id 配对失败 400。
                let tool_call_id = result.id.to_string();
                let content = tools::serialize_action_result(result).unwrap_or_else(|| {
                    serde_json::json!({ "result": result.result.to_string() }).to_string()
                });
                messages.push(ChatMessage::from(ToolResponse::new(tool_call_id, content)));
            }
            AIAgentInput::InvokeSkill {
                skill, user_query, ..
            } => {
                let mut composed = format!(
                    "请按下面的技能 \"{}\" 指引执行任务:\n\n{}\n\n---\n",
                    skill.name, skill.content,
                );
                if let Some(uq) = user_query {
                    composed.push_str(&format!("用户进一步指令: {}", uq.query));
                }
                messages.push(ChatMessage::user(composed));
            }
            AIAgentInput::ResumeConversation { context } => {
                // BYOP 没有 server 端 resume prompt 注入层。LRC auto-resume 时必须显式
                // 重带当前 PTY 上下文,否则错误恢复轮会退化成普通对话并重新选择 shell 工具。
                let mut prefixes: Vec<String> = Vec::new();
                if let Some(lrc_prefix) = render_lrc_request_context(params) {
                    prefixes.push(lrc_prefix);
                }
                let user_attachments = user_context::collect_user_attachments(context);
                if let Some(p) = &user_attachments.prefix {
                    prefixes.push(p.clone());
                }
                if !prefixes.is_empty() {
                    let full_text = format!("{}\n\nContinue.", prefixes.join("\n\n"));
                    messages.push(build_user_message_with_binaries(
                        full_text,
                        user_attachments.binaries,
                        api_type,
                        model_id,
                    ));
                }
            }
            AIAgentInput::SummarizeConversation {
                prompt,
                overflow: _,
            } => {
                // OpenWarp BYOP 本地会话压缩入口 — 1:1 对齐 opencode `compaction.ts processCompaction`。
                //
                // 此前 messages loop 已根据 `summarize_head_end` 把序列切到 head(去掉 tail);
                // 这里追加最后一条 user message:`build_prompt(previous_summary, plugin_context)`,
                // 它包含 SUMMARY_TEMPLATE(9 段 Markdown 模板)+ 增量摘要锚点。
                //
                // 模型会 emit 一段结构化 Markdown 摘要文本,controller 接到 stream 完成
                // 后把它写回 conversation.compaction_state(参见 Phase 6 controller 改动)。
                let prev_summary = params
                    .compaction_state
                    .as_ref()
                    .and_then(|s| s.previous_summary())
                    .map(str::to_string);
                let mut anchor_context: Vec<String> = Vec::new();
                if let Some(custom) = prompt.as_ref().filter(|p| !p.is_empty()) {
                    // /compact <自定义指令> 走这里 — 把用户指令拼到 plugin_context 段
                    anchor_context
                        .push(format!("Additional instructions from the user:\n{custom}"));
                }
                let nextp =
                    byop_compaction::prompt::build_prompt(prev_summary.as_deref(), &anchor_context);
                messages.push(ChatMessage::user(nextp));
            }
            AIAgentInput::AutoCodeDiffQuery { .. }
            | AIAgentInput::CreateNewProject { .. }
            | AIAgentInput::CodeReview { .. } => {
                // 暂时忽略
            }
            _ => {}
        }
    }

    // 防御性 sanitize: 确保每个 assistant tool_calls 后面跟着对应每个 call_id 的
    // ToolResponse。warp 自家协议有时把 tool result 消化成下一轮 AgentOutput,
    // 上游若未保留 ToolCallResult,会让 tool_calls 配对失败。
    sanitize_tool_call_pairs(&mut messages);

    // 防御性 sanitize: 确保 messages 末尾不是 assistant。
    // Anthropic / 部分网关不接受末尾为 assistant 的请求(prefill 仅特定模型支持),
    // 而 warp 的 `AIAgentInput::ResumeConversation`(handoff/auto-resume after error 等)
    // 不附加新 user 消息,会让序列末尾停在历史 assistant 上。
    // 这里统一兜底:末尾若是 assistant,追加一条隐式 user 消息让上游继续。
    ensure_ends_with_user(&mut messages);

    let tools_array = build_tools_array(params);

    // OpenWarp:整体 sanitize system / messages / tools 中所有会进入 JSON body 的字符串,
    // 移除 < 0x20 / DEL 控制字符(除 \n \r \t),并把 `\xNN` 这类危险字面量替换为
    // 普通文字,避免 Anthropic 或中间代理把它们误当成 JSON escape 后 400。
    // nvim 等 alt-screen TUI 的 grid_contents、tool result、工具描述和 schema description
    // 都可能带这些片段,所以不能只清理 user message 的 first_text。
    let system_text = sanitize_text_for_json(&system_text);
    let messages: Vec<ChatMessage> = messages
        .into_iter()
        .map(sanitize_chat_message_for_request)
        .collect();
    let tools_array: Vec<GenaiTool> = tools_array
        .into_iter()
        .map(sanitize_tool_for_request)
        .collect();

    let mut req = ChatRequest::from_messages(messages).with_system(system_text);
    if !tools_array.is_empty() {
        req = req.with_tools(tools_array);
    }
    req
}

/// 移除字符串中所有可能让 JSON 序列化产生非法转义的字符:
/// - 所有 ASCII 控制字符替换成空格,包括换行、回车和 tab
/// - DEL(0x7f)替换成空格
/// - 反斜杠替换成 `/`,双引号替换成单引号
///
/// 用途:防 ANSI escape 序列、Windows 路径、换行、字符串内引号等内容透到 BYOP 请求体。
/// 标准 JSON 允许这些 escape,但部分 Anthropic 兼容代理会在转发时把 escape
/// 处理坏并返回 `invalid escaped character in string`,因此这里统一压平成安全字符。
fn sanitize_text_for_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            c if (c as u32) < 0x20 => out.push(' '),
            '\u{7f}' => out.push(' '),
            '\\' => out.push('/'),
            '"' => out.push('\''),
            _ => out.push(c),
        }
    }
    replace_dangerous_escape_literals(out)
}

fn replace_dangerous_escape_literals(mut text: String) -> String {
    for (from, to) in [
        ("\\n", " "),
        ("\\r", " "),
        ("\\t", " "),
        ("\\x1b", "ESC"),
        ("\\x1B", "ESC"),
        ("\\x03", "Ctrl-C"),
        ("\\x04", "Ctrl-D"),
        ("\\x07", "BEL"),
        ("\\a", "BEL"),
        ("\\v", "vertical tab"),
    ] {
        text = text.replace(from, to);
    }
    text
}

fn sanitize_json_value_for_request(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(sanitize_text_for_json(&s)),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(sanitize_json_value_for_request)
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, sanitize_json_value_for_request(value)))
                .collect(),
        ),
        other => other,
    }
}

fn sanitize_chat_message_for_request(mut message: ChatMessage) -> ChatMessage {
    let parts = message
        .content
        .into_parts()
        .into_iter()
        .map(|part| match part {
            ContentPart::Text(text) => ContentPart::Text(sanitize_text_for_json(&text)),
            // ToolResponse.content 与 ToolCall.fn_arguments 本身就是
            // `serde_json::to_string` / `serde_json::json!` 产出的合法 JSON,
            // 让模型按 JSON 协议解析。再过一遍 sanitize_text_for_json 会把
            // `"` → `'`、`\` → `/`、控制字符压平,把合法 JSON 变成 Python-like
            // repr,模型彻底无法解析 retry 提示,陷入死循环改格式。
            // sanitize 仅对 prose(Text / Reasoning / ThoughtSignature)生效,
            // 结构化字段一律直通。
            ContentPart::ToolResponse(response) => ContentPart::ToolResponse(response),
            ContentPart::ToolCall(call) => ContentPart::ToolCall(call),
            ContentPart::ThoughtSignature(signature) => {
                ContentPart::ThoughtSignature(sanitize_text_for_json(&signature))
            }
            ContentPart::ReasoningContent(reasoning) => {
                ContentPart::ReasoningContent(sanitize_text_for_json(&reasoning))
            }
            ContentPart::Custom(mut custom) => {
                custom.data = sanitize_json_value_for_request(custom.data);
                ContentPart::Custom(custom)
            }
            other => other,
        })
        .collect::<Vec<_>>();
    message.content = MessageContent::from_parts(parts);
    message
}

fn sanitize_tool_for_request(mut tool: GenaiTool) -> GenaiTool {
    tool.description = tool
        .description
        .map(|description| sanitize_text_for_json(&description));
    tool.schema = tool.schema.map(sanitize_json_value_for_request);
    tool
}

/// 重排 messages 中所有 Tool 消息,确保:
/// 每个含 tool_calls 的 Assistant message 后面紧跟**且仅紧跟**一条 Tool message,
/// 内含该 Assistant **每个** call_id 的 ToolResponse(按 tool_calls 顺序,缺失补 placeholder)。
///
/// **为什么需要重排,而不是只补 placeholder / 剔孤儿**:
/// `build_chat_request` 按时间戳 chronological 排序合并所有 task 的历史 messages。
/// 当模型在一轮内发起多个 tool_call,且这些 tool 的执行时长差异较大时(如 read_skill
/// 立即返回错误,而 git/PowerShell 命令稍慢),后到的 ToolCallResult 时间戳可能晚于
/// 模型紧接着发起的**下一轮** Assistant tool_calls,导致历史 messages 被排成:
///
/// ```text
/// Asst-X(tc_a, tc_b, tc_c)
/// Tool(tc_c real)         ← read_skill 错误,快
/// Asst-Y(tc_d, tc_e)      ← 模型基于 tc_c 错误立刻发了下一轮
/// Tool(tc_a real)         ← git 命令慢,落到 Asst-Y 后面
/// Tool(tc_b real)
/// ```
///
/// Anthropic API 把连续 Tool block 合并视作"上一条 Assistant 的 tool_results",
/// 于是 Asst-Y 后面的 Tool block 含 tc_a/tc_b 这种 Asst-Y 不认识的 call_id → 400
/// `unexpected tool_use_id ... no corresponding tool_use block in the previous message`。
///
/// 旧实现只做"剔孤儿(整序列匹配)+补 placeholder(向前看相邻 Tool)",前者不会剔除
/// 这类**位置错误但 call_id 合法**的 ToolResponse,后者也不会重定位 — 所以 400 重现。
///
/// 新实现:抽出所有 ToolResponse 进 `call_id → response` 表,然后按每个 Assistant
/// tool_calls 的 call_id 顺序重新组装紧随其后的 Tool message。剩余未消费的 ToolResponse
/// (call_id 完全不在历史 Assistant tool_calls 里)即真孤儿,丢弃。
fn sanitize_tool_call_pairs(messages: &mut Vec<ChatMessage>) {
    use std::collections::HashMap;

    // 抽取所有 ToolResponse(同 call_id 后到的覆盖前面,符合"最新结果优先"语义)。
    let mut response_by_call_id: HashMap<String, ToolResponse> = HashMap::new();
    let original = std::mem::take(messages);
    let mut non_tool_msgs: Vec<ChatMessage> = Vec::with_capacity(original.len());
    for msg in original {
        if msg.role == genai::chat::ChatRole::Tool {
            for resp in msg.content.tool_responses() {
                response_by_call_id.insert(resp.call_id.clone(), (*resp).clone());
            }
        } else {
            non_tool_msgs.push(msg);
        }
    }

    // 重组:每个 Assistant 含 tool_calls 后紧跟一条 Tool message,按 call_id 顺序绑定。
    let mut placeholders_inserted: Vec<String> = Vec::new();
    for msg in non_tool_msgs {
        let call_ids: Vec<String> = msg
            .content
            .tool_calls()
            .iter()
            .map(|tc| tc.call_id.clone())
            .collect();
        let is_assistant = msg.role == genai::chat::ChatRole::Assistant;
        messages.push(msg);

        if is_assistant && !call_ids.is_empty() {
            let bundled: Vec<ToolResponse> = call_ids
                .iter()
                .map(|cid| {
                    response_by_call_id.remove(cid).unwrap_or_else(|| {
                        placeholders_inserted.push(cid.clone());
                        ToolResponse::new(cid.clone(), "(tool 执行结果未保留)".to_owned())
                    })
                })
                .collect();
            messages.push(ChatMessage::from(bundled));
        }
    }

    // 剩余 response_by_call_id 是真孤儿(没有任何 Assistant tool_call 与之配对),丢弃。
    if !response_by_call_id.is_empty() {
        let orphan_ids: Vec<&String> = response_by_call_id.keys().collect();
        log::warn!(
            "[byop-diag] sanitize_tool_call_pairs: 丢弃 {} 个孤儿 ToolResponse: \
             orphan_call_ids={:?}",
            response_by_call_id.len(),
            orphan_ids
        );
    }
    if !placeholders_inserted.is_empty() {
        log::warn!(
            "[byop-diag] sanitize_tool_call_pairs: 给 {} 个 ToolCall 补 placeholder \
             ToolResponse: missing_call_ids={:?}",
            placeholders_inserted.len(),
            placeholders_inserted
        );
    }
}

/// 兜底:确保 messages 末尾是 user(或 tool 响应)。
///
/// 触发场景:`AIAgentInput::ResumeConversation` 不附加新 user 消息,直接重发历史。
/// Anthropic 原生 API 拒绝末尾为 assistant 的请求(`This model does not support
/// assistant message prefill. The conversation must end with a user message.`),
/// 重试 3 次都同 payload → UI 渲染 error block 触发 flex panic。
///
/// 末尾是 assistant 时追加 `ChatMessage::user("Continue.")`,提示模型继续即可。
/// Tool 角色作为 user 输入的一种(模型会把 tool 响应当作下一轮起点)不动。
/// 空 messages 不触发,避免给空对话凭空塞内容。
fn ensure_ends_with_user(messages: &mut Vec<ChatMessage>) {
    use genai::chat::ChatRole;
    if let Some(last) = messages.last() {
        if last.role == ChatRole::Assistant {
            messages.push(ChatMessage::user("Continue."));
        }
    }
}

/// 反向: 把内部 `tool_call::Tool` variant 序列化成 (function name, arguments JSON Value)
/// 用于多轮历史回放。这里的 (name, args) 必须与 `tools::REGISTRY` 中各 tool 的 `name`
/// 与 `from_args` 期望的 schema 严格对齐。
fn serialize_outgoing_tool_call(
    tc: &api::message::ToolCall,
    mcp_ctx: Option<&crate::ai::agent::MCPContext>,
    server_message_data: &str,
) -> (String, Value) {
    use api::message::tool_call::Tool;

    // BYOP from_args 解析失败 carrier 还原:由 make_tool_call_carrier_message 写入,
    // tool oneof = None,原始 `<fn_name>\n<args_str>` 编码在 server_message_data。
    // 必须在主 match 之前优先识别,否则会落到下面 None=>"warp_internal_empty",
    // 上游模型看到一个不存在的工具名会更困惑、也不知道是哪个 call 失败了。
    if tc.tool.is_none() {
        if let Some((fn_name, raw_args)) = server_message_data.split_once('\n') {
            if !fn_name.is_empty() {
                let args_value = serde_json::from_str(raw_args)
                    .unwrap_or_else(|_| Value::String(raw_args.to_owned()));
                return (fn_name.to_owned(), args_value);
            }
        }
    }

    // 大多数旧实现返回 (String, String);这里改成 (String, Value),把字符串再 parse 一次。
    let (name, args_str) = match &tc.tool {
        Some(Tool::CallMcpTool(c)) => tools::mcp::serialize_outgoing_call(c, mcp_ctx),
        Some(Tool::ReadMcpResource(r)) => tools::mcp::serialize_outgoing_read_resource(r, mcp_ctx),
        Some(Tool::RunShellCommand(c)) => (
            "run_shell_command".to_owned(),
            json!({
                "command": c.command,
                "is_read_only": c.is_read_only,
                "uses_pager": c.uses_pager,
                "is_risky": c.is_risky,
            })
            .to_string(),
        ),
        Some(Tool::ReadFiles(r)) => {
            let files: Vec<Value> = r
                .files
                .iter()
                .map(|f| {
                    json!({
                        "path": f.name,
                        "line_ranges": f.line_ranges.iter().map(|lr| json!({
                            "start": lr.start, "end": lr.end
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            (
                "read_files".to_owned(),
                json!({ "files": files }).to_string(),
            )
        }
        Some(Tool::Grep(g)) => (
            "grep".to_owned(),
            json!({ "queries": g.queries, "path": g.path }).to_string(),
        ),
        Some(Tool::SearchCodebase(s)) => (
            "search_codebase".to_owned(),
            json!({
                "query": s.query,
                "path_filters": s.path_filters,
                "codebase_path": s.codebase_path,
            })
            .to_string(),
        ),
        Some(Tool::AskUserQuestion(a)) => {
            let questions: Vec<Value> = a
                .questions
                .iter()
                .map(|q| {
                    let (options, recommended_index, multi_select, supports_other) =
                        match &q.question_type {
                            Some(
                                api::ask_user_question::question::QuestionType::MultipleChoice(mc),
                            ) => (
                                mc.options
                                    .iter()
                                    .map(|o| o.label.clone())
                                    .collect::<Vec<_>>(),
                                mc.recommended_option_index,
                                mc.is_multiselect,
                                mc.supports_other,
                            ),
                            None => (vec![], 0, false, false),
                        };
                    json!({
                        "question": q.question,
                        "options": options,
                        "recommended_index": recommended_index,
                        "multi_select": multi_select,
                        "supports_other": supports_other,
                    })
                })
                .collect();
            (
                "ask_user_question".to_owned(),
                json!({ "questions": questions }).to_string(),
            )
        }
        Some(Tool::FileGlobV2(g)) => (
            "file_glob".to_owned(),
            json!({
                "patterns": g.patterns,
                "search_dir": g.search_dir,
                "limit": g.max_matches,
            })
            .to_string(),
        ),
        Some(Tool::ApplyFileDiffs(a)) => {
            let mut operations: Vec<Value> = Vec::new();
            for d in &a.diffs {
                operations.push(json!({
                    "op": "edit",
                    "file_path": d.file_path,
                    "search": d.search,
                    "replace": d.replace,
                }));
            }
            for f in &a.new_files {
                operations.push(json!({
                    "op": "create",
                    "file_path": f.file_path,
                    "content": f.content,
                }));
            }
            for f in &a.deleted_files {
                operations.push(json!({
                    "op": "delete",
                    "file_path": f.file_path,
                }));
            }
            (
                "apply_file_diffs".to_owned(),
                json!({ "summary": a.summary, "operations": operations }).to_string(),
            )
        }
        Some(Tool::WriteToLongRunningShellCommand(w)) => {
            use api::message::tool_call::write_to_long_running_shell_command::mode::Mode as M;
            let mode = match w.mode.as_ref().and_then(|m| m.mode.as_ref()) {
                Some(M::Raw(_)) => "raw",
                Some(M::Block(_)) => "block",
                _ => "line",
            };
            (
                "write_to_long_running_shell_command".to_owned(),
                json!({
                    "command_id": w.command_id,
                    "input": String::from_utf8_lossy(&w.input).to_string(),
                    "mode": mode,
                })
                .to_string(),
            )
        }
        Some(Tool::ReadDocuments(r)) => {
            let docs: Vec<Value> = r
                .documents
                .iter()
                .map(|d| {
                    json!({
                        "document_id": d.document_id,
                        "line_ranges": d.line_ranges.iter().map(|lr| json!({
                            "start": lr.start, "end": lr.end
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            (
                "read_documents".to_owned(),
                json!({ "documents": docs }).to_string(),
            )
        }
        Some(Tool::EditDocuments(e)) => {
            let diffs: Vec<Value> = e
                .diffs
                .iter()
                .map(|d| {
                    json!({
                        "document_id": d.document_id,
                        "search": d.search,
                        "replace": d.replace,
                    })
                })
                .collect();
            (
                "edit_documents".to_owned(),
                json!({ "diffs": diffs }).to_string(),
            )
        }
        Some(Tool::CreateDocuments(c)) => {
            let new_documents: Vec<Value> = c
                .new_documents
                .iter()
                .map(|d| json!({ "title": d.title, "content": d.content }))
                .collect();
            (
                "create_documents".to_owned(),
                json!({ "new_documents": new_documents }).to_string(),
            )
        }
        Some(Tool::SuggestNewConversation(s)) => (
            "suggest_new_conversation".to_owned(),
            json!({ "message_id": s.message_id }).to_string(),
        ),
        Some(Tool::SuggestPrompt(s)) => {
            use api::message::tool_call::suggest_prompt::DisplayMode;
            let (prompt, label) = match &s.display_mode {
                Some(DisplayMode::PromptChip(c)) => (c.prompt.clone(), c.label.clone()),
                Some(DisplayMode::InlineQueryBanner(b)) => (b.query.clone(), b.title.clone()),
                None => (String::new(), String::new()),
            };
            (
                "suggest_prompt".to_owned(),
                json!({ "prompt": prompt, "label": label }).to_string(),
            )
        }
        Some(Tool::OpenCodeReview(_)) => ("open_code_review".to_owned(), "{}".to_owned()),
        Some(Tool::TransferShellCommandControlToUser(t)) => (
            "transfer_shell_command_control_to_user".to_owned(),
            json!({ "reason": t.reason }).to_string(),
        ),
        Some(Tool::ReadSkill(r)) => {
            use api::message::tool_call::read_skill::SkillReference;
            let path = match &r.skill_reference {
                Some(SkillReference::SkillPath(p)) => p.clone(),
                Some(SkillReference::BundledSkillId(id)) => format!("bundled:{id}"),
                None => String::new(),
            };
            (
                "read_skill".to_owned(),
                json!({ "skill_path": path }).to_string(),
            )
        }
        Some(Tool::ReadShellCommandOutput(r)) => {
            use api::message::tool_call::read_shell_command_output::Delay;
            let delay_seconds = match &r.delay {
                Some(Delay::Duration(d)) => Some(d.seconds),
                Some(Delay::OnCompletion(_)) | None => None,
            };
            let mut args = json!({ "command_id": r.command_id });
            if let Some(s) = delay_seconds {
                args["delay_seconds"] = json!(s);
            }
            ("read_shell_command_output".to_owned(), args.to_string())
        }
        Some(other) => {
            let variant_name = format!("{other:?}")
                .split('(')
                .next()
                .unwrap_or("UnknownVariant")
                .to_owned();
            (format!("warp_internal_{}", variant_name), "{}".to_owned())
        }
        None => ("warp_internal_empty".to_owned(), "{}".to_owned()),
    };
    let args_value: Value =
        serde_json::from_str(&args_str).unwrap_or(Value::Object(Default::default()));
    (name, args_value)
}

// ---------------------------------------------------------------------------
// Tools 数组
// ---------------------------------------------------------------------------

/// 本轮 input 是否含 `/plan` 触发的 `UserQueryMode::Plan`。
///
/// per-turn 语义:只看本轮 `params.input` 是否带 Plan 标记。历史 task message
/// 当前的持久化路径(`make_user_query_message`)用 `..Default::default()` 写入
/// 上游 proto,**不带 mode 字段**;所以 plan 状态不会自动跨轮 sticky,用户每条
/// 想保持只读的 query 都需重新加 `/plan ` 前缀。这是有意为之的 MVP 形态:
/// - 实施成本最低(无需改 proto schema、无需新会话级状态机)
/// - 与 Claude Code `EnterPlanMode` 的"显式进入/退出"语义一致 —— 只是这里把
///   退出动作隐含在"下一条不带 /plan"
fn is_plan_mode_turn(input: &[AIAgentInput]) -> bool {
    input.iter().any(|i| {
        matches!(
            i,
            AIAgentInput::UserQuery {
                user_query_mode: UserQueryMode::Plan,
                ..
            }
        )
    })
}

/// Plan Mode 下硬过滤的写/执行类内置工具名。
///
/// 逻辑兜底,即使模型无视 `partials/plan_mode.j2` 的引导也无法触发副作用 ——
/// 工具不在 tool list 里就调用不到(provider 协议层会直接拒绝 unknown function)。
///
/// **没被 BLOCK 的写类工具**:`create_documents` / `edit_documents`。它们只动
/// Warp Drive 本地文档存储(AIDocumentModel),不碰文件系统、不跑命令,语义上
/// 恰好是 Plan Mode 的产出归档动作 —— 模型把最终 plan 沉淀为 Drive 文档,
/// 用户后续可在 Drive UI 中查看 / 编辑 / 拖入自建的 PLAN 文件夹复用。
///
/// 留下的只读 + Drive 写子集:`read_files / grep / file_glob_v2 /
/// read_shell_command_output / ask_user_question / read_skill / read_documents /
/// create_documents / edit_documents / webfetch / websearch / mcp/*`。
const PLAN_MODE_BLOCKED_TOOLS: &[&str] = &[
    "run_shell_command",
    "apply_file_diffs",
    "write_to_long_running_shell_command",
    "open_code_review",
    "transfer_shell_command_control_to_user",
    "suggest_prompt",
];

/// 列出本轮真正会喂给上游模型的 tool name(内置 REGISTRY + 当前 MCP 工具),
/// 与 `build_tools_array` 共享同一套 gating(LRC / `web_search_enabled` /
/// `suggest_new_conversation` / `plan_mode`)。供 `prompt_renderer` 注入到
/// system prompt,让模板按实际可用列表动态渲染,不再硬编码白/黑名单。
pub fn available_tool_names(params: &RequestParams) -> Vec<String> {
    let is_lrc = params.lrc_command_id.is_some();
    let web_enabled = params.web_search_enabled;
    let plan_mode = is_plan_mode_turn(&params.input);
    let mut names: Vec<String> = tools::REGISTRY
        .iter()
        .filter(|t| {
            if is_lrc && t.name == "run_shell_command" {
                return false;
            }
            if !web_enabled
                && (t.name == tools::webfetch::TOOL_NAME || t.name == tools::websearch::TOOL_NAME)
            {
                return false;
            }
            if t.name == "suggest_new_conversation" {
                return false;
            }
            if plan_mode && PLAN_MODE_BLOCKED_TOOLS.contains(&t.name) {
                return false;
            }
            true
        })
        .map(|t| t.name.to_owned())
        .collect();
    if let Some(ctx) = params.mcp_context.as_ref() {
        for (name, _description, _parameters) in tools::mcp::build_mcp_tool_defs(ctx) {
            names.push(name);
        }
    }
    names
}

fn build_tools_array(params: &RequestParams) -> Vec<GenaiTool> {
    // OpenWarp A2:LRC tag-in 场景剔除 `run_shell_command`,迫使模型选 PTY 操作类工具。
    //
    // 在 alt-screen 长命令(nvim/htop)+ 用户 tag-in 状态下,**模型最容易犯的错**是
    // 调 `run_shell_command` 跑 `taskkill nvim` / `Stop-Process nvim`(开新进程),
    // 这跟当前正在跑的 PTY 没关系,杀不到目标。**正确做法**是
    // `write_to_long_running_shell_command(command_id, input=":q\n", mode=raw)`,
    // 直接给当前 PTY 发指令。
    //
    // 实测带 system prompt 引导 + RunningCommand context prefix 都不够强,
    // 模型仍然倾向 run_shell_command(更简单)。最干净的硬约束就是从 tools 列表
    // 直接移除该工具,模型只能在 PTY 操作类工具中选。
    //
    // 其他工具保留(read_files/grep/ask_user_question 等),允许模型做必要的
    // 信息收集和反问。
    let is_lrc = params.lrc_command_id.is_some();
    let web_enabled = params.web_search_enabled;
    let plan_mode = is_plan_mode_turn(&params.input);
    // OpenWarp BYOP:`suggest_prompt` chip UI 已通过 view 层订阅
    // PromptSuggestionExecutorEvent 恢复(见 `terminal/view.rs::
    // handle_suggest_prompt_executor_event`),可以暴露给模型。
    // `suggest_new_conversation` 仍 filter:UX 没有现成弹窗组件,executor 已改为
    // fast-fail Cancelled(见 `action_model/execute/suggest_new_conversation.rs`),
    // filter 是冗余防御以避免无效调用噪声。
    // 动态占位替换:某些工具描述含 `{{year}}`(如 websearch,对齐 opencode
    // websearch.ts:30-32 的 description getter),build 时替换成当前年份。
    // 模型每次看到的描述都带正确年份,不会被训练数据里的旧年份污染。
    let current_year = chrono::Local::now().format("%Y").to_string();
    let mut out: Vec<GenaiTool> = tools::REGISTRY
        .iter()
        .filter(|t| {
            if is_lrc && t.name == "run_shell_command" {
                return false;
            }
            // BYOP web 工具按 profile.web_search_enabled gating(用户已关闭隐私
            // 开关时不暴露给上游模型,避免误调外网请求)。
            if !web_enabled
                && (t.name == tools::webfetch::TOOL_NAME || t.name == tools::websearch::TOOL_NAME)
            {
                return false;
            }
            // suggest_new_conversation:无 UI 实现,executor 在 OpenWarp 改为
            // fast-fail Cancelled。这里 filter 掉避免模型调用产生无意义的
            // tool_call→cancelled 往返(纯 token 浪费)。
            if t.name == "suggest_new_conversation" {
                return false;
            }
            // Plan Mode:`/plan` 触发的只读模式硬护栏,移除写/执行类工具。
            // 与 system prompt 的 plan_mode.j2 引导双重保险 —— 即便模型无视
            // 提示词,工具不在列表里也无法触发副作用(provider 协议层
            // 会直接拒绝 unknown function)。
            if plan_mode && PLAN_MODE_BLOCKED_TOOLS.contains(&t.name) {
                return false;
            }
            true
        })
        .map(|t| {
            let description = if t.description.contains("{{year}}") {
                t.description.replace("{{year}}", &current_year)
            } else {
                t.description.to_owned()
            };
            GenaiTool::new(t.name)
                .with_description(description)
                .with_schema((t.parameters)())
        })
        .collect();

    if let Some(ctx) = params.mcp_context.as_ref() {
        for (name, description, parameters) in tools::mcp::build_mcp_tool_defs(ctx) {
            out.push(
                GenaiTool::new(name)
                    .with_description(description)
                    .with_schema(parameters),
            );
        }
    }
    if is_lrc {
        log::info!(
            "[byop] LRC tag-in: tools array filtered (removed run_shell_command), \
             total tools={}",
            out.len()
        );
    }
    if plan_mode {
        log::info!(
            "[byop] Plan Mode: tools array filtered (removed write/exec tools: {:?}), \
             total tools={}",
            PLAN_MODE_BLOCKED_TOOLS,
            out.len()
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Client / 路由
// ---------------------------------------------------------------------------

/// 把 `AgentProviderApiType` 一对一映射到 genai `AdapterKind`。
fn adapter_kind_for(api_type: AgentProviderApiType) -> AdapterKind {
    match api_type {
        AgentProviderApiType::OpenAi => AdapterKind::OpenAI,
        AgentProviderApiType::OpenAiResp => AdapterKind::OpenAIResp,
        AgentProviderApiType::Gemini => AdapterKind::Gemini,
        AgentProviderApiType::Anthropic => AdapterKind::Anthropic,
        AgentProviderApiType::Ollama => AdapterKind::Ollama,
        AgentProviderApiType::DeepSeek => AdapterKind::DeepSeek,
    }
}

/// 规范化用户填写的 `base_url`,产出供 genai adapter 拼接 service path 的 endpoint URL。
///
/// genai 0.6.x 所有 adapter 都假设 endpoint 以 `/` 结尾、且已经包含版本路径段:
/// - Anthropic:`format!("{base_url}messages")` 期望 `…/v1/`
/// - Gemini:`format!("{base_url}models/{m}:streamGenerateContent")` 期望 `…/v1beta/`
/// - OpenAI / OpenAIResp / DeepSeek:`Url::join("chat/completions" 或 "responses")` 期望 `…/v1/`
/// - Ollama:`format!("{base_url}api/chat")` 期望根路径 `…/`
///
/// 用户实际三种填法:
/// 1. 纯 host(`https://ai.zerx.dev`)— 早期默认行为只补尾 `/` 会拼成 `https://ai.zerx.dev/messages`
///    漏掉 `/v1/` 导致 404。**这里按 api_type 自动追加默认版本路径段**(Anthropic/OpenAI 系→`/v1/`,
///    Gemini→`/v1beta/`,Ollama 不补)。
/// 2. 完整带版本路径(`https://ai.zerx.dev/v1`)— 仅补尾 `/`,不动 path。
/// 3. 留空 — 用 [`AgentProviderApiType::default_base_url`]。
fn normalize_endpoint_url(api_type: AgentProviderApiType, base_url: &str) -> String {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return api_type.default_base_url().to_owned();
    }

    // 解析失败(用户填了畸形 URL)→ 退化到原"补尾 /"行为,让上游报错而不是这里 panic。
    let parsed = match url::Url::parse(trimmed) {
        Ok(u) => u,
        Err(_) => {
            let stripped = trimmed.trim_end_matches('/');
            return format!("{stripped}/");
        }
    };

    // path == "/" 或为空 → 用户只填了 host,自动补上 api_type 默认版本路径段。
    if parsed.path() == "/" || parsed.path().is_empty() {
        // 从 default_base_url 抽 path 部分(如 "/v1/" / "/v1beta/" / "/")。
        let default_path = url::Url::parse(api_type.default_base_url())
            .ok()
            .map(|u| u.path().to_owned())
            .unwrap_or_else(|| "/".to_owned());
        let host_part = trimmed.trim_end_matches('/');
        return format!("{host_part}{default_path}");
    }

    // 用户已自带 path → 仅确保尾随 `/`(genai format!/Url::join 都依赖)。
    let stripped = trimmed.trim_end_matches('/');
    format!("{stripped}/")
}

/// 构造 genai Client。每次请求新建(开销低 — Client 内部只是 reqwest::Client + adapter 表)。
/// `ServiceTargetResolver` capture 当前请求的 endpoint/key/api_type 后,把每次 exec_chat_stream
/// 都强制路由到指定 AdapterKind,完全绕过 genai 默认的"按模型名识别"。
pub(super) fn build_client(
    api_type: AgentProviderApiType,
    base_url: String,
    api_key: String,
) -> Client {
    let adapter_kind = adapter_kind_for(api_type);
    let endpoint_url = normalize_endpoint_url(api_type, &base_url);
    log::info!("[byop] build_client: adapter={adapter_kind:?} endpoint_url={endpoint_url}");
    let key_for_resolver = api_key.clone();
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            let ServiceTarget { model, .. } = service_target;
            let endpoint = Endpoint::from_owned(endpoint_url.clone());
            let auth = AuthData::from_single(key_for_resolver.clone());
            // 用我们指定的 AdapterKind 覆盖 genai 的"按模型名"识别结果,
            // 但保留 model_name 以便上游服务正确寻址模型。
            let model = ModelIden::new(adapter_kind, model.model_name);
            Ok(ServiceTarget {
                endpoint,
                auth,
                model,
            })
        },
    );

    // OpenWarp BYOP:SSE 流必须不带 gzip。`Accept-Encoding: gzip` 会让 nginx
    // 类代理把响应压缩,server 必须 flush 完整 deflate frame 客户端才能解出
    // 明文,流式语义被破坏成 ~K 字节 burst,体感"几百毫秒一卡"。zed/opencode
    // 用 native fetch / std HTTP 不主动协商 gzip on SSE,所以同代理无问题。
    //
    // 这里显式构造 `WebConfig` 即使 genai default 已经 `gzip=false`(fork 修改)。
    let web_config = WebConfig {
        gzip: false,
        ..WebConfig::default()
    };
    Client::builder()
        .with_web_config(web_config)
        .with_service_target_resolver(resolver)
        .build()
}

/// 判定是否给 DashScope(阿里云百炼,OpenAI 兼容路径)注入 `enable_thinking: true`。
///
/// 对齐 opencode `transform.ts:931-938`(provider/transform.ts L926+ 的注释):
/// 「DashScope 默认不开 thinking,qwen3 / qwq / deepseek-r1 / kimi-k2.5 / qwen-plus
/// 等 reasoning 模型必须显式 `enable_thinking: true` 才会输出 reasoning_content」。
///
/// 命中条件(全部满足):
/// 1. `api_type == OpenAi`(DashScope 走 OpenAI 兼容路径)
/// 2. `effort_setting != Off`(用户主动关思考时尊重之,不注入)
/// 3. base_url 含 `dashscope.aliyuncs.com` / `dashscope.cn` / `dashscope-intl.aliyuncs.com`
/// 4. model_id 不含 `kimi-k2-thinking`(opencode 排除,该模型默认就 thinking)
/// 5. model_id 命中 reasoning 子串白名单:`qwen3` / `qwq` / `deepseek-r1` / `kimi-k2.5` /
///    `kimi-k2-` / `qwen-plus`(避免给 qwen-turbo / qwen2.5 等纯 chat 模型乱塞)
fn dashscope_needs_enable_thinking(
    api_type: AgentProviderApiType,
    base_url: &str,
    model_id: &str,
    effort_setting: crate::settings::ReasoningEffortSetting,
) -> bool {
    if !matches!(api_type, AgentProviderApiType::OpenAi) {
        return false;
    }
    if matches!(effort_setting, crate::settings::ReasoningEffortSetting::Off) {
        return false;
    }
    let url = base_url.to_ascii_lowercase();
    let is_dashscope = url.contains("dashscope.aliyuncs.com")
        || url.contains("dashscope.cn")
        || url.contains("dashscope-intl.aliyuncs.com");
    if !is_dashscope {
        return false;
    }
    let id = model_id.to_ascii_lowercase();
    if id.contains("kimi-k2-thinking") {
        return false;
    }
    id.contains("qwen3")
        || id.contains("qwq")
        || id.contains("deepseek-r1")
        || id.contains("kimi-k2.5")
        || id.contains("kimi-k2-")
        || id.contains("qwen-plus")
}

fn build_chat_options(
    api_type: AgentProviderApiType,
    base_url: &str,
    model_id: &str,
    effort_setting: crate::settings::ReasoningEffortSetting,
) -> ChatOptions {
    let mut opts = ChatOptions::default()
        .with_capture_content(true)
        .with_capture_tool_calls(true)
        .with_capture_reasoning_content(true)
        .with_capture_usage(true)
        // 让 genai 把 DeepSeek-style 模型在 content 中夹带的 <think>...</think>
        // 段抽出来归到 reasoning chunk,UI 显示更干净。仅对支持该格式的 adapter 生效。
        .with_normalize_reasoning_content(true);

    // 仅在用户显式选了非 Auto 档位 **且** 模型支持 reasoning 时才注入。
    // - Auto:不传,让 genai 走"模型名后缀推断"(OpenAI/Anthropic adapter 内部)。
    // - 非 Auto + 模型不支持:也不传,避免向 claude-3-5-haiku / gpt-4o / gemini-1.5-pro
    //   等老模型注入 thinking 参数被上游 400 拒绝。
    if let Some(effort) = effort_setting.to_genai() {
        if super::reasoning::model_supports_reasoning(api_type, model_id) {
            // DeepSeek 关闭思考必须走 `extra_body.thinking.type=disabled`,
            // 服务端不接受 `reasoning_effort: "none"`(400 unknown variant)。
            // 其他 provider 的 Off 档维持原 reasoning_effort 路径。
            let deepseek_off = matches!(api_type, AgentProviderApiType::DeepSeek)
                && matches!(effort_setting, crate::settings::ReasoningEffortSetting::Off);
            if deepseek_off {
                log::info!(
                    "[byop] DeepSeek Off → extra_body thinking.type=disabled (model={model_id})"
                );
                opts = opts.with_extra_body(json!({"thinking": {"type": "disabled"}}));
            } else {
                log::info!(
                    "[byop] reasoning_effort injected: model={model_id} setting={effort_setting:?}"
                );
                opts = opts.with_reasoning_effort(effort);
            }
        } else {
            log::info!(
                "[byop] reasoning_effort SKIPPED: model={model_id} not in capability list \
                 (api_type={api_type:?} setting={effort_setting:?}); request sent without thinking params"
            );
        }
    }

    // DashScope(阿里云百炼)OpenAI 兼容路径需显式 `enable_thinking: true` 才会
    // 输出 reasoning。详见 `dashscope_needs_enable_thinking` 注释。
    // 与上面 DeepSeek Off 的 extra_body 互斥(DeepSeek 走 DeepSeek api_type,
    // DashScope 走 OpenAI api_type),不会同时 fire。
    if dashscope_needs_enable_thinking(api_type, base_url, model_id, effort_setting) {
        log::info!(
            "[byop] DashScope reasoning model → extra_body enable_thinking=true \
             (model={model_id} setting={effort_setting:?})"
        );
        opts = opts.with_extra_body(json!({"enable_thinking": true}));
    }

    opts
}

fn map_genai_error(err: genai::Error) -> OpenAiCompatibleError {
    use genai::Error as G;
    match err {
        // 真·解析失败:JSON 反序列化阶段
        G::StreamParse { .. }
        | G::SerdeJson(_)
        | G::JsonValueExt(_)
        | G::InvalidJsonResponseElement { .. } => OpenAiCompatibleError::Decode(format!("{err}")),

        // 网络/流式发送阶段失败(reqwest 连接、TLS、DNS、超时、流中断等)
        G::WebStream { .. } | G::WebAdapterCall { .. } | G::WebModelCall { .. } => {
            OpenAiCompatibleError::Stream(format!("{err}"))
        }

        // 服务端返回的 HTTP 错误状态
        G::HttpError {
            status,
            body,
            canonical_reason,
        } => OpenAiCompatibleError::Status {
            status: status.as_u16(),
            body: if canonical_reason.is_empty() {
                body
            } else {
                format!("{canonical_reason}: {body}")
            },
        },

        // 其余(请求构造、鉴权、能力不支持等)归为通用错误,避免误导成"解析失败"
        other => OpenAiCompatibleError::Other(format!("{other}")),
    }
}

// ---------------------------------------------------------------------------
// 主流程
// ---------------------------------------------------------------------------

/// 标题生成所需的 BYOP 配置。可能与主请求同 provider 也可能不同(用户在 Profile
/// Editor 里独立选了 title_model)。
pub struct TitleGenInput {
    pub base_url: String,
    pub api_key: String,
    pub model_id: String,
    pub api_type: AgentProviderApiType,
    pub reasoning_effort: crate::settings::ReasoningEffortSetting,
}

/// `task_id`: conversation 的 root task id(controller 端从 history model 取)。
/// `target_task_id`: 本轮模型输出应该写入的 task id;普通对话等于 root,
/// CLI subagent 后续轮为已有 subtask。
/// `needs_create_task`: 仅首轮(root 还是 Optimistic)需要 emit `CreateTask`。
pub async fn generate_byop_output(
    params: RequestParams,
    base_url: String,
    api_key: String,
    model_id: String,
    api_type: AgentProviderApiType,
    reasoning_effort: crate::settings::ReasoningEffortSetting,
    task_id: String,
    target_task_id: String,
    needs_create_task: bool,
    // LRC 场景绑定的 CLI subagent `command_id`(= LRC block id 字符串)。
    lrc_command_id: Option<String>,
    // 仅 tag-in 首轮为 true:流头会合成虚拟 `tool_call::Subagent` + CreateTask,
    // 用 server subtask 升级 master 路径已经创建的 optimistic CLI subtask。
    lrc_should_spawn_subagent: bool,
    // 选中模型的 context window(tokens)。Some 时:流末用 genai captured_usage
    // 计算 (prompt_tokens + completion_tokens) / context_window 写回
    // ConversationUsageMetadata,驱动 footer 的 "X% context remaining" 实时更新。
    // None ⇒ 跳过(用户未填 + catalog 无),UI 维持 100% 占位。
    context_window: Option<u32>,
    _cancellation_rx: futures::channel::oneshot::Receiver<()>,
) -> Result<ResponseStream, ConvertToAPITypeError> {
    let force_echo_reasoning = super::reasoning::model_requires_reasoning_echo(api_type, &model_id);
    let chat_req = build_chat_request(&params, force_echo_reasoning, api_type, &model_id);
    let chat_opts = build_chat_options(api_type, &base_url, &model_id, reasoning_effort);
    let client = build_client(api_type, base_url, api_key);
    let conversation_id = params
        .conversation_token
        .as_ref()
        .map(|t| t.as_str().to_string())
        .unwrap_or_default();
    let request_id = Uuid::new_v4().to_string();
    let mcp_context = params.mcp_context.clone();

    // ⚠️ BYOP 持久化关键:warp 自家路径下,以下 ClientAction 都是 server 端 emit
    // 让 client 端把 UserQuery / ToolCallResult 等"非模型产出"的 message
    // 写回 task.messages,从而让下一轮请求的 `params.tasks` snapshot 完整。
    //
    // BYOP 去云化客户端自管,server 端不存在,必须我们自己 emit 这些写回事件,
    // 否则下一轮 `compute_active_tasks` 只看到模型产出(reasoning/output/tool_call),
    // 缺失对应的 user_query 和 tool_call_result,模型 context 严重断裂。
    //
    // 这里在流开始后 emit 两类事件:
    //   1. AddMessagesToTask{UserQuery}    ← 当前轮所有 UserQuery input
    //   2. AddMessagesToTask{ToolCallResult} ← 当前轮所有 ActionResult input
    //
    // emit 时机必须在 CreateTask 之后(任务已升级为 Server 状态),
    // 在模型响应开始之前(UI 顺序:user 显示 → thinking/answer)。
    // OpenWarp:历史轮多模态保活。除 query 文本外,把当前轮 UserQuery.context 里的所有
    // multimodal binary(image / pdf / audio / ...)一并打包进 `UserQuery.context.images`
    // 持久化(proto 字段叫 images,语义上是通用 BinaryFile —— `bytes data + mime_type`,
    // 跟 opencode FilePart 等价),使 build_chat_request 下一轮重建 messages 时能从历史
    // message 上恢复 binary,继续以 ContentPart::Binary 注入上游(模型不支持的 mime 由
    // build_user_message_with_binaries 替换为 ERROR 文本,与 opencode unsupportedParts 一致)。
    // 上游 warp 自家路径不需要这步因为云端 server 持有 InputContext;BYOP 直连必须客户端自管。
    let pending_user_queries: Vec<(String, Vec<user_context::UserBinary>)> = params
        .input
        .iter()
        .filter_map(|i| match i {
            AIAgentInput::UserQuery { query, context, .. } => {
                let attachments = user_context::collect_user_attachments(context);
                Some((query.clone(), attachments.binaries))
            }
            _ => None,
        })
        .collect();
    // ToolCallResult 持久化:用 `tools::serialize_action_result` 把 ActionResult
    // 序列化为 JSON 字符串,装进 Message 的 server_message_data 字段
    // (warp protobuf 的 `tool_call_result.result` oneof 都是结构化 variant,
    // 没有通用 string 兜底;但 server_message_data 是自由字符串字段,刚好够用)。
    // 下一轮 build_chat_request 在 ToolCallResult 分支:result=None 时从
    // server_message_data 读 content,result=Some 时走 tools::serialize_result。
    let pending_tool_results: Vec<(String, String)> = params
        .input
        .iter()
        .filter_map(|i| match i {
            AIAgentInput::ActionResult { result, .. } => {
                let id = result.id.to_string();
                let content = tools::serialize_action_result(result).unwrap_or_else(|| {
                    serde_json::json!({ "result": result.result.to_string() }).to_string()
                });
                Some((id, content))
            }
            _ => None,
        })
        .collect();

    // INFO 级别一行总览 + 每条 message 一行简报(role + 文本长度 + tool 计数 + reasoning 标记),
    // 默认日志配置即可看到,便于诊断"历史是否完整传上去"等问题。
    log::info!(
        "[byop] adapter={:?} model={} system_len={} messages={} tools={}",
        adapter_kind_for(api_type),
        model_id,
        chat_req.system.as_deref().map(str::len).unwrap_or(0),
        chat_req.messages.len(),
        chat_req.tools.as_ref().map(|t| t.len()).unwrap_or(0),
    );
    for (idx, m) in chat_req.messages.iter().enumerate() {
        let role = format!("{:?}", m.role);
        let text_len = m.content.first_text().map(str::len).unwrap_or(0);
        let tc_count = m.content.tool_calls().len();
        let tr_count = m.content.tool_responses().len();
        // reasoning_content 检测 — genai 把它存为 ContentPart::ReasoningContent,
        // 没有公开 getter,这里通过 size() 与 first_text+tool_count 的和差异粗判。
        log::info!(
            "[byop]  [{idx}] role={role} text_len={text_len} tool_calls={tc_count} tool_responses={tr_count}"
        );
    }

    // 诊断:构造包含 system / messages / tools 的完整 ChatRequest JSON dump,保存到
    // stream 闭包。真实 Anthropic wire body 会由 genai adapter 再转换一层,但这里已经
    // 覆盖所有传入 BYOP 的原始字符串,足够定位非法 escape 来自 prompt、工具描述、
    // schema 还是 tool result。
    let diag_body_json = serde_json::to_string(&json!({
        "model": &model_id,
        "chat_request": &chat_req,
    }))
    .unwrap_or_default();
    log::info!("[byop] diag_body_approx_len={}", diag_body_json.len());
    log::info!("[byop-diag] full_request_json={diag_body_json}");

    // 主动扫描原始文本里的"可疑反斜杠序列":serde_json 把源字符串里的字面
    // `\` 序列化为 `\\`,所以 wire body 里出现"两个连续反斜杠 + u/x" 才意味着
    // 原文有字面 `\u` / `\x`,这是 proxy 误"还原 `\\u` → `\u`"触发 invalid escape
    // 的真实风险点。源字符串里的 `\n` / `\r` / `\t` 经 serde_json 输出为单个反斜杠 +
    // 字母,本身就是合法 JSON escape,proxy 不会再二次还原,不算可疑。
    fn scan_suspicious_backslash(label: &str, s: &str) {
        let bytes = s.as_bytes();
        let mut bs_hits: Vec<(usize, String)> = Vec::new();
        let mut ctrl_hits: Vec<(usize, u8)> = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            // 字面 `\\u` / `\\x` 序列(源字符串中含 `\u` / `\x`)。
            if b == b'\\'
                && i + 2 < bytes.len()
                && bytes[i + 1] == b'\\'
                && matches!(bytes[i + 2], b'u' | b'x')
            {
                let end = (i + 10).min(bytes.len());
                let snippet = String::from_utf8_lossy(&bytes[i..end]).to_string();
                if bs_hits.len() < 5 {
                    bs_hits.push((i, snippet));
                }
                // 跳过这一对,避免对同一位置触发多次。
                i += 3;
                continue;
            }
            // raw 控制字符(byte 0x00-0x08, 0x0B-0x0C, 0x0E-0x1F)。
            // serde_json 会 escape 为 `\u00XX`,合法 JSON;但部分 strict proxy
            // 或经过 base64 / 中间编码层时这些字节最容易出错。
            if (b < 0x20 && !matches!(b, b'\t' | b'\n' | b'\r')) && ctrl_hits.len() < 10 {
                ctrl_hits.push((i, b));
            }
            i += 1;
        }
        if !bs_hits.is_empty() {
            log::warn!("[byop] {label} suspicious literal '\\\\u'/'\\\\x' patterns: {bs_hits:?}");
        }
        if !ctrl_hits.is_empty() {
            log::warn!("[byop] {label} contains raw control chars (offset, byte): {ctrl_hits:?}");
        }
    }
    scan_suspicious_backslash("full_request_json", &diag_body_json);
    if let Some(sys) = chat_req.system.as_deref() {
        scan_suspicious_backslash("system", sys);
    }
    for (idx, m) in chat_req.messages.iter().enumerate() {
        if let Some(t) = m.content.first_text() {
            scan_suspicious_backslash(&format!("msg[{idx}]"), t);
        }
    }

    let stream = async_stream::stream! {
        // 1) StreamInit — 始终先发,UI 能立刻显示 "thinking..."
        yield Ok(api::ResponseEvent {
            r#type: Some(api::response_event::Type::Init(
                api::response_event::StreamInit {
                    request_id: request_id.clone(),
                    conversation_id,
                    run_id: String::new(),
                },
            )),
        });

        // 2) 首轮:CreateTask 升级 Optimistic root → Server。
        if needs_create_task {
            yield Ok(create_task_event(&task_id));
        }

        // 3) 持久化 input 里的 UserQuery / ToolCallResult 到 task.messages。
        //    (warp server 路径由后端 emit;BYOP 客户端必须自己 emit,见上方注释。)
        //    tag-in 首轮先写 root,再由下面的 spawn 分支复制到新 subtask;已有 CLI
        //    subagent 的后续轮直接写 target_task_id。
        let persistence_task_id = if lrc_should_spawn_subagent {
            task_id.as_str()
        } else {
            target_task_id.as_str()
        };
        let mut persistence_messages: Vec<api::Message> = Vec::new();
        for (q, imgs) in &pending_user_queries {
            persistence_messages.push(make_user_query_message(
                persistence_task_id,
                &request_id,
                q.clone(),
                imgs,
            ));
        }
        for (call_id, content) in &pending_tool_results {
            persistence_messages.push(make_tool_call_result_message(
                persistence_task_id,
                &request_id,
                call_id.clone(),
                content.clone(),
            ));
        }
        if !persistence_messages.is_empty() {
            yield Ok(make_add_messages_event(persistence_task_id, persistence_messages));
        }

        // 3.5) LRC subagent spawn(对齐上游云端的 cli subagent 注入路径)。
        //
        // 当请求来自 alt-screen + agent tagged-in 状态时,`lrc_command_id` 携带当前 LRC
        // block 的 id 字符串。此处客户端合成两条事件:
        //   a) AddMessagesToTask(root, [<虚拟 subagent tool_call>])
        //      在 root.messages 里挂一条 ToolCall::Subagent { task_id=<新 subtask>,
        //      metadata: Cli { command_id }, payload: "" }。
        //      conversation `Task::new_subtask` 会从 parent.messages 里按 task_id 匹配
        //      这条 subagent_call,提取出 SubagentParams 挂到 subtask。
        //   b) CreateTask(api::Task { id=<新 subtask>, dependencies.parent_task_id=root })
        //      触发 `apply_client_action::CreateTask`,因 parent_id 非空走 `new_subtask`,
        //      接着 emit `BlocklistAIHistoryEvent::CreatedSubtask` →
        //      `cli_controller::handle_history_model_event` 看到 cli_subagent_block_id
        //      非空,emit `CLISubagentEvent::SpawnedSubagent` → terminal_view 创建
        //      `CLISubagentView` 浮窗,挂进 `cli_subagent_views` map。
        //
        // 切换后续 chunk emit 的 task_id 到 subtask_id,让模型 reasoning/output/tool_call
        // 全部进 subtask,subagent_view 据此渲染浮窗内容。
        //
        // 时序约束:必须在 root CreateTask + UserQuery 持久化之后,模型流之前。
        // 否则 conversation 找不到 root task / 找不到 user query 引用对。
        let mut current_task_id = if lrc_should_spawn_subagent {
            task_id.clone()
        } else {
            target_task_id.clone()
        };
        if lrc_should_spawn_subagent {
            let Some(command_id) = lrc_command_id.clone() else {
                log::warn!("[byop] LRC spawn requested without command_id");
                yield Err(Arc::new(AIApiError::Other(anyhow::anyhow!(
                    "BYOP LRC spawn requested without command_id"
                ))));
                return;
            };
            let subtask_id = Uuid::new_v4().to_string();
            let tool_call_id = Uuid::new_v4().to_string();
            log::info!(
                "[byop] LRC tag-in: spawning cli subagent subtask={subtask_id} \
                 command_id={command_id} parent={task_id}"
            );

            let subagent_tool = api::message::tool_call::Tool::Subagent(
                api::message::tool_call::Subagent {
                    task_id: subtask_id.clone(),
                    payload: String::new(),
                    metadata: Some(
                        api::message::tool_call::subagent::Metadata::Cli(
                            api::message::tool_call::subagent::CliSubagent {
                                command_id,
                            },
                        ),
                    ),
                },
            );
            let subagent_msg = make_tool_call_message(
                &task_id,
                &request_id,
                &tool_call_id,
                subagent_tool,
            );
            // a) 把 subagent tool_call 挂到 root.messages,供 new_subtask 反查 SubagentParams。
            yield Ok(make_add_messages_event(&task_id, vec![subagent_msg]));
            // b) 创建带 parent_task_id 的 subtask;conversation 检测 parent_id 非空 →
            //    走 `Task::new_subtask` 路径,自动绑定 SubagentParams。
            yield Ok(create_subtask_event(&subtask_id, &task_id));

            // c) OpenWarp A1:把当前轮的 UserQuery 也复制一份到 subtask,初始化 subtask 的
            //    exchange.output.messages。否则 CLISubagentView 渲染时 subtask 的 exchanges
            //    output 为空,浮窗永远只显示 49.6 高度的空对话框,看不到任何内容。
            //    上游云端在 cli subagent 任务上有完整 ClientAction 序列填 exchange.output,
            //    BYOP 客户端自管必须显式注入。
            //
            //    只复制本轮 UserQuery(`pending_user_queries`),不动 root 的副本(root
            //    保留 user query 引用以避免 exchange.input 为空导致状态机错乱)。
            //    后续模型 chunks 走 `current_task_id = subtask_id`,append 到这个起点之后。
            if !pending_user_queries.is_empty() {
                let mut subtask_messages: Vec<api::Message> = Vec::new();
                for (q, imgs) in &pending_user_queries {
                    subtask_messages.push(make_user_query_message(
                        &subtask_id,
                        &request_id,
                        q.clone(),
                        imgs,
                    ));
                }
                yield Ok(make_add_messages_event(&subtask_id, subtask_messages));
            }

            // 后续 chunk emit 切到 subtask。
            current_task_id = subtask_id;
        }

        log::info!("[byop] opening stream: model={model_id}");
        let mut sdk_stream = match client
            .exec_chat_stream(&model_id, chat_req, Some(&chat_opts))
            .await
        {
            Ok(resp) => {
                log::info!("[byop] stream opened OK (HTTP request accepted)");
                resp.stream
            }
            Err(e) => {
                let mapped = map_genai_error(e);
                log::error!("[byop] open stream failed: {mapped:#}");
                yield Err(Arc::new(AIApiError::Other(anyhow::anyhow!(
                    "BYOP open stream failed: {mapped}"
                ))));
                return;
            }
        };

        // 流式状态:文本 / 推理各自的 message id 在第一次 chunk 到达时生成,
        // 之后的 chunk 走 AppendToMessageContent 增量追加。
        let mut text_msg_id: Option<String> = None;
        let mut reasoning_msg_id: Option<String> = None;
        // tool_call 按 call_id 累积 — genai 流式发的 ToolCallChunk 已带完整 ToolCall
        // (since 0.4.0 行为),但跨 chunk 同一 call_id 可能多次出现 args 增量,
        // 用 HashMap 按 id 累积后在流末统一 emit。
        let mut tool_bufs: HashMap<String, ToolCall> = HashMap::new();
        // call_id → 首帧占位 ToolCall message 的 id。
        // 首次 ToolCallChunk 到达且可解析时立即 emit 一条占位卡(让 UI 在 stream End
        // 之前就能看到"调用 X 工具"反馈),流末用 update_message 原地刷新为最终 args。
        // 不在表里的 call_id(首帧 parse 失败 / web 工具)走老路径在 End 后一次性 emit。
        let mut tool_msg_ids: HashMap<String, String> = HashMap::new();
        // call_id → 上次 update_message 增量刷新的时刻。
        // 长 args 工具(create_or_edit_document、长 grep query)args 跨多 chunk 累积时,
        // 节流 ≥ 200ms reparse + update,体感跟文本流一样连续而不是首帧定格到 End。
        let mut tool_last_update: HashMap<String, std::time::Instant> = HashMap::new();
        // 增量刷新节流阈值:小于此值的连续 chunk 不再 update_message,避免频繁 UI 重排。
        // 注:SDK stream 每个 ChatStreamEvent 独立 await,多 tool 并发时本就是顺序到达,
        // 同 tick batch emit 在此层意义不大;真正降抖在节流上,这条注释提醒后续不要瞎引入 batch。
        const TOOL_ARGS_UPDATE_THROTTLE_MS: u64 = 200;
        // 诊断:统计 stream 各类事件计数,流末打 INFO log。
        // 用于排查"消息静默消失"——如果 chunk_count=0 且 tool_count=0,说明上游返回空内容。
        let mut start_count: u32 = 0;
        let mut chunk_count: u32 = 0;
        let mut chunk_bytes: usize = 0;
        let mut reasoning_count: u32 = 0;
        let mut reasoning_bytes: usize = 0;
        let mut tool_chunk_count: u32 = 0;
        let mut end_count: u32 = 0;
        let mut other_count: u32 = 0;
        // 累积本轮 token 使用量。genai 在 ChatStreamEvent::End 事件里携带
        // captured_usage(Option<Usage>),其 prompt_tokens 是本轮整段 history
        // (Anthropic / OpenAI 都按"完整请求 prompt"计),completion_tokens 是模型输出。
        // 二者相加除以 context_window 即为"context 占用率",和 warp 自家 server 路径语义一致。
        let mut captured_prompt_tokens: i32 = 0;
        let mut captured_completion_tokens: i32 = 0;

        while let Some(item) = sdk_stream.next().await {
            let event = match item {
                Ok(ev) => ev,
                Err(e) => {
                    let mapped = map_genai_error(e);
                    let err_text = format!("{mapped:#}");
                    log::error!("[byop] stream chunk error: {err_text}");
                    log::error!("[byop-diag] full_request_json_on_error={diag_body_json}");
                    // 从错误消息里 parse "column N",dump diag_body_json 该位置 ±200 char 上下文 + 字节 hex。
                    if let Some(col) = err_text
                        .split("column ")
                        .nth(1)
                        .and_then(|s| s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse::<usize>().ok())
                    {
                        let body = &diag_body_json;
                        let byte_len = body.len();
                        let start = col.saturating_sub(200).min(byte_len);
                        let end = (col + 200).min(byte_len);
                        let context = body.get(start..end).unwrap_or("(slice failed: 非 char 边界)");
                        log::error!(
                            "[byop] error column={col} diag_body_len={byte_len} context[{start}..{end}]={context:?}"
                        );
                        let hex_start = col.saturating_sub(20).min(byte_len);
                        let hex_end = (col + 20).min(byte_len);
                        if let Some(slice) = body.as_bytes().get(hex_start..hex_end) {
                            log::error!("[byop] error bytes[{hex_start}..{hex_end}] hex={slice:02x?}");
                        }
                    }
                    yield Err(Arc::new(AIApiError::Other(anyhow::anyhow!(
                        "BYOP stream error: {mapped}"
                    ))));
                    return;
                }
            };

            match event {
                ChatStreamEvent::Start => {
                    // unit event;UI 已通过 StreamInit 显示 thinking,这里 no-op
                    start_count += 1;
                }
                ChatStreamEvent::Chunk(c) if !c.content.is_empty() => {
                    chunk_count += 1;
                    chunk_bytes += c.content.len();
                    if let Some(id) = text_msg_id.clone() {
                        yield Ok(make_append_event(&current_task_id, &id, AppendKind::Text(c.content)));
                    } else {
                        let new_id = Uuid::new_v4().to_string();
                        let mut msg = make_agent_output_message(&current_task_id, &request_id, c.content);
                        msg.id = new_id.clone();
                        text_msg_id = Some(new_id);
                        yield Ok(make_add_messages_event(&current_task_id, vec![msg]));
                    }
                }
                ChatStreamEvent::Chunk(_) => {}
                ChatStreamEvent::ReasoningChunk(c) if !c.content.is_empty() => {
                    reasoning_count += 1;
                    reasoning_bytes += c.content.len();
                    if let Some(id) = reasoning_msg_id.clone() {
                        yield Ok(make_append_event(&current_task_id, &id, AppendKind::Reasoning(c.content)));
                    } else {
                        let new_id = Uuid::new_v4().to_string();
                        let mut msg = make_reasoning_message(&current_task_id, &request_id, c.content);
                        msg.id = new_id.clone();
                        reasoning_msg_id = Some(new_id);
                        yield Ok(make_add_messages_event(&current_task_id, vec![msg]));
                    }
                }
                ChatStreamEvent::ReasoningChunk(_) => {}
                ChatStreamEvent::ToolCallChunk(tc) => {
                    tool_chunk_count += 1;
                    let mut call = tc.tool_call;
                    // 极个别 provider(自建 ollama 代理等)不发 call_id,本地 uuid 兜底。
                    if call.call_id.is_empty() {
                        call.call_id = Uuid::new_v4().to_string();
                    }
                    // 首次见到该 call_id → 立即 push 占位 ToolCall 消息到 pending_placeholders,
                    // 让 UI 在 stream End 之前就出现"调用 X 工具"卡片。
                    // 多 tool 同 tick 内来时:本循环结束前统一 batch emit 一次 add_messages,
                    // 减少 view tree 重排次数。
                    // 已在表里(占位已发)且 args 又来新 chunk → 节流 ≥ 200ms reparse + update_message
                    // 增量刷新 args,长 args 工具(create_or_edit_document、长 grep 等)体感连续。
                    // web 工具(webfetch/websearch)走自己的 loading 帧链路(L2102 区域),
                    // 这里跳过避免双卡。
                    if call.fn_name != tools::webfetch::TOOL_NAME
                        && call.fn_name != tools::websearch::TOOL_NAME
                    {
                        if let Some(msg_id) = tool_msg_ids.get(&call.call_id).cloned() {
                            // 已 emit 占位 → 节流增量刷新。
                            let now = std::time::Instant::now();
                            let last = tool_last_update.get(&call.call_id).copied();
                            let elapsed_ok = last
                                .map(|t| now.duration_since(t).as_millis() as u64 >= TOOL_ARGS_UPDATE_THROTTLE_MS)
                                .unwrap_or(true);
                            if elapsed_ok {
                                if let Ok(parsed) =
                                    parse_incoming_tool_call(&call, mcp_context.as_ref())
                                {
                                    let mut updated = make_tool_call_message(
                                        &current_task_id,
                                        &request_id,
                                        &call.call_id,
                                        parsed,
                                    );
                                    updated.id = msg_id;
                                    tool_last_update.insert(call.call_id.clone(), now);
                                    yield Ok(make_update_message_event(
                                        &current_task_id,
                                        updated,
                                        vec!["tool_call".to_owned()],
                                    ));
                                }
                                // reparse 失败(intermediate 状态):静默,等下次 chunk。
                            }
                        } else if let Ok(parsed) =
                            parse_incoming_tool_call(&call, mcp_context.as_ref())
                        {
                            // 首次 parse 成功 → 立即 emit 占位卡。
                            // 每个 chunk 在未 emit 占位前都会重 parse(即"retry on every
                            // chunk"),所以即便首帧 args 不全,后续任意 chunk 完整时
                            // 都会立刻触发占位 emit—— 这就是 P1-4 的覆盖路径,
                            // 不再需要 generic placeholder variant。
                            let msg_id = Uuid::new_v4().to_string();
                            let mut placeholder = make_tool_call_message(
                                &current_task_id,
                                &request_id,
                                &call.call_id,
                                parsed,
                            );
                            placeholder.id = msg_id.clone();
                            tool_msg_ids.insert(call.call_id.clone(), msg_id);
                            tool_last_update.insert(
                                call.call_id.clone(),
                                std::time::Instant::now(),
                            );
                            yield Ok(make_add_messages_event(
                                &current_task_id,
                                vec![placeholder],
                            ));
                        }
                        // 首帧 parse 失败(args 还不完整 / 未知工具):暂不 emit,
                        // 等下次 chunk 再尝试或 End 时走老路径,避免视觉抖动。
                    }
                    // 同一 call_id 多次 chunk:后到的覆盖(genai 已合并 args)。
                    tool_bufs.insert(call.call_id.clone(), call);
                }
                ChatStreamEvent::End(end) => {
                    end_count += 1;
                    // genai >= 0.4.0 的 captured_content 含 tool_calls。
                    // 优先用 captured_content 里的 tool_calls(更完整),
                    // 否则用 streaming 中累积的 tool_bufs。
                    if let Some(content) = end.captured_content.as_ref() {
                        for call in content.tool_calls() {
                            tool_bufs.entry(call.call_id.clone()).or_insert_with(|| call.clone());
                        }
                    }
                    if let Some(usage) = end.captured_usage.as_ref() {
                        // 多次 End 取最大值兜底(理论上单次 stream 只有一次 End)。
                        if let Some(p) = usage.prompt_tokens {
                            captured_prompt_tokens = captured_prompt_tokens.max(p);
                        }
                        if let Some(c) = usage.completion_tokens {
                            captured_completion_tokens = captured_completion_tokens.max(c);
                        }
                    }
                }
                _ => {
                    other_count += 1;
                    // ThoughtSignatureChunk 等暂不处理(Gemini 3 thoughts 需要回传给后续轮次,
                    // 当前 BYOP 不持久化 thought_signatures,接受降级)
                }
            }
        }

        // 流统计 INFO log。chunk_count=0 && tool_count=0 时上游返回为空,
        // 大概率是 model_id 不被识别 / max_tokens 缺失 / Anthropic API 兼容代理返回 200 但 body 空。
        let total_tools = tool_bufs.len();
        log::info!(
            "[byop] stream stats: start={start_count} chunks={chunk_count} ({chunk_bytes}B) \
             reasoning={reasoning_count} ({reasoning_bytes}B) tool_chunks={tool_chunk_count} \
             ends={end_count} other={other_count} captured_tools={total_tools}"
        );
        if chunk_count == 0 && reasoning_count == 0 && total_tools == 0 {
            log::warn!(
                "[byop] stream returned 0 content / 0 reasoning / 0 tool_calls — \
                 上游可能返回空响应(model_id 错? max_tokens 缺? proxy 异常?)"
            );
        }

        // 流结束:把累积的 tool_calls 一次性发出。
        let mut final_messages: Vec<api::Message> = Vec::new();
        for call in tool_bufs.into_values() {
            // 诊断:dump 模型实际发的 tool_call raw payload
            // (call_id / fn_name / fn_arguments JSON 原文 + 类型标注),
            // 便于核对模型是否按 schema 出入参(常见问题:bool 字段被字符串化、
            // 数字被加引号、嵌套对象塌成字符串等)。
            // debug 级:只在排查 schema 问题时开 RUST_LOG=debug,平时不污染 INFO。
            // info 级保留一行不带 args 的简短摘要,便于看流式时序。
            log::info!(
                "[byop] tool_call_in: name={} call_id={}",
                call.fn_name,
                call.call_id,
            );
            if log::log_enabled!(log::Level::Debug) {
                let args_repr = if call.fn_arguments.is_string() {
                    format!("string({:?})", call.fn_arguments.as_str().unwrap_or(""))
                } else {
                    format!(
                        "{}({})",
                        match &call.fn_arguments {
                            Value::Object(_) => "object",
                            Value::Array(_) => "array",
                            Value::Bool(_) => "bool",
                            Value::Number(_) => "number",
                            Value::Null => "null",
                            Value::String(_) => "string",
                        },
                        call.fn_arguments
                    )
                };
                log::debug!(
                    "[byop] tool_call_in_args: name={} call_id={} args={}",
                    call.fn_name,
                    call.call_id,
                    args_repr,
                );
            }

            // OpenWarp BYOP web 工具拦截:webfetch / websearch 不映射到 protobuf
            // executor variant,在这里直接跑本地 HTTP,合成 (carrier ToolCall,
            // ToolCallResult) 一对消息,绕开 parse_incoming_tool_call。
            //
            // UI:对齐 cloud 模式,前后各 emit 一条 `Message::WebSearch` /
            // `Message::WebFetch` 状态消息,触发 inline_action `WebSearchView` /
            // `WebFetchView` 渲染:Searching/Fetching loading 卡片 → Success(URL 列表)
            // / Error 折叠卡。这两条不进 final_messages,直接 yield 让 UI 实时更新;
            // carrier + result 仍走 final_messages 给下一轮模型推理用。
            if call.fn_name == tools::webfetch::TOOL_NAME
                || call.fn_name == tools::websearch::TOOL_NAME
            {
                let args_str = if call.fn_arguments.is_string() {
                    call.fn_arguments.as_str().unwrap_or("").to_owned()
                } else {
                    call.fn_arguments.to_string()
                };
                let is_search = call.fn_name == tools::websearch::TOOL_NAME;

                // 预解析 args 抽 query / url 给 UI loading 卡。args 解析失败也要 emit
                // (用空字段兜底),保证 UI 至少看到一帧 loading,后续 dispatch
                // 仍会返回 invalid_arguments → 切到 Error 卡。
                let preview_query = if is_search {
                    serde_json::from_str::<tools::web_runtime::SearchToolArgs>(&args_str)
                        .map(|a| a.query)
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                let preview_urls: Vec<String> = if !is_search {
                    serde_json::from_str::<tools::web_runtime::FetchArgs>(&args_str)
                        .map(|a| vec![a.url])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                // Searching/Fetching loading 帧与最终 Success/Error 帧必须共用同一个
                // message.id —— `block.rs::handle_web_search_messages` 按 id 复用
                // WebSearchView,id 不同会创建两张独立卡。
                let web_msg_id = Uuid::new_v4().to_string();
                let mut loading_msg = if is_search {
                    make_web_search_searching_message(
                        &current_task_id,
                        &request_id,
                        preview_query.clone(),
                    )
                } else {
                    make_web_fetch_fetching_message(
                        &current_task_id,
                        &request_id,
                        preview_urls.clone(),
                    )
                };
                loading_msg.id = web_msg_id.clone();
                yield Ok(make_add_messages_event(&current_task_id, vec![loading_msg]));

                let result_json = dispatch_byop_web_tool(&call.fn_name, &args_str).await;

                let mut done_msg = if is_search {
                    make_web_search_status_from_result(
                        &current_task_id,
                        &request_id,
                        &preview_query,
                        &result_json,
                    )
                } else {
                    make_web_fetch_status_from_result(
                        &current_task_id,
                        &request_id,
                        &preview_urls,
                        &result_json,
                    )
                };
                done_msg.id = web_msg_id;
                // 第二帧不能再用 AddMessagesToTask —— 那会往 task.messages 追加第二条
                // 同 id 记录,`output.rs::WebSearch` 渲染分支按 message 数量 add_child,
                // 显示成两张并排卡。改用 UpdateTaskMessage + FieldMask:`task::upsert_message`
                // 找到同 id 现有 message 后走 FieldMaskOperation::update 原地合并,
                // task.messages 仍只有一条 → UI 一张卡 set_status 切换。
                let mask_path = if is_search { "web_search" } else { "web_fetch" };
                yield Ok(make_update_message_event(
                    &current_task_id,
                    done_msg,
                    vec![mask_path.to_owned()],
                ));

                let result_content = serde_json::to_string(&result_json)
                    .unwrap_or_else(|_| r#"{"status":"serialize_error"}"#.to_owned());
                final_messages.push(make_tool_call_carrier_message(
                    &current_task_id,
                    &request_id,
                    &call.call_id,
                    &call.fn_name,
                    &args_str,
                ));
                final_messages.push(make_tool_call_result_message(
                    &current_task_id,
                    &request_id,
                    call.call_id.clone(),
                    result_content,
                ));
                continue;
            }

            match parse_incoming_tool_call(&call, mcp_context.as_ref()) {
                Ok(warp_tool) => {
                    // 如果 ToolCallChunk 阶段已经 emit 过占位卡(同 call_id),
                    // 改用 update_message 原地刷新为最终 args(覆盖 chunk 中可能后到
                    // 的 args delta)。占位与终帧共用同一 message.id,
                    // task::upsert_message 走 FieldMaskOperation::update,
                    // task.messages 仍只有一条 → UI 一张卡 in-place 刷新,不双卡。
                    if let Some(msg_id) = tool_msg_ids.get(&call.call_id).cloned() {
                        let mut updated = make_tool_call_message(
                            &current_task_id,
                            &request_id,
                            &call.call_id,
                            warp_tool,
                        );
                        updated.id = msg_id;
                        yield Ok(make_update_message_event(
                            &current_task_id,
                            updated,
                            vec!["tool_call".to_owned()],
                        ));
                    } else {
                        final_messages.push(make_tool_call_message(
                            &current_task_id,
                            &request_id,
                            &call.call_id,
                            warp_tool,
                        ));
                    }
                }
                Err(e) => {
                    // 关键:不再把 from_args 失败吞成纯文本(原实现:emit AgentOutput),
                    // 因为模型那一轮以为自己调了 tool 在等 result,看到一段中文 assistant 文字
                    // 完全不知道是参数类型错,无法定向修正重试。
                    // 改成 emit 一对 ToolCall(carrier) + ToolCallResult(error JSON),
                    // 让模型在下一轮看到标准 tool_result error,可以按惯例改 args 重试或换工具。
                    //
                    // ToolCall 的 `tool` oneof 留 None(没有合适的结构化 variant),原始
                    // fn_name + args_str 通过 server_message_data 携带,
                    // serialize_outgoing_tool_call 的 carrier 分支会优先还原。
                    let args_str = if call.fn_arguments.is_string() {
                        call.fn_arguments.as_str().unwrap_or("").to_owned()
                    } else {
                        call.fn_arguments.to_string()
                    };
                    log::warn!(
                        "[byop] tool_call parse failed → emit synthetic error tool_result: \
                         tool={} call_id={} err={e:#}",
                        call.fn_name,
                        call.call_id
                    );
                    let error_payload = serde_json::json!({
                        "error": "invalid_arguments",
                        "detail": e.to_string(),
                        "tool": call.fn_name,
                        "received_args": &args_str,
                        "hint": "Arguments did not match the tool's JSON Schema. \
                                 Re-emit the tool call with corrected types / required fields, \
                                 or pick a different tool.",
                    });
                    let error_content = serde_json::to_string(&error_payload)
                        .unwrap_or_else(|_| r#"{"error":"invalid_arguments"}"#.to_owned());
                    final_messages.push(make_tool_call_carrier_message(
                        &current_task_id,
                        &request_id,
                        &call.call_id,
                        &call.fn_name,
                        &args_str,
                    ));
                    final_messages.push(make_tool_call_result_message(
                        &current_task_id,
                        &request_id,
                        call.call_id.clone(),
                        error_content,
                    ));
                }
            }
        }
        if !final_messages.is_empty() {
            yield Ok(make_add_messages_event(&current_task_id, final_messages));
        }

        // 把 captured token usage 折算成 ConversationUsageMetadata.context_window_usage
        // 注入 StreamFinished — controller 的 handle_response_stream_finished 会把它写到
        // conversation.conversation_usage_metadata,footer 监听 UpdatedStreamingExchange/
        // AppendedExchange 事件即在每轮末实时刷新 "X% context remaining" 工具提示。
        let usage_metadata = context_window.and_then(|cw| {
            if cw == 0 || (captured_prompt_tokens == 0 && captured_completion_tokens == 0) {
                return None;
            }
            let used = (captured_prompt_tokens + captured_completion_tokens).max(0) as f32;
            let pct = (used / cw as f32).clamp(0.0, 1.0);
            log::info!(
                "[byop] context usage: prompt={} completion={} window={} → {:.1}%",
                captured_prompt_tokens,
                captured_completion_tokens,
                cw,
                pct * 100.0
            );
            Some(api::response_event::stream_finished::ConversationUsageMetadata {
                context_window_usage: pct,
                summarized: false,
                credits_spent: 0.0,
                #[allow(deprecated)]
                token_usage: Vec::new(),
                tool_usage_metadata: None,
                warp_token_usage: std::collections::HashMap::new(),
                byok_token_usage: std::collections::HashMap::new(),
            })
        });
        yield Ok(make_finished_done(usage_metadata));
    };

    Ok(Box::pin(stream))
}

/// 用独立 BYOP 配置发一个短的非工具请求,让模型对首条 user query 生成会话标题。
/// 所有错误吞掉(返回 Err 让上游打 warn log,不影响主流程)。
///
/// 实现委托给 `oneshot::byop_oneshot_completion`,这里只负责拼 prompt 和清洗输出。
///
/// ## prompt 设计
///
/// - **system**: 见 `prompts/tasks/title_system.md`,结构化 task/rules/examples,
///   覆盖中英双语示例,显式禁止 "回答用户问题 / 拒绝 / 加引号"。
/// - **user**: 把原始 `user_query` 包在 `<user>...</user>` 里,前置一句明确的
///   "Generate a title for this conversation:",避免弱模型把 user 当主指令直接答复
///   (典型坏 case:user="你是谁" → 模型答"我是 Claude"被当作标题)。
/// - **temperature**: 0.3 — opencode title agent 用 0.5,这里更保守,降低跑题。
pub(crate) async fn generate_title_via_byop(
    tg: &TitleGenInput,
    user_query: &str,
) -> Result<Option<String>, anyhow::Error> {
    let cfg = super::oneshot::OneshotConfig {
        base_url: tg.base_url.clone(),
        api_key: tg.api_key.clone(),
        model_id: tg.model_id.clone(),
        api_type: tg.api_type,
        reasoning_effort: tg.reasoning_effort,
    };
    let system = include_str!("prompts/tasks/title_system.md");
    let user_prompt = format!(
        "Generate a title for this conversation:\n<user>{}</user>",
        user_query
    );
    let opts = super::oneshot::OneshotOptions {
        max_chars: Some(1000),
        temperature: Some(0.3),
        ..Default::default()
    };
    let raw = super::oneshot::byop_oneshot_completion(&cfg, system, &user_prompt, &opts).await?;
    Ok(sanitize_title(&raw))
}

/// 清洗 title 文本。空字符串 → None(让上游跳过 emit)。
///
/// 处理顺序:
/// 1. 剥 `<think>...</think>` / `<reasoning>...</reasoning>` 思考块(reasoning 模型常见前缀)。
/// 2. 取首行非空内容(模型常前置"好的,标题是:"再换行给标题)。
/// 3. 剥 `Title:` / `标题:` / `Thread:` / `Subject:` 等前缀(大小写不敏感)。
/// 4. 剥首尾引号 / 反引号(中英文)。
/// 5. 去尾标点。
/// 6. 50 字符截断(按 char,保护 CJK),超过则尾部加 `…`。
fn sanitize_title(raw: &str) -> Option<String> {
    // 1. 剥 reasoning 标签(可能有多个,DOTALL 模式)。
    let mut s = raw.to_owned();
    for tag in &["think", "reasoning", "thought", "scratchpad"] {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        while let (Some(start), Some(end_rel)) =
            (s.find(&open), s.find(&close).map(|e| e + close.len()))
        {
            if end_rel <= start {
                break;
            }
            s.replace_range(start..end_rel, "");
        }
    }

    // 2. 取首行非空。
    let first_line = s
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_owned();
    let mut s = first_line;

    // 3. 剥前缀(循环剥,处理 "Title: 标题: foo" 这类双前缀)。
    let prefixes = [
        "title:",
        "subject:",
        "thread:",
        "标题:",
        "标题：",
        "主题:",
        "主题：",
    ];
    loop {
        let lower = s.to_lowercase();
        let mut stripped = false;
        for p in &prefixes {
            if lower.starts_with(p) {
                s = s[p.len()..].trim_start().to_owned();
                stripped = true;
                break;
            }
        }
        if !stripped {
            break;
        }
    }

    // 4. 剥首尾引号(中英文)。
    let quotes = ['"', '\'', '`', '“', '”', '‘', '’', '《', '》', '「', '」'];
    while let Some(c) = s.chars().next() {
        if quotes.contains(&c) {
            s.remove(0);
        } else {
            break;
        }
    }
    while let Some(c) = s.chars().last() {
        if quotes.contains(&c) {
            let new_len = s.len() - c.len_utf8();
            s.truncate(new_len);
        } else {
            break;
        }
    }

    // 5. 去尾标点。
    while let Some(c) = s.chars().last() {
        if matches!(
            c,
            '.' | '。' | '!' | '！' | '?' | '？' | ',' | '，' | ';' | '；' | ':' | '：'
        ) {
            let new_len = s.len() - c.len_utf8();
            s.truncate(new_len);
        } else {
            break;
        }
    }

    let s = s.trim().to_owned();
    if s.is_empty() {
        return None;
    }

    // 6. 50 字符截断(按 char,保护 CJK)。超长加省略号。
    const MAX_CHARS: usize = 50;
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > MAX_CHARS {
        let mut truncated: String = chars.iter().take(MAX_CHARS - 1).collect();
        truncated.push('…');
        Some(truncated)
    } else {
        Some(s)
    }
}

// ---------------------------------------------------------------------------
// Event 构造辅助
// ---------------------------------------------------------------------------

enum AppendKind {
    Reasoning(String),
    Text(String),
}

fn make_add_messages_event(task_id: &str, messages: Vec<api::Message>) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                actions: vec![api::ClientAction {
                    action: Some(api::client_action::Action::AddMessagesToTask(
                        api::client_action::AddMessagesToTask {
                            task_id: task_id.to_owned(),
                            messages,
                        },
                    )),
                }],
            },
        )),
    }
}

/// 用 `UpdateTaskMessage` + FieldMask 替换已有 message 的部分字段。controller
/// `conversation::Action::UpdateTaskMessage` → `task::upsert_message` →
/// `FieldMaskOperation::update` 原地合并,id 已存在则不会 push 重复记录。
/// 用于 BYOP web 工具 loading → success/error 状态切换(见拦截分支)。
fn make_update_message_event(
    task_id: &str,
    message: api::Message,
    mask_paths: Vec<String>,
) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                actions: vec![api::ClientAction {
                    action: Some(api::client_action::Action::UpdateTaskMessage(
                        api::client_action::UpdateTaskMessage {
                            task_id: task_id.to_owned(),
                            message: Some(message),
                            mask: Some(prost_types::FieldMask { paths: mask_paths }),
                        },
                    )),
                }],
            },
        )),
    }
}

fn make_append_event(task_id: &str, message_id: &str, kind: AppendKind) -> api::ResponseEvent {
    let (msg_inner, mask_path) = match kind {
        AppendKind::Reasoning(r) => (
            api::message::Message::AgentReasoning(api::message::AgentReasoning {
                reasoning: r,
                finished_duration: None,
            }),
            "agent_reasoning.reasoning",
        ),
        AppendKind::Text(t) => (
            api::message::Message::AgentOutput(api::message::AgentOutput { text: t }),
            "agent_output.text",
        ),
    };
    let message = api::Message {
        id: message_id.to_owned(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(msg_inner),
        request_id: String::new(),
        timestamp: None,
    };
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                actions: vec![api::ClientAction {
                    action: Some(api::client_action::Action::AppendToMessageContent(
                        api::client_action::AppendToMessageContent {
                            task_id: task_id.to_owned(),
                            message: Some(message),
                            mask: Some(prost_types::FieldMask {
                                paths: vec![mask_path.to_owned()],
                            }),
                        },
                    )),
                }],
            },
        )),
    }
}

/// BYOP web 工具(`webfetch` / `websearch`)的本地分发器。
///
/// 不通过 protobuf executor —— 直接在本地用 reqwest 跑 HTTP,把结构化结果
/// 序列化成 JSON Value 给上游 LLM。错误也序列化成 `{status:"error", ...}`,
/// 让模型看到标准 tool_result。
async fn dispatch_byop_web_tool(tool_name: &str, args_str: &str) -> Value {
    use tools::web_runtime;
    // 短超时 + 默认安全配置;系统全局共享一个 client 也可,这里每次新建以避免污染。
    let client = match reqwest::Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[byop] reqwest client build failed: {e:#}");
            return web_runtime::error_to_json(tool_name, &anyhow::anyhow!(e.to_string()));
        }
    };
    if tool_name == tools::webfetch::TOOL_NAME {
        match serde_json::from_str::<web_runtime::FetchArgs>(args_str) {
            Ok(args) => match web_runtime::run_webfetch(&client, args).await {
                Ok(out) => web_runtime::fetch_output_to_json(&out),
                Err(e) => {
                    log::warn!("[byop][webfetch] error: {e:#}");
                    web_runtime::error_to_json(tool_name, &e)
                }
            },
            Err(e) => web_runtime::error_to_json(
                tool_name,
                &anyhow::anyhow!(format!("invalid arguments: {e}")),
            ),
        }
    } else {
        // websearch
        match serde_json::from_str::<web_runtime::SearchToolArgs>(args_str) {
            Ok(args) => {
                let api_key = std::env::var("EXA_API_KEY").ok();
                match web_runtime::run_websearch(&client, args, api_key.as_deref(), None).await {
                    Ok(out) => web_runtime::search_output_to_json(&out),
                    Err(e) => {
                        log::warn!("[byop][websearch] error: {e:#}");
                        web_runtime::error_to_json(tool_name, &e)
                    }
                }
            }
            Err(e) => web_runtime::error_to_json(
                tool_name,
                &anyhow::anyhow!(format!("invalid arguments: {e}")),
            ),
        }
    }
}

fn parse_incoming_tool_call(
    call: &ToolCall,
    mcp_ctx: Option<&crate::ai::agent::MCPContext>,
) -> anyhow::Result<api::message::tool_call::Tool> {
    // genai ToolCall.fn_arguments 是 Value;tools::* 的 from_args 期望 &str,
    // 把 Value 序列化回字符串后传入(原协议就是字符串 JSON)。
    let args_str = if call.fn_arguments.is_string() {
        call.fn_arguments.as_str().unwrap_or("").to_owned()
    } else {
        call.fn_arguments.to_string()
    };
    if tools::mcp::is_mcp_function(&call.fn_name) {
        return tools::mcp::parse_mcp_tool_call(&call.fn_name, &args_str, mcp_ctx);
    }
    let Some(tool) = tools::lookup(&call.fn_name) else {
        anyhow::bail!("unknown tool name: {}", call.fn_name);
    };
    match (tool.from_args)(&args_str) {
        Ok(t) => Ok(t),
        Err(e) => {
            // 第一次失败:大概率是模型把 bool/数字/数组 序列化成了字符串。
            // 拿工具自身的 schema 跑一次类型 coerce,再 retry。
            let schema = (tool.parameters)();
            if let Some(coerced) = tools::coerce::coerce_args_against_schema(&args_str, &schema) {
                match (tool.from_args)(&coerced) {
                    Ok(t) => {
                        log::info!(
                            "[byop] from_args coerced ok: tool={} original_err={e:#}",
                            call.fn_name
                        );
                        return Ok(t);
                    }
                    Err(e2) => {
                        log::warn!(
                            "[byop] from_args failed (after coerce): tool={} err={e2:#} original_err={e:#} coerced_args={coerced} args_str={args_str}",
                            call.fn_name
                        );
                        return Err(e2);
                    }
                }
            }
            // 诊断:解析失败时把 from_args 实际拿到的字符串原样打出来,
            // 配合上层 [byop] tool_call_in 的 args= 行可以判断:
            //   1. 是否模型出参类型错(bool→"true" / 数字→"1" 等)
            //   2. 是否 genai Value→string 转换中 escape 出问题
            //   3. 是否 fn_arguments 整段被字符串化(应该 object 却是 string)
            log::warn!(
                "[byop] from_args failed: tool={} err={e:#} args_str={args_str}",
                call.fn_name
            );
            Err(e)
        }
    }
}

fn make_reasoning_message(task_id: &str, request_id: &str, reasoning: String) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::AgentReasoning(
            api::message::AgentReasoning {
                reasoning,
                finished_duration: None,
            },
        )),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_agent_output_message(task_id: &str, request_id: &str, text: String) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::AgentOutput(
            api::message::AgentOutput { text },
        )),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_user_query_message(
    task_id: &str,
    request_id: &str,
    query: String,
    binaries: &[user_context::UserBinary],
) -> api::Message {
    // OpenWarp:把 multimodal binary(image / pdf / audio 等)写进 `UserQuery.context.images`
    // (InputContext 字段,proto Image 实际是 `bytes data + string mime_type` 通用容器,
    // 字段名叫 images 历史原因)。UserBinary.data 是 base64 字符串,proto.data 是 raw bytes,
    // 这里 decode 一次;decode 失败的条目跳过,不阻塞模型流(decode 失败本来就意味着这条
    // 当轮也没真送上游,丢就丢了,不影响 history 一致性)。
    let proto_binaries: Vec<api::input_context::Image> = binaries
        .iter()
        .filter_map(|b| {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(&b.data)
                .ok()
                .map(|bytes| api::input_context::Image {
                    data: bytes,
                    mime_type: b.content_type.clone(),
                })
        })
        .collect();
    let context = if proto_binaries.is_empty() {
        None
    } else {
        Some(api::InputContext {
            images: proto_binaries,
            ..Default::default()
        })
    };
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::UserQuery(api::message::UserQuery {
            query,
            context,
            ..Default::default()
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

/// BYOP 拦截 websearch 时,emit `Message::WebSearch(Searching{query})`,UI 据此渲染
/// "Searching the web for \"query\"" loading 卡(`inline_action::web_search`)。
fn make_web_search_searching_message(
    task_id: &str,
    request_id: &str,
    query: String,
) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::WebSearch(api::message::WebSearch {
            status: Some(api::message::web_search::Status {
                r#type: Some(api::message::web_search::status::Type::Searching(
                    api::message::web_search::status::Searching { query },
                )),
            }),
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

/// 从 exa MCP 返回的 results 字符串里抽 (url, title)。
///
/// 实际格式是行式 metadata block,以 `---` 分隔多条结果:
/// ```
/// Title: Announcing Rust 1.95.0 | Rust Blog
/// URL: https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/
/// Published: 2026-04-16T00:00:00.000Z
/// Author: N/A
/// Highlights:
/// ...
/// ---
/// Title: ...
/// ```
/// 扫到 `Title: X` 缓存 candidate,紧随的第一条 `URL: Y` 配对成 (Y, X) 入列,去重。
/// 兼容兜底:也扫 `[title](url)` markdown link 形式(若 exa 模板未来切换)。
fn extract_search_pages_from_exa_results(s: &str) -> Vec<(String, String)> {
    let mut pages = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 路线 1:Title:/URL: 行式
    let mut current_title: Option<String> = None;
    for line in s.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("Title:") {
            current_title = Some(rest.trim().to_owned());
        } else if let Some(rest) = trimmed.strip_prefix("URL:") {
            let url = rest.trim().to_owned();
            let title = current_title.take().unwrap_or_default();
            if (url.starts_with("http://") || url.starts_with("https://"))
                && seen.insert(url.clone())
            {
                pages.push((url, title));
            }
        }
    }

    // 路线 2:markdown link `[title](url)` 兜底(去重已生效,不会重复)
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(rel_close_text) = s[i + 1..].find("](") {
                let text_end = i + 1 + rel_close_text;
                let url_start = text_end + 2;
                if let Some(rel_close_url) = s[url_start..].find(')') {
                    let url_end = url_start + rel_close_url;
                    let title = s[i + 1..text_end].trim().to_owned();
                    let url = s[url_start..url_end].trim().to_owned();
                    if (url.starts_with("http://") || url.starts_with("https://"))
                        && seen.insert(url.clone())
                    {
                        pages.push((url, title));
                    }
                    i = url_end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    pages
}

/// BYOP websearch 完成后,根据 `result_json` 决定 Success / Error 状态。
///
/// `pages` 从 `result_json["results"]` 这段 exa 拼好的 markdown 里扫 `[title](url)` 抽。
fn make_web_search_status_from_result(
    task_id: &str,
    request_id: &str,
    query: &str,
    result_json: &Value,
) -> api::Message {
    let is_error = result_json.get("status").and_then(|v| v.as_str()) == Some("error");
    let r#type = if is_error {
        api::message::web_search::status::Type::Error(())
    } else {
        let pages = result_json
            .get("results")
            .and_then(|v| v.as_str())
            .map(extract_search_pages_from_exa_results)
            .unwrap_or_default()
            .into_iter()
            .map(
                |(url, title)| api::message::web_search::status::success::SearchedPage {
                    url,
                    title,
                },
            )
            .collect();
        api::message::web_search::status::Type::Success(api::message::web_search::status::Success {
            query: query.to_owned(),
            pages,
        })
    };
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::WebSearch(api::message::WebSearch {
            status: Some(api::message::web_search::Status {
                r#type: Some(r#type),
            }),
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

/// BYOP 拦截 webfetch 时,emit `Message::WebFetch(Fetching{urls})`,UI 据此渲染
/// "Fetching N URLs" loading 卡(`inline_action::web_fetch`)。
fn make_web_fetch_fetching_message(
    task_id: &str,
    request_id: &str,
    urls: Vec<String>,
) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::WebFetch(api::message::WebFetch {
            status: Some(api::message::web_fetch::Status {
                r#type: Some(api::message::web_fetch::status::Type::Fetching(
                    api::message::web_fetch::status::Fetching { urls },
                )),
            }),
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

/// BYOP webfetch 完成后,从 `FetchOutput` JSON 抽 `url` + HTTP `status` 组装 Success
/// 卡;status="error" 走 Error 卡。
fn make_web_fetch_status_from_result(
    task_id: &str,
    request_id: &str,
    fallback_urls: &[String],
    result_json: &Value,
) -> api::Message {
    let is_error = result_json.get("status").and_then(|v| v.as_str()) == Some("error");
    let r#type = if is_error {
        api::message::web_fetch::status::Type::Error(())
    } else {
        let url = result_json
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| fallback_urls.first().cloned().unwrap_or_default());
        // FetchOutput.status 是 HTTP 状态码,2xx 算 success。
        let success = result_json
            .get("status")
            .and_then(|v| v.as_u64())
            .map(|c| (200..300).contains(&c))
            .unwrap_or(true);
        api::message::web_fetch::status::Type::Success(api::message::web_fetch::status::Success {
            pages: vec![api::message::web_fetch::status::success::FetchedPage {
                url,
                title: String::new(),
                success,
            }],
        })
    };
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::WebFetch(api::message::WebFetch {
            status: Some(api::message::web_fetch::Status {
                r#type: Some(r#type),
            }),
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_tool_call_result_message(
    task_id: &str,
    request_id: &str,
    tool_call_id: String,
    content: String,
) -> api::Message {
    // ToolCallResult 持久化:warp protobuf 的 `tool_call_result.result` oneof 都是
    // 结构化 variant(RunShellCommand / Grep / ReadFiles / ...),没有通用的字符串
    // 兜底 variant。BYOP 已经在 chat_stream 自己把 result 序列化为 JSON 字符串,
    // 不再需要按 warp 协议结构化 — 直接把字符串存到 `server_message_data` 这个
    // 自由字符串字段,并把 `result` oneof 留 None。下一轮 build_chat_request 在
    // `Message::ToolCallResult` 分支需要特判:result=None 时从 server_message_data
    // 读 content(否则走 tools::serialize_result 反序列化结构化 variant)。
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: content,
        citations: vec![],
        message: Some(api::message::Message::ToolCallResult(
            api::message::ToolCallResult {
                tool_call_id,
                context: None,
                result: None,
            },
        )),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

/// BYOP `from_args` 解析失败时,emit 占位 ToolCall 作 carrier:
/// `tool` oneof 留 None(没有合适的结构化 variant),原始 fn_name + args_str 编码到
/// `server_message_data` 为 `<fn_name>\n<args_str>`。下一轮 build_chat_request →
/// `serialize_outgoing_tool_call` 的 carrier 分支据此还原,保证上游模型看到的
/// tool_use name / args 与原 call 一致(否则用 "warp_internal_empty" 占位会让模型
/// 困惑,也对不上紧随的 ToolCallResult error 上下文)。
fn make_tool_call_carrier_message(
    task_id: &str,
    request_id: &str,
    tool_call_id: &str,
    fn_name: &str,
    args_str: &str,
) -> api::Message {
    let carrier = format!("{}\n{}", fn_name, args_str);
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: carrier,
        citations: vec![],
        message: Some(api::message::Message::ToolCall(api::message::ToolCall {
            tool_call_id: tool_call_id.to_owned(),
            tool: None,
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn make_tool_call_message(
    task_id: &str,
    request_id: &str,
    tool_call_id: &str,
    tool: api::message::tool_call::Tool,
) -> api::Message {
    api::Message {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_owned(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::ToolCall(api::message::ToolCall {
            tool_call_id: tool_call_id.to_owned(),
            tool: Some(tool),
        })),
        request_id: request_id.to_owned(),
        timestamp: None,
    }
}

fn create_task_event(task_id: &str) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                actions: vec![api::ClientAction {
                    action: Some(api::client_action::Action::CreateTask(
                        api::client_action::CreateTask {
                            task: Some(api::Task {
                                id: task_id.to_owned(),
                                description: String::new(),
                                dependencies: None,
                                messages: vec![],
                                summary: String::new(),
                                server_data: String::new(),
                            }),
                        },
                    )),
                }],
            },
        )),
    }
}

/// 构造一条 `Action::CreateTask` 表示新 subtask,带 `dependencies.parent_task_id`。
/// conversation 在 `apply_client_action::CreateTask` 看到 `task.parent_id()` 非空 →
/// 走 `Task::new_subtask` 路径,从 parent.messages 找匹配的 subagent tool_call、
/// 抽 `SubagentParams` 挂到 subtask、emit `BlocklistAIHistoryEvent::CreatedSubtask`。
/// LRC tag-in 浮窗 spawn 链路依赖此事件。
fn create_subtask_event(subtask_id: &str, parent_task_id: &str) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::ClientActions(
            api::response_event::ClientActions {
                actions: vec![api::ClientAction {
                    action: Some(api::client_action::Action::CreateTask(
                        api::client_action::CreateTask {
                            task: Some(api::Task {
                                id: subtask_id.to_owned(),
                                description: String::new(),
                                dependencies: Some(api::task::Dependencies {
                                    parent_task_id: parent_task_id.to_owned(),
                                }),
                                messages: vec![],
                                summary: String::new(),
                                server_data: String::new(),
                            }),
                        },
                    )),
                }],
            },
        )),
    }
}

fn make_finished_done(
    usage_metadata: Option<api::response_event::stream_finished::ConversationUsageMetadata>,
) -> api::ResponseEvent {
    api::ResponseEvent {
        r#type: Some(api::response_event::Type::Finished(
            api::response_event::StreamFinished {
                reason: Some(api::response_event::stream_finished::Reason::Done(
                    api::response_event::stream_finished::Done {},
                )),
                conversation_usage_metadata: usage_metadata,
                token_usage: vec![],
                should_refresh_model_config: false,
                request_cost: None,
            },
        )),
    }
}

#[cfg(test)]
mod assistant_buffer_tests {
    use super::*;
    use genai::chat::{ChatRole, ToolCall};

    fn reasoning_part(msg: &ChatMessage) -> Option<&str> {
        for p in msg.content.parts() {
            if let ContentPart::ReasoningContent(r) = p {
                return Some(r.as_str());
            }
        }
        None
    }

    /// gate=false + 真实 reasoning → **丢弃**(zerx-lab/warp #25 修复点)。
    /// Cerebras / Groq / OpenRouter 等 OpenAI-strict provider 见到字段就 400。
    #[test]
    fn no_echo_drops_real_reasoning_text() {
        let mut buf = AssistantBuffer::new(false);
        buf.text = Some("Hi".to_string());
        buf.reasoning = Some("internal thought".to_string());
        let mut msgs = Vec::new();
        buf.flush_into(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, ChatRole::Assistant);
        assert!(
            reasoning_part(&msgs[0]).is_none(),
            "must not echo reasoning"
        );
    }

    /// gate=false + tool_calls + 真实 reasoning → tool_calls 这条也不挂 reasoning。
    #[test]
    fn no_echo_drops_reasoning_on_tool_calls_message() {
        let mut buf = AssistantBuffer::new(false);
        buf.text = Some("calling".to_string());
        buf.tool_calls = vec![ToolCall {
            call_id: "c1".to_string(),
            fn_name: "echo".to_string(),
            fn_arguments: serde_json::json!({}),
            thought_signatures: None,
        }];
        buf.reasoning = Some("planning".to_string());
        let mut msgs = Vec::new();
        buf.flush_into(&mut msgs);
        assert_eq!(msgs.len(), 2, "text + tool_calls flush 成两条");
        for m in &msgs {
            assert!(
                reasoning_part(m).is_none(),
                "any-msg reasoning must be absent"
            );
        }
    }

    /// gate=true + 真实 reasoning → 挂真实值(DeepSeek / Kimi 路径)。
    #[test]
    fn echo_keeps_real_reasoning() {
        let mut buf = AssistantBuffer::new(true);
        buf.text = Some("ok".to_string());
        buf.reasoning = Some("thinking...".to_string());
        let mut msgs = Vec::new();
        buf.flush_into(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(reasoning_part(&msgs[0]), Some("thinking..."));
    }

    /// gate=true + 无 reasoning → 挂占位符(满足"字段必须存在"校验)。
    #[test]
    fn echo_inserts_placeholder_when_empty() {
        let mut buf = AssistantBuffer::new(true);
        buf.text = Some("ok".to_string());
        buf.reasoning = None;
        let mut msgs = Vec::new();
        buf.flush_into(&mut msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(reasoning_part(&msgs[0]), Some(REASONING_ECHO_PLACEHOLDER));
    }

    /// gate=true + tool_calls + 真实 reasoning → text 这条占位,tool_calls 那条挂真实值。
    #[test]
    fn echo_with_tool_calls_splits_correctly() {
        let mut buf = AssistantBuffer::new(true);
        buf.text = Some("calling".to_string());
        buf.tool_calls = vec![ToolCall {
            call_id: "c1".to_string(),
            fn_name: "echo".to_string(),
            fn_arguments: serde_json::json!({}),
            thought_signatures: None,
        }];
        buf.reasoning = Some("plan".to_string());
        let mut msgs = Vec::new();
        buf.flush_into(&mut msgs);
        assert_eq!(msgs.len(), 2);
        // text 这条:占位
        assert_eq!(reasoning_part(&msgs[0]), Some(REASONING_ECHO_PLACEHOLDER));
        // tool_calls 这条:真实 reasoning + 含 ToolCall part
        assert_eq!(reasoning_part(&msgs[1]), Some("plan"));
        assert!(
            !msgs[1].content.tool_calls().is_empty(),
            "second message must carry tool_calls"
        );
    }
}

#[cfg(test)]
mod dashscope_thinking_tests {
    use super::*;
    use crate::settings::ReasoningEffortSetting as R;

    const DASHSCOPE_CN: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1/";
    const DASHSCOPE_INTL: &str = "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/";

    #[test]
    fn dashscope_qwen3_triggers() {
        assert!(dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "qwen3-235b-a22b",
            R::High
        ));
    }

    #[test]
    fn dashscope_qwq_triggers() {
        assert!(dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_INTL,
            "qwq-32b",
            R::Medium
        ));
    }

    #[test]
    fn dashscope_deepseek_r1_triggers() {
        assert!(dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "deepseek-r1",
            R::High
        ));
    }

    #[test]
    fn dashscope_kimi_k2_thinking_excluded() {
        // opencode 注释:kimi-k2-thinking 默认就开,不重复注入
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "kimi-k2-thinking",
            R::High
        ));
    }

    #[test]
    fn dashscope_off_setting_skips() {
        // 用户主动关思考时尊重之
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "qwen3-30b",
            R::Off
        ));
    }

    #[test]
    fn dashscope_non_reasoning_model_skips() {
        // qwen-turbo / qwen2.5 等纯 chat 模型不该被注入
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "qwen-turbo",
            R::High
        ));
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            DASHSCOPE_CN,
            "qwen2.5-72b",
            R::High
        ));
    }

    #[test]
    fn non_dashscope_url_skips() {
        // OpenAI / Cerebras / Groq 等不是 DashScope 的 base_url
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            "https://api.openai.com/v1/",
            "qwen3-30b",
            R::High
        ));
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::OpenAi,
            "https://api.cerebras.ai/v1/",
            "qwen3-30b",
            R::High
        ));
    }

    #[test]
    fn non_openai_api_type_skips() {
        // Anthropic / Gemini / DeepSeek api_type 不走这条路径
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::Anthropic,
            DASHSCOPE_CN,
            "qwen3-30b",
            R::High
        ));
        assert!(!dashscope_needs_enable_thinking(
            AgentProviderApiType::DeepSeek,
            DASHSCOPE_CN,
            "deepseek-r1",
            R::High
        ));
    }
}

```


**Tool Call: app\\src\\ai\\blocklist\\controller.rs**
Status: Completed

Diff: app\src\ai\blocklist\controller.rs
```
//! This module contains core business logic for Agent Mode, primarily sending input to an AI
//! model and receiving output.
//!
//! The `BlocklistAIController` orchestrates state updates and service calls to power the
//! Agent Mode UI.
pub mod input_context;
mod pending_response_streams;
pub mod response_stream;
pub(super) mod shared_session;
mod slash_command;
use input_context::{input_context_for_request, parse_context_attachments};
pub use slash_command::*;

use self::response_stream::{PendingTitleGeneration, ResponseStream, ResponseStreamEvent};
use super::agent_view::AgentViewEntryOrigin;
use super::ResponseStreamId;
use super::{
    action_model::{BlocklistAIActionEvent, BlocklistAIActionModel},
    agent_view::{AgentViewController, AgentViewControllerEvent},
    context_model::BlocklistAIContextModel,
    history_model::{BlocklistAIHistoryEvent, BlocklistAIHistoryModel},
    input_model::InputConfig,
    BlocklistAIInputModel, InputType,
};
use crate::ai::agent::api::{self, ServerConversationToken};
use crate::ai::agent::conversation::{AIConversation, ConversationStatus};
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{
    AIAgentActionResult, CancellationReason, PassiveSuggestionResultType, PassiveSuggestionTrigger,
    PassiveSuggestionTriggerType, RunningCommand,
};
use crate::ai::agent::{DocumentContentAttachmentSource, FileContext};
use crate::ai::ambient_agents::AmbientAgentTaskId;
use crate::ai::document::ai_document_model::{
    AIDocumentId, AIDocumentModel, AIDocumentUserEditStatus,
};
use crate::ai::llms::LLMId;
use crate::ai::{
    agent::{
        conversation::AIConversationId, AIAgentActionResultType, AIAgentAttachment, AIAgentContext,
        AIAgentExchangeId, AIAgentInput, AIAgentOutputStatus, AIIdentifiers, EntrypointType,
        FinishedAIAgentOutput, MessageId, RenderableAIError, RequestCost, RequestMetadata,
        StaticQueryType, UserQueryMode,
    },
    llms::LLMPreferences,
    AIRequestUsageModel,
};
use crate::cloud_object::model::persistence::CloudModel;
use crate::features::FeatureFlag;
use crate::global_resource_handles::GlobalResourceHandlesProvider;
use crate::network::NetworkStatus;
use crate::notebooks::editor::model::FileLinkResolutionContext;
use crate::persistence::ModelEvent;
use crate::search::slash_command_menu::static_commands::commands;
use crate::server::server_api::AIApiError;
use crate::terminal::model::block::{
    formatted_terminal_contents_for_input, BlockId, CURSOR_MARKER,
};
use crate::terminal::ssh::util::InteractiveSshCommand;
use crate::terminal::view::inline_banner::ZeroStatePromptSuggestionType;
use crate::terminal::{
    model::session::{active_session::ActiveSession, SessionType},
    model::terminal_model::TerminalModel,
    ShellLaunchData,
};
use crate::workspaces::update_manager::TeamUpdateManager;
use crate::workspaces::user_workspaces::UserWorkspaces;
use crate::{send_telemetry_from_ctx, server::telemetry::TelemetryEvent};
use anyhow::anyhow;
use chrono::{DateTime, Local};
use itertools::Itertools;
use parking_lot::FairMutex;
use pending_response_streams::PendingResponseStreams;
use session_sharing_protocol::common::ParticipantId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use warp_core::assertions::safe_assert;
use warp_multi_agent_api::{
    client_action::{Action, UpdateTaskDescription},
    message, ClientAction, Task, ToolType,
};
use warpui::r#async::{SpawnedFutureHandle, Timer};

use super::orchestration_events::{OrchestrationEventService, OrchestrationEventServiceEvent};
use warpui::{AppContext, Entity, EntityId, ModelContext, ModelHandle, SingletonEntity};

#[derive(Debug, Clone)]
pub struct SessionContext {
    session_type: Option<SessionType>,
    shell: Option<ShellLaunchData>,
    current_working_directory: Option<String>,
    /// OpenWarp:legacy SSH session(用户在本地 PTY 手敲 `ssh xxx@yyy`,
    /// 远端没装 warp shell hook)的连接信息。`session_type` 仍是 `Local`,
    /// 但 PTY 实际跑在远端,需要在 prompt 里告知 LLM,否则模型默认在本地 OS。
    ssh_connection_info: Option<InteractiveSshCommand>,
    /// 是否为 legacy SSH 会话(`IsLegacySSHSession::Yes`)。
    is_legacy_ssh: bool,
}

impl SessionContext {
    pub fn from_session(session: &ActiveSession, app: &AppContext) -> Self {
        let session_arc = session.session(app);
        let ssh_connection_info = session_arc
            .as_ref()
            .and_then(|s| s.subshell_info().as_ref())
            .and_then(|info| info.ssh_connection_info.clone());
        let is_legacy_ssh = session_arc
            .as_ref()
            .map(|s| s.is_legacy_ssh_session())
            .unwrap_or(false);
        SessionContext {
            session_type: session.session_type(app),
            shell: session.shell_launch_data(app),
            current_working_directory: session.current_working_directory().cloned(),
            ssh_connection_info,
            is_legacy_ssh,
        }
    }

    pub fn session_type(&self) -> &Option<SessionType> {
        &self.session_type
    }

    pub fn shell(&self) -> &Option<ShellLaunchData> {
        &self.shell
    }

    pub fn current_working_directory(&self) -> &Option<String> {
        &self.current_working_directory
    }

    /// Returns the remote host ID if this is a `WarpifiedRemote` session with
    /// a connected `RemoteServerClient`.
    pub fn host_id(&self) -> Option<&warp_core::HostId> {
        match &self.session_type {
            Some(SessionType::WarpifiedRemote { host_id }) => host_id.as_ref(),
            Some(SessionType::Local) | None => None,
        }
    }

    /// Returns `true` if this is a remote session (regardless of whether
    /// the remote server client is connected).
    pub fn is_remote(&self) -> bool {
        matches!(self.session_type, Some(SessionType::WarpifiedRemote { .. }))
    }

    /// OpenWarp:legacy SSH 连接信息(host/port),仅在 `is_legacy_ssh()` 为 true 时有意义。
    pub fn ssh_connection_info(&self) -> Option<&InteractiveSshCommand> {
        self.ssh_connection_info.as_ref()
    }

    /// OpenWarp:本会话是否为 legacy SSH(用户手敲 ssh,远端无 warp hook)。
    /// 这种会话 `session_type` 仍是 `Local`,但 PTY 实际跑在远端,
    /// `host_info`/`shell` 等画像反映的是本地客户端而非远端 shell。
    pub fn is_legacy_ssh(&self) -> bool {
        self.is_legacy_ssh
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        SessionContext {
            session_type: None,
            shell: None,
            current_working_directory: None,
            ssh_connection_info: None,
            is_legacy_ssh: false,
        }
    }
}

pub enum BlocklistAIControllerEvent {
    /// Emitted when a request is sent to the AI agent API.
    SentRequest {
        contains_user_query: bool,
        /// True when this request is the first send of a previously queued prompt (e.g.
        /// via `/queue` or the auto-queue toggle) rather than a direct user submission.
        /// Subscribers that perform user-submission side effects (e.g. clearing the input
        /// buffer) should skip those effects when this is true — the user may have typed
        /// new input while the agent was busy and we don't want to wipe it.
        is_queued_prompt: bool,
        /// The model ID used for this request. None for slash commands that don't
        /// send a model request (e.g., /fork).
        model_id: LLMId,
        /// The ID of the response stream for this request.
        stream_id: ResponseStreamId,
    },

    /// Emitted when an AI output response is fully received, particularly relevant when output is
    /// being streamed.
    FinishedReceivingOutput {
        stream_id: ResponseStreamId,
        conversation_id: AIConversationId,
    },

    /// Emitted when the export-to-file slash command is executed.
    ExportConversationToFile {
        filename: Option<String>,
    },

    FreeTierLimitCheckTriggered,
}

#[derive(Debug)]
pub struct RequestInput {
    pub conversation_id: AIConversationId,
    pub input_messages: HashMap<TaskId, Vec<AIAgentInput>>,
    pub working_directory: Option<String>,
    pub model_id: LLMId,
    pub coding_model_id: LLMId,
    pub cli_agent_model_id: LLMId,
    pub computer_use_model_id: LLMId,
    pub shared_session_response_initiator: Option<ParticipantId>,
    pub request_start_ts: DateTime<Local>,
    pub supported_tools_override: Option<Vec<ToolType>>,
}

impl RequestInput {
    fn for_task(
        inputs: Vec<AIAgentInput>,
        task_id: TaskId,
        active_session: &ModelHandle<ActiveSession>,
        shared_session_response_initiator: Option<ParticipantId>,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        app: &AppContext,
    ) -> Self {
        let mut me = Self::new_with_common_fields(
            conversation_id,
            active_session,
            shared_session_response_initiator,
            terminal_view_id,
            app,
        );
        me.input_messages.insert(task_id, inputs);
        me
    }

    fn for_actions_results(
        action_results: Vec<AIAgentActionResult>,
        context: Arc<[AIAgentContext]>,
        active_session: &ModelHandle<ActiveSession>,
        shared_session_response_initiator: Option<ParticipantId>,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        app: &AppContext,
    ) -> Self {
        let mut me = Self::new_with_common_fields(
            conversation_id,
            active_session,
            shared_session_response_initiator,
            terminal_view_id,
            app,
        );
        for result in action_results.into_iter() {
            me.input_messages
                .entry(result.task_id.clone())
                .or_default()
                .push(AIAgentInput::ActionResult {
                    result,
                    context: context.clone(),
                });
        }
        me
    }

    pub fn all_inputs(&self) -> impl Iterator<Item = &AIAgentInput> {
        self.input_messages.values().flatten()
    }

    pub fn with_supported_tools(mut self, tools: Vec<ToolType>) -> Self {
        self.supported_tools_override = Some(tools);
        self
    }

    fn new_with_common_fields(
        conversation_id: AIConversationId,
        active_session: &ModelHandle<ActiveSession>,
        shared_session_response_initiator: Option<ParticipantId>,
        terminal_view_id: EntityId,
        app: &AppContext,
    ) -> Self {
        let llm_prefs = LLMPreferences::as_ref(app);
        let model_id = llm_prefs
            .get_active_base_model(app, Some(terminal_view_id))
            .id
            .clone();
        let coding_model_id = llm_prefs
            .get_active_coding_model(app, Some(terminal_view_id))
            .id
            .clone();
        let cli_agent_model_id = llm_prefs
            .get_active_cli_agent_model(app, Some(terminal_view_id))
            .id
            .clone();
        let computer_use_model_id = llm_prefs
            .get_active_computer_use_model(app, Some(terminal_view_id))
            .id
            .clone();
        let working_directory = active_session
            .as_ref(app)
            .current_working_directory()
            .cloned();

        Self {
            conversation_id,
            input_messages: Default::default(),
            working_directory,
            model_id,
            coding_model_id,
            cli_agent_model_id,
            computer_use_model_id,
            shared_session_response_initiator,
            request_start_ts: Local::now(),
            supported_tools_override: None,
        }
    }
}

/// Controller for Blocklist AI.
///
/// This is responsible for managing and updating blocklist AI state in a single terminal pane.
pub struct BlocklistAIController {
    active_session: ModelHandle<ActiveSession>,
    input_model: ModelHandle<BlocklistAIInputModel>,
    context_model: ModelHandle<BlocklistAIContextModel>,
    action_model: ModelHandle<BlocklistAIActionModel>,
    terminal_model: Arc<FairMutex<TerminalModel>>,

    in_flight_response_streams: PendingResponseStreams,

    /// The ID of the terminal view this controller is associated with.
    terminal_view_id: EntityId,

    should_refresh_available_llms_on_stream_finish: bool,

    shared_session_state: shared_session::SharedSessionState,

    /// Ambient agent task ID attached to this controller. This is a property of the controller, and not an individual
    /// conversation, because the ambient agent task driver owns the entire Warp window working on a task, and any
    /// sessions within it. In the future, one task may span several sessions with background processes.
    ambient_agent_task_id: Option<AmbientAgentTaskId>,

    /// Per-session directory for downloading file attachments.
    /// Set by the agent driver based on the workspace directory (e.g. `{working_dir}/.warp/attachments`).
    attachments_download_dir: Option<std::path::PathBuf>,

    /// Pending auto-resume tasks that are waiting for network connectivity.
    /// These should be cancelled when a new request is sent for the same conversation.
    pending_auto_resume_handles: HashMap<AIConversationId, SpawnedFutureHandle>,
    /// Passive conversations explicitly requested to follow up after actions complete.
    pending_passive_follow_ups: HashSet<AIConversationId>,
    /// Passive suggestion results that should be included with the next request
    /// for a given conversation (e.g. accepted/iterated code diffs that weren't
    /// auto-resumed).
    pending_passive_suggestion_results: HashMap<
        AIConversationId,
        Vec<(
            PassiveSuggestionResultType,
            Option<PassiveSuggestionTrigger>,
        )>,
    >,
}

enum InputQueryType {
    /// The user submitted query from the input. This may map to [`AIAgentInput::UserQuery`] but may
    /// map to other `AIAgentInput` types depending on various factors.
    UserSubmittedQueryFromInput {
        query: String,
        static_query_type: Option<StaticQueryType>,
        running_command: Option<RunningCommand>,
    },
    /// A custom [`AIInputType`].
    AIInputType { ai_input: AIAgentInput },
}

enum WhichTask {
    NewConversation,
    Task {
        conversation_id: AIConversationId,
        task_id: TaskId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FollowUpTrigger {
    Auto,
    UserRequested,
}

struct InputQuery {
    which_task: WhichTask,
    input_query: InputQueryType,
    /// Additional referenced attachments to include in the query
    /// (e.g. file path references from shared session file uploads).
    additional_attachments: HashMap<String, AIAgentAttachment>,
}

impl InputQuery {
    fn query(&self) -> String {
        match &self.input_query {
            InputQueryType::UserSubmittedQueryFromInput { query, .. } => query.clone(),
            InputQueryType::AIInputType { ai_input } => ai_input.user_query().unwrap_or_default(),
        }
    }
}

impl BlocklistAIController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_model: ModelHandle<BlocklistAIInputModel>,
        context_model: ModelHandle<BlocklistAIContextModel>,
        action_model: ModelHandle<BlocklistAIActionModel>,
        active_session: ModelHandle<ActiveSession>,
        agent_view_controller: ModelHandle<AgentViewController>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(&action_model, move |me, event, ctx| {
            let BlocklistAIActionEvent::FinishedAction {
                conversation_id,
                cancellation_reason,
                ..
            } = event
            else {
                return;
            };
            let action_model = me.action_model.as_ref(ctx);
            if action_model.has_unfinished_actions_for_conversation(*conversation_id) {
                return;
            }

            let history_model = BlocklistAIHistoryModel::handle(ctx);
            let Some((is_viewing_shared_session, is_entirely_passive_code_diff)) = history_model
                .as_ref(ctx)
                .conversation(conversation_id)
                .map(|conversation| {
                    (
                        conversation.is_viewing_shared_session(),
                        conversation.is_entirely_passive_code_diff(),
                    )
                })
            else {
                return;
            };

            // Viewer sessions should not send follow-ups.
            // They only act as passive viewers of the action stream.
            if is_viewing_shared_session {
                return;
            }

            let Some(finished_action_results) =
                action_model.get_finished_action_results(*conversation_id)
            else {
                return;
            };
            let is_passive_code_diff = is_entirely_passive_code_diff
                && finished_action_results.last().is_some_and(|result| {
                    matches!(result.result, AIAgentActionResultType::RequestFileEdits(_))
                });
            let has_manual_follow_up = me.pending_passive_follow_ups.contains(conversation_id);

            let is_lrc_command_completed =
                cancellation_reason.is_some_and(|reason| reason.is_lrc_command_completed());
            let should_trigger_follow_up_request = (!is_passive_code_diff
                && !is_lrc_command_completed
                && finished_action_results
                    .iter()
                    .any(|result| result.result.should_trigger_request_upon_completion()))
                || has_manual_follow_up;
            if !should_trigger_follow_up_request {
                // We also check if there's an in-flight req, because it's possible that this
                // subscription callback was queued in response to auto-cancelling pending actions
                // in the process of constructing a request. In such cases, we don't want to update
                // conversation status to Cancelled/Success.
                if !me
                    .in_flight_response_streams
                    .has_active_stream_for_conversation(*conversation_id, ctx)
                {
                    // If the completed actions do not trigger a follow-up request, update conversation
                    // status based on the outcome of the actions.
                    //
                    // (It would otherwise remain `InProgress`, which would be correct, since we'd be
                    // immediately triggering a follow-up request).
                    //
                    // In practice, the only time where this codepath gets triggered is upon completion
                    // of a passive code diff action, where we don't autosend the next request.
                    //
                    // With passive code diffs, its most appropriate to mark the conversation
                    // successful if the passive diff was accepted. In practice, there's only ever
                    // one RequestFileEdits action, so `finished_action_results` at this point
                    // should only have a single element.
                    //
                    // If the user does end up following up on the passive diff-originated conversation,
                    // the status will once again be updated to `InProgress`.
                    let updated_conversation_status = if finished_action_results
                        .iter()
                        .all(|result| result.result.is_successful())
                        || is_lrc_command_completed
                    {
                        ConversationStatus::Success
                    } else {
                        // This is an imperfect heuristic that practically speaking should have no effect.
                        //
                        // If we actually need to differentiate between the state of a conversation
                        // where actions completed with mixed result statuses (e.g. a mix of
                        // cancelled, error, and success) _and_ we don't automatically send back action
                        // results to the agent, then it'd be worth considering adding a new status
                        // variant.
                        ConversationStatus::Cancelled
                    };
                    history_model.update(ctx, |history_model, ctx| {
                        history_model.update_conversation_status(
                            me.terminal_view_id,
                            *conversation_id,
                            updated_conversation_status,
                            ctx,
                        );
                    });
                }
                return;
            }
            let trigger = if has_manual_follow_up {
                FollowUpTrigger::UserRequested
            } else {
                FollowUpTrigger::Auto
            };
            me.send_follow_up_for_conversation(*conversation_id, trigger, ctx);
        });

        ctx.subscribe_to_model(&agent_view_controller, |me, event, ctx| {
            let AgentViewControllerEvent::ExitedAgentView {
                conversation_id,
                final_exchange_count,
                ..
            } = event
            else {
                return;
            };

            // If we exited a brand-new empty conversation, there's nothing meaningful to cancel.
            if *final_exchange_count == 0 {
                return;
            }

            let history = BlocklistAIHistoryModel::handle(ctx);
            let Some(conversation) = history.as_ref(ctx).conversation(conversation_id) else {
                return;
            };

            // Viewer sessions should not send cancellations.
            if conversation.is_viewing_shared_session() {
                return;
            }

            if conversation.status().is_in_progress() {
                me.cancel_conversation_progress(
                    *conversation_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
            }
        });

        // Subscribe to the orchestration event service to inject events
        // (e.g. MessagesReceivedFromAgents) into conversations that receive inter-agent messages.
        if FeatureFlag::Orchestration.is_enabled() {
            let svc = OrchestrationEventService::handle(ctx);
            ctx.subscribe_to_model(&svc, move |me, event, ctx| {
                let OrchestrationEventServiceEvent::EventsReady { conversation_id } = event;
                me.handle_pending_events_ready(*conversation_id, ctx);
            });
        }
        Self {
            input_model,
            context_model,
            action_model,
            active_session,
            terminal_model,
            in_flight_response_streams: PendingResponseStreams::new(),
            terminal_view_id,
            should_refresh_available_llms_on_stream_finish: false,
            shared_session_state: shared_session::SharedSessionState::default(),
            ambient_agent_task_id: None,
            attachments_download_dir: None,
            pending_auto_resume_handles: HashMap::new(),
            pending_passive_follow_ups: HashSet::new(),
            pending_passive_suggestion_results: HashMap::new(),
        }
    }

    /// Internal method to send a query to the AI model. External callers should use either
    /// `send_user_query_in_conversation`, `send_user_in_conversation`, or
    /// `send_custom_ai_input_query` instead.
    ///
    /// When the request is sent, a `BlocklistAIEvent::SentRequest` event is emitted containing the
    /// query itself as well as a oneshot `Receiver` that can be `await`-ed to receive the response
    /// from the AI.
    fn send_query(
        &mut self,
        input_query: InputQuery,
        entrypoint_type: EntrypointType,
        // The shared session participant who initiated this query
        // (None if this is not a shared session).
        shared_session_participant_id: Option<ParticipantId>,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        // Store the participant who initiated this query before sending
        // so that send_query can use it when creating the exchange.
        if let Some(participant_id) = shared_session_participant_id {
            self.set_current_response_initiator(participant_id);
        }

        let query = input_query.query().to_owned();
        let (conversation_id, task_id) = match input_query.which_task {
            WhichTask::NewConversation => {
                let conversation = self.start_new_conversation_for_request(ctx);
                (conversation.id(), conversation.get_root_task_id().clone())
            }
            WhichTask::Task {
                conversation_id,
                task_id,
            } => (conversation_id, task_id),
        };

        // Drain any queued passive suggestion results for this conversation
        // *before* cancelling progress, since cancel_conversation_progress
        // clears the pending map.
        let pending_passive_results = self
            .pending_passive_suggestion_results
            .remove(&conversation_id)
            .unwrap_or_default();

        let cancellation_reason =
            self.cancel_active_conversation_for_follow_up(conversation_id, ctx);

        if let Some(slash_command_request) = SlashCommandRequest::from_query(query.as_str()) {
            slash_command_request.send_request(self, is_queued_prompt, ctx);
            return;
        }

        let (query, user_query_mode) = if let Some(q) =
            commands::strip_command_prefix(&query, commands::PLAN_NAME)
        {
            (q, UserQueryMode::Plan)
        } else if let Some(q) = commands::strip_command_prefix(&query, commands::ORCHESTRATE_NAME) {
            (q, UserQueryMode::Orchestrate)
        } else {
            (query, UserQueryMode::Normal)
        };

        let should_prepend_finished_action_results = matches!(
            input_query.input_query,
            InputQueryType::UserSubmittedQueryFromInput { .. }
        );

        let completed_action_results = self.action_model.update(ctx, |action_model, ctx| {
            action_model.cancel_all_pending_actions(
                conversation_id,
                Some(cancellation_reason),
                ctx,
            );
            action_model.drain_finished_action_results(conversation_id)
        });

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            vec![],
            ctx,
        );
        let mut inputs = if should_prepend_finished_action_results {
            completed_action_results
                .into_iter()
                .map(|result| AIAgentInput::ActionResult {
                    result,
                    context: context.clone(),
                })
                .collect_vec()
        } else {
            // Custom AI inputs like CodeReview and FetchReviewComments are encoded as
            // top-level request variants (`request::input::Type::CodeReview`,
            // `request::input::Type::FetchReviewComments`, etc.), and `convert_input`
            // only emits those variants in the single-input path.
            //
            // Tool call results are encoded differently: they only exist inside
            // `request::input::Type::UserInputs` as `user_input::Input::ToolCallResult`.
            // There is no proto request shape that can represent both a top-level
            // CodeReview-style input and a ToolCallResult in the same request.
            //
            // So if we prepend an ActionResult here, `convert_input` has to fall back
            // to the multi-input `UserInputs` path, where CodeReview / FetchReviewComments
            // are ignored entirely. The stale tool result is preserved, but the custom
            // AI input disappears from the request.
            vec![]
        };

        // Append any queued passive suggestion results that were drained
        // earlier (before cancel_conversation_progress).
        for (suggestion, trigger) in pending_passive_results {
            inputs.push(AIAgentInput::PassiveSuggestionResult {
                trigger,
                suggestion,
                context: context.clone(),
            });
        }

        let additional_attachments = input_query.additional_attachments;
        let ai_input = match input_query.input_query {
            InputQueryType::UserSubmittedQueryFromInput {
                static_query_type,
                running_command,
                ..
            } => input_for_query(
                query,
                &task_id,
                conversation_id,
                static_query_type,
                user_query_mode,
                running_command,
                additional_attachments,
                self.context_model.as_ref(ctx),
                self.active_session.as_ref(ctx),
                ctx,
            ),
            InputQueryType::AIInputType { ai_input } => ai_input,
        };
        inputs.push(ai_input);

        if let Err(e) = self.send_request_input(
            RequestInput::for_task(
                inputs,
                task_id,
                &self.active_session,
                self.get_current_response_initiator(),
                conversation_id,
                self.terminal_view_id,
                ctx,
            ),
            Some(RequestMetadata {
                is_autodetected_user_query: !self.input_model.as_ref(ctx).is_input_type_locked(),
                entrypoint: entrypoint_type,
                is_auto_resume_after_error: false,
            }),
            /*default_to_follow_up_on_success*/ true,
            /*can_attempt_resume_on_error*/ true,
            is_queued_prompt,
            ctx,
        ) {
            log::error!("Failed to send agent request: {e:?}");
        }
    }

    /// Populates plan documents from user query to AIDocumentModel if not already present.
    /// Parses attachments from query and creates AI documents for any user-attached plans.
    /// This is split from parse_context_attachments to run later in the pipeline when new conversations are created.
    fn maybe_populate_plans_for_ai_document_model(
        &self,
        referenced_attachments: &HashMap<String, AIAgentAttachment>,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        // Get file link resolution context from active session
        let session = self.active_session.as_ref(ctx);
        let file_link_resolution_context =
            session
                .current_working_directory()
                .cloned()
                .map(|working_directory| FileLinkResolutionContext {
                    working_directory,
                    shell_launch_data: session.shell_launch_data(ctx),
                });

        for attachment in referenced_attachments.values() {
            let AIAgentAttachment::DocumentContent {
                document_id,
                content,
                source,
                ..
            } = attachment
            else {
                continue;
            };
            if !matches!(*source, DocumentContentAttachmentSource::UserAttached) {
                continue;
            }
            let document_id = match AIDocumentId::try_from(document_id.as_str()) {
                Ok(id) => id,
                Err(_) => {
                    log::warn!("Invalid ai_document_id in document content: {document_id}");
                    continue;
                }
            };

            // Skip if document already exists in the model
            let ai_document_model = AIDocumentModel::as_ref(ctx);
            if ai_document_model
                .get_current_document(&document_id)
                .is_some()
            {
                continue;
            }

            // Look up notebook to get title and sync_id
            let cloud_model = CloudModel::as_ref(ctx);
            let notebook_data = cloud_model
                .get_all_active_notebooks()
                .find(|nb| nb.model().ai_document_id.as_ref() == Some(&document_id))
                .map(|nb| (nb.model().title.clone(), nb.id));

            if let Some((title, sync_id)) = notebook_data {
                AIDocumentModel::handle(ctx).update(ctx, |model, model_ctx| {
                    model.create_document_from_notebook(
                        document_id,
                        sync_id,
                        title,
                        content,
                        conversation_id,
                        file_link_resolution_context.clone(),
                        model_ctx,
                    );
                });
            } else {
                log::warn!("Notebook not found for ai_document_id: {document_id}");
            }
        }
    }

    pub fn send_user_query_in_new_conversation(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        entrypoint_type: EntrypointType,
        participant_id: Option<ParticipantId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_new_conversation_internal(
            query,
            static_query_type,
            entrypoint_type,
            participant_id,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the first submission of a previously queued user prompt into a new conversation.
    /// Same as [`Self::send_user_query_in_new_conversation`] but marks the emitted
    /// `SentRequest` event so UI subscribers (e.g. the input editor) know not to treat
    /// this as a direct user submission and therefore not clear the input buffer.
    pub fn send_queued_user_query_in_new_conversation(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        entrypoint_type: EntrypointType,
        participant_id: Option<ParticipantId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_new_conversation_internal(
            query,
            static_query_type,
            entrypoint_type,
            participant_id,
            /*is_queued_prompt*/ true,
            ctx,
        );
    }

    fn send_user_query_in_new_conversation_internal(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        entrypoint_type: EntrypointType,
        participant_id: Option<ParticipantId>,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let participant_id = participant_id.or_else(|| self.get_sharer_participant_id());
        let running_command = {
            let terminal_model = self.terminal_model.lock();
            get_running_command(&terminal_model)
        };
        if let Some(running_command) = running_command {
            let conversation_id = self.start_new_conversation_for_request(ctx).id();
            let history_model = BlocklistAIHistoryModel::handle(ctx);
            let task_id = match history_model.update(ctx, |history_model, ctx| {
                history_model.create_cli_subagent_task_for_conversation(
                    running_command.block_id.clone(),
                    conversation_id,
                    self.terminal_view_id,
                    ctx,
                )
            }) {
                Ok(task_id) => task_id,
                Err(e) => {
                    log::error!("Could not create CLI subagent task optimistically: {e:?}");
                    return;
                }
            };
            self.send_query(
                InputQuery {
                    which_task: WhichTask::Task {
                        conversation_id,
                        task_id,
                    },
                    input_query: InputQueryType::UserSubmittedQueryFromInput {
                        query,
                        static_query_type,
                        running_command: Some(running_command),
                    },
                    additional_attachments: HashMap::new(),
                },
                entrypoint_type,
                participant_id,
                is_queued_prompt,
                ctx,
            );
        } else {
            self.send_query(
                InputQuery {
                    which_task: WhichTask::NewConversation,
                    input_query: InputQueryType::UserSubmittedQueryFromInput {
                        query,
                        static_query_type,
                        running_command: None,
                    },
                    additional_attachments: HashMap::new(),
                },
                entrypoint_type,
                participant_id,
                is_queued_prompt,
                ctx,
            );
        }
    }

    /// Sends a query into an existing conversation as an agent-initiated request.
    /// This is the agent-initiated counterpart to `send_user_query_in_conversation`.
    pub fn send_agent_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            None,
            false,
            HashMap::new(),
            EntrypointType::AgentInitiated,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the given user query to the AI model.
    pub fn send_user_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        participant_id: Option<ParticipantId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            participant_id,
            false, // skip_running_command_detection
            HashMap::new(),
            EntrypointType::UserInitiated,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the first submission of a previously queued user prompt into an existing conversation.
    /// Same as [`Self::send_user_query_in_conversation`] but marks the emitted `SentRequest`
    /// event so UI subscribers (e.g. the input editor) know not to treat this as a direct
    /// user submission and therefore not clear the input buffer.
    pub fn send_queued_user_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        participant_id: Option<ParticipantId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            participant_id,
            false, // skip_running_command_detection
            HashMap::new(),
            EntrypointType::UserInitiated,
            /*is_queued_prompt*/ true,
            ctx,
        );
    }

    /// Sends the given user query to the AI model, with additional referenced attachments.
    pub fn send_user_query_in_conversation_with_attachments(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        participant_id: Option<ParticipantId>,
        additional_attachments: HashMap<String, AIAgentAttachment>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            participant_id,
            false, // skip_running_command_detection
            additional_attachments,
            EntrypointType::UserInitiated,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the given user query to the AI model, skipping long running command detection.
    /// We use this when we fork a conversation and immediately send an initial query, to avoid
    /// a race condition where restored command blocks may appear long running when the initial query is sent,
    /// causing the query to go to the lrc subagent.
    pub fn send_user_query_in_conversation_no_lrc_subagent(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        participant_id: Option<ParticipantId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            participant_id,
            true, // skip_running_command_detection
            HashMap::new(),
            EntrypointType::UserInitiated,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn send_user_query_in_conversation_internal(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        participant_id: Option<ParticipantId>,
        skip_running_command_detection: bool,
        additional_attachments: HashMap<String, AIAgentAttachment>,
        entrypoint_type: EntrypointType,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let is_viewer = self
            .terminal_model
            .lock()
            .shared_session_status()
            .is_viewer();
        if is_viewer {
            log::error!("Viewers should never attempt to send queries directly");
        }

        // Ensure we capture all pending context blocks before promoting and attaching them to the conversation.
        let context_block_ids = self
            .context_model
            .as_ref(ctx)
            .pending_context_block_ids()
            .clone();

        let (promoted_blocks, task_id, running_command) = {
            let mut terminal_model = self.terminal_model.lock();
            terminal_model
                .block_list_mut()
                .associate_blocks_with_conversation(context_block_ids.iter(), conversation_id);

            // Promote all blocks that are pending for this conversation to attached.
            // This happens at query submission time, making blocks permanently associated with the conversation.
            let promoted_blocks = terminal_model
                .block_list_mut()
                .promote_blocks_to_attached_from_conversation(conversation_id);

            let active_block = terminal_model.block_list().active_block();
            let running_command_opt = if !skip_running_command_detection {
                get_running_command(&terminal_model)
            } else {
                None
            };

            let (task_id, running_command) = if let Some(running_command) = running_command_opt {
                let history_model = BlocklistAIHistoryModel::handle(ctx);
                match history_model.update(ctx, |history_model, ctx| {
                    history_model.create_cli_subagent_task_for_conversation(
                        running_command.block_id.clone(),
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    )
                }) {
                    Ok(task_id) => (task_id, Some(running_command)),
                    Err(e) => {
                        log::error!("Could not create CLI subagent task optimistically: {e:?}");
                        return;
                    }
                }
            } else if let Some(task_id) = active_block
                .is_agent_monitoring()
                .then(|| active_block.agent_interaction_metadata())
                .flatten()
                .filter(|metadata| metadata.conversation_id() == &conversation_id)
                .and_then(|metadata| metadata.subagent_task_id().cloned())
            {
                (task_id, None)
            } else {
                let history_model = BlocklistAIHistoryModel::as_ref(ctx);
                let Some(conversation) = history_model.conversation(&conversation_id) else {
                    log::error!(
                        "Tried to send follow-up query for non-existent conversation: {conversation_id:?}"
                    );
                    return;
                };

                (conversation.get_root_task_id().clone(), None)
            };

            (promoted_blocks, task_id, running_command)
        };

        // Persist the updated visibility for each promoted block
        if !promoted_blocks.is_empty() {
            if let Some(sender) = GlobalResourceHandlesProvider::as_ref(ctx)
                .get()
                .model_event_sender
                .as_ref()
            {
                for (block_id, agent_view_visibility) in promoted_blocks {
                    if let Err(e) = sender.send(ModelEvent::UpdateBlockAgentViewVisibility {
                        block_id: block_id.to_string(),
                        agent_view_visibility: agent_view_visibility.into(),
                    }) {
                        log::error!("Error sending UpdateBlockAgentViewVisibility event: {e:?}");
                    }
                }
            }
        }

        let participant_id = participant_id.or_else(|| self.get_sharer_participant_id());
        self.send_query(
            InputQuery {
                which_task: WhichTask::Task {
                    conversation_id,
                    task_id,
                },
                input_query: InputQueryType::UserSubmittedQueryFromInput {
                    query,
                    static_query_type: None,
                    running_command,
                },
                additional_attachments,
            },
            entrypoint_type,
            participant_id,
            is_queued_prompt,
            ctx,
        );
    }

    /// Sends a request triggered by a zero-state prompt suggestion.
    pub fn send_zero_state_prompt_suggestion(
        &mut self,
        query_type: ZeroStatePromptSuggestionType,
        ctx: &mut ModelContext<Self>,
    ) {
        let participant_id = self.get_sharer_participant_id();
        self.send_query(
            InputQuery {
                which_task: WhichTask::NewConversation,
                input_query: InputQueryType::UserSubmittedQueryFromInput {
                    query: query_type.query().to_string(),
                    static_query_type: query_type.static_query_type(),
                    running_command: None,
                },
                additional_attachments: HashMap::new(),
            },
            EntrypointType::ZeroStateAgentModePromptSuggestion,
            participant_id,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends a custom [`AIAgentInput`] query.
    pub fn send_custom_ai_input_query(
        &mut self,
        ai_input: AIAgentInput,
        ctx: &mut ModelContext<Self>,
    ) {
        let participant_id = self.get_sharer_participant_id();
        let which_task = match self.context_model.as_ref(ctx).selected_conversation_id(ctx) {
            Some(id) => {
                let Some(conversation) = BlocklistAIHistoryModel::as_ref(ctx).conversation(&id)
                else {
                    log::error!(
                        "Tried to send custom AI input query as follow-up in non-existent conversation"
                    );
                    return;
                };
                WhichTask::Task {
                    conversation_id: conversation.id(),
                    task_id: conversation.get_root_task_id().clone(),
                }
            }
            None => WhichTask::NewConversation,
        };
        self.send_query(
            InputQuery {
                which_task,
                input_query: InputQueryType::AIInputType { ai_input },
                additional_attachments: HashMap::new(),
            },
            EntrypointType::UserInitiated,
            participant_id,
            /*is_queued_prompt*/ false,
            ctx,
        )
    }

    pub fn send_slash_command_request(
        &mut self,
        slash_command: SlashCommandRequest,
        ctx: &mut ModelContext<Self>,
    ) {
        // Slash commands are a fresh user turn; mirror `send_query`'s
        // cancel-and-resend so we don't trip `send_request_input`'s in-flight
        // invariant.
        if let Some(conversation_id) = slash_command.conversation_id(self, ctx) {
            self.cancel_active_conversation_for_follow_up(conversation_id, ctx);
        }
        slash_command.send_request(self, /*is_queued_prompt*/ false, ctx);
    }

    /// Cancel any in-flight progress on the active conversation in preparation
    /// for sending a follow-up turn that will land on `target_conversation_id`.
    /// Without this pre-cancel, [`Self::send_request_input`] would trip its
    /// in-flight invariant when the new turn re-uses an existing conversation.
    ///
    /// Returns the [`CancellationReason::FollowUpSubmitted`] reason used so
    /// callers can reuse it for downstream side effects (e.g. cancelling
    /// pending actions on the target conversation).
    fn cancel_active_conversation_for_follow_up(
        &mut self,
        target_conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) -> CancellationReason {
        let active_conversation_id =
            BlocklistAIHistoryModel::as_ref(ctx).active_conversation_id(self.terminal_view_id);
        let reason = CancellationReason::FollowUpSubmitted {
            is_for_same_conversation: active_conversation_id
                .is_some_and(|id| id == target_conversation_id),
        };
        if let Some(active_conversation_id) = active_conversation_id {
            self.cancel_conversation_progress(active_conversation_id, reason, ctx);
        }
        reason
    }

    /// Same as [`Self::send_slash_command_request`] but marks the emitted `SentRequest`
    /// event as a queued prompt submission so UI subscribers (e.g. the input editor)
    /// don't clear the input buffer on the auto-send.
    pub fn send_queued_slash_command_request(
        &mut self,
        slash_command: SlashCommandRequest,
        ctx: &mut ModelContext<Self>,
    ) {
        slash_command.send_request(self, /*is_queued_prompt*/ true, ctx);
    }

    /// Mark a conversation to follow up after its actions complete and attempt to send immediately
    /// if results are already available.
    pub fn request_follow_up_after_actions(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.pending_passive_follow_ups.insert(conversation_id);

        if self
            .in_flight_response_streams
            .has_active_stream_for_conversation(conversation_id, ctx)
        {
            return;
        }

        let has_pending_actions = self
            .action_model
            .as_ref(ctx)
            .get_pending_actions_for_conversation(&conversation_id)
            .next()
            .is_some();
        if has_pending_actions {
            return;
        }

        let finished_action_results = self
            .action_model
            .as_ref(ctx)
            .get_finished_action_results(conversation_id);
        if finished_action_results.is_some_and(|results| !results.is_empty()) {
            self.send_follow_up_for_conversation(
                conversation_id,
                FollowUpTrigger::UserRequested,
                ctx,
            );
        }
    }

    /// Sends a custom AI input, building context from the current session.
    pub fn send_ai_input_with_context(
        &mut self,
        build_input: impl FnOnce(Arc<[AIAgentContext]>) -> AIAgentInput,
        ctx: &mut ModelContext<Self>,
    ) {
        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            None,
            vec![],
            ctx,
        );
        self.send_custom_ai_input_query(build_input(context), ctx);
    }

    /// Sends the result of a passive suggestion (accepted/rejected code diff or
    /// prompt) back to the model so it can continue with accurate context.
    pub fn send_passive_suggestion_result(
        &mut self,
        conversation_id: Option<AIConversationId>,
        suggestion: PassiveSuggestionResultType,
        trigger: Option<PassiveSuggestionTrigger>,
        ctx: &mut ModelContext<Self>,
    ) {
        let which_task = match conversation_id {
            Some(id) => {
                let Some(conversation) = BlocklistAIHistoryModel::as_ref(ctx).conversation(&id)
                else {
                    log::error!("[passive-suggestion-result] conversation not found for id {id:?}");
                    return;
                };
                WhichTask::Task {
                    conversation_id: conversation.id(),
                    task_id: conversation.get_root_task_id().clone(),
                }
            }
            None => WhichTask::NewConversation,
        };

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            conversation_id,
            vec![],
            ctx,
        );

        let participant_id = self.get_sharer_participant_id();
        let trigger_type = trigger.as_ref().map(PassiveSuggestionTriggerType::from);
        log::debug!(
            "[passive-suggestions] sending result: trigger={}, trigger_type={:?}",
            if trigger.is_some() { "Some" } else { "None" },
            trigger_type,
        );
        self.send_query(
            InputQuery {
                which_task,
                input_query: InputQueryType::AIInputType {
                    ai_input: AIAgentInput::PassiveSuggestionResult {
                        trigger,
                        suggestion,
                        context,
                    },
                },
                additional_attachments: HashMap::new(),
            },
            EntrypointType::TriggerPassiveSuggestion {
                trigger: trigger_type,
            },
            participant_id,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Queues a passive suggestion result to be included with the next request
    /// for the given conversation. Use this instead of `send_passive_suggestion_result`
    /// when the result should not trigger an immediate server request (e.g. the user
    /// accepted a code diff without auto-resuming).
    pub fn queue_passive_suggestion_result(
        &mut self,
        conversation_id: AIConversationId,
        suggestion: PassiveSuggestionResultType,
        trigger: Option<PassiveSuggestionTrigger>,
    ) {
        self.pending_passive_suggestion_results
            .entry(conversation_id)
            .or_default()
            .push((suggestion, trigger));
    }

    fn send_follow_up_for_conversation(
        &mut self,
        conversation_id: AIConversationId,
        trigger: FollowUpTrigger,
        ctx: &mut ModelContext<Self>,
    ) {
        if self
            .in_flight_response_streams
            .has_active_stream_for_conversation(conversation_id, ctx)
        {
            return;
        }

        BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
            history.set_active_conversation_id(conversation_id, self.terminal_view_id, ctx);
        });

        if !FeatureFlag::AgentView.is_enabled() && trigger == FollowUpTrigger::Auto {
            // If `AgentView` is enabled, the conversation is guaranteed to be active while the
            // conversation is in-progress and thus while actions are executing/finishing.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.set_pending_query_state_for_existing_conversation(
                    conversation_id,
                    AgentViewEntryOrigin::AutoFollowUp,
                    ctx,
                );
            });
        }

        let finished_results = self.action_model.update(ctx, |action_model, _| {
            action_model.drain_finished_action_results(conversation_id)
        });
        if finished_results.is_empty() {
            return;
        }

        // Check whether any result will trigger a server-side subagent (e.g. CLI
        // subagent for LRC), or if one is already active. If so, we must not
        // piggyback orchestration events because the subagent cannot interpret
        // them and inserting events breaks tool_use/tool_result ordering.
        let will_trigger_server_subagent = finished_results
            .iter()
            .any(|r| r.result.triggers_server_subagent());
        let has_active_subagent = BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(&conversation_id)
            .is_some_and(|c| c.has_active_subagent());

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            vec![],
            ctx,
        );
        let mut request_input = RequestInput::for_actions_results(
            finished_results,
            context,
            &self.active_session,
            self.get_current_response_initiator(),
            conversation_id,
            self.terminal_view_id,
            ctx,
        );

        // Include any pending orchestration events in this follow-up rather
        // than waiting for a separate idle injection turn. Skip when a server
        // subagent is or will be active — events will be delivered via the idle
        // path once the subagent session ends.
        let mut has_piggybacked_events = false;
        if FeatureFlag::Orchestration.is_enabled() {
            if will_trigger_server_subagent || has_active_subagent {
                log::debug!(
                    "Skipping event piggyback for conversation {conversation_id:?}: \
                     {}",
                    if will_trigger_server_subagent {
                        "results will trigger a server-side subagent"
                    } else {
                        "a subagent is currently active"
                    }
                );
            } else if let Some((event_inputs, task_id)) = OrchestrationEventService::handle(ctx)
                .update(ctx, |svc, ctx| {
                    svc.drain_events_for_request(conversation_id, ctx)
                })
            {
                has_piggybacked_events = true;
                request_input
                    .input_messages
                    .entry(task_id)
                    .or_default()
                    .extend(event_inputs);
            }
        }

        let result = self.send_request_input(
            request_input,
            None,
            /*default_to_follow_up_on_success*/ false,
            /*can_attempt_resume_on_error*/ true,
            /*is_queued_prompt*/ false,
            ctx,
        );

        if has_piggybacked_events && result.is_err() {
            OrchestrationEventService::handle(ctx).update(ctx, |svc, ctx| {
                svc.requeue_awaiting_events(conversation_id, ctx);
            });
        }

        self.pending_passive_follow_ups.remove(&conversation_id);
    }

    /// Handles the EventsReady signal. Checks readiness, drains
    /// pending events from the service, and injects them into the conversation.
    fn handle_pending_events_ready(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        let owns = BlocklistAIHistoryModel::as_ref(ctx)
            .all_live_conversations_for_terminal_view(self.terminal_view_id)
            .any(|c| c.id() == conversation_id);
        if !owns {
            return;
        }

        if self
            .in_flight_response_streams
            .has_active_stream_for_conversation(conversation_id, ctx)
        {
            return;
        }

        // Only drain when the conversation is actually idle.
        let is_success = BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(&conversation_id)
            .is_some_and(|c| matches!(c.status(), ConversationStatus::Success));
        if !is_success {
            return;
        }

        let Some((inputs, task_id)) = OrchestrationEventService::handle(ctx)
            .update(ctx, |svc, ctx| {
                svc.drain_events_for_request(conversation_id, ctx)
            })
        else {
            return;
        };

        if self
            .send_request_input(
                RequestInput::for_task(
                    inputs,
                    task_id,
                    &self.active_session,
                    self.get_current_response_initiator(),
                    conversation_id,
                    self.terminal_view_id,
                    ctx,
                ),
                None,
                /*default_to_follow_up_on_success*/ true,
                /*can_attempt_resume_on_error*/ true,
                /*is_queued_prompt*/ false,
                ctx,
            )
            .is_err()
        {
            OrchestrationEventService::handle(ctx).update(ctx, |svc, ctx| {
                svc.requeue_awaiting_events(conversation_id, ctx);
            });
        }
    }

    pub fn resume_conversation(
        &mut self,
        conversation_id: AIConversationId,
        can_attempt_resume_on_error: bool,
        is_auto_resume_after_error: bool,
        additional_context: Vec<AIAgentContext>,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) =
            BlocklistAIHistoryModel::as_ref(ctx).conversation(&conversation_id)
        else {
            log::error!("Tried to resume non-existent conversation: {conversation_id:?}");
            return;
        };
        let task_id = {
            let terminal_model = self.terminal_model.lock();
            let active_block = terminal_model.block_list().active_block();
            if let Some(agent_interaction_metadata) = active_block
                .agent_interaction_metadata()
                .filter(|metadata| {
                    metadata.conversation_id() == &conversation_id && metadata.is_agent_in_control()
                })
            {
                agent_interaction_metadata
                    .subagent_task_id()
                    .cloned()
                    .unwrap_or_else(|| conversation.get_root_task_id().clone())
            } else {
                conversation.get_root_task_id().clone()
            }
        };

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            additional_context,
            ctx,
        );

        let inputs = vec![AIAgentInput::ResumeConversation { context }];
        let metadata = if is_auto_resume_after_error {
            Some(RequestMetadata {
                is_autodetected_user_query: false,
                entrypoint: EntrypointType::ResumeConversation,
                is_auto_resume_after_error: true,
            })
        } else {
            None
        };
        let _ = self.send_request_input(
            RequestInput::for_task(
                inputs,
                task_id,
                &self.active_session,
                self.get_current_response_initiator(),
                conversation_id,
                self.terminal_view_id,
                ctx,
            ),
            metadata,
            /*default_to_follow_up_on_success*/ true,
            can_attempt_resume_on_error,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    pub fn send_passive_code_diff_request(
        &mut self,
        query: String,
        block_id: &BlockId,
        file_contexts: Vec<FileContext>,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<(AIConversationId, ResponseStreamId)> {
        let mut input_context = file_contexts
            .into_iter()
            .map(AIAgentContext::File)
            .collect_vec();
        if let Some(block_context) = self
            .context_model
            .as_ref(ctx)
            .transform_block_to_context(block_id, false)
        {
            input_context.push(block_context);
        }

        let new_conversation = self.start_new_conversation_for_request(ctx);
        self.send_request_input(
            RequestInput::for_task(
                vec![AIAgentInput::AutoCodeDiffQuery {
                    query,
                    context: input_context.into(),
                }],
                new_conversation.get_root_task_id().clone(),
                &self.active_session,
                self.get_current_response_initiator(),
                new_conversation.id(),
                self.terminal_view_id,
                ctx,
            ),
            Some(RequestMetadata {
                is_autodetected_user_query: false,
                entrypoint: EntrypointType::PromptSuggestion {
                    is_static: false,
                    is_coding: true,
                },
                is_auto_resume_after_error: false,
            }),
            /*default_to_follow_up_on_success=*/ false,
            /*can_attempt_resume_on_error*/ true,
            /*is_queued_prompt*/ false,
            ctx,
        )
    }

    /// Builds request params for an out-of-band passive suggestions request.
    ///
    /// This reads conversation state read-only and does NOT create exchanges,
    /// register response streams, or modify conversation status. The caller
    /// is responsible for spawning the API call and handling the response.
    ///
    /// If `followup_conversation_id` is provided, the conversation's task context
    /// and server token are included so the server can use prior context.
    /// Otherwise, a new conversation is created to anchor the request.
    /// Builds request params for an out-of-band passive suggestions request.
    ///
    /// This is read-only and does NOT create exchanges, register response
    /// streams, or modify conversation history. The caller is responsible for
    /// spawning the API call and handling the response.
    ///
    /// If `followup_conversation_id` is provided, the conversation's task
    /// context and server token are included so the server can use prior
    /// context. Otherwise a fresh, ephemeral conversation ID is generated
    /// without touching the history model.
    pub fn build_passive_suggestions_request_params(
        &self,
        followup_conversation_id: Option<AIConversationId>,
        trigger: PassiveSuggestionTrigger,
        supported_tools: Vec<ToolType>,
        ctx: &ModelContext<Self>,
    ) -> anyhow::Result<(AIConversationId, api::RequestParams)> {
        let history_model = BlocklistAIHistoryModel::as_ref(ctx);

        // Resolve conversation state. For follow-ups we read from history;
        // for new triggers we generate a fresh ID without persisting anything.
        let (conversation_id, task_id, conversation_data) = if let Some(conversation_id) =
            followup_conversation_id
        {
            let Some(conversation) = history_model.conversation(&conversation_id) else {
                return Err(anyhow!(
                    "Tried to build passive suggestions request params for non-existent conversation with ID {conversation_id:?}"
                ));
            };
            let task_id = conversation.get_root_task_id().clone();
            let conversation_data = api::ConversationData {
                id: conversation_id,
                tasks: conversation.compute_active_tasks(),
                server_conversation_token: conversation.server_conversation_token().cloned(),
                forked_from_conversation_token: conversation
                    .forked_from_server_conversation_token()
                    .cloned(),
                ambient_agent_task_id: self.ambient_agent_task_id,
                existing_suggestions: None,
            };
            (conversation_id, task_id, conversation_data)
        } else if !matches!(
            trigger,
            PassiveSuggestionTrigger::AgentResponseCompleted { .. }
        ) {
            // Generate a fresh, ephemeral conversation ID without mutating history.
            let conversation_id = AIConversationId::new();
            let task_id = TaskId::new(uuid::Uuid::new_v4().to_string());
            let conversation_data = api::ConversationData {
                id: conversation_id,
                tasks: vec![],
                server_conversation_token: None,
                forked_from_conversation_token: None,
                ambient_agent_task_id: self.ambient_agent_task_id,
                existing_suggestions: None,
            };
            (conversation_id, task_id, conversation_data)
        } else {
            return Err(anyhow!(
                "Tried to use agent response completed trigger to generate passive suggestions without a conversation ID"
            ));
        };

        let inputs = vec![AIAgentInput::TriggerPassiveSuggestion {
            context: input_context_for_request(
                false,
                self.context_model.as_ref(ctx),
                self.active_session.as_ref(ctx),
                Some(conversation_id),
                vec![],
                ctx,
            ),
            attachments: vec![],
            trigger: trigger.clone(),
        }];

        let request_input = RequestInput::for_task(
            inputs,
            task_id,
            &self.active_session,
            self.get_current_response_initiator(),
            conversation_id,
            self.terminal_view_id,
            ctx,
        )
        .with_supported_tools(supported_tools);

        let metadata = Some(RequestMetadata {
            is_autodetected_user_query: false,
            entrypoint: EntrypointType::TriggerPassiveSuggestion {
                trigger: Some((&trigger).into()),
            },
            is_auto_resume_after_error: false,
        });

        let request_params = api::RequestParams::new(
            Some(self.terminal_view_id),
            SessionContext::from_session(self.active_session.as_ref(ctx), ctx),
            &request_input,
            conversation_data,
            metadata,
            ctx,
        );

        Ok((conversation_id, request_params))
    }

    pub fn send_unit_test_suggestions_request(
        &mut self,
        block_output: String,
        trigger: PassiveSuggestionTrigger,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<(AIConversationId, ResponseStreamId)> {
        let attachments = vec![AIAgentAttachment::PlainText(block_output.to_string())];
        let trigger_type = (&trigger).into();
        let inputs = vec![AIAgentInput::TriggerPassiveSuggestion {
            context: input_context_for_request(
                false,
                self.context_model.as_ref(ctx),
                self.active_session.as_ref(ctx),
                None,
                vec![],
                ctx,
            ),
            attachments,
            trigger,
        }];

        let new_conversation = self.start_new_conversation_for_request(ctx);
        self.send_request_input(
            RequestInput::for_task(
                inputs,
                new_conversation.get_root_task_id().clone(),
                &self.active_session,
                self.get_current_response_initiator(),
                new_conversation.id(),
                self.terminal_view_id,
                ctx,
            ),
            Some(RequestMetadata {
                is_autodetected_user_query: false,
                entrypoint: EntrypointType::TriggerPassiveSuggestion {
                    trigger: Some(trigger_type),
                },
                is_auto_resume_after_error: false,
            }),
            /*default_to_follow_up_on_success*/ false,
            /*can_attempt_resume_on_error*/ true,
            /*is_queued_prompt*/ false,
            ctx,
        )
    }

    /// Set the ID of the ambient agent task which owns this controller and its backing session.
    pub fn set_ambient_agent_task_id(
        &mut self,
        id: Option<AmbientAgentTaskId>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.ambient_agent_task_id = id;
        self.action_model.update(ctx, |action_model, ctx| {
            action_model.set_ambient_agent_task_id(id, ctx);
        });
    }

    /// Set the per-session directory for downloading file attachments.
    pub fn set_attachments_download_dir(&mut self, dir: std::path::PathBuf) {
        self.attachments_download_dir = Some(dir);
    }

    fn start_new_conversation_for_request<'a>(
        &self,
        ctx: &'a mut ModelContext<Self>,
    ) -> &'a AIConversation {
        let is_autoexecute_override = self
            .context_model
            .as_ref(ctx)
            .pending_query_autoexecute_override(ctx)
            .is_autoexecute_any_action();
        let history_model = BlocklistAIHistoryModel::handle(ctx);
        let id = history_model.update(ctx, |history_model, ctx| {
            // We don't mark passive conversations as "the active conversation" (at least when they first appear).
            history_model.start_new_conversation(
                self.terminal_view_id,
                is_autoexecute_override,
                false,
                ctx,
            )
        });
        history_model
            .as_ref(ctx)
            .conversation(&id)
            .expect("Conversation exists- was just created.")
    }

    /// Attempts to send a request to the AI model API. Adds context to the input if it
    /// contains a user query. Returns `Err` if the AI input was not able to be sent due to an
    /// existing in-flight request. Emits an event containing a receiver for the AI's output.
    /// If conversation_id is Some, we follow up in that conversation.
    /// If it's None or we can't find a conversation with that ID, we start a new one.
    /// Returns the conversation ID of affected conversation and response stream ID.
    ///
    ///  This function does not handle cancelling any in flight requests (and sending them back as
    /// input) for an existing conversation. Consider calling [`Self::send_custom_ai_input_query`] if
    /// you're trying to send a query with a custom [`AIAgentInput`] type where you'd like the "normal"
    /// flow that handles existing conversations properly.
    fn send_request_input(
        &mut self,
        request_input: RequestInput,
        query_metadata: Option<RequestMetadata>,
        default_to_follow_up_on_success: bool,
        can_attempt_resume_on_error: bool,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<(AIConversationId, ResponseStreamId)> {
        let history_model = BlocklistAIHistoryModel::handle(ctx);
        let (
            conversation_id,
            conversation_server_token,
            conversation_forked_from_token,
            active_tasks,
            parent_agent_id,
            agent_name,
        ) = {
            let Some(conversation) = history_model
                .as_ref(ctx)
                .conversation(&request_input.conversation_id)
            else {
                return Err(anyhow!(
                    "Tried to send request for non-existent conversation with ID {:?}",
                    request_input.conversation_id
                ));
            };

            let active_tasks = conversation.compute_active_tasks();

            (
                conversation.id(),
                conversation.server_conversation_token().cloned(),
                conversation
                    .forked_from_server_conversation_token()
                    .cloned(),
                active_tasks,
                conversation.parent_agent_id().map(str::to_string),
                conversation.agent_name().map(str::to_string),
            )
        };

        // Cancel any pending auto-resume for this conversation, since the user is sending a new
        // request.
        if let Some(handle) = self
            .pending_auto_resume_handles
            .remove(&request_input.conversation_id)
        {
            handle.abort();
        }

        // Make sure there's no existing response stream for the conversation. If
        // there is, something has gone wrong.
        if self
            .in_flight_response_streams
            .has_active_stream_for_conversation(conversation_id, ctx)
        {
            send_telemetry_from_ctx!(
                TelemetryEvent::AIInputNotSent {
                    entrypoint: query_metadata.map(|metadata| metadata.entrypoint),
                    inputs: request_input
                        .all_inputs()
                        .cloned()
                        .map(|input| input.into())
                        .collect(),
                    active_server_conversation_id: conversation_server_token.clone(),
                    active_client_conversation_id: Some(conversation_id),
                },
                ctx
            );
            const AI_INPUT_NOT_SENT_ERROR_STR: &str =
                "Not sending AI input because there is an in-flight request";
            safe_assert!(false, "{}", AI_INPUT_NOT_SENT_ERROR_STR);
            return Err(anyhow::anyhow!(AI_INPUT_NOT_SENT_ERROR_STR));
        }

        let conversation_data = api::ConversationData {
            id: conversation_id,
            tasks: active_tasks,
            server_conversation_token: conversation_server_token,
            forked_from_conversation_token: conversation_forked_from_token,
            ambient_agent_task_id: self.ambient_agent_task_id,
            existing_suggestions: history_model
                .as_ref(ctx)
                .existing_suggestions_for_conversation(conversation_id)
                .cloned(),
        };

        // Log an error if tool call results do not have corresponding tool calls in task context
        validate_tool_call_results(
            request_input.all_inputs(),
            &conversation_data.tasks,
            &conversation_data.server_conversation_token,
        );

        let mut request_params = api::RequestParams::new(
            Some(self.terminal_view_id),
            SessionContext::from_session(self.active_session.as_ref(ctx), ctx),
            &request_input,
            conversation_data.clone(),
            query_metadata,
            ctx,
        );
        request_params.parent_agent_id = parent_agent_id;
        request_params.agent_name = agent_name;

        // OpenWarp BYOP 本地会话压缩:
        //   1. 自动 prune 旧 tool output(对齐 opencode `compaction.ts:297-341` prune)
        //   2. 把 conversation.compaction_state.clone() 注入 request_params
        //
        // chat_stream::build_chat_request 会据此投影 messages(隐去已压缩区间 + 替换 compacted tool output);
        // SummarizeConversation input 路径还会切 head + 拼 SUMMARY_TEMPLATE。
        // 非 BYOP 路径(走 server protobuf)不读这个字段,无副作用。
        let compaction_cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
        history_model.update(ctx, |history_model, _ctx| {
            if let Some(convo) = history_model.conversation_mut(&conversation_id) {
                crate::ai::byop_compaction::commit::prune_now(convo, &compaction_cfg);
            }
        });
        if let Some(convo) = history_model.as_ref(ctx).conversation(&conversation_id) {
            request_params.compaction_state = Some(convo.compaction_state.clone());
        }

        // OpenWarp BYOP:检测当前请求是否绑定 LRC(alt-screen 长命令)。
        // - tag-in 首轮:注入 command_id + running_command,并让 chat_stream 合成 subagent
        //   CreateTask 事件来升级 master 路径已经创建的 optimistic CLI subtask。
        // - 已进入 agent control 的后续轮:auto-resume / tool result 仍要注入 command_id
        //   与最新 PTY 快照,但不能重复 spawn subagent。
        {
            let terminal_model = self.terminal_model.lock();
            let active_block = terminal_model.block_list().active_block();
            let is_lrc_tagged_in = active_block.is_agent_tagged_in();
            let is_matching_lrc_agent = active_block.is_agent_in_control()
                && active_block
                    .agent_interaction_metadata()
                    .is_some_and(|metadata| metadata.conversation_id() == &conversation_id);
            if is_lrc_tagged_in || is_matching_lrc_agent {
                request_params.lrc_command_id = Some(active_block.id().to_string());
                request_params.lrc_should_spawn_subagent = is_lrc_tagged_in;

                // OpenWarp A3:把完整 RunningCommand 注入到本轮 UserQuery 中,
                // 严格对齐上游 `get_running_command` 的 grid_contents 提取逻辑
                // (alt-screen 走 alt_screen.grid_handler,非 alt-screen 走 output_grid)。
                // 之前用 `output_to_string_force_full_grid_contents()` 在 nvim 等
                // alt-screen TUI 下取到空字符串,导致 prefix 块为空,模型说看不到 command_id。
                if let Some(running_command) = byop_get_running_command_for_lrc(&terminal_model) {
                    request_params.lrc_running_command = Some(running_command.clone());
                    let total_inputs = request_params.input.len();
                    let mut filled_count = 0usize;
                    for input in request_params.input.iter_mut() {
                        if let crate::ai::agent::AIAgentInput::UserQuery {
                            running_command: rc_slot @ None,
                            ..
                        } = input
                        {
                            *rc_slot = Some(running_command.clone());
                            filled_count += 1;
                        }
                    }
                    log::info!(
                        "[byop-diag] LRC running_command filled: {filled_count}/{total_inputs} \
                         UserQuery slot(s); should_spawn={} grid_contents_len={} command={:?} is_alt_screen={}",
                        request_params.lrc_should_spawn_subagent,
                        running_command.grid_contents.len(),
                        running_command.command,
                        running_command.is_alt_screen_active
                    );
                } else {
                    log::warn!(
                        "[byop-diag] LRC detected but byop_get_running_command_for_lrc \
                         returned None (active_block 状态不符)"
                    );
                }
            }
        }

        let server_conversation_token_for_identifiers =
            conversation_data.server_conversation_token.clone();

        let response_stream = ctx.add_model(|ctx| {
            // Create AIIdentifiers for the response stream
            let ai_identifiers = AIIdentifiers {
                server_output_id: None, // Will be populated by the successful response
                server_conversation_id: server_conversation_token_for_identifiers.map(Into::into),
                client_conversation_id: Some(conversation_data.id),
                client_exchange_id: None,
                model_id: Some(request_params.model.clone()),
            };
            ResponseStream::new(
                request_params.clone(),
                ai_identifiers,
                can_attempt_resume_on_error,
                ctx,
            )
        });
        let response_stream_id = response_stream.as_ref(ctx).id().clone();
        let response_stream_clone = response_stream.clone();
        let input_contains_user_query = request_input
            .all_inputs()
            .any(|input| input.is_user_query());
        ctx.subscribe_to_model(&response_stream, move |me, event, ctx| {
            me.handle_response_stream_event(
                input_contains_user_query,
                event,
                &response_stream_clone,
                ctx,
            );
        });

        let is_passive_request = request_input
            .all_inputs()
            .any(|input| input.is_passive_request());

        for input in request_input.all_inputs() {
            if let AIAgentInput::UserQuery {
                referenced_attachments,
                ..
            } = input
            {
                self.maybe_populate_plans_for_ai_document_model(
                    referenced_attachments,
                    conversation_data.id,
                    ctx,
                );
            }
        }

        history_model.update(ctx, |history_model, ctx| {
            match history_model.update_conversation_for_new_request_input(
                request_input,
                response_stream_id.clone(),
                self.terminal_view_id,
                ctx,
            ) {
                Ok(_) => {
                    history_model.update_conversation_status(
                        self.terminal_view_id,
                        conversation_data.id,
                        ConversationStatus::InProgress,
                        ctx,
                    );
                }
                Err(e) => {
                    log::warn!("Failed to push new exchange to AI conversation: {e:?}");
                }
            }
        });

        self.in_flight_response_streams.register_new_stream(
            response_stream_id.clone(),
            conversation_data.id,
            response_stream,
            CancellationReason::FollowUpSubmitted {
                is_for_same_conversation: true,
            },
            ctx,
        );

        if input_contains_user_query {
            // Get the pending document ID before clearing context
            let pending_document_id = self.context_model.as_ref(ctx).pending_document_id();

            // Reset the context state to the default.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.reset_context_to_default(ctx);
            });

            // Update the document status to UpToDate after query submission
            if let Some(doc_id) = pending_document_id {
                AIDocumentModel::handle(ctx).update(ctx, |model, mctx| {
                    model.set_user_edit_status(&doc_id, AIDocumentUserEditStatus::UpToDate, mctx);
                });
            }
        }

        ctx.emit(BlocklistAIControllerEvent::SentRequest {
            contains_user_query: input_contains_user_query,
            is_queued_prompt,
            model_id: request_params.model.clone(),
            stream_id: response_stream_id.clone(),
        });
        if !is_passive_request {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history_model, ctx| {
                history_model.set_active_conversation_id(
                    conversation_data.id,
                    self.terminal_view_id,
                    ctx,
                )
            });
        }

        // Trigger a snapshot save to persist the agent view state when a user query is sent.
        // This ensures the agent view is restored if the app restarts.
        if input_contains_user_query {
            ctx.dispatch_global_action("workspace:save_app", ());
        }

        // If `AgentView` is enabled, the agent view is guaranteed to be active when the agent
        // input is sent, so logic to ensure follow-ups is redundant.
        if !FeatureFlag::AgentView.is_enabled() && default_to_follow_up_on_success {
            // Set the input mode to AI but allow autodetection to run
            self.input_model.update(ctx, |input_model, ctx| {
                input_model.set_input_config_for_classic_mode(
                    InputConfig {
                        input_type: InputType::AI,
                        is_locked: false,
                    },
                    ctx,
                );
            });
            // After making an AI query, default to asking a follow up.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.set_pending_query_state_for_existing_conversation(
                    conversation_data.id,
                    AgentViewEntryOrigin::AutoFollowUp,
                    ctx,
                )
            });
        }

        Ok((conversation_data.id, response_stream_id))
    }

    /// Cancels a pending AI request response stream, given the exchange ID, if it exists.
    /// Returns true if a pending stream was found and canceled, false otherwise.
    pub fn try_cancel_pending_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        self.in_flight_response_streams
            .try_cancel_stream(stream_id, reason, ctx)
    }

    /// Cancels 'progress' for the active conversation if there is one:
    ///  * If there is an in-flight request, cancels it.
    ///  * Else, if the request finished, but actions from the response are pending or mid-execution, cancels all of them.
    pub fn cancel_conversation_progress(
        &mut self,
        conversation_id: AIConversationId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) {
        // Cancel any pending auto-resume for this conversation.
        if let Some(handle) = self.pending_auto_resume_handles.remove(&conversation_id) {
            handle.abort();
        }

        // Discard any queued passive suggestion results for this conversation.
        self.pending_passive_suggestion_results
            .remove(&conversation_id);

        if !self
            .in_flight_response_streams
            .try_cancel_streams_for_conversation(conversation_id, reason, ctx)
        {
            // Otherwise, cancel pending actions and update the input state.
            self.action_model.update(ctx, |action_model, ctx| {
                action_model.cancel_all_pending_actions(conversation_id, Some(reason), ctx);
            });
            self.set_input_mode_for_cancellation(ctx);
        }
    }

    /// Clears finished action results for a conversation. Used when reverting.
    pub fn clear_finished_action_results(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.action_model.update(ctx, |action_model, _| {
            action_model.clear_finished_action_results(conversation_id);
        });
    }

    /// Cancels the in-flight request for the given conversation, if there is one.
    ///
    /// Returns `true` if a request was actually cancelled.
    pub fn cancel_request(
        &mut self,
        response_stream_id: &ResponseStreamId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        self.in_flight_response_streams
            .try_cancel_stream(response_stream_id, reason, ctx)
    }

    fn start_title_generation(
        &mut self,
        pending_title_generation: PendingTitleGeneration,
        stream_id: ResponseStreamId,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        let terminal_view_id = self.terminal_view_id;
        let _ = ctx.spawn(
            async move {
                let result = crate::ai::agent_providers::chat_stream::generate_title_via_byop(
                    &pending_title_generation.input,
                    &pending_title_generation.user_query,
                )
                .await;
                (pending_title_generation.task_id, result)
            },
            move |me, (task_id, result), ctx| match result {
                Ok(Some(title)) => {
                    log::info!("[byop] title generated: {title:?}");
                    let client_actions = vec![ClientAction {
                        action: Some(Action::UpdateTaskDescription(UpdateTaskDescription {
                            task_id,
                            description: title,
                        })),
                    }];
                    let response_event = warp_multi_agent_api::ResponseEvent {
                        r#type: Some(warp_multi_agent_api::response_event::Type::ClientActions(
                            warp_multi_agent_api::response_event::ClientActions {
                                actions: client_actions.clone(),
                            },
                        )),
                    };
                    if FeatureFlag::AgentSharedSessions.is_enabled() {
                        let participant_id = me
                            .get_current_response_initiator()
                            .or_else(|| me.get_sharer_participant_id());
                        let mut model = me.terminal_model.lock();
                        if model.shared_session_status().is_sharer() {
                            model.send_agent_response_for_shared_session(
                                &response_event,
                                participant_id,
                                None,
                            );
                        }
                    }
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history_model, ctx| {
                        match history_model.apply_client_actions(
                            &stream_id,
                            client_actions,
                            conversation_id,
                            terminal_view_id,
                            ctx,
                        ) {
                            Ok(()) => {
                                ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationMetadata {
                                    terminal_view_id: Some(terminal_view_id),
                                    conversation_id,
                                });
                            }
                            Err(e) => {
                                log::warn!("[byop] title update failed: {e:#}");
                            }
                        }
                    });
                }
                Ok(None) => {
                    log::warn!("[byop] title gen returned empty content; skip");
                }
                Err(e) => {
                    log::warn!("[byop] title gen failed: {e:#}; skip");
                }
            },
        );
    }

    fn handle_response_stream_event(
        &mut self,
        did_input_contain_user_query: bool,
        event: &ResponseStreamEvent,
        response_stream: &ModelHandle<ResponseStream>,
        ctx: &mut ModelContext<Self>,
    ) {
        let stream_id = response_stream.as_ref(ctx).id().clone();

        match event {
            ResponseStreamEvent::ReceivedEvent(event) => {
                // Dynamic lookup handles conversation splits mid-stream.
                let Some(conversation_id) = BlocklistAIHistoryModel::as_ref(ctx)
                    .conversation_for_response_stream(&stream_id)
                else {
                    log::warn!("Could not find conversation for response stream: {stream_id:?}");
                    return;
                };
                let Some(event) = event.consume() else {
                    debug_assert!(
                        false,
                        "This model should only have a single subscriber that takes ownership over the event."
                    );
                    return;
                };
                let history_model = BlocklistAIHistoryModel::handle(ctx);
                match event {
                    Ok(event) => {
                        // If this controller is part of a shared session, forward the entire response event to viewers first.
                        if FeatureFlag::AgentSharedSessions.is_enabled() {
                            let mut model = self.terminal_model.lock();
                            if model.shared_session_status().is_sharer() {
                                // Get the participant who initiated this response, falling back to the sharer if needed.
                                let participant_id = self
                                    .get_current_response_initiator()
                                    .or_else(|| self.get_sharer_participant_id());

                                // For forked conversations (e.g. when loading from cloud), include
                                // the original conversation token so viewers can link the new
                                // server-assigned token to their existing conversation.
                                //
                                // This token is cleared after the first Init event (see below),
                                // so it's only sent once per forked conversation.
                                let forked_from_token = history_model
                                    .as_ref(ctx)
                                    .conversation(&conversation_id)
                                    .and_then(|conv| {
                                        conv.forked_from_server_conversation_token()
                                            .map(|t| t.as_str().to_string())
                                    });

                                model.send_agent_response_for_shared_session(
                                    &event,
                                    participant_id,
                                    forked_from_token,
                                );
                            }
                        }
                        let Some(event) = event.r#type else {
                            return;
                        };
                        match event {
                            warp_multi_agent_api::response_event::Type::Init(init_event) => {
                                history_model.update(ctx, |history_model, ctx| {
                                    history_model.initialize_output_for_response_stream(
                                        &stream_id,
                                        conversation_id,
                                        self.terminal_view_id,
                                        init_event,
                                        ctx,
                                    );

                                    // Clear the forked_from token after the first Init event.
                                    // For forked conversations, we only need to send this once so
                                    // viewers can update their conversation's server token. After
                                    // that, the viewer's conversation uses the new token directly.
                                    if let Some(conversation) =
                                        history_model.conversation_mut(&conversation_id)
                                    {
                                        conversation.clear_forked_from_server_conversation_token();
                                    }
                                });
                            }
                            warp_multi_agent_api::response_event::Type::Finished(
                                finished_event,
                            ) => {
                                let completed_successfully = matches!(
                                    finished_event.reason.as_ref(),
                                    Some(
                                        warp_multi_agent_api::response_event::stream_finished::Reason::Done(_)
                                    ) | None
                                );
                                if completed_successfully {
                                    if let Some(pending_title_generation) =
                                        response_stream.update(ctx, |response_stream, _| {
                                            response_stream.take_pending_title_generation()
                                        })
                                    {
                                        self.start_title_generation(
                                            pending_title_generation,
                                            stream_id.clone(),
                                            conversation_id,
                                            ctx,
                                        );
                                    }
                                }
                                // OpenWarp BYOP 本地会话压缩:在 stream finished 前拿 summarization 标志
                                let summarize_overflow =
                                    response_stream.as_ref(ctx).summarization_overflow();
                                self.handle_response_stream_finished(
                                    &stream_id,
                                    finished_event,
                                    conversation_id,
                                    did_input_contain_user_query,
                                    summarize_overflow,
                                    ctx,
                                );
                            }
                            warp_multi_agent_api::response_event::Type::ClientActions(actions) => {
                                let client_actions = actions.actions;
                                let apply_result =
                                    history_model.update(ctx, |history_model, ctx| {
                                        history_model.apply_client_actions(
                                            &stream_id,
                                            client_actions,
                                            conversation_id,
                                            self.terminal_view_id,
                                            ctx,
                                        )
                                    });
                                if let Err(e) = apply_result {
                                    log::error!(
                                        "Failed to apply client actions to conversation: {e:?}"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if matches!(e.as_ref(), AIApiError::QuotaLimit) {
                            // If the error is a quota limit, we want to refresh workspace metadata
                            // So the current state of AI overages is immediately up to date.
                            TeamUpdateManager::handle(ctx).update(
                                ctx,
                                |team_update_manager, ctx| {
                                    std::mem::drop(
                                        team_update_manager.refresh_workspace_metadata(ctx),
                                    );
                                },
                            );
                            AIRequestUsageModel::handle(ctx).update(ctx, |model, ctx| {
                                model.enable_buy_credits_banner(ctx);
                            });
                        }

                        let mut renderable_error: RenderableAIError = e.as_ref().into();
                        if let RenderableAIError::Other {
                            will_attempt_resume,
                            waiting_for_network,
                            ..
                        } = &mut renderable_error
                        {
                            let should_attempt_resume = response_stream
                                .as_ref(ctx)
                                .should_resume_conversation_after_stream_finished();
                            *will_attempt_resume |= should_attempt_resume;
                            if should_attempt_resume {
                                let network_status = NetworkStatus::as_ref(ctx);
                                *waiting_for_network = !network_status.is_online();
                            }
                        }

                        history_model.update(ctx, |history_model, ctx| {
                            history_model.mark_response_stream_completed_with_error(
                                renderable_error,
                                &stream_id,
                                conversation_id,
                                self.terminal_view_id,
                                ctx,
                            );
                        });
                    }
                }
            }
            ResponseStreamEvent::AfterStreamFinished { cancellation } => {
                // Cancellations provide conversation_id (survives truncation); otherwise use dynamic lookup.
                let conversation_id = match &cancellation {
                    Some(stream_cancellation) => stream_cancellation.conversation_id,
                    None => {
                        let Some(id) = BlocklistAIHistoryModel::as_ref(ctx)
                            .conversation_for_response_stream(&stream_id)
                        else {
                            log::warn!(
                                "Could not find conversation for response stream: {stream_id:?}"
                            );
                            return;
                        };
                        id
                    }
                };

                let history_model = BlocklistAIHistoryModel::handle(ctx);
                let Some(conversation) = history_model.as_ref(ctx).conversation(&conversation_id)
                else {
                    log::warn!("Conversation not found.");
                    return;
                };
                let new_exchange_ids = conversation.new_exchange_ids_for_response(&stream_id);
                let mut was_passive_request = false;
                let mut is_any_exchange_unfinished = false;
                let mut actions_to_queue = vec![];
                // OpenWarp BYOP:收集本轮新加 message id,稍后用于在 EMPTY 分支检测
                // synthetic invalid_arguments 错误标记。**只看本轮 added** 才能避免
                // 在历史里反复命中导致 auto-resume 死循环(标记一旦持久化就永远在)。
                let mut newly_added_message_ids: std::collections::HashSet<MessageId> =
                    std::collections::HashSet::new();

                for new_exchange_id in new_exchange_ids {
                    let Some(exchange) = conversation.exchange_with_id(new_exchange_id) else {
                        log::warn!("Exchange not found.");
                        return;
                    };
                    was_passive_request |= exchange.has_passive_request();
                    is_any_exchange_unfinished |= !exchange.output_status.is_finished();
                    newly_added_message_ids.extend(exchange.added_message_ids.iter().cloned());

                    if let AIAgentOutputStatus::Finished {
                        finished_output: FinishedAIAgentOutput::Success { output },
                        ..
                    } = &exchange.output_status
                    {
                        actions_to_queue.extend(output.get().actions().cloned());
                    }
                }

                if let Some(stream_cancellation) = &cancellation {
                    // If this is a shared session, send a synthetic StreamFinished event to notify viewers
                    // of any user-initiated cancellation. We skip FollowUpSubmitted because that's an internal
                    // cancellation for continuing the conversation.
                    if FeatureFlag::AgentSharedSessions.is_enabled()
                        && !stream_cancellation
                            .reason
                            .is_follow_up_for_same_conversation()
                    {
                        self.send_cancellation_to_viewers(ctx);
                    }

                    history_model.update(ctx, |history_model, ctx| {
                        history_model.mark_response_stream_cancelled(
                            &stream_id,
                            conversation_id,
                            self.terminal_view_id,
                            stream_cancellation.reason,
                            ctx,
                        );
                    });

                    if !was_passive_request {
                        self.set_input_mode_for_cancellation(ctx);
                    }
                } else if is_any_exchange_unfinished {
                    log::warn!(
                        "generate_multi_agent_output stream ended without emitting StreamFinished event."
                    );

                    let error_message = "Request did not successfully complete";
                    history_model.update(ctx, |history_model, ctx| {
                        history_model.mark_response_stream_completed_with_error(
                            RenderableAIError::Other {
                                error_message: error_message.to_string(),
                                will_attempt_resume: false,
                                waiting_for_network: false,
                            },
                            &stream_id,
                            conversation_id,
                            self.terminal_view_id,
                            ctx,
                        );
                    });
                } else if !actions_to_queue.is_empty() {
                    log::info!(
                        "[byop-diag] queue_actions: count={} ids=[{}] conversation_id={:?}",
                        actions_to_queue.len(),
                        actions_to_queue
                            .iter()
                            .map(|a| format!("{}", a.id))
                            .collect::<Vec<_>>()
                            .join(", "),
                        conversation_id,
                    );
                    // OpenWarp:LRC tag-in 首轮自动授权 agent 工具执行。
                    //
                    // 触发条件:发起本轮请求时 active_block 处于
                    // InteractionMode::User { did_user_tag_in_agent: true }。不能用当前
                    // active_block 的 monitored metadata 兜底,否则同一 CLI subagent 会话里的
                    // 后续普通请求也会被自动确认,导致确认 UI 不显示。
                    let auto_accept_for_lrc_tag_in =
                        response_stream.as_ref(ctx).is_lrc_tag_in_request();
                    if auto_accept_for_lrc_tag_in {
                        log::info!(
                            "[byop] LRC tag-in: queue with auto-accept ({} action(s))",
                            actions_to_queue.len()
                        );
                    }
                    self.action_model.update(ctx, |action_model, ctx| {
                        action_model.queue_actions_with_options(
                            actions_to_queue,
                            conversation_id,
                            auto_accept_for_lrc_tag_in,
                            ctx,
                        );
                    });
                } else {
                    // OpenWarp BYOP:from_args 解析失败时,chat_stream 走 fallback emit
                    // carrier ToolCall(tool=None) + synthetic error ToolCallResult(result=None,
                    // server_message_data 是 invalid_arguments JSON)。两者都走 NoClientRepresentation,
                    // 不入 actions_to_queue,exchange 静默结束 → 模型永远收不到错误反馈,
                    // 用户必须手动再发消息才能让模型重试。
                    //
                    // 检测最近 ~16 条 messages 是否含 BYOP synthetic 错误标记;有的话复用
                    // line 2695+ 的 auto-resume 路径触发重发,让模型立即基于 error tool_result
                    // 修正参数重试。`can_attempt_resume_on_error=false` 防 LLM 持续输出坏 args 导致死循环。
                    // 只在本轮新加的 messages 里查找 synthetic 错误标记,避免历史持久化的
                    // 同标记反复命中导致死循环。
                    // OpenWarp BYOP:两类 synthetic ToolCallResult 需要 auto-resume
                    // (二者都不入 actions_to_queue,exchange 静默结束 → 模型卡死等结果)。
                    // 1. invalid_arguments — from_args 解析失败兜底(原始)
                    // 2. _byop_intercepted — webfetch / websearch 等本地拦截工具结果
                    //    (chat_stream::dispatch_byop_web_tool 不走 protobuf executor,
                    //     直接合成 result,没有 AIAgentAction 入队)
                    let needs_byop_local_resume = conversation.all_tasks().any(|task| {
                        task.messages().any(|msg| {
                            newly_added_message_ids.contains(&MessageId::new(msg.id.clone()))
                                && matches!(
                                    msg.message,
                                    Some(message::Message::ToolCallResult(
                                        message::ToolCallResult { result: None, .. },
                                    )),
                                )
                                && (msg
                                    .server_message_data
                                    .contains(r#""error":"invalid_arguments""#)
                                    || msg
                                        .server_message_data
                                        .contains(r#""_byop_intercepted":true"#))
                        })
                    });
                    if needs_byop_local_resume {
                        log::info!(
                            "[byop] detected synthetic local tool_result (invalid_arguments \
                             or _byop_intercepted) without queued action → schedule auto-resume. \
                             conversation_id={conversation_id:?}"
                        );
                        let network_status = NetworkStatus::handle(ctx);
                        let wait_for_online = network_status.as_ref(ctx).wait_until_online();
                        let handle = ctx.spawn(wait_for_online, move |me, _, ctx| {
                            me.pending_auto_resume_handles.remove(&conversation_id);
                            me.resume_conversation(
                                conversation_id,
                                /*can_attempt_resume_on_error*/
                                false,
                                /*is_auto_resume_after_error*/
                                true,
                                vec![],
                                ctx,
                            );
                        });
                        self.pending_auto_resume_handles
                            .insert(conversation_id, handle);
                    }
                }

                // Cancelled streams will handle pending_response_stream updates synchronously.
                if cancellation.is_none() {
                    self.in_flight_response_streams.cleanup_stream(&stream_id);

                    // Now that the stream is cleaned up, re-check for pending
                    // orchestration events that couldn't be drained earlier.
                    if FeatureFlag::Orchestration.is_enabled() {
                        self.handle_pending_events_ready(conversation_id, ctx);
                    }
                }

                // Before cleaning up the response stream, check if we should attempt to resume.
                if response_stream
                    .as_ref(ctx)
                    .should_resume_conversation_after_stream_finished()
                {
                    let network_status = NetworkStatus::handle(ctx);
                    let wait_for_online = network_status.as_ref(ctx).wait_until_online();
                    let handle = ctx.spawn(wait_for_online, move |me, _, ctx| {
                        // Clean up the pending handle now that the resume is executing.
                        me.pending_auto_resume_handles.remove(&conversation_id);
                        me.resume_conversation(
                            conversation_id,
                            // Don't allow a second resume-on-error to prevent a persistent
                            // loop.
                            /*can_attempt_resume_on_error*/
                            false,
                            /*is_auto_resume_after_error*/
                            true,
                            vec![],
                            ctx,
                        );
                    });
                    self.pending_auto_resume_handles
                        .insert(conversation_id, handle);
                }

                // Clean up the response stream tracking entry now that the stream is complete.
                history_model.update(ctx, |history_model, _| {
                    if let Some(conversation) = history_model.conversation_mut(&conversation_id) {
                        conversation.cleanup_completed_response_stream(&stream_id);
                    }
                });
                ctx.unsubscribe_from_model(response_stream);

                if self.should_refresh_available_llms_on_stream_finish {
                    self.should_refresh_available_llms_on_stream_finish = false;
                    LLMPreferences::handle(ctx).update(ctx, |llm_preferences, ctx| {
                        llm_preferences.refresh_authed_models(ctx);
                    });
                }
                ctx.emit(BlocklistAIControllerEvent::FinishedReceivingOutput {
                    stream_id,
                    conversation_id,
                });
                AIRequestUsageModel::handle(ctx).update(ctx, |request_usage_model, ctx| {
                    request_usage_model.refresh_request_usage_async(ctx);
                });

                self.maybe_refresh_ai_overages(ctx);
            }
        }
    }

    /// Sets the terminal input state after an AI request is cancelled.
    /// From the user perspective, we downgrade the level of autonomy so:
    /// * Executing a task automatically -> interactive AI input
    /// * Interactive AI input -> interactive shell input
    fn set_input_mode_for_cancellation(&mut self, ctx: &mut ModelContext<Self>) {
        // If the request was cancelled, default to shell mode with autodetection
        // enabled.
        self.input_model.update(ctx, |input_model, ctx| {
            input_model.set_input_config_for_classic_mode(
                input_model
                    .input_config()
                    .with_shell_type()
                    .unlocked_if_autodetection_enabled(false, ctx),
                ctx,
            );
        });
    }

    /// Checks if we should refresh AI overage information after an AI request completes.
    /// This is used to ensure the UI matches the state of the workspace,
    /// especially because overages are not real-time communicated to clients.
    fn maybe_refresh_ai_overages(&mut self, ctx: &mut ModelContext<Self>) {
        let workspace = UserWorkspaces::as_ref(ctx).current_workspace();
        let Some(workspace) = workspace else {
            return;
        };

        // We want to minimize the number of times we ping our backend for updated usage information;
        // doing it after every AI query finishes would be very expensive.

        // If a user is below their personal limits, then we know that they won't eat into overages,
        // so we don't need to refresh.
        let has_no_requests_remaining = !AIRequestUsageModel::as_ref(ctx).has_requests_remaining();
        // If overages aren't enabled, we're not going to reap the benefit of refreshing at all anyway.
        let are_overages_enabled = workspace.are_overages_enabled();

        if are_overages_enabled && has_no_requests_remaining {
            // Give a one second delay to ensure that Stripe has been charged and the database is completely updated,
            // before syncing new AI overages data.
            ctx.spawn(
                async move { Timer::after(Duration::from_secs(1)).await },
                |_, _, ctx| {
                    UserWorkspaces::handle(ctx).update(ctx, |user_workspaces, ctx| {
                        user_workspaces.refresh_ai_overages(ctx);
                    });
                },
            );
        }
    }

    pub(super) fn handle_response_stream_finished(
        &mut self,
        stream_id: &ResponseStreamId,
        mut finished_event: warp_multi_agent_api::response_event::StreamFinished,
        conversation_id: AIConversationId,
        did_request_contain_user_query: bool,
        summarize_overflow: Option<bool>,
        ctx: &mut ModelContext<Self>,
    ) {
        // OpenWarp BYOP 本地会话压缩:在 token_usage move 进下面 closure 前先聚合,
        // 用于 auto overflow 检查(后面 Done 分支用)。
        let aggregate_token_count: usize = finished_event
            .token_usage
            .iter()
            .map(|u| (u.total_input + u.output + u.input_cache_read + u.input_cache_write) as usize)
            .max()
            .unwrap_or(0);

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        history_model.update(ctx, |history_model, _| {
            // Update conversation cost and usage information before updating and
            // persisting the conversation.
            history_model.update_conversation_cost_and_usage_for_request(
                conversation_id,
                finished_event
                    .request_cost
                    .map(|cost| RequestCost::new(cost.exact.into())),
                finished_event.token_usage,
                finished_event.conversation_usage_metadata.take(),
                did_request_contain_user_query,
            );
        });

        let history_model = BlocklistAIHistoryModel::handle(ctx);
        match finished_event.reason {
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::Done(_)) | None => {
                // OpenWarp BYOP 本地会话压缩 - 写回 summary
                if let Some(overflow) = summarize_overflow {
                    let compaction_cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
                    history_model.update(ctx, |history_model, _ctx| {
                        if let Some(convo) = history_model.conversation_mut(&conversation_id) {
                            crate::ai::byop_compaction::commit::commit_summarization(
                                convo,
                                overflow,
                                &compaction_cfg,
                            );
                        }
                    });
                }
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_successfully(
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });

                // OpenWarp BYOP 本地会话压缩 - auto overflow 触发(对齐 opencode `processor.ts:395-403`)
                // 仅在本流不是摘要本身时检查,防止递归。
                if summarize_overflow.is_none() {
                    let aggregate_count = aggregate_token_count;
                    if aggregate_count > 0 {
                        let cfg = crate::ai::byop_compaction::CompactionConfig::from_settings(ctx);
                        let model_limit =
                            crate::ai::byop_compaction::overflow::ModelLimit::FALLBACK;
                        let counts = crate::ai::byop_compaction::overflow::TokenCounts {
                            total: aggregate_count,
                            ..Default::default()
                        };
                        if crate::ai::byop_compaction::is_overflow(&cfg, counts, model_limit) {
                            log::info!(
                                "[byop-compaction] auto overflow detected: tokens={aggregate_count} usable={}",
                                crate::ai::byop_compaction::usable(&cfg, model_limit)
                            );
                            // 通过 SlashCommandRequest::Summarize 触发(与 /compact-and 同链路);
                            // overflow=true → chat_stream 拼摘要请求时携带 overflow 标记,
                            // commit_summarization 写回时也以 overflow=true 落 state(便于 UI 区分)。
                            self.send_slash_command_request(
                                crate::ai::blocklist::controller::SlashCommandRequest::Summarize {
                                    prompt: None,
                                    overflow: true,
                                },
                                ctx,
                            );
                        }
                    }
                }
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::Other(_)) => {
                let error_message = "Response stream finished unexpectedly (with finish reason `Other`).";
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::Other {
                            error_message: error_message.to_owned(),
                            will_attempt_resume: false,
                            waiting_for_network: false,
                        },
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::ContextWindowExceeded(_)) => {
                let error_message = "Input exceeded context window limit.";
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::ContextWindowExceeded(error_message.to_owned()),
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::QuotaLimit(_)) => {
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::QuotaLimit,
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::LlmUnavailable(_)) => {
                let error_message = "The LLM is currently unavailable.";
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::Other {
                            error_message: error_message.to_owned(),
                            will_attempt_resume: false,
                            waiting_for_network: false,
                        },
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::InvalidApiKey(details)) => {
                use warp_multi_agent_api::LlmProvider;
                let is_aws_bedrock = details
                    .provider
                    .try_into()
                    .ok()
                    .is_some_and(|p: LlmProvider| p == LlmProvider::AwsBedrock);

                let error = if is_aws_bedrock {
                    RenderableAIError::AwsBedrockCredentialsExpiredOrInvalid {
                        model_name: details.model_name,
                    }
                } else {
                    let provider = details.provider.try_into().ok().and_then(|p| match p {
                        LlmProvider::Google => Some("Google"),
                        LlmProvider::Anthropic => Some("Anthropic"),
                        LlmProvider::Openai => Some("OpenAI"),
                        LlmProvider::Xai => Some("xAI"),
                        LlmProvider::Openrouter => Some("OpenRouter"),
                        LlmProvider::AwsBedrock | LlmProvider::Unknown => None,
                    });
                    RenderableAIError::InvalidApiKey {
                        provider: provider.unwrap_or("Unknown").to_string(),
                        model_name: details.model_name,
                    }
                };

                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        error,
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::InternalError(
                warp_multi_agent_api::response_event::stream_finished::InternalError{ message})) => {
                let error_message = format!(
                    "Response stream finished unexpectedly with internal error: {message}",
                );
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::Other {
                            error_message,
                            will_attempt_resume: false,
                            waiting_for_network: false,
                        },
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
            Some(warp_multi_agent_api::response_event::stream_finished::Reason::MaxTokenLimit(_)) => {
                let error_message = "Input exceeded context window limit.";
                history_model.update(ctx, |history_model, ctx| {
                    history_model.mark_response_stream_completed_with_error(
                        RenderableAIError::ContextWindowExceeded(error_message.to_owned()),
                        stream_id,
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    );
                });
            }
        }

        if finished_event.should_refresh_model_config {
            LLMPreferences::handle(ctx).update(ctx, |llm_preferences, ctx| {
                llm_preferences.refresh_authed_models(ctx);
            });
            ctx.emit(BlocklistAIControllerEvent::FreeTierLimitCheckTriggered);
        }
    }
}

impl Entity for BlocklistAIController {
    type Event = BlocklistAIControllerEvent;
}

#[derive(Clone)]
pub struct ClientIdentifiers {
    pub conversation_id: AIConversationId,
    pub client_exchange_id: AIAgentExchangeId,
    /// Not populated for restored AI blocks.
    pub response_stream_id: Option<ResponseStreamId>,
}

#[allow(clippy::too_many_arguments)]
fn input_for_query(
    query: String,
    task_id: &TaskId,
    conversation_id: AIConversationId,
    static_query_type: Option<StaticQueryType>,
    user_query_mode: UserQueryMode,
    running_command: Option<RunningCommand>,
    additional_attachments: HashMap<String, AIAgentAttachment>,
    context_model: &BlocklistAIContextModel,
    active_session: &ActiveSession,
    app: &AppContext,
) -> AIAgentInput {
    let context = input_context_for_request(
        true,
        context_model,
        active_session,
        Some(conversation_id),
        vec![],
        app,
    );
    let intended_agent = BlocklistAIHistoryModel::as_ref(app)
        .conversation(&conversation_id)
        .and_then(|c| c.get_task(task_id))
        .and_then(|task| {
            if task.is_root_task() {
                Some(warp_multi_agent_api::AgentType::Primary)
            } else if task.is_cli_subagent() {
                Some(warp_multi_agent_api::AgentType::Cli)
            } else {
                None
            }
        });
    let mut referenced_attachments = parse_context_attachments(&query, context_model, app);
    referenced_attachments.extend(additional_attachments);
    AIAgentInput::UserQuery {
        query,
        context,
        static_query_type,
        referenced_attachments,
        user_query_mode,
        running_command,
        intended_agent,
    }
}

/// Validates that tool call results have corresponding tool calls in the task context, otherwise
/// logs a warning.
fn validate_tool_call_results<'a>(
    inputs: impl Iterator<Item = &'a AIAgentInput>,
    tasks: &[Task],
    server_conversation_token: &Option<ServerConversationToken>,
) {
    // Create a mapping from tool call IDs to their task IDs
    let mut tool_call_to_task_map: HashMap<String, String> = HashMap::new();
    for task in tasks {
        for message in &task.messages {
            if let Some(message::Message::ToolCall(tool_call)) = &message.message {
                tool_call_to_task_map
                    .insert(tool_call.tool_call_id.clone(), message.task_id.clone());
            }
        }
    }

    // Check each input for tool call results and validate they have corresponding tool calls
    for input in inputs {
        if let AIAgentInput::ActionResult { result, .. } = input {
            let action_id_str = result.id.to_string();
            let server_conversation_id = server_conversation_token
                .as_ref()
                .map(|token| token.as_str())
                .unwrap_or("None");

            if !tool_call_to_task_map.contains_key(&action_id_str) {
                log::warn!(
                    "Found tool call result with ID '{action_id_str}' but no corresponding tool \
                    call in task context. Server conversation ID: '{server_conversation_id}'"
                );
            }
        }
    }
}

fn get_running_command(terminal_model: &TerminalModel) -> Option<RunningCommand> {
    let active_block = terminal_model.block_list().active_block();
    if !active_block.is_active_and_long_running() || active_block.is_agent_monitoring() {
        return None;
    }
    let is_alt_screen_active = terminal_model.is_alt_screen_active();
    Some(RunningCommand {
        block_id: active_block.id().clone(),
        command: active_block.command_to_string(),
        grid_contents: if is_alt_screen_active {
            formatted_terminal_contents_for_input(
                terminal_model.alt_screen().grid_handler(),
                None,
                CURSOR_MARKER,
            )
        } else {
            formatted_terminal_contents_for_input(
                active_block.output_grid().grid_handler(),
                // TODO(vorporeal): This is probably too large.
                Some(1000),
                CURSOR_MARKER,
            )
        },
        cursor: CURSOR_MARKER.to_owned(),
        requested_command_id: active_block.requested_command_action_id().cloned(),
        is_alt_screen_active,
    })
}

/// OpenWarp BYOP 专用:LRC tag-in / agent-monitored 场景下提取 RunningCommand。
///
/// 上游 `get_running_command` 在 `is_agent_monitoring()` 时返回 None — 因为 Warp 自家
/// 路径下 LRC 已 spawn cli subagent 后,server 端持久该状态,后续轮 client 不必重发
/// running_command。但 BYOP 直连模型无服务端持久,**每轮都要把当前 PTY grid 内容
/// 重新带给模型**(否则模型只能看到首轮 grid_contents 之后的盲区)。
///
/// 条件放宽为 `is_agent_in_control_or_tagged_in()` — 覆盖:
///   - tag-in:`InteractionMode::User { did_user_tag_in_agent: true }`(spawn 前)
///   - monitored:`InteractionMode::Agent { ... }`(spawn 后)
///
/// 提取逻辑严格对齐上游:alt-screen 时取 `terminal_model.alt_screen().grid_handler()`
/// 而不是 `active_block.output_grid()`(后者在 alt-screen 期间是空的,
/// 不要再用 `output_to_string_force_full_grid_contents()`,那条路在 nvim 等 TUI 下
/// 会得到空字符串导致 `<attached_running_command>` 块为空,模型抱怨"看不到 command_id")。
fn byop_get_running_command_for_lrc(terminal_model: &TerminalModel) -> Option<RunningCommand> {
    let active_block = terminal_model.block_list().active_block();
    if !active_block.is_active_and_long_running() {
        return None;
    }
    if !active_block.is_agent_in_control_or_tagged_in() {
        return None;
    }
    let is_alt_screen_active = terminal_model.is_alt_screen_active();
    Some(RunningCommand {
        block_id: active_block.id().clone(),
        command: active_block.command_to_string(),
        grid_contents: if is_alt_screen_active {
            formatted_terminal_contents_for_input(
                terminal_model.alt_screen().grid_handler(),
                None,
                CURSOR_MARKER,
            )
        } else {
            formatted_terminal_contents_for_input(
                active_block.output_grid().grid_handler(),
                Some(1000),
                CURSOR_MARKER,
            )
        },
        cursor: CURSOR_MARKER.to_owned(),
        requested_command_id: active_block.requested_command_action_id().cloned(),
        is_alt_screen_active,
    })
}

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

! regex failed: regex parse error:
    CompletedCompaction {
                        ^
error: unclosed counted repetition, using literal match
→ Read app\src\ai\byop_compaction\state.rs [def]
app\src\ai\byop_compaction\state.rs
 44: pub struct CompletedCompaction {
  45| pub user_msg_id: String,
  46| pub assistant_msg_id: String,
  47| /// 本次摘要覆盖的 head 区 message ids,投影普通请求时全部隐藏。
  48| #[serde(default)]
  49| pub head_message_ids: Vec<String>,
  50| /// tail 起点 message id,用于 split 验证 / debug。
  51| #[serde(default, skip_serializing_if = "Option::is_none")]
  52| pub tail_start_id: Option<String>,
 161: fn cc(uid: &str, aid: &str, auto: bool) -> CompletedCompaction {
app\src\ai\byop_compaction\commit.rs
 77: let completed = CompletedCompaction {
app\src\ai\byop_compaction\state.rs
 162: CompletedCompaction {

**Tool Call: Read file `app\src\ai\agent\api\convert_conversation.rs` (lines 294-380)**
Status: Completed

```warp/app/src/ai/agent/api/convert_conversation.rs
impl ConvertToExchanges for &api::Task {
    /// Converts a list of tasks into AIAgentExchange objects.
    ///
    /// Note: for now, we only restore messages from the root task (task with no parent).
    fn into_exchanges(self) -> Vec<AIAgentExchange> {
        let mut exchanges = Vec::new();
        let mut todo_lists: Vec<AIAgentTodoList> = Vec::new();

        // Build a map of message_id -> message for quick lookup
        let mut message_map: HashMap<&str, &api::Message> = HashMap::new();
        // Build a map of tool_call_id -> tool_call for cancelled results
        let mut tool_call_map: HashMap<String, &api::message::ToolCall> = HashMap::new();
        for message in &self.messages {
            message_map.insert(message.id.as_str(), message);
            // If this is a tool call message, add it to the tool call map
            if let Some(api::message::Message::ToolCall(tool_call)) = &message.message {
                tool_call_map.insert(tool_call.tool_call_id.clone(), tool_call);
            }
        }

        // Process messages in chronological order
        let mut current_inputs = Vec::new();
        let mut current_outputs = Vec::new();
        let mut current_message_ids = HashSet::new();
        let mut document_versions: HashMap<AIDocumentId, AIDocumentVersion> = HashMap::new();
        let mut current_request_id: Option<String> = None;

        // Almost all messages should be ingested as outputs, except for some special cases:
        // 1. User queries
        // 2. System queries, but only if they are displayed as user queries/initiate conversations, like queries:
        //   * from the new project flow (displayed as user queries)
        //   * from the clone repository flow (displayed as user queries)
        //   * from auto code diff queries (initiate new conversations)
        // 3. tool call results (as we also render these like inputs)
        for api_message in self.messages.iter() {
            let Some(message) = &api_message.message else {
                continue;
            };

            let task_id = TaskId::new(api_message.task_id.clone());
            // Check if request_id has changed - if so, create an exchange from accumulated messages
            let message_request_id = if api_message.request_id.is_empty() {
                None
            } else {
                Some(api_message.request_id.clone())
            };

            // Create exchange if request_id changed and we have accumulated messages
            if message_request_id != current_request_id
                && current_request_id.is_some()
                && (!current_inputs.is_empty() || !current_outputs.is_empty())
            {
                let is_output_tool_call_canceled = current_inputs
                    .last()
                    .and_then(|input: &AIAgentInput| input.action_result())
                    .map(|result| result.result.is_cancelled())
                    .unwrap_or(false);

                if let Some(exchange) = create_exchange_from_messages(
                    &current_inputs,
                    &current_outputs,
                    is_output_tool_call_canceled,
                    &current_message_ids,
                    &message_map,
                    current_request_id.as_deref(),
                ) {
                    exchanges.push(exchange);
                }
                current_inputs.clear();
                current_outputs.clear();
                current_message_ids.clear();
            }

            // Update current_request_id
            current_request_id = message_request_id;

            // Track this message ID for the current exchange
            current_message_ids.insert(api_message.id.clone());

            let added_message_as_exchange_input = match message {
                api::message::Message::UserQuery(user_query) => {
                    // Add user query as input
                    current_inputs.push(AIAgentInput::UserQuery {
                        query: user_query.query.clone(),
                        context: convert_input_context(user_query.context.as_ref()),
                        static_query_type: None,
                        referenced_attachments: HashMap::new(),

```


**Tool Call: Read file `app\src\ai\agent\api\convert_conversation.rs` (lines 380-470)**
Status: Completed

```warp/app/src/ai/agent/api/convert_conversation.rs
                        referenced_attachments: HashMap::new(),
                        user_query_mode: convert_user_query_mode(user_query.mode.as_ref()),
                        running_command: None,
                        intended_agent: Some(user_query.intended_agent()),
                    });
                    true
                }
                api::message::Message::SystemQuery(query) => {
                    let Some(query_type) = &query.r#type else {
                        continue;
                    };

                    match query_type {
                        api::message::system_query::Type::CreateNewProject(auto_code_diff) => {
                            current_inputs.push(AIAgentInput::UserQuery {
                                query: auto_code_diff.query.clone(),
                                context: convert_input_context(query.context.as_ref()),
                                static_query_type: None,
                                referenced_attachments: HashMap::new(),
                                user_query_mode: UserQueryMode::default(), // SystemQuery doesn't have mode field
                                running_command: None,
                                intended_agent: None,
                            });
                            true
                        }
                        api::message::system_query::Type::CloneRepository(clone_repo) => {
                            current_inputs.push(AIAgentInput::CloneRepository {
                                clone_repo_url: CloneRepositoryURL::new(clone_repo.url.clone()),
                                context: convert_input_context(query.context.as_ref()),
                            });
                            true
                        }
                        api::message::system_query::Type::AutoCodeDiff(auto_code_diff) => {
                            current_inputs.push(AIAgentInput::AutoCodeDiffQuery {
                                query: auto_code_diff.query.clone(),
                                context: convert_input_context(query.context.as_ref()),
                            });
                            true
                        }
                        api::message::system_query::Type::FetchReviewComments(fetch) => {
                            current_inputs.push(AIAgentInput::FetchReviewComments {
                                repo_path: fetch.repo_path.clone(),
                                context: convert_input_context(query.context.as_ref()),
                            });
                            true
                        }
                        // TriggerSuggestPrompt is not rendered as user input, so we don't want to include it as an input in the exchange.
                        // ResumeConversation is actually added to the task's messages as a plain UserQuery, so we don't expect to encounter it in the task's messages.
                        api::message::system_query::Type::ResumeConversation(_)
                        | api::message::system_query::Type::GeneratePassiveSuggestions(_)
                        // TODO: Implement this for real. ZB adding this to bump proto version for unrelated API changes.
                        | api::message::system_query::Type::SummarizeConversation(_)=> false,
                    }
                }
                api::message::Message::ToolCallResult(tool_call_result) => {
                    // Try to convert tool call result - returns None for ServerToolCalls
                    if let Some(input) = convert_tool_call_result_to_input(
                        &task_id,
                        tool_call_result,
                        &tool_call_map,
                        &mut document_versions,
                    ) {
                        // Add tool call result as input
                        current_inputs.push(input);
                    }

                    true
                }
                api::message::Message::UpdateTodos(update) => {
                    if let Some(operation) = &update.operation {
                        update_todo_list_from_todo_op(&mut todo_lists, operation.clone());
                    }

                    false
                }
                api::message::Message::InvokeSkill(invoke_skill) => {
                    if let Some(api_skill) = invoke_skill.skill.clone() {
                        if let Ok(parsed_skill) = ParsedSkill::try_from(api_skill) {
                            let user_query = invoke_skill
                                .user_query
                                .clone()
                                .map(|user_query| crate::ai::agent::InvokeSkillUserQuery {
                                    query: user_query.query,
                                    // Restored conversations currently do not hydrate invoke-skill
                                    // inline attachments back into client-side attachment structs.
                                    // TODO(APP-3101): support rehydration of attachments.
                                    referenced_attachments: HashMap::new(),
                                });
                            let input = AIAgentInput::InvokeSkill {
                                context: Arc::new([]),
                                skill: parsed_skill,

```


**Tool Call: Read file `app\src\ai\agent\api\convert_conversation.rs` (lines 470-570)**
Status: Completed

```warp/app/src/ai/agent/api/convert_conversation.rs
                                skill: parsed_skill,
                                user_query,
                            };
                            current_inputs.push(input);
                        };
                    };

                    true
                }
                // Preserve EventsFromAgents as an explicit input in restored conversations
                // so orchestration state (including lifecycle timestamps) survives roundtrip.
                api::message::Message::EventsFromAgents(events) => {
                    current_inputs.push(AIAgentInput::EventsFromAgents {
                        events: events.agent_events.clone(),
                    });
                    true
                }
                api::message::Message::PassiveSuggestionResult(passive_result) => {
                    if let Some(input) =
                        convert_passive_suggestion_result_to_input(passive_result)
                    {
                        current_inputs.push(input);
                    }
                    true
                }
                api::message::Message::AgentOutput(_)
                | api::message::Message::AgentReasoning(_)
                | api::message::Message::Summarization(_)
                | api::message::Message::ToolCall(_)
                | api::message::Message::ServerEvent(_)
                | api::message::Message::UpdateReviewComments(_)
                | api::message::Message::CodeReview(_)
                // TODO(advait): Handle this for restored + forked conversations w/ web searches/fetches.
                | api::message::Message::WebSearch(_)
                | api::message::Message::WebFetch(_)
                | api::message::Message::DebugOutput(_)
                | api::message::Message::ArtifactEvent(_)
                | api::message::Message::MessagesReceivedFromAgents(_)
                | api::message::Message::ModelUsed(_) => false,
            };

            if !added_message_as_exchange_input {
                if let Ok(MaybeAIAgentOutputMessage::Message(output_msg)) = (*api_message)
                    .clone()
                    .to_client_output_message(ConversionParams {
                        current_todo_list: todo_lists.last(),
                        // TODO(alokedesai): Support persistence for the code review state.
                        active_code_review: None,
                        task_id: &TaskId::new(api_message.task_id.clone()),
                    })
                {
                    current_outputs.push(output_msg);
                }
            }
        }

        // At the end, if we have remaining inputs or outputs, create the last exchange.
        if !current_inputs.is_empty() || !current_outputs.is_empty() {
            // If the last message is a tool call (i.e. we have no corresponding result)
            // we will assume the tool call was cancelled.
            let is_output_tool_call_canceled = self.messages.last().is_some_and(|message| {
                matches!(message.message, Some(api::message::Message::ToolCall(_)))
            });

            if let Some(exchange) = create_exchange_from_messages(
                &current_inputs,
                &current_outputs,
                is_output_tool_call_canceled,
                &current_message_ids,
                &message_map,
                current_request_id.as_deref(),
            ) {
                exchanges.push(exchange);
            }
        }

        exchanges
    }
}

/// Convert a ToolCallResult to an AIAgentInput::ActionResult
/// Returns None if the tool call result is a ServerToolCallResult
/// `document_versions` tracks the latest version per document for CreateDocuments and EditDocuments results.
/// Each new document (CreateDocuments) starts at the default version; edits increment the specific document's version.
#[allow(clippy::single_range_in_vec_init)]
pub(crate) fn convert_tool_call_result_to_input(
    task_id: &TaskId,
    tool_call_result: &api::message::ToolCallResult,
    tool_call_map: &HashMap<String, &api::message::ToolCall>,
    document_versions: &mut HashMap<AIDocumentId, AIDocumentVersion>,
) -> Option<AIAgentInput> {
    use warp_multi_agent_api::message::tool_call_result::Result as ToolCallResultType;

    let tool_call_id = tool_call_result.tool_call_id.clone();
    let context = convert_input_context(tool_call_result.context.as_ref());

    match tool_call_result.result.as_ref() {
        Some(ToolCallResultType::RunShellCommand(result)) => {
            // Convert RunShellCommand result to RequestCommandOutputResult
            let command_output_result = match &result.result {
                Some(api::run_shell_command_result::Result::CommandFinished(finished)) => {

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

app\src\persistence\sqlite.rs
 3579: timestamp: Some(timestamp.naive_utc()),
 3599: oldest_timestamp: Some(oldest_timestamp.naive_utc()),
 3600: latest_timestamp: Some(latest_timestamp.naive_utc()),
 3602: processed_at_timestamp: Some(latest_processed_at_timestamp.naive_utc()),
app\src\server\cloud_objects\update_manager_test.rs
 4827: processed_at_timestamp: Some(timestamp),
 4838: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(10)),
 4914: processed_at_timestamp: Some(timestamp),
 4925: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(10)),
 4983: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(1)),
 4994: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(10)),
 5005: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(12)),
 5031: processed_at_timestamp: Some(timestamp),
 5042: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(1)),
 5053: processed_at_timestamp: Some(timestamp - chrono::Duration::minutes(10)),
app\src\server\graphql\schema\util.rs
 58: processed_at_timestamp: Some(s.processed_at_timestamp.utc()),
app\src\server\telemetry_ext.rs
 30: original_timestamp: Some(self.timestamp),
 103: original_timestamp: Some(timestamp),

**Tool Call: Read file `app\src\ai\blocklist\orchestration_event_streamer_tests.rs` (lines 100-130)**
Status: Completed

```warp/app/src/ai/blocklist/orchestration_event_streamer_tests.rs
        panic!("expected lifecycle event");
    };
    assert!(matches!(
        lifecycle.detail,
        Some(api::agent_event::lifecycle_event::Detail::InProgress(..))
    ));
}

#[test]
fn ai_conversation_new_restored_preserves_last_event_sequence() {
    // Guards against regressions that drop the field when wiring the restore
    // path: a conversation restored with `last_event_sequence: Some(N)`
    // should expose it via `conversation.last_event_sequence()`.
    use crate::ai::agent::conversation::{AIConversation, AIConversationId};
    use crate::persistence::model::AgentConversationData;

    let task = api::Task {
        id: "root".to_string(),
        messages: vec![api::Message {
            id: "m1".to_string(),
            task_id: "root".to_string(),
            server_message_data: String::new(),
            citations: vec![],
            message: Some(api::message::Message::AgentOutput(
                api::message::AgentOutput {
                    text: "hi".to_string(),
                },
            )),
            request_id: String::new(),
            timestamp: None,
        }],

```


**Tool Call: Read file `app\src\ai\blocklist\orchestration_event_streamer_tests.rs` (lines 130-170)**
Status: Completed

```warp/app/src/ai/blocklist/orchestration_event_streamer_tests.rs
        }],
        dependencies: None,
        description: String::new(),
        summary: String::new(),
        server_data: String::new(),
    };
    let data = AgentConversationData {
        server_conversation_token: None,
        conversation_usage_metadata: None,
        reverted_action_ids: None,
        forked_from_server_conversation_token: None,
        artifacts_json: None,
        parent_agent_id: None,
        agent_name: None,
        parent_conversation_id: None,
        run_id: None,
        autoexecute_override: None,
        last_event_sequence: Some(42),
    };
    let conversation =
        AIConversation::new_restored(AIConversationId::new(), vec![task], Some(data))
            .expect("should restore");
    assert_eq!(conversation.last_event_sequence(), Some(42));
}

// ---- Helpers for App-based poller tests ----

fn make_ambient_task_with_children(
    children: Vec<String>,
) -> crate::ai::ambient_agents::AmbientAgentTask {
    let mut task = make_ambient_task_with_event_seq(None);
    task.children = children;
    task
}

fn make_ambient_task_with_event_seq(
    last_event_sequence: Option<i64>,
) -> crate::ai::ambient_agents::AmbientAgentTask {
    use chrono::Utc;
    crate::ai::ambient_agents::AmbientAgentTask {
        task_id: "550e8400-e29b-41d4-a716-446655440000".parse().unwrap(),

```


**Tool Call: Run MCP tool `grep`**
Status: Completed

→ Read app\src\ai\agent\api\convert_to.rs (only match)
app\src\ai\agent\api\convert_to.rs
 17: fn local_datetime_to_timestamp(timestamp: DateTime<Local>) -> prost_types::Timestamp {
  18| prost_types::Timestamp {
  19| seconds: timestamp.timestamp(),
  20| nanos: timestamp.timestamp_subsec_nanos() as i32,
  21| }
  22| }
 18: prost_types::Timestamp {
 732: api_context.current_time = Some(prost_types::Timestamp {

**Tool Call: Read file `app\src\ai\byop_compaction\tests.rs` (lines 260-620)**
Status: Completed

```warp/app/src/ai/byop_compaction/tests.rs
    }
    fn user_compaction(id: u32) -> Self {
        Self {
            id,
            role: Role::User,
            is_compaction: true,
            is_summary: false,
            size: 0,
            tools: vec![],
        }
    }
    fn assistant(id: u32, size: usize) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: false,
            size,
            tools: vec![],
        }
    }
    fn summary(id: u32) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: true,
            size: 100,
            tools: vec![],
        }
    }
    fn assistant_with_tools(id: u32, size: usize, tools: Vec<ToolOutputRef<u32>>) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: false,
            size,
            tools,
        }
    }
}

impl MessageRef for M {
    type Id = u32;
    type CallId = u32;
    fn id(&self) -> u32 {
        self.id
    }
    fn role(&self) -> Role {
        self.role
    }
    fn is_compaction_marker(&self) -> bool {
        self.is_compaction
    }
    fn is_summary(&self) -> bool {
        self.is_summary
    }
    fn estimate_size(&self) -> usize {
        self.size
    }
    fn tool_outputs(&self) -> Vec<ToolOutputRef<u32>> {
        self.tools.clone()
    }
}

fn sum_size(slice: &[M]) -> usize {
    slice.iter().map(|m| m.size).sum()
}

#[test]
fn turns_basic() {
    let msgs = vec![
        M::user(1, 10),
        M::assistant(2, 20),
        M::user(3, 10),
        M::assistant(4, 30),
        M::user(5, 10),
    ];
    let t = turns(&msgs);
    assert_eq!(t.len(), 3);
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    assert_eq!(t[1].start, 2);
    assert_eq!(t[1].end, 4);
    assert_eq!(t[2].start, 4);
    assert_eq!(t[2].end, 5);
}

#[test]
fn turns_skips_compaction_marker() {
    let msgs = vec![
        M::user(1, 10),
        M::assistant(2, 20),
        M::user_compaction(99), // 不算 turn
        M::assistant(3, 30),
        M::user(4, 10),
    ];
    let t = turns(&msgs);
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].id, 1);
    assert_eq!(t[1].id, 4);
}

#[test]
fn turns_empty() {
    let msgs: Vec<M> = vec![];
    assert_eq!(turns(&msgs).len(), 0);
}

#[test]
fn select_keeps_recent_turns_within_budget() {
    let msgs = vec![
        M::user(1, 100),
        M::assistant(2, 100), // turn1 size 200
        M::user(3, 100),
        M::assistant(4, 100), // turn2 size 200
        M::user(5, 100),
        M::assistant(6, 100), // turn3 size 200
    ];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 2;
    cfg.preserve_recent_tokens = Some(500); // 足够装下最近 2 个 turn (各 200)
    let model = ModelLimit::FALLBACK;
    let r = select(&msgs, &cfg, model, sum_size);
    // tail 起点是第 2 个 turn 的 user(idx=2),head_end=2
    assert_eq!(r.head_end, 2);
    assert_eq!(r.tail_start_id, Some(3));
}

#[test]
fn select_split_turn_when_over_budget() {
    let msgs = vec![
        M::user(1, 100),
        M::user(2, 100), // turn 2 含 5 条消息共 500
        M::assistant(3, 100),
        M::assistant(4, 100),
        M::assistant(5, 100),
        M::assistant(6, 100),
    ];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 1;
    cfg.preserve_recent_tokens = Some(250); // 装不下 turn2 整体(500),触发 splitTurn
    let model = ModelLimit::FALLBACK;
    let r = select(&msgs, &cfg, model, sum_size);
    // splitTurn 从 turn2.start+1=2 开始找,messages[2..6]=400 > 250, [3..6]=300>250, [4..6]=200<=250 → start=4
    assert_eq!(r.head_end, 4);
    assert_eq!(r.tail_start_id, Some(5));
}

#[test]
fn select_returns_full_when_no_turns() {
    let msgs: Vec<M> = vec![];
    let cfg = CompactionConfig::default();
    let r = select(&msgs, &cfg, ModelLimit::FALLBACK, sum_size);
    assert_eq!(r.head_end, 0);
    assert_eq!(r.tail_start_id, None);
}

#[test]
fn select_tail_turns_zero_keeps_full() {
    let msgs = vec![M::user(1, 100), M::assistant(2, 100)];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 0;
    let r = select(&msgs, &cfg, ModelLimit::FALLBACK, sum_size);
    assert_eq!(r.head_end, msgs.len());
    assert_eq!(r.tail_start_id, None);
}

// -- prune ---------------------------------------------------------------

fn tool_output(call_id: u32, name: &str, size: usize) -> ToolOutputRef<u32> {
    ToolOutputRef {
        call_id,
        tool_name: name.to_string(),
        output_size: size,
        completed: true,
        already_compacted: false,
    }
}

#[test]
fn prune_below_minimum_returns_empty() {
    // 只有少量 tool output,不到 PRUNE_MINIMUM(20_000)
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![tool_output(101, "bash", 5_000)]),
        M::user(3, 10),
        M::assistant_with_tools(4, 0, vec![tool_output(102, "bash", 5_000)]),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_skips_protected_skill_tool() {
    // 大 skill tool + 大 bash tool;skill 受保护永不入 prune,bash 在 PRUNE_PROTECT 内也不剪
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(
            2,
            0,
            vec![
                tool_output(101, "skill", 50_000), // skip
                tool_output(102, "bash", 30_000),
            ],
        ),
        M::user(3, 10),
        M::assistant_with_tools(4, 0, vec![tool_output(103, "bash", 30_000)]),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    // 只剪超过 PRUNE_PROTECT(40_000)之外的部分,且总剪量 > PRUNE_MINIMUM(20_000)
    // 最近 2 个 user turn 不动:turn5..end / turn3..turn5 都保留
    // 这里第 1 个 turn 含 bash 30_000(超 PROTECT),所以应被剪
    // total 累计:30_000 (bash 102) + 30_000 (bash 103,但在 turn3 受保护 skip)...
    // 注意:user_turns_seen 在遇到 user(5) 时变 1,user(3) 时变 2,继续看更早消息
    // assistant(4) 的 tools 在 turn3 内,user_turns_seen=2 时还是 continue?
    //   循环:idx=4 user(5), seen=1 → continue
    //         idx=3 assistant(4), seen=1 → continue
    //         idx=2 user(3), seen=2 → 进入处理
    //         idx=1 assistant(2), seen=2 → 处理 tools(skill skip; bash 30_000 → total=30_000 > PROTECT(40_000)? no, 30<40 → continue)
    //         idx=0 user(1), seen=3 → 处理(无 tools)
    // total=30_000,pruned=0,小于 PRUNE_MINIMUM → 返回空
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_reaches_minimum_returns_decisions() {
    // 构造足够大的 tool output 触发 prune
    let big_tool = |id: u32| tool_output(id, "bash", 25_000);
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![big_tool(101), big_tool(102), big_tool(103)]),
        M::user(3, 10),
        M::assistant(4, 0),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    // 倒序遍历:
    //  idx=4 user(5) seen=1 continue
    //  idx=3 assistant(4) seen=1 continue
    //  idx=2 user(3) seen=2 continue (no tools)
    //  idx=1 assistant(2) seen=2,tools 倒序:103 → total=25_000 < 40_000 continue;
    //                                        102 → total=50_000 > 40_000 → pruned=25_000, push (2,102)
    //                                        101 → total=75_000 > 40_000 → pruned=50_000, push (2,101)
    //  idx=0 user(1) seen=3 continue
    // pruned=50_000 > PRUNE_MINIMUM(20_000) → 返回 [(2,102),(2,101)]
    assert_eq!(r.len(), 2);
    assert!(r.contains(&(2, 102)));
    assert!(r.contains(&(2, 101)));
}

#[test]
fn prune_stops_at_summary_boundary() {
    let big_tool = |id: u32| tool_output(id, "bash", 50_000);
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![big_tool(101)]),
        M::summary(3), // boundary
        M::user(4, 10),
        M::assistant(5, 0),
        M::user(6, 10),
    ];
    let r = prune_decisions(&msgs);
    // seen=2 时遇到 summary(3) 就 break,不会处理 idx=1 的 big_tool
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_stops_at_already_compacted() {
    let mut already = tool_output(101, "bash", 50_000);
    already.already_compacted = true;
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![already, tool_output(102, "bash", 50_000)]),
        M::user(3, 10),
        M::assistant(4, 0),
        M::user(5, 10),
    ];
    // 倒序 tools:102 size=50_000 → total=50_000 > 40_000 → pruned=50_000, push (2,102)
    //              101 already_compacted → break outer
    let r = prune_decisions(&msgs);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0], (2, 102));
}

```


**Tool Call: app\\src\\ai\\byop\_compaction\\tests.rs**
Status: Completed

Diff: app\src\ai\byop_compaction\tests.rs
```
//! Phase 1 单元测试 — 覆盖纯函数(token / overflow / prompt / config / algorithm)。
//!
//! Phase 3 (state + message_view) 落地后再补 e2e 集成测试。

use super::algorithm::{prune_decisions, select, turns, MessageRef, Role, ToolOutputRef};
use super::commit::commit_summarization;
use super::config::CompactionConfig;
use crate::ai::agent::conversation::{AIConversation, AIConversationId};
use warp_multi_agent_api as api;
use super::consts::*;
use super::overflow::{is_overflow, usable, ModelLimit, TokenCounts};
use super::prompt::{build_continue_message, build_prompt, SUMMARY_TEMPLATE};
use super::token::estimate;

// -- token ---------------------------------------------------------------

#[test]
fn token_estimate_empty() {
    assert_eq!(estimate(""), 0);
}

#[test]
fn token_estimate_short() {
    // "hello world" = 11 chars → round(11/4) = 3
    assert_eq!(estimate("hello world"), 3);
}

#[test]
fn token_estimate_aligned() {
    assert_eq!(estimate(&"a".repeat(40)), 10);
    assert_eq!(estimate(&"a".repeat(41)), 10); // 41/4 = 10.25 → 10 (banker's rounding 不影响)
    assert_eq!(estimate(&"a".repeat(42)), 11); // 42/4 = 10.5 → 11
}

// -- overflow ------------------------------------------------------------

fn cfg_default() -> CompactionConfig {
    CompactionConfig::default()
}

#[test]
fn usable_with_input_limit() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    // reserved = min(20_000, 8_000) = 8_000
    // usable = max(0, 180_000 - 8_000) = 172_000
    assert_eq!(usable(&cfg, model), 172_000);
}

#[test]
fn usable_without_input_limit() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 0,
        max_output: 8_000,
    };
    // 走第二分支:context - max_output = 192_000
    assert_eq!(usable(&cfg, model), 192_000);
}

#[test]
fn usable_zero_context() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 0,
        input: 0,
        max_output: 0,
    };
    assert_eq!(usable(&cfg, model), 0);
}

#[test]
fn usable_respects_cfg_reserved_override() {
    let mut cfg = cfg_default();
    cfg.reserved = Some(50_000);
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    // reserved 覆盖为 50_000 → 180_000 - 50_000 = 130_000
    assert_eq!(usable(&cfg, model), 130_000);
}

#[test]
fn is_overflow_auto_off() {
    let mut cfg = cfg_default();
    cfg.auto = false;
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    let tokens = TokenCounts {
        total: 999_999,
        ..Default::default()
    };
    assert!(!is_overflow(&cfg, tokens, model));
}

#[test]
fn is_overflow_at_threshold() {
    let cfg = cfg_default();
    let model = ModelLimit {
        context: 200_000,
        input: 180_000,
        max_output: 8_000,
    };
    let usable_n = usable(&cfg, model);
    let tokens = TokenCounts {
        total: usable_n,
        ..Default::default()
    };
    assert!(is_overflow(&cfg, tokens, model));
    let tokens_below = TokenCounts {
        total: usable_n - 1,
        ..Default::default()
    };
    assert!(!is_overflow(&cfg, tokens_below, model));
}

#[test]
fn token_counts_count_uses_total_when_present() {
    let t = TokenCounts {
        total: 100,
        input: 50,
        output: 60,
        cache_read: 10,
        cache_write: 5,
    };
    assert_eq!(t.count(), 100); // total 优先
}

#[test]
fn token_counts_count_sums_when_total_zero() {
    let t = TokenCounts {
        total: 0,
        input: 50,
        output: 60,
        cache_read: 10,
        cache_write: 5,
    };
    assert_eq!(t.count(), 125);
}

// -- preserve_recent_budget ----------------------------------------------

#[test]
fn preserve_recent_budget_default_formula() {
    let cfg = cfg_default();
    // usable=80_000 → 80_000/4 = 20_000 → max(2_000, 20_000)=20_000 → min(8_000, 20_000) = 8_000
    assert_eq!(
        cfg.preserve_recent_budget(80_000),
        MAX_PRESERVE_RECENT_TOKENS
    );
    // usable=4_000 → 1_000 → max(2_000, 1_000)=2_000 → min(8_000, 2_000)=2_000
    assert_eq!(
        cfg.preserve_recent_budget(4_000),
        MIN_PRESERVE_RECENT_TOKENS
    );
    // usable=20_000 → 5_000 → max(2_000, 5_000)=5_000 → min(8_000, 5_000)=5_000
    assert_eq!(cfg.preserve_recent_budget(20_000), 5_000);
}

#[test]
fn preserve_recent_budget_override() {
    let mut cfg = cfg_default();
    cfg.preserve_recent_tokens = Some(12_345);
    assert_eq!(cfg.preserve_recent_budget(80_000), 12_345);
}

// -- prompt --------------------------------------------------------------

#[test]
fn summary_template_contains_all_sections() {
    let must = [
        "## Goal",
        "## Constraints & Preferences",
        "## Progress",
        "### Done",
        "### In Progress",
        "### Blocked",
        "## Key Decisions",
        "## Next Steps",
        "## Critical Context",
        "## Relevant Files",
        "Rules:",
        "<template>",
        "</template>",
    ];
    for m in must {
        assert!(SUMMARY_TEMPLATE.contains(m), "missing section: {m}");
    }
}

#[test]
fn build_prompt_no_previous() {
    let s = build_prompt(None, &[]);
    assert!(s.starts_with("Create a new anchored summary from the conversation history above."));
    assert!(s.contains(SUMMARY_TEMPLATE));
}

#[test]
fn build_prompt_with_previous() {
    let s = build_prompt(Some("OLD-SUMMARY"), &[]);
    assert!(s.starts_with("Update the anchored summary below"));
    assert!(s.contains("<previous-summary>\nOLD-SUMMARY\n</previous-summary>"));
    assert!(s.contains(SUMMARY_TEMPLATE));
}

#[test]
fn build_prompt_with_plugin_context() {
    let ctx = vec!["EXTRA1".to_string(), "EXTRA2".to_string()];
    let s = build_prompt(None, &ctx);
    assert!(s.contains("EXTRA1"));
    assert!(s.contains("EXTRA2"));
}

#[test]
fn continue_message_overflow_branch() {
    let s = build_continue_message(true);
    assert!(s.contains("previous request exceeded"));
    assert!(s.contains("Continue if you have next steps"));
}

#[test]
fn continue_message_normal_branch() {
    let s = build_continue_message(false);
    assert!(!s.contains("previous request exceeded"));
    assert!(s.starts_with("Continue if you have next steps"));
}

// -- algorithm: turns / select / prune ----------------------------------

/// 测试用 mock message 实现。
#[derive(Debug, Clone)]
struct M {
    id: u32,
    role: Role,
    /// user 消息是否带 compaction 标记
    is_compaction: bool,
    /// assistant 消息是否是摘要
    is_summary: bool,
    size: usize,
    tools: Vec<ToolOutputRef<u32>>,
}

impl M {
    fn user(id: u32, size: usize) -> Self {
        Self {
            id,
            role: Role::User,
            is_compaction: false,
            is_summary: false,
            size,
            tools: vec![],
        }
    }
    fn user_compaction(id: u32) -> Self {
        Self {
            id,
            role: Role::User,
            is_compaction: true,
            is_summary: false,
            size: 0,
            tools: vec![],
        }
    }
    fn assistant(id: u32, size: usize) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: false,
            size,
            tools: vec![],
        }
    }
    fn summary(id: u32) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: true,
            size: 100,
            tools: vec![],
        }
    }
    fn assistant_with_tools(id: u32, size: usize, tools: Vec<ToolOutputRef<u32>>) -> Self {
        Self {
            id,
            role: Role::Assistant,
            is_compaction: false,
            is_summary: false,
            size,
            tools,
        }
    }
}

impl MessageRef for M {
    type Id = u32;
    type CallId = u32;
    fn id(&self) -> u32 {
        self.id
    }
    fn role(&self) -> Role {
        self.role
    }
    fn is_compaction_marker(&self) -> bool {
        self.is_compaction
    }
    fn is_summary(&self) -> bool {
        self.is_summary
    }
    fn estimate_size(&self) -> usize {
        self.size
    }
    fn tool_outputs(&self) -> Vec<ToolOutputRef<u32>> {
        self.tools.clone()
    }
}

fn sum_size(slice: &[M]) -> usize {
    slice.iter().map(|m| m.size).sum()
}

#[test]
fn turns_basic() {
    let msgs = vec![
        M::user(1, 10),
        M::assistant(2, 20),
        M::user(3, 10),
        M::assistant(4, 30),
        M::user(5, 10),
    ];
    let t = turns(&msgs);
    assert_eq!(t.len(), 3);
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    assert_eq!(t[1].start, 2);
    assert_eq!(t[1].end, 4);
    assert_eq!(t[2].start, 4);
    assert_eq!(t[2].end, 5);
}

#[test]
fn turns_skips_compaction_marker() {
    let msgs = vec![
        M::user(1, 10),
        M::assistant(2, 20),
        M::user_compaction(99), // 不算 turn
        M::assistant(3, 30),
        M::user(4, 10),
    ];
    let t = turns(&msgs);
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].id, 1);
    assert_eq!(t[1].id, 4);
}

#[test]
fn turns_empty() {
    let msgs: Vec<M> = vec![];
    assert_eq!(turns(&msgs).len(), 0);
}

#[test]
fn select_keeps_recent_turns_within_budget() {
    let msgs = vec![
        M::user(1, 100),
        M::assistant(2, 100), // turn1 size 200
        M::user(3, 100),
        M::assistant(4, 100), // turn2 size 200
        M::user(5, 100),
        M::assistant(6, 100), // turn3 size 200
    ];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 2;
    cfg.preserve_recent_tokens = Some(500); // 足够装下最近 2 个 turn (各 200)
    let model = ModelLimit::FALLBACK;
    let r = select(&msgs, &cfg, model, sum_size);
    // tail 起点是第 2 个 turn 的 user(idx=2),head_end=2
    assert_eq!(r.head_end, 2);
    assert_eq!(r.tail_start_id, Some(3));
}

#[test]
fn select_split_turn_when_over_budget() {
    let msgs = vec![
        M::user(1, 100),
        M::user(2, 100), // turn 2 含 5 条消息共 500
        M::assistant(3, 100),
        M::assistant(4, 100),
        M::assistant(5, 100),
        M::assistant(6, 100),
    ];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 1;
    cfg.preserve_recent_tokens = Some(250); // 装不下 turn2 整体(500),触发 splitTurn
    let model = ModelLimit::FALLBACK;
    let r = select(&msgs, &cfg, model, sum_size);
    // splitTurn 从 turn2.start+1=2 开始找,messages[2..6]=400 > 250, [3..6]=300>250, [4..6]=200<=250 → start=4
    assert_eq!(r.head_end, 4);
    assert_eq!(r.tail_start_id, Some(5));
}

#[test]
fn select_returns_full_when_no_turns() {
    let msgs: Vec<M> = vec![];
    let cfg = CompactionConfig::default();
    let r = select(&msgs, &cfg, ModelLimit::FALLBACK, sum_size);
    assert_eq!(r.head_end, 0);
    assert_eq!(r.tail_start_id, None);
}

#[test]
fn select_tail_turns_zero_keeps_full() {
    let msgs = vec![M::user(1, 100), M::assistant(2, 100)];
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 0;
    let r = select(&msgs, &cfg, ModelLimit::FALLBACK, sum_size);
    assert_eq!(r.head_end, msgs.len());
    assert_eq!(r.tail_start_id, None);
}

// -- prune ---------------------------------------------------------------

fn tool_output(call_id: u32, name: &str, size: usize) -> ToolOutputRef<u32> {
    ToolOutputRef {
        call_id,
        tool_name: name.to_string(),
        output_size: size,
        completed: true,
        already_compacted: false,
    }
}

#[test]
fn prune_below_minimum_returns_empty() {
    // 只有少量 tool output,不到 PRUNE_MINIMUM(20_000)
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![tool_output(101, "bash", 5_000)]),
        M::user(3, 10),
        M::assistant_with_tools(4, 0, vec![tool_output(102, "bash", 5_000)]),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_skips_protected_skill_tool() {
    // 大 skill tool + 大 bash tool;skill 受保护永不入 prune,bash 在 PRUNE_PROTECT 内也不剪
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(
            2,
            0,
            vec![
                tool_output(101, "skill", 50_000), // skip
                tool_output(102, "bash", 30_000),
            ],
        ),
        M::user(3, 10),
        M::assistant_with_tools(4, 0, vec![tool_output(103, "bash", 30_000)]),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    // 只剪超过 PRUNE_PROTECT(40_000)之外的部分,且总剪量 > PRUNE_MINIMUM(20_000)
    // 最近 2 个 user turn 不动:turn5..end / turn3..turn5 都保留
    // 这里第 1 个 turn 含 bash 30_000(超 PROTECT),所以应被剪
    // total 累计:30_000 (bash 102) + 30_000 (bash 103,但在 turn3 受保护 skip)...
    // 注意:user_turns_seen 在遇到 user(5) 时变 1,user(3) 时变 2,继续看更早消息
    // assistant(4) 的 tools 在 turn3 内,user_turns_seen=2 时还是 continue?
    //   循环:idx=4 user(5), seen=1 → continue
    //         idx=3 assistant(4), seen=1 → continue
    //         idx=2 user(3), seen=2 → 进入处理
    //         idx=1 assistant(2), seen=2 → 处理 tools(skill skip; bash 30_000 → total=30_000 > PROTECT(40_000)? no, 30<40 → continue)
    //         idx=0 user(1), seen=3 → 处理(无 tools)
    // total=30_000,pruned=0,小于 PRUNE_MINIMUM → 返回空
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_reaches_minimum_returns_decisions() {
    // 构造足够大的 tool output 触发 prune
    let big_tool = |id: u32| tool_output(id, "bash", 25_000);
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![big_tool(101), big_tool(102), big_tool(103)]),
        M::user(3, 10),
        M::assistant(4, 0),
        M::user(5, 10),
    ];
    let r = prune_decisions(&msgs);
    // 倒序遍历:
    //  idx=4 user(5) seen=1 continue
    //  idx=3 assistant(4) seen=1 continue
    //  idx=2 user(3) seen=2 continue (no tools)
    //  idx=1 assistant(2) seen=2,tools 倒序:103 → total=25_000 < 40_000 continue;
    //                                        102 → total=50_000 > 40_000 → pruned=25_000, push (2,102)
    //                                        101 → total=75_000 > 40_000 → pruned=50_000, push (2,101)
    //  idx=0 user(1) seen=3 continue
    // pruned=50_000 > PRUNE_MINIMUM(20_000) → 返回 [(2,102),(2,101)]
    assert_eq!(r.len(), 2);
    assert!(r.contains(&(2, 102)));
    assert!(r.contains(&(2, 101)));
}

#[test]
fn prune_stops_at_summary_boundary() {
    let big_tool = |id: u32| tool_output(id, "bash", 50_000);
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![big_tool(101)]),
        M::summary(3), // boundary
        M::user(4, 10),
        M::assistant(5, 0),
        M::user(6, 10),
    ];
    let r = prune_decisions(&msgs);
    // seen=2 时遇到 summary(3) 就 break,不会处理 idx=1 的 big_tool
    assert_eq!(r.len(), 0);
}

#[test]
fn prune_stops_at_already_compacted() {
    let mut already = tool_output(101, "bash", 50_000);
    already.already_compacted = true;
    let msgs = vec![
        M::user(1, 10),
        M::assistant_with_tools(2, 0, vec![already, tool_output(102, "bash", 50_000)]),
        M::user(3, 10),
        M::assistant(4, 0),
        M::user(5, 10),
    ];
    // 倒序 tools:102 size=50_000 → total=50_000 > 40_000 → pruned=50_000, push (2,102)
    //              101 already_compacted → break outer
    let r = prune_decisions(&msgs);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0], (2, 102));
}

// -- commit --------------------------------------------------------------

fn ts(seconds: i64) -> prost_types::Timestamp {
    prost_types::Timestamp { seconds, nanos: 0 }
}

fn user_query(id: &str, task_id: &str, request_id: &str, seconds: i64) -> api::Message {
    api::Message {
        id: id.to_string(),
        task_id: task_id.to_string(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::UserQuery(api::message::UserQuery {
            query: format!("query-{id}"),
            context: None,
            mode: None,
            referenced_attachments: Default::default(),
            intended_agent: Default::default(),
        })),
        request_id: request_id.to_string(),
        timestamp: Some(ts(seconds)),
    }
}

fn agent_output(id: &str, task_id: &str, request_id: &str, seconds: i64) -> api::Message {
    api::Message {
        id: id.to_string(),
        task_id: task_id.to_string(),
        server_message_data: String::new(),
        citations: vec![],
        message: Some(api::message::Message::AgentOutput(
            api::message::AgentOutput {
                text: format!("output-{id}"),
            },
        )),
        request_id: request_id.to_string(),
        timestamp: Some(ts(seconds)),
    }
}

fn conversation_with_messages(messages: Vec<api::Message>) -> AIConversation {
    let task = api::Task {
        id: "root".to_string(),
        messages,
        dependencies: None,
        description: String::new(),
        summary: String::new(),
        server_data: String::new(),
    };
    AIConversation::new_restored(AIConversationId::new(), vec![task], None).unwrap()
}

#[test]
fn commit_summarization_records_head_message_ids() {
    let mut conversation = conversation_with_messages(vec![
        user_query("u1", "root", "r1", 1),
        agent_output("a1", "root", "r1", 2),
        user_query("u2", "root", "r2", 3),
        agent_output("a2", "root", "r2", 4),
        user_query("u3", "root", "r3", 5),
        agent_output("a3", "root", "r3", 6),
    ]);
    let mut cfg = CompactionConfig::default();
    cfg.tail_turns = 1;
    cfg.preserve_recent_tokens = Some(1_000);

    assert!(commit_summarization(&mut conversation, false, &cfg));
    let completed = conversation.compaction_state.completed().last().unwrap();
    assert_eq!(completed.user_msg_id, "u3");
    assert_eq!(completed.assistant_msg_id, "a3");
    assert_eq!(completed.tail_start_id.as_deref(), Some("u3"));
    assert_eq!(completed.head_message_ids, ["u1", "a1", "u2", "a2"]);
}

```


## User

继续
