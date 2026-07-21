//! `okxx` — okx exchange adapter scaffold。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait（内存占位，非真实 HTTP）。

mod adapter;

pub use adapter::{AdapterState, OkxAdapter};
