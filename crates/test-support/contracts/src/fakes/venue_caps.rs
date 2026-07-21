//! Exchange capability Fakes（MarketData / Catalog / Execution / Account / VenueTime）。

use async_trait::async_trait;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use contracts::{
    AccountSource, ExecutionVenue, InstrumentCatalog, MarketDataSource, VenueTimeSource,
};
use decimalx::{Decimal, Price, Qty};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::sync::Mutex;

fn sample_meta(symbol: &str) -> SymbolMeta {
    SymbolMeta {
        symbol: symbol.to_string(),
        base: "BTC".into(),
        quote: "USDT".into(),
        tick_size: Decimal::new(1, 2),
        min_qty: Qty::new(Decimal::new(1, 4)),
    }
}

/// 内存行情源：预置空流（立即结束）。
#[derive(Debug, Default)]
pub struct FakeMarketDataSource;

#[async_trait]
impl MarketDataSource for FakeMarketDataSource {
    async fn subscribe_ticks(&self, _symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }

    async fn subscribe_orderbook(
        &self,
        _symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }

    async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

/// 内存交易对目录。
#[derive(Debug, Default)]
pub struct FakeInstrumentCatalog {
    inner: Mutex<HashMap<String, SymbolMeta>>,
}

impl FakeInstrumentCatalog {
    /// 新建空目录。
    pub fn new() -> Self {
        Self::default()
    }

    /// 预置一条元数据。
    pub fn with_symbol(self, meta: SymbolMeta) -> Self {
        if let Ok(mut g) = self.inner.lock() {
            g.insert(meta.symbol.clone(), meta);
        }
        self
    }
}

#[async_trait]
impl InstrumentCatalog for FakeInstrumentCatalog {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        let g = self.inner.lock().map_err(|_| XError::internal("catalog lock 中毒"))?;
        g.get(symbol).cloned().ok_or_else(|| XError::invalid(format!("未知交易对: {symbol}")))
    }
}

/// 内存执行场所：记录最近一次下单。
#[derive(Debug)]
pub struct FakeExecutionVenue {
    venue: VenueId,
    last_order: Mutex<Option<Order>>,
}

impl FakeExecutionVenue {
    /// 指定 venue id。
    pub fn new(venue: impl Into<VenueId>) -> Self {
        Self { venue: venue.into(), last_order: Mutex::new(None) }
    }

    /// 最近一次 place_order 的订单（测试辅助）。
    pub fn last_order(&self) -> Option<Order> {
        self.last_order.lock().ok().and_then(|g| g.clone())
    }
}

#[async_trait]
impl ExecutionVenue for FakeExecutionVenue {
    async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
        if let Ok(mut g) = self.last_order.lock() {
            *g = Some(order.clone());
        }
        Ok(OrderAck { id: order.id.clone(), status: OrderStatus::Pending, ts: 0 })
    }

    async fn cancel_order(&self, _request: &CancelOrderRequest) -> XResult<()> {
        Ok(())
    }

    async fn query_order(&self, _request: &CancelOrderRequest) -> XResult<OrderStatus> {
        Ok(OrderStatus::Open)
    }

    fn venue_id(&self) -> VenueId {
        self.venue.clone()
    }
}

/// 内存账户源。
#[derive(Debug, Default)]
pub struct FakeAccountSource {
    positions: Mutex<Vec<Position>>,
    balances: Mutex<Vec<Money>>,
}

impl FakeAccountSource {
    /// 新建空账户。
    pub fn new() -> Self {
        Self::default()
    }

    /// 预置持仓。
    pub fn with_position(self, pos: Position) -> Self {
        if let Ok(mut g) = self.positions.lock() {
            g.push(pos);
        }
        self
    }

    /// 预置余额。
    pub fn with_balance(self, money: Money) -> Self {
        if let Ok(mut g) = self.balances.lock() {
            g.push(money);
        }
        self
    }
}

#[async_trait]
impl AccountSource for FakeAccountSource {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        let g = self.positions.lock().map_err(|_| XError::internal("account lock 中毒"))?;
        Ok(g.clone())
    }

    async fn query_balance(&self) -> XResult<Vec<Money>> {
        let g = self.balances.lock().map_err(|_| XError::internal("account lock 中毒"))?;
        Ok(g.clone())
    }
}

/// 固定服务器时间。
#[derive(Debug, Clone)]
pub struct FakeVenueTimeSource {
    /// 固定返回值（纳秒 epoch 或实现约定刻度）。
    pub now_ns: i64,
}

impl FakeVenueTimeSource {
    /// 固定时间。
    pub fn new(now_ns: i64) -> Self {
        Self { now_ns }
    }
}

#[async_trait]
impl VenueTimeSource for FakeVenueTimeSource {
    async fn server_time(&self) -> XResult<i64> {
        Ok(self.now_ns)
    }
}

/// 供 suite 自测构造默认 meta。
pub fn default_symbol_meta(symbol: &str) -> SymbolMeta {
    sample_meta(symbol)
}

/// 构造最小可下单 [`Order`]。
pub fn sample_order(id: &str, symbol: &str) -> Order {
    Order {
        id: id.into(),
        symbol: symbol.into(),
        side: canonical::Side::Buy,
        price: Price::new(Decimal::new(100, 0)),
        qty: Qty::new(Decimal::new(1, 0)),
        status: OrderStatus::Pending,
    }
}
