//! `kafkax` — kafka adapter。
//!
//! - scaffold：[`KafkaAdapter`]
//! - mock 验证入口：[`MockKafkaBus`]（单调 `BusMessage.id`；**非**真实 Kafka）

mod adapter;
mod mock;

pub use adapter::KafkaAdapter;
pub use mock::MockKafkaBus;
