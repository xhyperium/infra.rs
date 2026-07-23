//! redisx — 生产默认的异步 Redis 客户端。
//!
//! ## 生产入口
//!
//! - [`RedisConfig`] / [`RedisConfigBuilder`]：私有字段配置（Standalone / Cluster / Sentinel + TLS）
//! - [`RedisPool`]：双后端（`ConnectionManager` | `ClusterConnection`）+ Semaphore 背压 + `close`
//! - [`RedisClient`]：KV 扩展 + 调用级 deadline + [`contracts::KeyValueStore`]
//! - 扩展：`pipeline_set` / `eval_script` / 带 fencing 的 `lock_*`
//! - resilience：基于 resiliencx 的重试包装
//! - feature `pubsub`：`RedisPubSub` / `RedisPubSubFacade`
//!
//! ## Scaffold（可选）
//!
//! feature `scaffold` 下保留旧 `RedisAdapter` / `InMemoryRedis` / `MockRedisAdapter`，
//! **禁止**当作生产 Redis。
//!
//! ## 兼容
//!
//! [`RedisLiveKv`] 是 [`RedisClient`] 的类型别名（旧 live 验证入口）。

#![forbid(unsafe_code)]

mod client;
mod config;
mod error_map;
mod ext;
mod pool;
mod resilience;

pub use client::RedisClient;
pub use config::{RedisConfig, RedisConfigBuilder, RedisMode};
pub use error_map::{map_redis_error, map_redis_result};
pub use ext::RedisLock;
pub use pool::{RedisPool, RedisPoolStats};
pub use resilience::{
    RedisAtomicity, RedisOperation, RedisRetryConfig, RedisRetrySafety, with_budget,
    with_budget_async, with_budget_async_noop, with_budget_async_safe, with_budget_async_safe_noop,
    with_budget_noop, with_budget_safe, with_budget_safe_noop, with_retry_async,
    with_retry_async_no_wait, with_retry_sync,
};

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
        let _ = RedisMode::Cluster;
        let _ = RedisMode::Sentinel;

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
        assert_type::<RedisOperation>();
        assert_type::<RedisRetrySafety>();
        assert_type::<RedisAtomicity>();
        assert!(RedisOperation::Get.allows_automatic_retry());
        assert!(!RedisOperation::Set.allows_automatic_retry());
        let _ = map_redis_error;
        let cfg = RedisRetryConfig::fixed(1, 0);
        let v = with_retry_sync(&cfg, "surface", || Ok(1_i32)).expect("retry");
        assert_eq!(v, 1);
    }
}
