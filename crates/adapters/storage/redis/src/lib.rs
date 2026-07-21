//! `redisx` — Redis adapter。
//!
//! - scaffold：[`RedisAdapter`]（忽略 TTL 的内存 KV）
//! - mock 验证入口：[`MockRedisAdapter`]（TTL 模拟 + 单调 PubSub id；**非**真实 Redis）

mod adapter;
mod mock;

pub use adapter::RedisAdapter;
pub use mock::MockRedisAdapter;
