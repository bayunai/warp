pub mod util;

// OpenWarp Wave 2-1:原有两个服务端 update result → 本地对象 update result
// 的 TryFrom impl 只服务云对象 RPC GraphQL 路径。该路径下线后 impl 变 dead code,
// 一并物理删除以拿走对 `warp_graphql::mutations::update_generic_string_object`
// 与 `warp_graphql::object::ObjectUpdateSuccess` 的依赖,使 21 个 mutation 文件
// 可以物理移除。`util` 子模块的 GraphQL ↔ 本地类型转换仍被本地化路径以外的
// 残余消费方(actions 解析等)间接引用,保留待 Wave 3 一起清理。
