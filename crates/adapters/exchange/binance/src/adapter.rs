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
}
