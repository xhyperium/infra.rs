//! `binancex` — Binance 交易所适配器。
//!
//! 实现 `ExchangeAdapter` trait。

use infra_contracts::exchange::{ExchangeAdapter, Ticker};
use infra_contracts::{AdapterState, Result};

/// Binance 适配器
pub struct BinanceAdapter {
    name: String,
    state: AdapterState,
    base_url: String,
}

impl BinanceAdapter {
    /// 创建新的 Binance 适配器
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: AdapterState::Uninitialized,
            base_url: base_url.into(),
        }
    }

    /// 默认测试网
    pub fn testnet() -> Self {
        Self::new("binance-testnet", "https://testnet.binance.vision")
    }

    /// 默认主网
    pub fn mainnet() -> Self {
        Self::new("binance-mainnet", "https://api.binance.com")
    }
}

impl ExchangeAdapter for BinanceAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn connect(&mut self) -> Result<()> {
        if self.state == AdapterState::Connected {
            return Err(infra_contracts::Error::AlreadyConnected);
        }
        self.state = AdapterState::Connected;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        if self.state != AdapterState::Connected {
            return Err(infra_contracts::Error::NotConnected);
        }
        self.state = AdapterState::Disconnected;
        Ok(())
    }

    fn state(&self) -> AdapterState {
        self.state
    }

    fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        if self.state != AdapterState::Connected {
            return Err(infra_contracts::Error::NotConnected);
        }
        // 返回占位 ticker（实际实现需 HTTP 请求）
        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: 0.0,
            ask: 0.0,
            last: 0.0,
            timestamp: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infra_contracts::exchange::ExchangeAdapter;

    #[test]
    fn test_connect_disconnect() {
        let mut adapter = BinanceAdapter::testnet();
        assert_eq!(adapter.state(), AdapterState::Uninitialized);

        adapter.connect().unwrap();
        assert_eq!(adapter.state(), AdapterState::Connected);

        adapter.disconnect().unwrap();
        assert_eq!(adapter.state(), AdapterState::Disconnected);
    }

    #[test]
    fn test_double_connect_fails() {
        let mut adapter = BinanceAdapter::testnet();
        adapter.connect().unwrap();
        assert!(adapter.connect().is_err());
    }

    #[test]
    fn test_disconnect_before_connect_fails() {
        let mut adapter = BinanceAdapter::testnet();
        assert!(adapter.disconnect().is_err());
    }

    #[test]
    fn test_fetch_ticker_requires_connect() {
        let adapter = BinanceAdapter::testnet();
        assert!(adapter.fetch_ticker("BTCUSDT").is_err());

        let mut adapter = BinanceAdapter::testnet();
        adapter.connect().unwrap();
        assert!(adapter.fetch_ticker("BTCUSDT").is_ok());
    }

    #[test]
    fn test_name() {
        let adapter = BinanceAdapter::mainnet();
        assert_eq!(adapter.name(), "binance-mainnet");
    }
}
