//! Binance `VenueAdapter` scaffold（非真实 HTTP）。

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use contracts::{
    AccountSource, ExecutionVenue, InstrumentCatalog, MarketDataSource, VenueAdapter,
    VenueTimeSource,
};
use decimalx::{Currency, Decimal, Price, Qty};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// 适配器连接状态（观测用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Disconnected,
    Connected,
}

/// K 线时间粒度（venue 扩展；非 contracts 面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

/// 单根 K 线 scaffold DTO。
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

/// Binance adapter scaffold。
pub struct BinanceAdapter {
    name: String,
    base_url: String,
    connected: AtomicBool,
}

impl BinanceAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { name: name.into(), base_url: base_url.into(), connected: AtomicBool::new(false) }
    }

    pub fn testnet() -> Self {
        Self::new("binance-testnet", "https://testnet.binance.vision")
    }

    pub fn mainnet() -> Self {
        Self::new("binance-mainnet", "https://api.binance.com")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn state(&self) -> AdapterState {
        if self.connected.load(Ordering::SeqCst) {
            AdapterState::Connected
        } else {
            AdapterState::Disconnected
        }
    }

    fn require_connected(&self) -> XResult<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(XError::unavailable("not connected"));
        }
        Ok(())
    }

    fn zero_price() -> XResult<Price> {
        Ok(Price(Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero price: {e}")))?))
    }

    fn zero_qty() -> XResult<Qty> {
        Ok(Qty(Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero qty: {e}")))?))
    }

    /// venue 扩展：占位 K 线（非 contracts 面）。
    pub async fn fetch_candles(
        &self,
        _symbol: &str,
        _timeframe: Timeframe,
        limit: Option<u32>,
    ) -> XResult<Vec<Candle>> {
        self.require_connected()?;
        let zero = Self::zero_price()?;
        let count = limit.unwrap_or(10) as usize;
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

#[async_trait]
impl VenueAdapter for BinanceAdapter {
    async fn connect(&self) -> XResult<()> {
        if self.connected.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err()
        {
            return Err(XError::conflict("already connected"));
        }
        Ok(())
    }

    async fn disconnect(&self) -> XResult<()> {
        if self.connected.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_err()
        {
            return Err(XError::unavailable("not connected"));
        }
        Ok(())
    }

    async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
        self.require_connected()?;
        Ok(OrderAck { id: order.id.clone(), status: OrderStatus::Open, ts: 0 })
    }

    #[allow(deprecated)]
    async fn cancel_order(&self, _id: &str) -> XResult<()> {
        self.require_connected()?;
        Ok(())
    }

    #[allow(deprecated)]
    async fn query_order(&self, _id: &str) -> XResult<OrderStatus> {
        self.require_connected()?;
        Ok(OrderStatus::Open)
    }

    async fn cancel_order_request(&self, _request: &CancelOrderRequest) -> XResult<()> {
        self.require_connected()?;
        Ok(())
    }

    async fn query_order_request(&self, _request: &CancelOrderRequest) -> XResult<OrderStatus> {
        self.require_connected()?;
        Ok(OrderStatus::Open)
    }

    async fn query_position(&self) -> XResult<Vec<Position>> {
        self.require_connected()?;
        Ok(Vec::new())
    }

    async fn query_balance(&self) -> XResult<Vec<Money>> {
        self.require_connected()?;
        let ccy = Currency::try_new(*b"USD").map_err(|e| XError::internal(format!("ccy: {e}")))?;
        let amt = Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("amt: {e}")))?;
        Ok(vec![Money::try_new(amt, ccy).map_err(|e| XError::internal(format!("money: {e}")))?])
    }

    async fn subscribe_ticks(&self, _symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn subscribe_orderbook(
        &self,
        _symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn server_time(&self) -> XResult<i64> {
        self.require_connected()?;
        Ok(0)
    }

    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        self.require_connected()?;
        Ok(SymbolMeta {
            symbol: symbol.to_string(),
            base: "BASE".into(),
            quote: "QUOTE".into(),
            tick_size: Decimal::try_new(1, 2)
                .map_err(|e| XError::internal(format!("tick: {e}")))?,
            min_qty: Self::zero_qty()?,
        })
    }

    fn venue_id(&self) -> &'static str {
        "binance"
    }
}

#[async_trait]
impl MarketDataSource for BinanceAdapter {
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        VenueAdapter::subscribe_ticks(self, symbol).await
    }
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        VenueAdapter::subscribe_orderbook(self, symbol).await
    }
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        VenueAdapter::subscribe_trades(self, symbol).await
    }
}

