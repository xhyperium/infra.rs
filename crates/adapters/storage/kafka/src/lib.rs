//! `kafkax` — kafka adapter scaffold。
//!
//! 实现 [`contracts::EventBus`]（进程内内存）。

mod adapter;

pub use adapter::KafkaAdapter;
