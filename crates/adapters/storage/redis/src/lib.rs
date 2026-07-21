//! redisx — 生产默认的异步 Redis 客户端。
//!
//! ## 生产入口
//!
//! - [`RedisConfig`] / [`RedisConfigBuilder`]：私有字段配置
//! - [`RedisPool`]：`ConnectionManager` + Semaphore 背压 + `close`
//! - [`RedisClient`]：KV 扩展 + [`contracts::KeyValueStore`]
//! - feature `pubsub`：[`RedisPubSub`] / [`RedisPubSubFacade`]
//!
//! ## Scaffold（可选）
//!
//! feature `scaffold` 下保留旧 [`RedisAdapter`] / [`InMemoryRedis`] / [`MockRedisAdapter`]，
//! **禁止**当作生产 Redis。
//!
//! ## 兼容
//!
//! [`RedisLiveKv`] 是 [`RedisClient`] 的类型别名（旧 live 验证入口）。

#![forbid(unsafe_code)]

mod client;
mod config;
mod error_map;
mod pool;

pub use client::RedisClient;
pub use config::{RedisConfig, RedisConfigBuilder, RedisMode};
pub use error_map::{map_redis_error, map_redis_result};
pub use pool::{RedisPool, RedisPoolStats};

/// 兼容旧名称：真实 Redis KV 客户端。
pub type RedisLiveKv = RedisClient;

#[cfg(feature = "pubsub")]
mod pubsub;
#[cfg(feature = "pubsub")]
pub use pubsub::{RedisPubSub, RedisPubSubFacade};

#[cfg(feature = "scaffold")]
mod scaffold;
#[cfg(feature = "scaffold")]
pub use scaffold::{InMemoryRedis, MockRedisAdapter, RedisAdapter};
