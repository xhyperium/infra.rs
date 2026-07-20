//! 库外消费者视角的公开 API 契约（integration）。
//!
//! 证明 `xhyper-canonical` 作为 workspace 成员可被独立 crate 引用，
//! 且关键 DTO/helper 行为与 SSOT §2–§3 一致。

use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderRef, OrderStatus, Position,
    PriceLevel, Side, SymbolMeta, TS_UNIT, Tick, Trade, VenueId, cancel_request_shape_ok,
    dto_ts_from_unix_millis, is_plausible_venue_slug, ns_from_unix_millis, unix_millis_from_ns,
};
use decimalx::{Decimal, Price, Qty};

#[test]
fn consumer_can_construct_and_roundtrip_core_dtos() {
    let cancel = CancelOrderRequest {
        venue: "okx".into(),
        instrument: "BTC-USDT".into(),
        id: OrderRef::Exchange("987".into()),
    };
    let json = serde_json::to_string(&cancel).expect("serialize");
    let back: CancelOrderRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, cancel);
    assert!(cancel_request_shape_ok(&cancel));
    assert!(is_plausible_venue_slug("okx"));

    let order = Order {
        id: "o1".into(),
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        price: Price(Decimal::new(50_000, 0)),
        qty: Qty(Decimal::new(1, 0)),
        status: OrderStatus::Open,
    };
    let ojson = serde_json::to_string(&order).expect("order serialize");
    let order_back: Order = serde_json::from_str(&ojson).expect("order deserialize");
    assert_eq!(order_back, order);

    let ack = OrderAck { id: "okx:987".into(), status: OrderStatus::Open, ts: 7 };
    assert_eq!(
        serde_json::to_string(&ack).expect("ack"),
        r#"{"id":"okx:987","status":"Open","ts":7}"#
    );
}

#[test]
fn consumer_time_helpers_are_nanoseconds() {
    assert!(TS_UNIT.contains("nano"));
    assert_eq!(ns_from_unix_millis(1000), Some(1_000_000_000));
    assert_eq!(unix_millis_from_ns(1_000_000_000), 1000);
    assert_eq!(dto_ts_from_unix_millis(1), Some(1_000_000));
    assert!(ns_from_unix_millis(i64::MAX).is_none());
}

#[test]
fn consumer_money_is_decimalx_reexport() {
    let m: Money = Money { amount: Decimal::new(1, 0), currency: "USD".parse().expect("currency") };
    let as_decimalx: decimalx::Money = m;
    assert_eq!(m, as_decimalx);
}

#[test]
fn consumer_inventory_types_constructible() {
    let _venue: VenueId = "binance".into();
    let _pos = Position {
        symbol: "ETHUSDT".into(),
        qty: Qty(Decimal::new(2, 0)),
        entry_price: Price(Decimal::new(3000, 0)),
    };
    let _tick = Tick {
        symbol: "ETHUSDT".into(),
        bid: Price(Decimal::new(1, 0)),
        ask: Price(Decimal::new(2, 0)),
        ts: 0,
    };
    let _level = PriceLevel { price: Price(Decimal::new(10, 0)), qty: Qty(Decimal::new(5, 0)) };
    let _book = OrderBookSnapshot { symbol: "ETHUSDT".into(), bids: vec![], asks: vec![], ts: 0 };
    let _trade = Trade {
        symbol: "ETHUSDT".into(),
        price: Price(Decimal::new(100, 0)),
        qty: Qty(Decimal::new(1, 0)),
        ts: 12,
    };
    let _meta = SymbolMeta {
        symbol: "ETHUSDT".into(),
        base: "ETH".into(),
        quote: "USDT".into(),
        tick_size: Decimal::new(1, 2),
        min_qty: Qty(Decimal::new(1, 0)),
    };
    let _client = OrderRef::Client("c-1".into());
    for status in [
        OrderStatus::Pending,
        OrderStatus::Open,
        OrderStatus::PartiallyFilled,
        OrderStatus::Filled,
        OrderStatus::Cancelled,
        OrderStatus::Rejected,
    ] {
        let _ = serde_json::to_string(&status).expect("status");
    }
}
