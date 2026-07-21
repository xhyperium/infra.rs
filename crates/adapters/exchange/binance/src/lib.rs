//! `binancex` — binance exchange adapter scaffold。
//! 完整 `contracts` venue trait 接入 DEFER。

mod adapter;
mod error;

pub use adapter::{BinanceAdapter, Candle, Ticker, Timeframe};
pub use error::{Error, Result};

/// 适配器状态（本 crate 本地）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    /// 未初始化
    Uninitialized,
    /// 已连接
    Connected,
    /// 断开
    Disconnected,
    /// 关闭
    Shutdown,
}
