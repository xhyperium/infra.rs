//! `okxx` — OKX 交易所适配器。
//!
//! 实现 `ExchangeAdapter` trait。

use decimalx::{Decimal, Price};
use infra_contracts::exchange::{ExchangeAdapter, Ticker};
use infra_contracts::{AdapterState, Result};

/// OKX 适配器
pub struct OkxAdapter {
    name: String,
    state: AdapterState,
    base_url: String,
}

impl OkxAdapter {
    /// 创建新的 OKX 适配器
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { name: name.into(), state: AdapterState::Uninitialized, base_url: base_url.into() }
    }

    /// 默认模拟盘
    pub fn demo() -> Self {
        Self::new("okx-demo", "https://www.okx.com")
    }

    /// 默认主网
    pub fn mainnet() -> Self {
        Self::new("okx-mainnet", "https://www.okx.com")
    }

    /// 配置的 REST base URL（scaffold 阶段供观测；真实 HTTP 未接入）。
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl ExchangeAdapter for OkxAdapter {
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
        // 占位 ticker（实际实现需 HTTP 请求）；价格使用 decimalx::Price
        let zero =
            Price(Decimal::try_new(0, 0).map_err(|e| {
                infra_contracts::Error::Internal(format!("zero price construct: {e}"))
            })?);
        Ok(Ticker { symbol: symbol.to_string(), bid: zero, ask: zero, last: zero, timestamp: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infra_contracts::exchange::ExchangeAdapter;

    #[test]
    fn test_connect_disconnect() {
        let mut adapter = OkxAdapter::demo();
        assert_eq!(adapter.state(), AdapterState::Uninitialized);

        adapter.connect().expect("connect");
        assert_eq!(adapter.state(), AdapterState::Connected);

        adapter.disconnect().expect("disconnect");
        assert_eq!(adapter.state(), AdapterState::Disconnected);
    }

    #[test]
    fn test_double_connect_fails() {
        let mut adapter = OkxAdapter::demo();
        adapter.connect().expect("connect");
        assert!(adapter.connect().is_err());
    }

    #[test]
    fn test_disconnect_before_connect_fails() {
        let mut adapter = OkxAdapter::demo();
        assert!(adapter.disconnect().is_err());
    }

    #[test]
    fn test_fetch_ticker_requires_connect() {
        let adapter = OkxAdapter::demo();
        assert!(adapter.fetch_ticker("BTC-USDT").is_err());

        let mut adapter = OkxAdapter::demo();
        adapter.connect().expect("connect");
        let ticker = adapter.fetch_ticker("BTC-USDT").expect("ticker");
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert_eq!(ticker.bid.0, Decimal::try_new(0, 0).expect("zero"));
    }

    #[test]
    fn test_name_and_base_url() {
        let adapter = OkxAdapter::mainnet();
        assert_eq!(adapter.name(), "okx-mainnet");
        assert_eq!(adapter.base_url(), "https://www.okx.com");
    }
}
