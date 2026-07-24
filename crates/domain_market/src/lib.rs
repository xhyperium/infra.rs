//! Market data domain model: Tick, Quote, Bar, OrderBook, and canonical types.

use serde::{Deserialize, Serialize};

// Re-exported types from domainx
pub use domainx::{Decimal, OrderSide, Timestamp};

mod book;
mod time;
pub use book::{
    BookError, asks_are_ascending, bids_are_descending, deltas_are_contiguous,
    looks_like_unix_millis, validate_level_ordering, validate_order_book, validate_update_ids,
    validate_update_type_shape,
};
pub use time::{
    TimeError, validate_bar_bounds, validate_bar_time, validate_event_vs_received,
    validate_quote_time, validate_tick_time,
};

// ---------------------------------------------------------------------------
// InstrumentKey
// ---------------------------------------------------------------------------

/// 当前 workspace 的 instrument 标识（`exchange` + `symbol`）。
///
/// 唯一 canonical owner 迁移由 `DM-CAN-001` 追踪；在 `xhyper-canonical`
/// 纳入 workspace 前，本类型即为公共契约，不得另起第二套 instrument 结构。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentKey {
    /// Exchange identifier (e.g. "binance").
    pub exchange: String,
    /// Trading symbol within the exchange (e.g. "BTCUSDT").
    pub symbol: String,
}

// ---------------------------------------------------------------------------
// ProductLine
// ---------------------------------------------------------------------------

/// Product line / asset class categorisation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProductLine {
    Spot,
    Future,
    Perpetual,
    Option,
}

// ---------------------------------------------------------------------------
// Tick
// ---------------------------------------------------------------------------

/// Direction of a trade (aggressor side).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TickDirection {
    Buy,
    Sell,
    Unknown,
}

/// A single trade (tick) — represents one match event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tick {
    /// Instrument traded.
    pub instrument: InstrumentKey,
    /// Execution price.
    pub price: Decimal,
    /// Quantity traded.
    pub quantity: Decimal,
    /// Aggressor side (optional).
    pub side: Option<TickDirection>,
    /// Exchange-assigned trade identifier (optional).
    pub trade_id: Option<String>,
    /// Trade timestamp (Unix ms).
    pub timestamp: Timestamp,
    /// Local receive timestamp (Unix ms).
    pub received_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Quote / PriceLevel
// ---------------------------------------------------------------------------

/// A single price level in the order book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLevel {
    /// Price of this level.
    pub price: Decimal,
    /// Quantity available at this level.
    pub quantity: Decimal,
    /// Number of orders at this level (optional, exchange-dependent).
    pub order_count: Option<u64>,
}

/// Top-of-book quote with optional depth snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Instrument.
    pub instrument: InstrumentKey,
    /// Best bid price.
    pub bid_price: Decimal,
    /// Best bid quantity.
    pub bid_quantity: Decimal,
    /// Best ask price.
    pub ask_price: Decimal,
    /// Best ask quantity.
    pub ask_quantity: Decimal,
    /// Full bid depth levels (optional).
    pub bid_levels: Option<Vec<PriceLevel>>,
    /// Full ask depth levels (optional).
    pub ask_levels: Option<Vec<PriceLevel>>,
    /// Exchange event timestamp (Unix ms).
    pub timestamp: Timestamp,
    /// Local receive timestamp (Unix ms).
    pub received_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Bar
// ---------------------------------------------------------------------------

/// Bar interval for OHLCV aggregation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BarInterval {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
    Weeks(u64),
    Months(u64),
}

/// OHLCV bar (candle) data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    /// Instrument.
    pub instrument: InstrumentKey,
    /// Aggregation interval.
    pub interval: BarInterval,
    /// Bar open timestamp (Unix ms).
    pub open_time: Timestamp,
    /// Bar close timestamp (Unix ms).
    pub close_time: Timestamp,
    /// Open price.
    pub open: Decimal,
    /// High price.
    pub high: Decimal,
    /// Low price.
    pub low: Decimal,
    /// Close price.
    pub close: Decimal,
    /// Volume in base asset.
    pub volume: Decimal,
    /// Volume in quote asset (optional).
    pub quote_volume: Option<Decimal>,
    /// Number of trades (optional).
    pub trade_count: Option<u64>,
    /// Taker buy volume (optional).
    pub taker_buy_volume: Option<Decimal>,
    /// Taker buy quote volume (optional).
    pub taker_buy_quote_volume: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// OrderBook
