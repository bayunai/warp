// OpenWarp(本地化,Phase 5):`TeamUpdateManager` 原本负责轮询 GraphQL 拉云端 team /
// workspace 元数据,本地化场景下无云端 team / workspace 概念,**整文件改为本地空 model**:
// - 所有公共方法保留签名兼容(被 `agent_sdk/common.rs::refresh_workspace_metadata` /
//   `blocklist/controller.rs` / `drive/index.rs` / `root_view.rs` / `auth/mod.rs::log_out` 调用)
// - 所有 RPC 调用、轮询、持久化路径删除,内部 no-op
// - `refresh_workspace_metadata` 返回立即完成的 oneshot Receiver,使 `agent_sdk` 通过 `try_join`
//   等待 setup 完成的链路直接 ready
// - `team_client` 字段删除(与 Phase 5 server_api 物理删配套)

use super::user_workspaces::UserWorkspaces;
use super::workspace::WorkspaceUid;
use crate::cloud_object::CloudObjectEventEntrypoint;
use crate::persistence::ModelEvent;
use crate::server::ids::ServerId;
use futures::channel::oneshot::{self, Receiver};
use std::sync::mpsc::SyncSender;
use warpui::{Entity, ModelContext, SingletonEntity};

pub enum TeamUpdateManagerEvent {
    LeaveSuccess,
    LeaveError,
    RenameTeamSuccess,
    RenameTeamError,
}

/// OpenWarp 本地化版本:`TeamUpdateManager` 退化为本地空 model,保留方法签名兼容性。
/// 本体不再创建 team / 拉取 workspace 元数据 / 触发任何网络请求。
pub struct TeamUpdateManager {
    model_event_sender: Option<SyncSender<ModelEvent>>,
}

impl TeamUpdateManager {
    pub fn new(
        model_event_sender: Option<SyncSender<ModelEvent>>,
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Self { model_event_sender }
    }

    #[cfg(test)]
    pub fn mock(_ctx: &mut ModelContext<Self>) -> Self {
        Self {
            model_event_sender: None,
        }
    }

    /// OpenWarp(本地化):无云端 polling,no-op。
    pub fn start_polling_for_workspace_metadata_updates(&mut self, _ctx: &mut ModelContext<Self>) {}

    /// OpenWarp(本地化):无云端 polling,no-op。
    pub fn stop_polling_for_workspace_metadata_updates(&mut self) {}

    /// OpenWarp(本地化):本地无云端 workspace 元数据,返回立即完成的 oneshot。
    /// `agent_sdk/common.rs::refresh_workspace_metadata` 通过 `try_join` 等 setup 完成,这里
    /// 直接 ready 让其继续。
    pub fn refresh_workspace_metadata(&mut self, _ctx: &mut ModelContext<Self>) -> Receiver<()> {
        let (tx, rx) = oneshot::channel::<()>();
        let _ = tx.send(());
        rx
    }

    /// OpenWarp(本地化):本地无云端 team 创建，不触发任何外部请求。
    pub fn create_team(
        &mut self,
        team_name: String,
        entrypoint: CloudObjectEventEntrypoint,
        discoverable: Option<bool>,
        _ctx: &mut ModelContext<Self>,
    ) {
        let _ = (team_name, entrypoint, discoverable);
    }

    /// OpenWarp(本地化):本地无云端 team,发 LeaveError 让 UI 不卡。
    pub fn leave_team(
        &mut self,
        team_uid: ServerId,
        entrypoint: CloudObjectEventEntrypoint,
        ctx: &mut ModelContext<Self>,
    ) {
        let _ = (team_uid, entrypoint);
        ctx.emit(TeamUpdateManagerEvent::LeaveError);
    }

    /// OpenWarp(本地化):本地无云端 team rename,no-op。
    pub fn rename_team(&mut self, new_name: String, ctx: &mut ModelContext<Self>) {
        let _ = new_name;
        ctx.emit(TeamUpdateManagerEvent::RenameTeamError);
    }

    /// OpenWarp(本地化):仅更新本地 UserWorkspaces 状态 + 写本地 sqlite。
    pub fn set_current_workspace_uid(
        &mut self,
        workspace_uid: WorkspaceUid,
        ctx: &mut ModelContext<Self>,
    ) {
        UserWorkspaces::handle(ctx).update(ctx, |user_workspaces, ctx| {
            user_workspaces.set_current_workspace_uid(workspace_uid, ctx);
        });

        // 本地 sqlite 写入仍保留(`current_workspace` 字段本地存储)
        if let Some(sender) = &self.model_event_sender {
            let _ = sender.send(ModelEvent::SetCurrentWorkspace { workspace_uid });
        }
    }
}

impl Entity for TeamUpdateManager {
    type Event = TeamUpdateManagerEvent;
}

impl SingletonEntity for TeamUpdateManager {}

// OpenWarp(本地化,Phase 5):`update_manager_tests.rs` 是 RPC polling 测试,全部不可达,物理删除。