#[async_trait]
impl InstrumentCatalog for BinanceAdapter {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        VenueAdapter::symbol_info(self, symbol).await
    }
}

#[async_trait]
impl ExecutionVenue for BinanceAdapter {
    async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
        VenueAdapter::place_order(self, order).await
    }
    async fn cancel_order(&self, request: &CancelOrderRequest) -> XResult<()> {
        self.cancel_order_request(request).await
    }
    async fn query_order(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        self.query_order_request(request).await
    }
    fn venue_id(&self) -> VenueId {
        "binance".to_string()
    }
}

#[async_trait]
impl AccountSource for BinanceAdapter {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        VenueAdapter::query_position(self).await
    }
    async fn query_balance(&self) -> XResult<Vec<Money>> {
        VenueAdapter::query_balance(self).await
    }
}

#[async_trait]
impl VenueTimeSource for BinanceAdapter {
    async fn server_time(&self) -> XResult<i64> {
        VenueAdapter::server_time(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::{OrderRef, Side};
    use decimalx::Qty;

    fn sample_order() -> Order {
        Order {
            id: "o1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: Price(Decimal::try_new(1, 0).unwrap()),
            qty: Qty(Decimal::try_new(1, 0).unwrap()),
            status: OrderStatus::Pending,
        }
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let a = BinanceAdapter::mainnet();
        assert_eq!(a.state(), AdapterState::Disconnected);
        VenueAdapter::connect(&a).await.unwrap();
        assert_eq!(a.state(), AdapterState::Connected);
        VenueAdapter::disconnect(&a).await.unwrap();
        assert_eq!(a.state(), AdapterState::Disconnected);
    }

    #[tokio::test]
    async fn double_connect_fails() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert!(VenueAdapter::connect(&a).await.is_err());
    }

    #[tokio::test]
    async fn place_order_requires_connect() {
        let a = BinanceAdapter::mainnet();
        assert!(VenueAdapter::place_order(&a, &sample_order()).await.is_err());
        VenueAdapter::connect(&a).await.unwrap();
        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
        assert_eq!(ack.status, OrderStatus::Open);
    }

    #[tokio::test]
    async fn cancel_query_request() {
        let a = BinanceAdapter::testnet();
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "binance".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Open);
    }

    #[tokio::test]
    async fn market_data_streams_empty() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let _ = VenueAdapter::subscribe_ticks(&a, "BTCUSDT").await.unwrap();
        let _ = VenueAdapter::subscribe_orderbook(&a, "BTCUSDT").await.unwrap();
        let _ = VenueAdapter::subscribe_trades(&a, "BTCUSDT").await.unwrap();
    }

    #[tokio::test]
    async fn candles_extension() {
        let a = BinanceAdapter::testnet();
        VenueAdapter::connect(&a).await.unwrap();
        let c = a.fetch_candles("BTCUSDT", Timeframe::M1, Some(3)).await.unwrap();
        assert_eq!(c.len(), 3);
    }

    #[tokio::test]
    async fn capability_traits_dispatch() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let v: &dyn VenueAdapter = &a;
        assert_eq!(v.venue_id(), "binance");
        let ex: &dyn ExecutionVenue = &a;
        assert_eq!(ex.venue_id(), "binance");
        let _ = VenueAdapter::server_time(&a).await.unwrap();
        let meta = VenueAdapter::symbol_info(&a, "BTCUSDT").await.unwrap();
        assert_eq!(meta.symbol, "BTCUSDT");
        assert!(VenueAdapter::query_position(&a).await.unwrap().is_empty());
        assert_eq!(VenueAdapter::query_balance(&a).await.unwrap().len(), 1);
    }
}