// ---------------------------------------------------------------------------

/// Order book update type.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderBookUpdateType {
    Snapshot,
    Delta,
}

/// Order book snapshot or delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    /// Instrument.
    pub instrument: InstrumentKey,
    /// Bid levels (sorted descending by price).
    pub bids: Vec<PriceLevel>,
    /// Ask levels (sorted ascending by price).
    pub asks: Vec<PriceLevel>,
    /// Monotonic sequence number (optional).
    pub sequence: Option<u64>,
    /// First update ID (Binance delta integrity).
    pub first_update_id: Option<u64>,
    /// Last update ID (Binance delta integrity).
    pub last_update_id: Option<u64>,
    /// Exchange event timestamp (Unix ms).
    pub timestamp: Timestamp,
    /// Update type (snapshot or delta).
    #[serde(rename = "updateType")]
    pub update_type: OrderBookUpdateType,
}

// ---------------------------------------------------------------------------
// MarketFactEnvelope & DataSource
// ---------------------------------------------------------------------------

/// Market data source / venue.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSource {
    Binance,
    Okx,
    Bybit,
    Bitget,
    KuCoin,
    Gate,
    Mexc,
    Htx,
    Coinbase,
    Hyperliquid,
    Lighter,
    Upbit,
    Coinglass,
}

/// Unified envelope for all market fact types flowing through the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketFactEnvelope {
    /// Related instrument.
    pub instrument: InstrumentKey,
    /// Data source / venue that produced this fact.
    pub source: DataSource,
    /// Discriminant string describing the fact type (e.g. "tick", "quote", "bar").
    pub fact_type: String,
    /// Fact payload as opaque JSON value.
    pub data: serde_json::Value,
    /// Event timestamp (Unix ms).
    pub timestamp: Timestamp,
    /// 可选管道序列号（DM-ENV-001；历史 fixture 可缺省）。
    #[serde(default)]
    pub sequence: Option<u64>,
}

// ---------------------------------------------------------------------------
// DM-ENV-001：typed fact / subject（与 JSON envelope 并存）
// ---------------------------------------------------------------------------

/// 市场事实主体：交易标的 vs 跨所聚合主体。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum MarketSubject {
    /// 标准交易所标的。
    Instrument(InstrumentKey),
    /// 聚合数据主体：`coin` + 可选被聚合交易所名。
    Aggregate { coin: String, exchange: Option<String> },
}

/// 类型化市场事实（DM-ENV-001）。
///
/// 与 [`MarketFactEnvelope`] 并存：envelope 保持兼容 JSON 管道；
/// typed fact 用于库内强类型路径，禁止把聚合点强塞进单一 instrument。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum MarketFact {
    Tick(Tick),
    Quote(Quote),
    Bar(Bar),
    OrderBook(OrderBook),
    OpenInterest(OpenInterestPoint),
    FundingRate(FundingRatePoint),
    Liquidation(LiquidationData),
    LongShortRatio(LongShortRatioData),
}

impl MarketFact {
    /// 推导主体：行情类 → Instrument；聚合类 → Aggregate。
    pub fn subject(&self) -> MarketSubject {
        match self {
            MarketFact::Tick(t) => MarketSubject::Instrument(t.instrument.clone()),
            MarketFact::Quote(q) => MarketSubject::Instrument(q.instrument.clone()),
            MarketFact::Bar(b) => MarketSubject::Instrument(b.instrument.clone()),
            MarketFact::OrderBook(o) => MarketSubject::Instrument(o.instrument.clone()),
            MarketFact::OpenInterest(p) => MarketSubject::Aggregate {
                coin: p.coin.clone(),
                exchange: Some(format!("{:?}", p.exchange)),
            },
            MarketFact::FundingRate(p) => MarketSubject::Aggregate {
                coin: p.coin.clone(),
                exchange: Some(format!("{:?}", p.exchange)),
            },
            MarketFact::Liquidation(p) => MarketSubject::Aggregate {
                coin: p.coin.clone(),
                exchange: Some(format!("{:?}", p.exchange)),
            },
            MarketFact::LongShortRatio(p) => MarketSubject::Aggregate {
                coin: p.coin.clone(),
                exchange: Some(format!("{:?}", p.exchange)),
            },
        }
    }

