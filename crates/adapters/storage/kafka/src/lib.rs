//! `kafkax` — 生产级 Kafka 适配（`rdkafka`）。
//!
//! ## 默认入口
//!
//! - [`KafkaConfig`] / [`KafkaConfig::from_env`]
//! - [`KafkaPool`]：`connect` / `producer` / `consumer` / `health` / `stats` / `close`
//! - [`KafkaProducer`]：`publish` 等待 delivery report
//! - [`KafkaConsumer`]：`subscribe` + 消息流
//! - [`KafkaEventBus`]：[`contracts::EventBus`] facade（**at-most-once**；见模块文档）
//!
//! ## scaffold feature
//!
//! 启用 `scaffold` 时额外导出旧的内存 [`KafkaAdapter`] / [`MockKafkaBus`]。
//!
//! ## 环境变量
//!
//! `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`

#![forbid(unsafe_code)]

mod bus;
mod config;
mod consumer;
mod error_map;
mod message;
mod pool;
mod producer;

pub use bus::KafkaEventBus;
pub use config::{DEFAULT_BROKERS, DEFAULT_SASL_MECHANISM, KafkaConfig};
pub use consumer::{ConsumerConfig, KafkaConsumer};
pub use message::{Delivery, KafkaMessage, encode_bus_id, parse_bus_id};
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
