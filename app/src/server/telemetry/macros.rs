// OpenWarp(Wave1-S4):telemetry 宏的类型检查 no-op 版本。
// 原 `send_telemetry_*` 宏会向 Rudderstack/Warp Inc 后端排队或同步发送 telemetry event。
// OpenWarp 不需要任何外部 telemetry,但仍保留宏签名以避免修改 146+ 调用点。
//
// 宏体只做"借用 + 类型检查":
// 1. 调用 `TelemetryEvent::name(&$event)` 形式约束 `$event` 实参确实实现了
//    `warp_core::telemetry::TelemetryEvent` trait,维持 `TelemetryEvent` 枚举的
//    完整使用义务(防止编译器把它当作 dead variants 优化掉、也防止调用点把不
//    相干的值塞进来)。
//    注:`TelemetryEvent::event_descs() -> impl Iterator` 非 object-safe,因此
//    不能用 `&dyn TelemetryEvent` 形式,改用 trait 方法调用形式做 bound check。
// 2. `let _ = &$ctx;` 抑制 unused_variables 警告。
//
// 运行时:无任何 I/O、无任何分配、无 future 创建。`name()` 返回 `&'static str`
// 编译期可被优化掉,实际执行只有一次借用 + 丢弃。
//
// 配套:`warp_core::telemetry::send_telemetry_from_{ctx,app_ctx}` 仍存在,
// 经由 `record_event` no-op 也不会真正外发,见 `server/telemetry/mod.rs` 注释。

#[macro_export]
macro_rules! send_telemetry_sync_from_ctx {
    ($event:expr, $ctx:expr) => {{
        // OpenWarp:类型检查 + 借用消费,运行时无副作用。
        let _ = warp_core::telemetry::TelemetryEvent::name(&$event);
        let _ = &$ctx;
    }};
}

#[macro_export]
macro_rules! send_telemetry_sync_from_app_ctx {
    ($event:expr, $app_ctx:expr) => {{
        // OpenWarp:类型检查 + 借用消费,运行时无副作用。
        let _ = warp_core::telemetry::TelemetryEvent::name(&$event);
        let _ = &$app_ctx;
    }};
}

#[macro_export]
macro_rules! send_telemetry_on_executor {
    ($auth_state:expr, $event:expr, $executor:expr) => {{
        // OpenWarp:类型检查 + 借用消费,运行时无副作用。
        let _ = warp_core::telemetry::TelemetryEvent::name(&$event);
        let _ = &$auth_state;
        let _ = &$executor;
    }};
}
