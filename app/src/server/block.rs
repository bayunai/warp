// OpenWarp Wave 6-8:历史上本文件承载 `share_block` 云端 RPC 的本地化数据传输类型
// (`Block` struct + `Block::new` / `embed_pixel_height` / 一组 pixel 常量),配合
// `BlockClient` trait 把终端 block 序列化推送到 warp.dev。
//
// Wave 6-8 物理删 `BlockClient` trait + `ShowBlocksView` 设置页后,`Block` struct
// 与 impl 全部 0 消费,随之整删,只保留 `DisplaySetting` enum —— 后者仍被本地
// telemetry 事件载荷 / terminal model `embed_pixel_height` 等纯本地路径消费。
use serde::{Deserialize, Serialize};

/// 表示分享 block 时希望嵌入哪些区段。
/// OpenWarp:历史上是 `share_block::DisplaySetting` GraphQL 输入类型的本地映射,云端
/// share block 路径已下线。该枚举仍被 telemetry 事件载荷 + 终端模型 + 设置页 UI
/// 等纯本地路径消费,故保留。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DisplaySetting {
    Command,
    Output,
    CommandAndOutput,
    Other(String),
}
