//! `redisx` — Redis adapter scaffold。
//!
//! 实现 [`contracts::KeyValueStore`] 与 [`contracts::PubSub`]（进程内内存）。

mod adapter;

pub use adapter::RedisAdapter;
