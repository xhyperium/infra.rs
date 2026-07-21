//! `natsx` — nats adapter。
//!
//! - scaffold：[`NatsAdapter`]
//! - mock 验证入口：[`MockNatsBus`]（单调 `BusMessage.id`；**非**真实 NATS）

mod adapter;
mod mock;

pub use adapter::NatsAdapter;
pub use mock::MockNatsBus;
