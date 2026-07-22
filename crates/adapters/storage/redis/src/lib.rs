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

#[cfg(test)]
mod public_api_surface {
    use super::*;

    /// 默认 feature crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        let builder: RedisConfigBuilder = RedisConfig::builder();
        let cfg: RedisConfig = builder.build().expect("cfg");
        let _dbg = format!("{cfg:?}");
        let _mode = RedisMode::Standalone;

        let ok: kernel::XResult<i32> = map_redis_result(Ok(7));
        assert_eq!(ok.unwrap(), 7);

        let stats = RedisPoolStats { open: 0, in_flight: 0, waiters: 0 };
        assert_eq!(stats.open, 0);

        fn assert_type<T: ?Sized>() {}
        assert_type::<RedisClient>();
        assert_type::<RedisPool>();
        assert_type::<RedisLiveKv>();
        assert_type::<RedisConfig>();
        assert_type::<RedisConfigBuilder>();
        assert_type::<RedisPoolStats>();
        let _ = map_redis_error;
    }
}
