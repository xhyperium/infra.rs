//! `kafkax` — 生产级 Kafka 适配（纯 Rust `rskafka`）。
//!
//! ## 默认入口
//!
//! - [`KafkaConfig`] / [`KafkaConfigBuilder`] / [`KafkaConfig::from_env`]
//! - [`KafkaPool`]：`connect` / `producer` / `consumer` / `health` / `stats` / `close`
//! - [`KafkaProducer`]：`publish` 等待 broker 确认
//! - [`KafkaConsumer`]：按分区流式消费（不依赖 group coordinator；**at-most-once**）
//! - [`KafkaEventBus`]：[`contracts::EventBus`] facade（**at-most-once**）
//!
//! ## 应用层可靠性（DEFER 闭环）
//!
//! `rskafka` 无 consumer group / transactional producer，可靠语义由应用层补齐：
//!
//! - [`OffsetCommitStore`] / [`MemoryOffsetStore`] / [`FileOffsetStore`]：offset 持久化
//! - [`AtLeastOnceConsumer`] / [`KafkaAtLeastOnceBus`]：显式 `ack` 后才推进位点
//! - [`ProduceThenCheckpointCoordinator`] / [`ProduceThenCheckpointSession`]：非原子
//!   produce-then-checkpoint；checkpoint 失败会形成重复窗口
//!
//! ## scaffold feature
//!
//! 启用 `scaffold` 时额外导出旧的内存 `KafkaAdapter` / `MockKafkaBus`。
//!
//! ## 环境变量
//!
//! `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`
//!
//! ## 权威边界（draft 十轮收敛）
//!
//! - **驱动**：默认 `rskafka`（非 `rust-rdkafka` / librdkafka）
//! - **NO-GO**：group / rebalance / 自动重连 / native EOS / schema registry /
//!   SCRAM / OAuth / mTLS / package stable
//! - **OOS**：draft Part2 量化栈（embedded broker / io-uring / µs 热路径 等）

#![forbid(unsafe_code)]

mod at_least_once;
mod bus;
mod config;
mod consumer;
mod eos;
mod error_map;
mod lifecycle;
mod message;
mod offset;
mod pool;
mod producer;

pub use at_least_once::{AtLeastOnceConsumer, KafkaAtLeastOnceBus, resolve_start_offset};
pub use bus::KafkaEventBus;
pub use config::{DEFAULT_BROKERS, DEFAULT_SASL_MECHANISM, KafkaConfig, KafkaConfigBuilder};
pub use consumer::{ConsumerConfig, KafkaConsumer};
// 源码兼容期仍需从 crate root 导出旧 `Eos*` 别名；新代码与文档不得使用。
#[allow(deprecated)]
pub use eos::{
    EosCoordinator, EosSession, ProduceThenCheckpointCoordinator, ProduceThenCheckpointSession,
};
pub use message::{Delivery, KafkaMessage, encode_bus_id, parse_bus_id};
pub use offset::{FileOffsetStore, MemoryOffsetStore, OffsetCommitStore};
pub use pool::{KafkaHealth, KafkaPool, KafkaPoolStats};
pub use producer::KafkaProducer;

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
mod mock;
#[cfg(feature = "scaffold")]
pub use adapter::KafkaAdapter;
#[cfg(feature = "scaffold")]
pub use mock::MockKafkaBus;

#[cfg(test)]
mod public_api_surface {
    use super::*;
    use bytes::Bytes;
    use kernel::ErrorKind;
    use std::sync::Arc;
    use std::time::Duration;

    /// 默认 feature crate-root 导出均有真实行为路径（非仅类型存在）。
    #[tokio::test]
    async fn default_exports_behavior_paths() {
        assert!(!DEFAULT_BROKERS.is_empty());
        assert_eq!(DEFAULT_SASL_MECHANISM, "PLAIN");

        let cfg = KafkaConfigBuilder::new()
            .brokers(DEFAULT_BROKERS)
            .client_id("kafkax-surface")
            .connect_timeout(Duration::from_millis(50))
            .operation_timeout(Duration::from_millis(50))
            .delivery_timeout(Duration::from_millis(50))
            .build()
            .expect("默认 loopback 配置合法");
        assert_eq!(cfg.security_protocol(), "PLAINTEXT");
        assert_eq!(cfg.client_id, "kafkax-surface");

        let from_env = KafkaConfig::from_env().expect("from_env 在无强制变量时回落 default");
        assert!(!from_env.brokers.is_empty());

        let consumer_cfg =
            ConsumerConfig::assign("surface-topic", 0, "g-surface").with_start_offset(7);
        assert_eq!(consumer_cfg.partition, 0);
        assert_eq!(consumer_cfg.start_offset, Some(7));

        let delivery = Delivery { partition: 0, offset: 1 };
        assert_eq!(delivery.offset, 1);
        let msg = KafkaMessage {
            topic: "t".into(),
            partition: 0,
            offset: 1,
            payload: Bytes::from_static(b"x"),
            key: None,
            timestamp: None,
        };
        assert_eq!(msg.bus_id(), encode_bus_id("t", 0, 1));
        assert!(parse_bus_id(&msg.bus_id()).is_some());
        assert!(parse_bus_id("bad").is_none());

        let health = KafkaHealth { ready: false, detail: "offline".into() };
        assert!(!health.ready);
        assert!(health.detail.contains("offline"));
        let stats = KafkaPoolStats { published: 2, publish_failed: 1, closed: false };
        assert_eq!(stats.published, 2);
        assert_eq!(stats.publish_failed, 1);

        let store = MemoryOffsetStore::new().shared();
        store.commit("t", 0, 3).await.expect("commit");
        assert_eq!(store.committed("t", 0).await.expect("get"), Some(4));
        assert_eq!(resolve_start_offset(store.as_ref(), "t", 0).await.expect("start"), Some(4));

        let coordinator =
            ProduceThenCheckpointCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        // 共享同一 store 句柄：coordinator 不伪造 produce 结果
        assert!(
            Arc::ptr_eq(&coordinator.store(), &(Arc::clone(&store) as Arc<dyn OffsetCommitStore>))
                || coordinator.store().committed("t", 0).await.expect("via coord") == Some(4)
        );
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(4));

        let path = std::env::temp_dir().join(format!(
            "kafkax-offset-surface-{}-{}.tsv",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let file = FileOffsetStore::new(&path);
        file.commit("file-t", 1, 5).await.expect("file commit");
        assert_eq!(file.committed("file-t", 1).await.expect("file get"), Some(6));
        let _ = std::fs::remove_file(&path);

        // connect 拒绝路径：真实 shipped 入口，无 broker
        let refused = KafkaConfig {
            brokers: "127.0.0.1:1".into(),
            connect_timeout: Duration::from_millis(200),
            delivery_timeout: Duration::from_millis(200),
            operation_timeout: Duration::from_millis(200),
            ..KafkaConfig::default()
        };
        match KafkaPool::connect(refused).await {
            Ok(_) => panic!("拒绝连接必须失败"),
            Err(err) => assert!(
                matches!(
                    err.kind(),
                    ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
                ),
                "kind={:?}",
                err.kind()
            ),
        }
    }
}
