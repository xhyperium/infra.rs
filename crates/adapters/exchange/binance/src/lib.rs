//! `binancex` — binance exchange adapter scaffold。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait。
//! 可选注入 [`transportx::HttpDriver`]（`BinanceAdapter::with_http`）走传输边界；
//! 默认仍为内存占位（非真实交易所协议）。

mod adapter;

pub use adapter::{AdapterState, BinanceAdapter, Candle, Timeframe};
