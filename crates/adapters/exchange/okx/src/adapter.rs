//! OKX `VenueAdapter` scaffold（非真实 HTTP）。

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

/// OKX adapter scaffold。
pub struct OkxAdapter {
    name: String,
    base_url: String,
    connected: AtomicBool,
}

impl OkxAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self { name: name.into(), base_url: base_url.into(), connected: AtomicBool::new(false) }
    }

    pub fn demo() -> Self {
        Self::new("okx-demo", "https://www.okx.com")
    }

    pub fn mainnet() -> Self {
        Self::new("okx-mainnet", "https://www.okx.com")
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

    fn zero_qty() -> XResult<Qty> {
        Ok(Qty(Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero qty: {e}")))?))
    }
}

#[async_trait]
impl VenueAdapter for OkxAdapter {
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
        "okx"
    }
}

#[async_trait]
impl MarketDataSource for OkxAdapter {
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
impl InstrumentCatalog for OkxAdapter {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        VenueAdapter::symbol_info(self, symbol).await
    }
}

#[async_trait]
impl ExecutionVenue for OkxAdapter {
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
        "okx".to_string()
    }
}

#[async_trait]
impl AccountSource for OkxAdapter {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        VenueAdapter::query_position(self).await
    }
    async fn query_balance(&self) -> XResult<Vec<Money>> {
        VenueAdapter::query_balance(self).await
    }
}

#[async_trait]
impl VenueTimeSource for OkxAdapter {
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
            symbol: "BTC-USDT".into(),
            side: Side::Buy,
            price: Price(Decimal::try_new(1, 0).unwrap()),
            qty: Qty(Decimal::try_new(1, 0).unwrap()),
            status: OrderStatus::Pending,
        }
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        VenueAdapter::disconnect(&a).await.unwrap();
    }

    #[tokio::test]
    async fn double_connect_fails() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert!(VenueAdapter::connect(&a).await.is_err());
    }

    #[tokio::test]
    async fn place_order_flow() {
        let a = OkxAdapter::demo();
        assert!(VenueAdapter::place_order(&a, &sample_order()).await.is_err());
        VenueAdapter::connect(&a).await.unwrap();
        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
    }

    #[tokio::test]
    async fn cancel_query_request() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Open);
    }

    #[tokio::test]
    async fn capability_traits() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert_eq!(VenueAdapter::venue_id(&a), "okx");
        assert_eq!(ExecutionVenue::venue_id(&a), "okx");
        let _ = VenueAdapter::server_time(&a).await.unwrap();
        assert!(VenueAdapter::query_position(&a).await.unwrap().is_empty());
    }
}
