//! canonical 公开面：shape/wire/time helpers + 全 DTO 构造与 committed serde。

use canonical::{
    COMMITTED_WIRE_V1, COMMITTED_WIRE_V1_1, COMMITTED_WIRE_V1_2, COMMITTED_WIRE_V1_3,
    CURRENT_PAYLOAD_SCHEMA_VERSION, CancelOrderRequest, ENVELOPE_SCHEMA_VERSION, Envelope,
    InstrumentId, Money, Order, OrderAck, OrderBookSnapshot, OrderRef, OrderStatus,
    PROPOSED_TS_UNIT, Position, PriceLevel, Side, SymbolMeta, TS_UNIT, Tick, Trade, VenueId,
    WireCommitment, cancel_request_shape_ok, dto_ts_from_unix_millis, is_nonempty_token,
    is_plausible_instrument_id, is_plausible_venue_slug, ns_from_unix_millis,
    order_ref_payload_nonempty, proposed_dto_ts_from_unix_millis, proposed_ns_from_unix_millis,
    proposed_unix_millis_from_ns, unix_millis_from_ns, wire_commitment,
};
use decimalx::{Currency, Decimal, Price, Qty};

fn assert_roundtrip<T>(value: &T)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(value, &back);
}

#[test]
fn shape_and_time_and_wire_surface() {
    assert!(is_nonempty_token("abc"));
    assert!(!is_nonempty_token(""));
    assert!(is_plausible_venue_slug("okx"));
    assert!(!is_plausible_venue_slug(""));
    assert!(is_plausible_instrument_id("BTC-USDT"));
    assert!(!is_plausible_instrument_id(""));

    let r = OrderRef::Client("c1".into());
    assert!(order_ref_payload_nonempty(&r));
    let r2 = OrderRef::Exchange("e1".into());
    assert!(order_ref_payload_nonempty(&r2));
    assert!(!order_ref_payload_nonempty(&OrderRef::Client(String::new())));

    let cancel = CancelOrderRequest { venue: "okx".into(), instrument: "BTC-USDT".into(), id: r };
    assert!(cancel_request_shape_ok(&cancel));
    assert!(!cancel_request_shape_ok(&CancelOrderRequest {
        venue: "".into(),
        instrument: "BTC-USDT".into(),
        id: OrderRef::Client("x".into()),
    }));

    assert!(TS_UNIT.contains("nano"));
    assert_eq!(PROPOSED_TS_UNIT, TS_UNIT);
    assert_eq!(ns_from_unix_millis(1), Some(1_000_000));
    assert_eq!(proposed_ns_from_unix_millis(1), Some(1_000_000));
    assert_eq!(unix_millis_from_ns(1_000_000), 1);
    assert_eq!(proposed_unix_millis_from_ns(1_000_000), 1);
    assert_eq!(dto_ts_from_unix_millis(1), Some(1_000_000));
    assert_eq!(proposed_dto_ts_from_unix_millis(1), Some(1_000_000));
    assert!(ns_from_unix_millis(i64::MAX).is_none());

    assert!(!COMMITTED_WIRE_V1.is_empty());
    assert!(!COMMITTED_WIRE_V1_1.is_empty());
    assert!(!COMMITTED_WIRE_V1_2.is_empty());
    assert!(!COMMITTED_WIRE_V1_3.is_empty());
    for name in COMMITTED_WIRE_V1
        .iter()
        .chain(COMMITTED_WIRE_V1_1.iter())
        .chain(COMMITTED_WIRE_V1_2.iter())
        .chain(COMMITTED_WIRE_V1_3.iter())
    {
        assert_eq!(wire_commitment(name), WireCommitment::CommittedV1);
    }
    assert_eq!(wire_commitment("NotAType"), WireCommitment::Uncommitted);
    assert_eq!(wire_commitment("Money"), WireCommitment::Uncommitted);

    let _vid: VenueId = "binance".into();
    let _iid: InstrumentId = "BTCUSDT".into();

    // Money re-export ≡ decimalx::Money
    let ccy = Currency::try_new(*b"USD").unwrap();
    let money = Money::try_new(Decimal::new(1, 0), ccy).unwrap();
    assert_eq!(money.amount().mantissa(), 1);
    assert_roundtrip(&money);

    // Envelope 公开面
    assert_eq!(ENVELOPE_SCHEMA_VERSION, 1);
    assert_eq!(CURRENT_PAYLOAD_SCHEMA_VERSION, 1);
    let env =
        Envelope::wrap_current(OrderAck { id: "e1".into(), status: OrderStatus::Open, ts: 9 });
    assert_eq!(env.schema_version, 1);
    assert_roundtrip(&env);
    assert_eq!(env.validate_version(1).unwrap().id, "e1");
}

#[test]
fn all_dto_construct_and_committed_serde() {
    for status in [
        OrderStatus::Pending,
        OrderStatus::Open,
        OrderStatus::PartiallyFilled,
        OrderStatus::Filled,
        OrderStatus::Cancelled,
        OrderStatus::Rejected,
    ] {
        assert_roundtrip(&status);
    }
    assert_roundtrip(&Side::Buy);
    assert_roundtrip(&Side::Sell);

    let order = Order {
        id: "1".into(),
        symbol: "S".into(),
        side: Side::Sell,
        price: Price::new(Decimal::new(1, 0)),
        qty: Qty::new(Decimal::new(1, 0)),
        status: OrderStatus::Filled,
    };
    assert_roundtrip(&order);
    assert_eq!(order.side, Side::Sell);

    let ack = OrderAck { id: "1".into(), status: OrderStatus::Filled, ts: 1 };
    assert_roundtrip(&ack);
    assert_eq!(ack.ts, 1);

    let pos = Position {
        symbol: "S".into(),
        qty: Qty::new(Decimal::new(1, 0)),
        entry_price: Price::new(Decimal::new(1, 0)),
    };
    assert_roundtrip(&pos);

    let tick = Tick {
        symbol: "S".into(),
        bid: Price::new(Decimal::new(1, 0)),
        ask: Price::new(Decimal::new(2, 0)),
        ts: 0,
    };
    assert_roundtrip(&tick);

    let level =
        PriceLevel { price: Price::new(Decimal::new(1, 0)), qty: Qty::new(Decimal::new(1, 0)) };
    assert_roundtrip(&level);

    let book = OrderBookSnapshot { symbol: "S".into(), bids: vec![level], asks: vec![], ts: 0 };
    assert_roundtrip(&book);

    let trade = Trade {
        symbol: "S".into(),
        price: Price::new(Decimal::new(1, 0)),
        qty: Qty::new(Decimal::new(1, 0)),
        ts: 0,
    };
    assert_roundtrip(&trade);

    let meta = SymbolMeta {
        symbol: "S".into(),
        base: "B".into(),
        quote: "Q".into(),
        tick_size: Decimal::new(1, 2),
        min_qty: Qty::new(Decimal::new(1, 0)),
    };
    assert_roundtrip(&meta);

    let cancel = CancelOrderRequest {
        venue: "okx".into(),
        instrument: "BTC-USDT".into(),
        id: OrderRef::Exchange("987".into()),
    };
    assert_roundtrip(&cancel);
    assert_roundtrip(&OrderRef::Client("c".into()));
    assert_roundtrip(&OrderRef::Exchange("e".into()));
}

#[test]
fn modules_reachable() {
    assert_eq!(canonical::wire::wire_commitment("Order"), WireCommitment::CommittedV1);
    assert!(canonical::shape::is_nonempty_token("x"));
    assert_eq!(canonical::proposed_time::unix_millis_from_ns(1_000_000), 1);
}
