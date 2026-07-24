//! DE-API-001 / DE-LIFE-001 / DE-REST-001：mock VenueAdapter 生命周期与负能力。

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, InstrumentMeta, OrderAmend, StreamType, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{
    Decimal, ExecType, ExecutionReport, Order, OrderId, OrderSide, OrderStatus, OrderType,
    TimeInForce,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn sample_instrument() -> InstrumentKey {
    InstrumentKey { exchange: "mock".into(), symbol: "BTCUSDT".into() }
}

fn sample_order() -> Order {
    Order {
        order_id: "o1".into(),
        instrument: "BTCUSDT".into(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        status: OrderStatus::New,
        price: Some(Decimal::new(50000, 0)),
        stop_price: None,
        quantity: Decimal::new(1, 0),
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::new(1, 0),
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
        client_order_id: None,
    }
}

/// 有连接状态的 mock：未连接时交易/查询返回明确错误。
struct StatefulMock {
    connected: AtomicBool,
    connect_count: AtomicUsize,
    disconnect_count: AtomicUsize,
}

impl StatefulMock {
    fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            connect_count: AtomicUsize::new(0),
            disconnect_count: AtomicUsize::new(0),
        }
    }

    fn require_connected(&self) -> Result<(), AdapterError> {
        if self.connected.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(AdapterError::InvalidRequest("not connected".into()))
        }
    }
}

#[async_trait]
impl VenueAdapter for StatefulMock {
    async fn connect(&self) -> Result<(), AdapterError> {
        self.connected.store(true, Ordering::SeqCst);
        self.connect_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        self.connected.store(false, Ordering::SeqCst);
        self.disconnect_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        self.require_connected()?;
        Ok(())
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        self.require_connected()?;
        Ok(())
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        self.require_connected()?;
        Ok(())
    }

    async fn place_order(&self, order: &Order) -> Result<ExecutionReport, AdapterError> {
        self.require_connected()?;
        Ok(ExecutionReport {
            report_id: "r1".into(),
            order_id: order.order_id.clone(),
            exec_type: ExecType::New,
            order_status: OrderStatus::New,
            instrument: order.instrument.clone(),
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            price: order.price,
            quantity: order.quantity,
            last_filled_price: None,
            last_filled_quantity: None,
            cumulative_filled_quantity: Decimal::ZERO,
            remaining_quantity: order.quantity,
            commission: None,
            trade_id: None,
            reject_reason: None,
            occurred_at: 1_700_000_000_000,
        })
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        self.require_connected()?;
        Ok(())
    }

    async fn amend_order(&self, amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        self.require_connected()?;
        Ok(ExecutionReport {
            report_id: "r2".into(),
            order_id: amend.order_id.clone(),
            exec_type: ExecType::Replaced,
            order_status: OrderStatus::New,
            instrument: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: amend.price,
            quantity: amend.quantity.unwrap_or(Decimal::new(1, 0)),
            last_filled_price: None,
            last_filled_quantity: None,
            cumulative_filled_quantity: Decimal::ZERO,
            remaining_quantity: amend.quantity.unwrap_or(Decimal::new(1, 0)),
            commission: None,
            trade_id: None,
            reject_reason: None,
            occurred_at: 1_700_000_000_000,
        })
    }

    async fn get_order(
        &self,
        order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        self.require_connected()?;
        let mut order = sample_order();
        order.order_id = order_id.clone();
        Ok(order)
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        self.require_connected()?;
        Ok(vec![sample_order()])
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        self.require_connected()?;
        Ok(AccountInfo {
            account_id: "mock-acc".into(),
            balances: vec![],
            can_trade: true,
            can_withdraw: false,
            can_deposit: true,
            update_time: 1_700_000_000_000,
        })
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        self.require_connected()?;
        Ok(vec![])
    }

