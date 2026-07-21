//! 交易所适配器合约。
//!
//! 定义所有交易所适配器必须实现的统一接口。

use crate::{AdapterState, Result};

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

/// 交易所返回的统一 ticker 结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub timestamp: u64,
}

/// 交易所适配器 trait
///
/// 每个交易所（binance、okx 等）实现此 trait 以提供统一接口。
pub trait ExchangeAdapter: Send + Sync {
    /// 返回交易所名称
    fn name(&self) -> &str;

    /// 连接
    fn connect(&mut self) -> Result<()>;

    /// 断开
    fn disconnect(&mut self) -> Result<()>;

    /// 当前状态
    fn state(&self) -> AdapterState;

    /// 获取 ticker
    fn fetch_ticker(&self, symbol: &str) -> Result<Ticker>;
}
