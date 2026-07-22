//! `kafkax` — 生产级 Kafka 适配（纯 Rust `rskafka`）。
//!
//! ## 默认入口
//!
//! - [`KafkaConfig`] / [`KafkaConfig::from_env`]
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
//! - [`EosCoordinator`] / [`EosSession`]：produce 成功后才允许 commit（应用级 EOS）
//!
//! ## scaffold feature
//!
//! 启用 `scaffold` 时额外导出旧的内存 [`KafkaAdapter`] / [`MockKafkaBus`]。
//!
//! ## 环境变量
//!
//! `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`

#![forbid(unsafe_code)]

mod at_least_once;
mod bus;
mod config;
mod consumer;
mod eos;
mod error_map;
mod message;
mod offset;
mod pool;
mod producer;

pub use at_least_once::{AtLeastOnceConsumer, KafkaAtLeastOnceBus, resolve_start_offset};
pub use bus::KafkaEventBus;
pub use config::{DEFAULT_BROKERS, DEFAULT_SASL_MECHANISM, KafkaConfig};
pub use consumer::{ConsumerConfig, KafkaConsumer};
pub use eos::{EosCoordinator, EosSession};
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
    use std::sync::Arc;

    /// 默认 feature crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        assert!(!DEFAULT_BROKERS.is_empty());
        assert!(!DEFAULT_SASL_MECHANISM.is_empty());
        let _cfg = KafkaConfig::default();
        let _consumer_cfg = ConsumerConfig::subscribe("t", "g");

        let delivery = Delivery { partition: 0, offset: 1 };
        assert_eq!(delivery.offset, 1);
        let msg = KafkaMessage {
            topic: "t".into(),
            partition: 0,
            offset: 1,
            payload: Bytes::from_static(b"x"),
            key: None,
        };
        assert_eq!(msg.topic, "t");
        let health = KafkaHealth { ready: false, detail: "offline".into() };
        assert!(!health.ready);
        let stats = KafkaPoolStats { published: 0, publish_failed: 0, closed: false };
        assert!(!stats.closed);

        let id = encode_bus_id("t", 0, 1);
        let _ = parse_bus_id(&id).expect("id");

        let store = MemoryOffsetStore::new().shared();
        let _eos = EosCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        let _file = FileOffsetStore::new("/tmp/kafkax-offset-surface.tsv");

        fn assert_type<T: ?Sized>() {}
        assert_type::<KafkaPool>();
        assert_type::<KafkaProducer>();
        assert_type::<KafkaConsumer>();
        assert_type::<KafkaEventBus>();
        assert_type::<AtLeastOnceConsumer>();
        assert_type::<KafkaAtLeastOnceBus>();
        assert_type::<EosCoordinator>();
        assert_type::<EosSession>();
        assert_type::<MemoryOffsetStore>();
        assert_type::<FileOffsetStore>();
    }
}
