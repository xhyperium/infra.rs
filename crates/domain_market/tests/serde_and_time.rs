//! DM-SER-001 / DM-TIME-001 / DM-BOOK 纯检查：fixture + 真实类型路径。

use domain_market::{
    Bar, MarketFactEnvelope, OrderBook, Quote, Tick, validate_bar_time, validate_event_vs_received,
    validate_order_book, validate_quote_time, validate_tick_time,
};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn dm_ser_tick_fixture_round_trip_and_time() {
    let raw = include_str!("fixtures/tick.json");
    let tick: Tick = serde_json::from_str(raw).expect("tick fixture");
    assert_eq!(tick.instrument.exchange, "binance");
    assert_eq!(tick.price, Decimal::from_str("50000.00").unwrap());
    assert_eq!(tick.quantity.scale(), 4);
    validate_tick_time(&tick).expect("time gate");

    let value = serde_json::to_value(&tick).unwrap();
    assert!(value.get("receivedAt").is_some());
    assert!(value.get("tradeId").is_some());
    assert!(value.get("received_at").is_none());
    let again: Tick = serde_json::from_value(value).unwrap();
    assert_eq!(tick, again);
}

#[test]
fn dm_ser_quote_fixture_levels_and_time() {
    let raw = include_str!("fixtures/quote.json");
    let quote: Quote = serde_json::from_str(raw).expect("quote fixture");
    assert_eq!(quote.bid_price, Decimal::from_str("49999.5").unwrap());
    validate_quote_time(&quote).expect("quote time");
    let bids = quote.bid_levels.as_ref().expect("bid levels");
    assert!(bids[0].price >= bids[1].price);
    let asks = quote.ask_levels.as_ref().expect("ask levels");
    assert!(asks[0].price <= asks[1].price);

    let value = serde_json::to_value(&quote).unwrap();
    assert!(value.get("bidPrice").is_some());
    assert!(value.get("askQuantity").is_some());
    let again: Quote = serde_json::from_value(value).unwrap();
    assert_eq!(quote, again);
}

#[test]
fn dm_ser_bar_fixture_bounds() {
    let raw = include_str!("fixtures/bar.json");
    let bar: Bar = serde_json::from_str(raw).expect("bar fixture");
    // untagged interval：整数 5 → Seconds(5) 或 Minutes 取决于反序列化顺序
    // 契约：字段存在且时间边界合法；interval 具体变体由 untagged 解析决定
    validate_bar_time(&bar).expect("bar bounds");
    assert!(bar.close_time > bar.open_time);
    assert_eq!(bar.volume, Decimal::from_str("1234.5678").unwrap());

    let value = serde_json::to_value(&bar).unwrap();
    assert!(value.get("openTime").is_some());
    assert!(value.get("takerBuyVolume").is_some());
    let again: Bar = serde_json::from_value(value).unwrap();
    assert_eq!(bar, again);
}

#[test]
fn dm_book_snapshot_fixture_ordering_and_ids() {
    let raw = include_str!("fixtures/order_book_snapshot.json");
    let book: OrderBook = serde_json::from_str(raw).expect("book fixture");
    assert_eq!(book.update_type, domain_market::OrderBookUpdateType::Snapshot);
    assert_eq!(book.first_update_id, Some(1000));
    assert_eq!(book.last_update_id, Some(1010));
    validate_order_book(&book).expect("book pure checks");

    let value = serde_json::to_value(&book).unwrap();
    assert_eq!(value["updateType"], "snapshot");
    assert!(value.get("firstUpdateId").is_some());
    let again: OrderBook = serde_json::from_value(value).unwrap();
    assert_eq!(book, again);
}

#[test]
fn dm_ser_envelope_fixture() {
    let raw = include_str!("fixtures/envelope.json");
    let env: MarketFactEnvelope = serde_json::from_str(raw).expect("envelope");
    assert_eq!(env.source, domain_market::DataSource::Coinbase);
    assert_eq!(env.fact_type, "tick");
    let value = serde_json::to_value(&env).unwrap();
    assert!(value.get("factType").is_some());
    let again: MarketFactEnvelope = serde_json::from_value(value).unwrap();
    assert_eq!(env, again);
}

#[test]
fn dm_time_rejects_seconds_written_as_timestamp() {
    let err = validate_event_vs_received(1_700_000_000, 1_700_000_001).expect_err("sec");
    assert!(matches!(err, domain_market::TimeError::Unit(_)));
}

#[test]
fn dm_decimal_large_and_trailing_zeros_on_quote() {
    let mut quote: Quote =
        serde_json::from_str(include_str!("fixtures/quote.json")).expect("quote");
    quote.bid_price = Decimal::from_str("123456789012345.123456789").unwrap();
    quote.ask_price = Decimal::from_str("0.1000").unwrap();
    assert_eq!(quote.ask_price.scale(), 4);
    let json = serde_json::to_string(&quote).unwrap();
    assert!(
        json.contains("123456789012345.123456789"),
        "large decimal must not float-cast: {json}"
    );
    let again: Quote = serde_json::from_str(&json).unwrap();
    assert_eq!(again.bid_price, quote.bid_price);
    assert_eq!(again.ask_price, quote.ask_price);
}
