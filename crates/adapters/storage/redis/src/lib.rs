//! redisx — Redis 存储适配。
//!
//! - 默认：[`RedisAdapter`] 进程内 scaffold（**非**生产客户端）。
//! - feature `live`：[`RedisLiveKv`] 真实 redis 客户端（infra-s9t.2 验证入口）。

#![forbid(unsafe_code)]

mod adapter;
pub use adapter::RedisAdapter;

#[cfg(feature = "live")]
mod live;
#[cfg(feature = "live")]
pub use live::RedisLiveKv;
