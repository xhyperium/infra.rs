use crate::{AdapterState, Error, Result};
use decimalx::{Decimal, Price};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Price,
    pub ask: Price,
    pub last: Price,
    pub timestamp: u64,
}

pub struct BinanceAdapter {
    name: String,
    state: AdapterState,
    base_url: String,
}

impl BinanceAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { name: name.into(), state: AdapterState::Uninitialized, base_url: base_url.into() }
    }
    pub fn testnet() -> Self { Self::new("binance-testnet", "https://testnet.binance.vision") }
    pub fn mainnet() -> Self { Self::new("binance-mainnet", "https://api.binance.com") }
    pub fn name(&self) -> &str { &self.name }
    pub fn base_url(&self) -> &str { &self.base_url }
    pub fn state(&self) -> AdapterState { self.state }
    pub fn connect(&mut self) -> Result<()> {
        if self.state == AdapterState::Connected { return Err(Error::AlreadyConnected); }
        self.state = AdapterState::Connected; Ok(())
    }
    pub fn disconnect(&mut self) -> Result<()> {
        if self.state != AdapterState::Connected { return Err(Error::NotConnected); }
        self.state = AdapterState::Disconnected; Ok(())
    }
    pub fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        if self.state != AdapterState::Connected { return Err(Error::NotConnected); }
        let zero = Price(Decimal::try_new(0, 0).map_err(|e| Error::Internal(format!("zero: {e}")))?);
        Ok(Ticker { symbol: symbol.to_string(), bid: zero, ask: zero, last: zero, timestamp: 0 })
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
}
