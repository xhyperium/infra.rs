//! `okxx` — okx exchange adapter scaffold。
//! 完整 `contracts` venue trait 接入 DEFER。

mod adapter;
mod error;

pub use adapter::{OkxAdapter, Ticker};
pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Uninitialized,
    Connected,
    Disconnected,
    Shutdown,
}