    async fn get_order_book(
        &self,
        instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        self.require_connected()?;
        Ok(OrderBook {
            instrument: instrument.clone(),
            bids: vec![],
            asks: vec![],
            sequence: Some(1),
            first_update_id: None,
            last_update_id: None,
            timestamp: 1_700_000_000_000,
            update_type: domain_market::OrderBookUpdateType::Snapshot,
        })
    }
}

/// REST-only mock（Coinglass 风格）：WS/交易方法返回 Unsupported，不伪装 Network。
struct RestOnlyMock {
    connected: AtomicBool,
}

impl RestOnlyMock {
    fn new() -> Self {
        Self { connected: AtomicBool::new(false) }
    }
}

#[async_trait]
impl VenueAdapter for RestOnlyMock {
    async fn connect(&self) -> Result<(), AdapterError> {
        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn subscribe_ticker(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported("REST-only: WebSocket ticker 不支持".into()))
    }

    async fn subscribe_order_book(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported("REST-only: WebSocket depth 不支持".into()))
    }

    async fn subscribe_trades(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported("REST-only: WebSocket trades 不支持".into()))
    }

    async fn place_order(&self, _: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 交易不支持".into()))
    }

    async fn cancel_order(&self, _: &OrderId, _: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 撤单不支持".into()))
    }

    async fn amend_order(&self, _: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 改单不支持".into()))
    }

    async fn get_order(&self, _: &OrderId, _: &InstrumentKey) -> Result<Order, AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 订单查询不支持".into()))
    }

    async fn get_open_orders(&self, _: &InstrumentKey) -> Result<Vec<Order>, AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 未结订单查询不支持".into()))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Unsupported("REST-only: 账户不支持".into()))
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        // 公开 REST 元数据允许
        Ok(vec![])
    }

    async fn get_order_book(
        &self,
        instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        // REST 深度快照允许
        Ok(OrderBook {
            instrument: instrument.clone(),
            bids: vec![],
            asks: vec![],
            sequence: None,
            first_update_id: None,
            last_update_id: None,
            timestamp: 1_700_000_000_000,
            update_type: domain_market::OrderBookUpdateType::Snapshot,
        })
    }
}

#[tokio::test]
async fn de_life_connect_disconnect_idempotent_and_gate() {
    let adapter = StatefulMock::new();
    let ik = sample_instrument();

    // 未连接 → 拒绝
    let err = adapter.subscribe_ticker(&ik).await.expect_err("gated");
    assert!(matches!(err, AdapterError::InvalidRequest(_)));
    assert_eq!(err.to_string(), "Invalid request: not connected");

    // connect 幂等
    adapter.connect().await.expect("connect");
    adapter.connect().await.expect("connect again");
    assert_eq!(adapter.connect_count.load(Ordering::SeqCst), 2);
    assert!(adapter.connected.load(Ordering::SeqCst));

    adapter.subscribe_ticker(&ik).await.expect("sub after connect");
    let report = adapter.place_order(&sample_order()).await.expect("place");
    assert_eq!(report.order_id, "o1");
    assert_eq!(report.exec_type, ExecType::New);

    let book = adapter.get_order_book(&ik, Some(20)).await.expect("order book");
    assert_eq!(book.instrument.symbol, "BTCUSDT");

    adapter.disconnect().await.expect("disconnect");
    adapter.disconnect().await.expect("disconnect again");
    assert!(!adapter.connected.load(Ordering::SeqCst));
    let err = adapter.get_account_info().await.expect_err("after disconnect");
    assert!(matches!(err, AdapterError::InvalidRequest(_)));
}

