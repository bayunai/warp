//! Telemetry events for the in-app notification mailbox / toast stack.
//!
//! 这是 002ce467 cloud-removal 时一并删掉的 `AgentManagementTelemetryEvent` 的最小裁剪版,
//! 仅保留通知中心(`item_rendering.rs`)实际仍在用的 variant —— artifact 点击事件 +
//! tombstone 已经不存在但保留 schema 以维持向后兼容/未来重建。

use serde::Serialize;
use serde_json::json;
use strum_macros::{EnumDiscriminants, EnumIter};
use warp_core::telemetry::{EnablementState, TelemetryEvent, TelemetryEventDesc};

/// 通知 artifact 类型(用于 telemetry)。
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Plan,
    Branch,
    PullRequest,
    File,
}

/// 通知中心相关的 telemetry 事件。
#[derive(Serialize, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum NotificationsTelemetryEvent {
    /// 用户在通知项里点击了 artifact 按钮(plan / branch / PR / file)
    ArtifactClicked { artifact_type: ArtifactType },
}

impl TelemetryEvent for NotificationsTelemetryEvent {
    fn name(&self) -> &'static str {
        NotificationsTelemetryEventDiscriminants::from(self).name()
    }

    fn payload(&self) -> Option<serde_json::Value> {
        match self {
            NotificationsTelemetryEvent::ArtifactClicked { artifact_type } => {
                Some(json!({ "artifact_type": artifact_type }))
            }
        }
    }

    fn description(&self) -> &'static str {
        NotificationsTelemetryEventDiscriminants::from(self).description()
    }

    fn enablement_state(&self) -> EnablementState {
        NotificationsTelemetryEventDiscriminants::from(self).enablement_state()
    }

    fn contains_ugc(&self) -> bool {
        false
    }

    fn event_descs() -> impl Iterator<Item = Box<dyn TelemetryEventDesc>> {
        warp_core::telemetry::enum_events::<Self>()
    }
}

impl TelemetryEventDesc for NotificationsTelemetryEventDiscriminants {
    fn name(&self) -> &'static str {
        match self {
            Self::ArtifactClicked => "Notifications.ArtifactClicked",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ArtifactClicked => "User clicked an artifact button in a notification",
        }
    }

    fn enablement_state(&self) -> EnablementState {
        EnablementState::Always
    }
}

warp_core::register_telemetry_event!(NotificationsTelemetryEvent);
