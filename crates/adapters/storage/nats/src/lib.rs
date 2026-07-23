//! `natsx` — NATS 适配（`async-nats` Core NATS + JetStream durable pull）。
//!
//! ## 默认入口
//!
//! - [`NatsConfig`] / [`NatsConfig::from_env`] / [`TlsPolicy`]
//! - [`NatsPool`]：`connect` / `publish` / `subscribe` / `ping` / `health` / `close`
//! - [`NatsEventBus`]：[`contracts::EventBus`]（**at-most-once**）
//! - [`JetStream`]：持久 publish、stream 管理与显式确认 consumer
//! - [`JetStreamConsumer`] / [`JetStreamDelivery`]：有限拉取与 ack/nak/progress/term
//!
//! ## TLS 默认策略
//!
//! - loopback → [`TlsPolicy::Prefer`]（允许明文）
//! - 非 loopback → [`TlsPolicy::Require`]（`ConnectOptions::require_tls(true)`）
//! - 环境：`FOUNDATIONX_NATS_TLS` / `FOUNDATIONX_NATS_TLS_POLICY`
//!
//! ## scaffold feature
//!
//! 启用 `scaffold` 时额外导出旧内存 `NatsAdapter` / `MockNatsBus`。
//!
//! ## 环境变量
//!
//! `FOUNDATIONX_NATS_{URL,USER,PASSWORD,TLS,TLS_POLICY,JETSTREAM}` 或 `FOUNDATIONX_NATSX_*`。

#![forbid(unsafe_code)]

mod bus;
mod config;
mod jetstream;
mod pool;

pub use bus::NatsEventBus;
pub use config::{DEFAULT_URL, NatsConfig, TlsPolicy, url_is_loopback};
pub use jetstream::{
    JetStream, JetStreamConsumer, JetStreamConsumerConfig, JetStreamDelivery,
    JetStreamDeliveryMetadata, PullConsumerConfig, StreamConfig, validate_consumer_name,
    validate_operation_timeout, validate_stream_name,
};
pub use pool::{NatsHealth, NatsMessage, NatsPool, NatsPoolStats, NatsSubscription};

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
mod mock;
#[cfg(feature = "scaffold")]
pub use adapter::NatsAdapter;
#[cfg(feature = "scaffold")]
pub use mock::MockNatsBus;

#[cfg(test)]
mod public_api_surface {
    use super::*;

    #[test]
    fn default_exports_named() {
        assert!(!DEFAULT_URL.is_empty());
        let cfg = NatsConfig::default();
        assert_eq!(cfg.effective_tls_policy(), TlsPolicy::Prefer);
        assert!(validate_stream_name("EVENTS").is_ok());
        let _sc = StreamConfig::new("S", "s.>");
        let _pc = PullConsumerConfig::durable("d");
        let _durable = JetStreamConsumerConfig::durable("durable");
        fn assert_type<T: ?Sized>() {}
        assert_type::<NatsPool>();
        assert_type::<NatsEventBus>();
        assert_type::<JetStream>();
        assert_type::<JetStreamConsumer>();
        assert_type::<JetStreamDelivery>();
        assert_type::<TlsPolicy>();
        assert!(url_is_loopback("nats://127.0.0.1:4222"));
    }
}
