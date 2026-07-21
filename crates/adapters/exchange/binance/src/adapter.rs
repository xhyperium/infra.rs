//! Binance exchange adapter scaffold.

use crate::{AdapterState, Error, Result};
use decimalx::{Decimal, Price};

/// Ticker snapshot (scaffold DTO).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Price,
    pub ask: Price,
    pub last: Price,
    pub timestamp: u64,
}

/// K 线时间粒度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

/// 单根 K 线（scaffold DTO）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    pub open_time: u64,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Price,
    pub close_time: u64,
}

/// 订单方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

/// 订单状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

/// 订单（scaffold DTO）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Price,
    pub status: OrderStatus,
}

/// 资产余额（scaffold DTO）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    pub asset: String,
    pub free: Price,
    pub locked: Price,
}

/// Binance adapter (HTTP 接入 DEFER)。
pub struct BinanceAdapter {
    name: String,
    state: AdapterState,
    base_url: String,
}

impl BinanceAdapter {
    /// 创建适配器。
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { name: name.into(), state: AdapterState::Uninitialized, base_url: base_url.into() }
    }

    /// 测试网。
    pub fn testnet() -> Self {
        Self::new("binance-testnet", "https://testnet.binance.vision")
    }

    /// 主网。
    pub fn mainnet() -> Self {
        Self::new("binance-mainnet", "https://api.binance.com")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// base URL。
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 状态。
    pub fn state(&self) -> AdapterState {
        self.state
    }

    /// 连接。
    pub fn connect(&mut self) -> Result<()> {
        if self.state == AdapterState::Connected {
            return Err(Error::AlreadyConnected);
        }
        self.state = AdapterState::Connected;
        Ok(())
    }

    /// 断开。
    pub fn disconnect(&mut self) -> Result<()> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        self.state = AdapterState::Disconnected;
        Ok(())
    }

    /// 获取 ticker（占位）。
    pub fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        let zero =
            Price(Decimal::try_new(0, 0).map_err(|e| Error::Internal(format!("zero: {e}")))?);
        Ok(Ticker { symbol: symbol.to_string(), bid: zero, ask: zero, last: zero, timestamp: 0 })
    }

    /// 获取 K 线（占位）。
    pub fn fetch_candles(
        &self,
        symbol: &str,
        _timeframe: Timeframe,
        limit: Option<u32>,
    ) -> Result<Vec<Candle>> {
        let _ = symbol;
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        let count = limit.unwrap_or(10) as usize;
        let zero =
            Price(Decimal::try_new(0, 0).map_err(|e| Error::Internal(format!("zero: {e}")))?);
        Ok((0..count)
            .map(|i| Candle {
                open_time: i as u64,
                open: zero,
                high: zero,
                low: zero,
                close: zero,
                volume: zero,
                close_time: (i + 1) as u64,
            })
            .collect())
    }

    /// 下单（占位）。
    pub fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        price: Price,
        quantity: Price,
    ) -> Result<Order> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        Ok(Order {
            id: "scaffold-1".into(),
            symbol: symbol.into(),
            side,
            order_type,
            price,
            quantity,
            status: OrderStatus::New,
        })
    }

    /// 撤单（占位）。
    pub fn cancel_order(&self, order_id: &str) -> Result<Order> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        Ok(Order {
            id: order_id.into(),
            symbol: "unknown".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Price(Decimal::try_new(0, 0).unwrap()),
            quantity: Price(Decimal::try_new(0, 0).unwrap()),
            status: OrderStatus::Canceled,
        })
    }

    /// 查询订单（占位）。
    pub fn query_order(&self, order_id: &str) -> Result<Order> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        Ok(Order {
            id: order_id.into(),
            symbol: "BTCUSDT".into(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Price(Decimal::try_new(0, 0).unwrap()),
            quantity: Price(Decimal::try_new(0, 0).unwrap()),
            status: OrderStatus::Filled,
        })
    }

    /// 查询余额（占位）。
    pub fn fetch_balances(&self) -> Result<Vec<Balance>> {
        if self.state != AdapterState::Connected {
            return Err(Error::NotConnected);
        }
        let zero = Price(Decimal::try_new(0, 0).unwrap());
        Ok(vec![
            Balance { asset: "BTC".into(), free: zero, locked: zero },
            Balance { asset: "USDT".into(), free: zero, locked: zero },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_disconnect() {
        let mut a = BinanceAdapter::mainnet();
        a.connect().unwrap();
        a.disconnect().unwrap();
    }

    #[test]
    fn double_connect_fails() {
        let mut a = BinanceAdapter::mainnet();
        a.connect().unwrap();
        assert!(a.connect().is_err());
    }

    #[test]
    fn ticker_requires_connect() {
        let a = BinanceAdapter::mainnet();
        assert!(a.fetch_ticker("BTC").is_err());
    }

    #[test]
    fn fetch_candles_requires_connect() {
        let a = BinanceAdapter::testnet();
        assert!(a.fetch_candles("BTCUSDT", Timeframe::M1, None).is_err());
    }

    #[test]
    fn fetch_candles_default_limit() {
        let mut a = BinanceAdapter::testnet();
        a.connect().unwrap();
        let candles = a.fetch_candles("BTCUSDT", Timeframe::H1, None).expect("candles");
        assert_eq!(candles.len(), 10);
    }

    #[test]
    fn fetch_candles_custom_limit() {
        let mut a = BinanceAdapter::testnet();
        a.connect().unwrap();
        let candles = a.fetch_candles("ETHUSDT", Timeframe::D1, Some(5)).expect("candles");
        assert_eq!(candles.len(), 5);
    }

    #[test]
    fn place_order_requires_connect() {
        let a = BinanceAdapter::testnet();
        let zero = Price(Decimal::try_new(0, 0).unwrap());
        assert!(a.place_order("BTCUSDT", OrderSide::Buy, OrderType::Limit, zero, zero).is_err());
    }

    #[test]
    fn place_and_cancel_order() {
        let mut a = BinanceAdapter::testnet();
        a.connect().unwrap();
        let zero = Price(Decimal::try_new(0, 0).unwrap());
        let order = a
            .place_order("ETHUSDT", OrderSide::Sell, OrderType::Market, zero, zero)
            .expect("place");
        assert_eq!(order.symbol, "ETHUSDT");
        assert_eq!(order.status, OrderStatus::New);

        let canceled = a.cancel_order(&order.id).expect("cancel");
        assert_eq!(canceled.status, OrderStatus::Canceled);
    }

    #[test]
    fn query_order_requires_connect() {
        let a = BinanceAdapter::testnet();
        assert!(a.query_order("id-1").is_err());
    }

    #[test]
    fn query_order_scaffold() {
        let mut a = BinanceAdapter::testnet();
        a.connect().unwrap();
        let order = a.query_order("scaffold-42").expect("query");
        assert_eq!(order.id, "scaffold-42");
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn fetch_balances_requires_connect() {
        let a = BinanceAdapter::testnet();
        assert!(a.fetch_balances().is_err());
    }

    #[test]
    fn fetch_balances_scaffold() {
        let mut a = BinanceAdapter::testnet();
        a.connect().unwrap();
        let balances = a.fetch_balances().expect("balances");
        assert_eq!(balances.len(), 2);
        assert!(balances.iter().any(|b| b.asset == "BTC"));
        assert!(balances.iter().any(|b| b.asset == "USDT"));
    }
}