#[tokio::test]
async fn de_life_all_thirteen_methods_reachable() {
    let adapter = Arc::new(StatefulMock::new());
    adapter.connect().await.unwrap();
    let ik = sample_instrument();
    let order = sample_order();

    adapter.subscribe_ticker(&ik).await.unwrap();
    adapter.subscribe_order_book(&ik).await.unwrap();
    adapter.subscribe_trades(&ik).await.unwrap();
    adapter.place_order(&order).await.unwrap();
    adapter.cancel_order(&order.order_id, &ik).await.unwrap();
    adapter
        .amend_order(&OrderAmend {
            order_id: order.order_id.clone(),
            price: Some(Decimal::new(51000, 0)),
            quantity: None,
            stop_price: None,
            new_client_order_id: None,
        })
        .await
        .unwrap();
    let got = adapter.get_order(&order.order_id, &ik).await.unwrap();
    assert_eq!(got.order_id, "o1");
    assert_eq!(adapter.get_open_orders(&ik).await.unwrap().len(), 1);
    assert!(adapter.get_account_info().await.unwrap().can_trade);
    assert!(adapter.get_instruments().await.unwrap().is_empty());
    adapter.get_order_book(&ik, None).await.unwrap();
    adapter.disconnect().await.unwrap();
}

#[tokio::test]
async fn de_rest_only_returns_unsupported_not_network() {
    let adapter = RestOnlyMock::new();
    adapter.connect().await.unwrap();
    let ik = sample_instrument();

    for (label, result) in [
        ("ticker", adapter.subscribe_ticker(&ik).await.err()),
        ("book_ws", adapter.subscribe_order_book(&ik).await.err()),
        ("trades", adapter.subscribe_trades(&ik).await.err()),
        ("place", adapter.place_order(&sample_order()).await.err()),
        ("cancel", adapter.cancel_order(&"o1".into(), &ik).await.err()),
        (
            "amend",
            adapter
                .amend_order(&OrderAmend {
                    order_id: "o1".into(),
                    price: Some(Decimal::new(1, 0)),
                    quantity: None,
                    stop_price: None,
                    new_client_order_id: None,
                })
                .await
                .err(),
        ),
        ("get_order", adapter.get_order(&"o1".into(), &ik).await.err()),
        ("open_orders", adapter.get_open_orders(&ik).await.err()),
        ("account", adapter.get_account_info().await.err()),
    ] {
        let err = result.unwrap_or_else(|| panic!("{label} must be Err"));
        let s = err.to_string();
        assert!(s.starts_with("Unsupported:"), "{label}: {s}");
        match &err {
            AdapterError::Unsupported(msg) => {
                assert!(msg.contains("REST-only") || msg.contains("不支持"), "{label}: {msg}");
            }
            other => panic!("{label}: expected Unsupported, got {other}"),
        }
    }

    // REST 允许路径
    adapter.get_instruments().await.expect("instruments ok");
    adapter.get_order_book(&ik, Some(5)).await.expect("rest depth ok");
}

#[test]
fn de_api_adapter_error_display_all_variants() {
    let cases = [
        (AdapterError::InvalidRequest("x".into()), "Invalid request: x"),
        (AdapterError::Authentication("x".into()), "Authentication error: x"),
        (AdapterError::RateLimit("x".into()), "Rate limit: x"),
        (
            AdapterError::rate_limit_detailed(domain_exchange::RateLimitDetail {
                message: "x".into(),
                retry_after_ms: None,
                scope: None,
                provider_code: None,
                http_status: None,
                request_id: None,
            }),
            "Rate limit: x",
        ),
        (AdapterError::Network("x".into()), "Network error: x"),
        (AdapterError::WebSocket("x".into()), "WebSocket error: x"),
        (AdapterError::Parse("x".into()), "Parse error: x"),
        (AdapterError::Internal("x".into()), "Internal error: x"),
        (AdapterError::Unsupported("x".into()), "Unsupported: x"),
    ];
    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected);
    }
}

#[test]
fn de_api_stream_type_serde_camel_case() {
    let v = serde_json::to_value(StreamType::MiniTicker).unwrap();
    assert_eq!(v, "miniTicker");
    let back: StreamType = serde_json::from_value(v).unwrap();
    assert_eq!(back, StreamType::MiniTicker);
}
