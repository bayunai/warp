# LSP Agent Tool 化调研

> 调研日期: 2026-05-05
> 触发: 用户反馈"openwarp 模型说没 web/LSP 工具",定位发现 web 是提示词漏改 + gating,LSP 是真没注册成 agent tool。本文评估把 LSP 暴露成 BYOP agent tool 的可行性与工程量。
> 状态: 待落地。修改方案见 [#五、推荐首期范围](#五推荐首期范围--开发顺序)

---

## 一、现状摘要

openWarp 内置 LSP 客户端(`crates/lsp/`),仅服务**编辑器 UI**:

- 鼠标 hover → `app/src/code/language_server_extension.rs::LspHoverState` 渲染浮窗
- 编辑器右下 footer → 服务器状态/手动启停(`app/src/code/footer.rs`)
- 编辑时 didOpen/didChange 同步;collect server-push 的 `publishDiagnostics`

**模型层(BYOP agent)无任何 LSP 工具可调**。`app/src/ai/agent_providers/tools/mod.rs::REGISTRY` 没有 `lsp_*` descriptor,system prompt 旧的 `tool_aliases.j2` 还把 `LSP` 列在 "unavailable" 黑名单(已在本次提示词动态化中删除黑名单段)。

## 二、`crates/lsp/` 架构层级

自底向上:

| 层 | 文件 | 职责 |
|---|---|---|
| 传输 | `transport.rs::ProcessTransport` | stdio 子进程 + JSON-RPC `Content-Length` 框架 |
| 协议 | `service.rs::LspService` | 高级 async API(definition / hover / references / format + 文档同步) |
| 实例 | `model.rs::LspServerModel` | warpui Entity,持有 `LspState::Available { service: Arc<LspService> }`,缓存 `diagnostics_by_path: HashMap<PathBuf, DocumentDiagnostics>` |
| 全局 | `manager.rs::LspManagerModel` | **SingletonEntity**,`workspace_root → Vec<ModelHandle<LspServerModel>>`,`server_for_path()` 路由,`external_file_servers` 处理跳出 workspace 的定义跳转 |
| 集成 | `app/src/code/language_server_extension.rs` | 编辑器 UI(目前唯一消费方) |

入口 `crates/lsp/src/lib.rs:134-136`:
```rust
pub fn init(app: &mut AppContext) {
    app.add_singleton_model(|_| LspManagerModel::new());
}
```

## 三、支持的语言

`crates/lsp/src/supported_servers.rs:39-45`:

| 服务器 | 语言 | 自动安装 |
|---|---|---|
| `RustAnalyzer` | Rust | ✅ data_dir |
| `GoPls` | Go | ❌ 系统 PATH |
| `Pyright` | Python | ✅ data_dir(node + JS) |
| `TypeScriptLanguageServer` | TS / TSX / JS / JSX | ✅ data_dir |
| `Clangd` | C / C++ | ✅ data_dir |

Windows ConPTY 启动闪 cmd 漏洞已修(`crates/lsp/src/servers/rust.rs::find_installed_binary_in_data_dir`)。

## 四、`LspService` 已有 async 方法

`crates/lsp/src/service.rs` 实测公共 API:

| 方法 | LSP 请求 | 返回 |
|---|---|---|
| `definition(path, position)` | `textDocument/definition` | `Vec<LspDefinitionLocation>` |
| `hover(path, position)` | `textDocument/hover` | `Option<HoverResult>` |
| `references(path, position)` | `textDocument/references` | `Vec<ReferenceLocation>`(默认 `include_declaration: true`) |
| `format(path, options)` | `textDocument/formatting` | `Option<Vec<TextEdit>>` |
| `did_open` / `did_change` / `did_close` | 文档同步通知 | — |

**未暴露,需在 `service.rs` 加 wrapper(每个约 30-50 行)**:
- `documentSymbol` / `workspaceSymbol`
- `goToImplementation`
- `callHierarchy`(prepare → incoming/outgoing,两步交互)

**diagnostics 不需要 LSP 请求** — server push 来的 `textDocument/publishDiagnostics` 已经被 `LspServerModel::diagnostics_by_path` 缓存,**直接读 model 即可**。

## 五、推荐首期范围 + 开发顺序

### Phase A:覆盖 80% 用例

工程量约等于 webfetch tool 一套(参考 `app/src/ai/agent_providers/tools/webfetch.rs` + `web_runtime.rs`)。

- `lsp_definition` — `LspService::definition` 现成
- `lsp_references` — `LspService::references` 现成
- `lsp_hover` — `LspService::hover` 现成
- `lsp_diagnostics` — 读 `LspServerModel::diagnostics_by_path`,不需要新请求

### Phase B:符号检索

需先扩 `LspService` 增加 wrapper。

- `lsp_document_symbol`
- `lsp_workspace_symbol`
- `lsp_implementation`

### Phase C:可能不做

- `lsp_call_hierarchy` — opencode / Claude Code 用得很少,优先级最低,result 类型嵌套深,UI 渲染另想

### 不建议的设计

**不要复制 Claude Code 的 `LSP(operation, file, line, char)` 单工具多操作 schema**。BYOP 下每个 operation 一个 tool 更清晰:模型选择负担小、schema 校验严、UI 卡片可分别定制。

## 六、开工前必须解决的坑

### 6.1 路由策略

`LspManagerModel::server_for_path(path)` 在该语言的 server 没启动时返回 None。

| 选项 | 体感 | 推荐度 |
|---|---|---|
| A. fail fast 返 `"no LSP for this file type / not started"` | 模型立即知道,转 grep | ⭐⭐⭐ |
| B. auto-start 触发启动 | 启动秒级 ~ 分钟级,模型卡住,LSP 索引慢的话更糟 | ❌ |

推荐 A + 一个独立 `lsp_status` tool 让模型自查"哪些语言可用"。

### 6.2 didOpen 时序

LSP spec 规定**必须先 didOpen 才能 query**。但编辑器 UI 的 didOpen 跟着 view 走,**agent 给的 path 可能不是当前打开的 view**。

| 方案 | 说明 |
|---|---|
| executor 临时 didOpen + query + didClose | 通用,但 LSP 服务器内部缓存可能被反复构建/丢弃,影响后续命中率 |
| executor didOpen 后保留 | 跟编辑器 didOpen 对齐,LSP 索引复用,**推荐** |
| 强制要求模型只问当前 view | 太严苛,模型会绕道 grep |

### 6.3 Position 单位

LSP 标准:`Position` 是 **0-based, UTF-16 code unit**。跟模型/人类直觉(1-based char)不同。

**给模型的 schema 推荐**:`line: integer (1-based)`、`character: integer (1-based char)`。executor 内部:
- `line - 1` → LSP line
- 把行内 char offset 转 UTF-16 code unit:Rust 用 `&line_text[..char_offset].encode_utf16().count()`

### 6.4 跨线程桥接

`LspManagerModel` 是 `SingletonEntity`,只能在 **main thread `AppContext`** 上下文里读取。BYOP tool executor 走 tokio 异步路径,需要桥到 main thread 拿 `Arc<LspService>` 后再下到 tokio 等 await。

参考实现:`app/src/ai/agent_providers/tools/web_runtime.rs`(webfetch / websearch 同样不在 main thread 但能调 reqwest)。
跟踪 BYOP tool executor 现有的 `app.send_action` / `executor.spawn_local` 模式,先确认现成 warp tool(read_files / grep)是怎么从 tokio 拿到 model handle 的。

## 七、命名建议(对齐 opencode 风格,避免与 Claude Code 同名冲淆)

| Phase | tool name | 说明 |
|---|---|---|
| A | `lsp_definition` | goToDefinition,接受 `file_path` + `line` + `character` |
| A | `lsp_references` | findReferences,同上签名,`include_declaration: bool = true` |
| A | `lsp_hover` | hover,同上签名 |
| A | `lsp_diagnostics` | 读缓存,签名 `file_path` 即可 |
| A | `lsp_status` | 列当前已启动的 server 类型 + workspace,给模型自查用 |
| B | `lsp_document_symbol` | 一个文件全部符号 |
| B | `lsp_workspace_symbol` | 跨文件模糊符号搜索,签名 `query: string` |
| B | `lsp_implementation` | trait 方法 → 实现类 |

## 八、何时落地

- 当前(2026-05-05)BYOP 提示词工具列表已动态化(本次改动覆盖 webfetch/websearch);
- LSP 单独 PR 落地,**不阻塞**当前提示词修复 commit;
- 优先级建议:Phase A 在 BYOP 语义稳定后排上日程,实测能省多少 grep 流量再决定是否做 Phase B。

---

## 附:文件位置索引

- `crates/lsp/src/lib.rs:62-123` — `spawn_lsp_service` 启动入口
- `crates/lsp/src/lib.rs:134-136` — `init` 注册 SingletonEntity
- `crates/lsp/src/service.rs:598-740` — definition / hover / references / format
- `crates/lsp/src/model.rs:101-111` — `LspServerModel` 字段(含 `diagnostics_by_path`)
- `crates/lsp/src/manager.rs:31-100` — `LspManagerModel` 路由 API
- `crates/lsp/src/supported_servers.rs:39-45` — `LSPServerType` 5 种
- `app/src/ai/agent_providers/tools/webfetch.rs` — BYOP-only tool descriptor 模板
- `app/src/ai/agent_providers/tools/web_runtime.rs` — 跨线程异步执行参考
- `app/src/ai/agent_providers/tools/mod.rs:76-104` — `REGISTRY` 注册位置
- `app/src/ai/agent_providers/chat_stream.rs::available_tool_names` — 与 `build_tools_array` 共享 gating,新增 lsp 工具的开关条件加在这里
