//! `natsx` — 生产级 NATS 适配（`async-nats` Core NATS）。
//!
//! ## 默认入口
//!
//! - [`NatsConfig`] / [`NatsConfig::from_env`]
//! - [`NatsPool`]：`connect` / `publish` / `subscribe` / `ping` / `health` / `close`
//! - [`NatsEventBus`]：[`contracts::EventBus`]（**at-most-once**）
//!
//! ## scaffold feature
//!
//! 启用 `scaffold` 时额外导出旧内存 [`NatsAdapter`] / [`MockNatsBus`]。
//!
//! ## 环境变量
//!
//! `FOUNDATIONX_NATS_{URL,USER,PASSWORD}` 或 `FOUNDATIONX_NATSX_*`。

#![forbid(unsafe_code)]

mod bus;
mod config;
mod pool;

pub use bus::NatsEventBus;
pub use config::{DEFAULT_URL, NatsConfig};
pub use pool::{NatsHealth, NatsMessage, NatsPool, NatsPoolStats, NatsSubscription};

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
mod mock;
#[cfg(feature = "scaffold")]
pub use adapter::NatsAdapter;
#[cfg(feature = "scaffold")]
pub use mock::MockNatsBus;