    /// 事件时间（ms）。
    pub fn timestamp(&self) -> Timestamp {
        match self {
            MarketFact::Tick(t) => t.timestamp,
            MarketFact::Quote(q) => q.timestamp,
            MarketFact::Bar(b) => b.close_time,
            MarketFact::OrderBook(o) => o.timestamp,
            MarketFact::OpenInterest(p) => p.timestamp,
            MarketFact::FundingRate(p) => p.timestamp,
            MarketFact::Liquidation(p) => p.timestamp,
            MarketFact::LongShortRatio(p) => p.timestamp,
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-exchange aggregate data types
// ---------------------------------------------------------------------------

/// Exchange identifier for cross-exchange data.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExchangeId {
    Binance,
    Okx,
    Coinbase,
    Hyperliquid,
    Bybit,
    Bitget,
    KuCoin,
    Gate,
    Kraken,
    Bitfinex,
    Htx,
    Mexc,
    Other(String),
}

/// Open interest data point (cross-exchange).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterestPoint {
    /// Coin / asset identifier.
    pub coin: String,
    /// Exchange.
    pub exchange: ExchangeId,
    /// Open interest in contracts.
    pub oi: Decimal,
    /// Open interest notional value (USD).
    pub oi_value: Decimal,
    /// Data timestamp (Unix ms).
    pub timestamp: Timestamp,
}

/// Funding rate data point (cross-exchange).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingRatePoint {
    /// Coin / asset identifier.
    pub coin: String,
    /// Exchange.
    pub exchange: ExchangeId,
    /// Signed funding rate (e.g. 0.0001).
    pub rate: Decimal,
    /// Data timestamp (Unix ms).
    pub timestamp: Timestamp,
}

/// Side of a liquidation event.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiquidationSide {
    Long,
    Short,
}

/// Liquidation data point (cross-exchange).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationData {
    /// Coin / asset identifier.
    pub coin: String,
    /// Exchange.
    pub exchange: ExchangeId,
    /// Liquidation side.
    pub side: LiquidationSide,
    /// Liquidation amount (USD).
    pub amount: Decimal,
    /// Liquidation price.
    pub price: Decimal,
    /// Data timestamp (Unix ms).
    pub timestamp: Timestamp,
}

