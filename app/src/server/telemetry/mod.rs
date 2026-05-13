// OpenWarp(Wave5-2)：TelemetryApi 隔层物理删。
//
// 历史职责：构造 Rudderstack 批次 + 序列化到磁盘 + 以 http_client 回传 Warp Inc 后端。
// 经 P2 send 网络层(commit 60e37e160)、Wave1-S1..S4 voice/collector/mod_tests 拆除,
// 以及本次 Wave5-2 进一步:
// - 删除 `TelemetryApi` struct 物理本体(连同 `flush_events` / `flush_persisted_events_to_rudder` /
//   `flush_and_persist_events` / `send_telemetry_event` 所有 no-op 方法)
// - 该隔层原被 `ServerApi.telemetry_api: TelemetryApi` 字段透传调用，现
//   `ServerApi.send_telemetry_event` 直接折成本体 no-op，字段与隔层一同删
// - `flush_telemetry_events` / `flush_persisted_events_to_rudder` / `flush_and_persist_events`
//   在 ServerApi 层上也 0 外部消费 — 同步删除。
//
// `TelemetryEvent` 枚举(events.rs 6841 行)与 146 文件外部 `import` 保持不动 —
// macros.rs 仍做轻量类型检查以防枚举 variant 被当成 dead。

pub mod context_provider;
mod events;
mod macros;

pub use events::*;

use serde_json::{json, Value};
use std::sync::OnceLock;

/// OpenWarp(Wave1-S4)：原 `context.rs` 已删,但 session_sharing 协议 `InitPayload`
/// 仍含 `TelemetryContext(Value)` 字段(viewer/sharer 之间的协议字段)。这里保留
/// 一个最小 stub：返回一个永远为空 JSON 对象的 `TelemetryContext`，以维持协议兼容。
pub struct TelemetryContext(Value);

impl TelemetryContext {
    pub fn as_value(&self) -> Value {
        self.0.clone()
    }
}

static TELEMETRY_CONTEXT: OnceLock<TelemetryContext> = OnceLock::new();

/// OpenWarp(Wave1-S4)：空 telemetry context。原实现会序列化 OS / 用户 agent 等
/// 信息附在 Rudderstack 事件上；现在 Rudderstack 路径已死，仅 session_sharing
/// 协议字段消费，返回空对象即可。
pub fn telemetry_context() -> &'static TelemetryContext {
    TELEMETRY_CONTEXT.get_or_init(|| TelemetryContext(json!({})))
}