/// Long / short ratio data point (cross-exchange).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongShortRatioData {
    /// Coin / asset identifier.
    pub coin: String,
    /// Exchange.
    pub exchange: ExchangeId,
    /// Long ratio (percentage).
    pub long_ratio: Decimal,
    /// Short ratio (percentage).
    pub short_ratio: Decimal,
    /// Data timestamp (Unix ms).
    pub timestamp: Timestamp,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_instrument() -> InstrumentKey {
        InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() }
    }

    #[test]
    fn test_instrument_key() {
        let ik = sample_instrument();
        assert_eq!(ik.exchange, "binance");
        assert_eq!(ik.symbol, "BTCUSDT");
    }

    #[test]
    fn test_tick_creation() {
        let tick = Tick {
            instrument: sample_instrument(),
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            side: Some(TickDirection::Buy),
            trade_id: Some("t1".into()),
            timestamp: 1_000_000_000_000,
            received_at: 1_000_000_000_001,
        };
        assert_eq!(tick.side, Some(TickDirection::Buy));
    }

    #[test]
    fn test_tick_serialization() {
        let tick = Tick {
            instrument: sample_instrument(),
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            side: None,
            trade_id: None,
            timestamp: 1_000_000_000_000,
            received_at: 1_000_000_000_001,
        };
        let json = serde_json::to_string(&tick).expect("serialize tick");
        let deserialized: Tick = serde_json::from_str(&json).expect("deserialize tick");
        assert_eq!(tick, deserialized);
    }

    #[test]
    fn test_quote_with_levels() {
        let quote = Quote {
            instrument: sample_instrument(),
            bid_price: Decimal::new(49900, 0),
            bid_quantity: Decimal::new(10, 0),
            ask_price: Decimal::new(50100, 0),
            ask_quantity: Decimal::new(5, 0),
            bid_levels: Some(vec![
                PriceLevel {
                    price: Decimal::new(49900, 0),
                    quantity: Decimal::new(10, 0),
                    order_count: Some(1),
                },
                PriceLevel {
                    price: Decimal::new(49800, 0),
                    quantity: Decimal::new(20, 0),
                    order_count: Some(2),
                },
            ]),
            ask_levels: Some(vec![PriceLevel {
                price: Decimal::new(50100, 0),
                quantity: Decimal::new(5, 0),
                order_count: Some(1),
            }]),
            timestamp: 1_000_000_000_000,
            received_at: 1_000_000_000_001,
        };
        assert_eq!(quote.bid_levels.as_ref().unwrap().len(), 2);
        assert_eq!(quote.ask_levels.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_bar() {
        let bar = Bar {
            instrument: sample_instrument(),
            interval: BarInterval::Minutes(5),
            open_time: 1_000_000_000_000,
            close_time: 1_000_000_300_000,
            open: Decimal::new(50000, 0),
            high: Decimal::new(50100, 0),
            low: Decimal::new(49900, 0),
            close: Decimal::new(50050, 0),
            volume: Decimal::new(1000, 0),
            quote_volume: Some(Decimal::new(50_000_000, 0)),
            trade_count: Some(500),
            taker_buy_volume: Some(Decimal::new(600, 0)),
            taker_buy_quote_volume: Some(Decimal::new(30_000_000, 0)),
        };
        assert_eq!(bar.interval, BarInterval::Minutes(5));
        assert!(bar.close > bar.open);
    }

    #[test]
    fn test_order_book() {
        let ob = OrderBook {
            instrument: sample_instrument(),
            bids: vec![PriceLevel {
                price: Decimal::new(49900, 0),
                quantity: Decimal::new(10, 0),
                order_count: None,
            }],
            asks: vec![PriceLevel {
                price: Decimal::new(50100, 0),
                quantity: Decimal::new(5, 0),
                order_count: None,
            }],
            sequence: Some(12345),
            first_update_id: Some(100),
            last_update_id: Some(200),
            timestamp: 1_000_000_000_000,
            update_type: OrderBookUpdateType::Snapshot,
        };
        assert_eq!(ob.bids.len(), 1);
        assert_eq!(ob.asks.len(), 1);
    }

    #[test]
    fn test_market_fact_envelope() {
        let envelope = MarketFactEnvelope {
            instrument: sample_instrument(),
            source: DataSource::Binance,
            fact_type: "tick".into(),
            data: serde_json::json!({"price": "50000", "quantity": "1"}),
            timestamp: 1_000_000_000_000,
            sequence: None,
        };
        let json = serde_json::to_string(&envelope).expect("serialize envelope");
        let deserialized: MarketFactEnvelope =
            serde_json::from_str(&json).expect("deserialize envelope");
        assert_eq!(envelope, deserialized);
    }

    #[test]
    fn test_cross_exchange_types() {
        let oi = OpenInterestPoint {
            coin: "BTC".into(),
            exchange: ExchangeId::Binance,
            oi: Decimal::new(100_000, 0),
            oi_value: Decimal::new(5_000_000_000, 0),
            timestamp: 1_000_000_000_000,
        };
        assert_eq!(oi.coin, "BTC");

        let funding = FundingRatePoint {
            coin: "ETH".into(),
            exchange: ExchangeId::Bybit,
            rate: Decimal::new(1, 4),
            timestamp: 1_000_000_000_000,
        };
        assert!(funding.rate > Decimal::ZERO);

        let liq = LiquidationData {
            coin: "BTC".into(),
            exchange: ExchangeId::Okx,
            side: LiquidationSide::Long,
            amount: Decimal::new(1_000_000, 0),
            price: Decimal::new(48000, 0),
            timestamp: 1_000_000_000_000,
        };
        assert_eq!(liq.side, LiquidationSide::Long);

        let lsr = LongShortRatioData {
            coin: "BTC".into(),
            exchange: ExchangeId::Hyperliquid,
            long_ratio: Decimal::new(55, 0),
            short_ratio: Decimal::new(45, 0),
            timestamp: 1_000_000_000_000,
        };
        assert!(lsr.long_ratio > lsr.short_ratio);

        // Round-trip serialization
        let json = serde_json::to_string(&oi).expect("serialize oi");
        let _: OpenInterestPoint = serde_json::from_str(&json).expect("deserialize oi");
    }

    #[test]
    fn test_product_line() {
        match ProductLine::Spot {
            ProductLine::Spot
            | ProductLine::Future
            | ProductLine::Perpetual
            | ProductLine::Option => {}
        }
    }
}
