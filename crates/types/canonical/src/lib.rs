//! canonical —— /types/ 跨层共享 DTO（ADR-001，spec §4.2）。
//!
//! 只放纯数据形状，无业务逻辑。`Money`/`Decimal` 族复用自 [`decimalx`]（ADR-007）。
//!
//! # 生产就绪（诚实边界）
//!
//! - **不是** Production Ready / package stable / 跨版本 wire 全面承诺。
//! - `ts: i64` = Unix epoch **纳秒**（CAN-TIME-001 Approved 2026-07-17；与 kernel 同刻度）。
//! - 新执行接口优先 [`OrderRef`] / [`CancelOrderRequest`]（CAN-ID Approved）。
//! - 订单 wire id 为普通 [`String`]（`OrderId` 类型别名已删除）。
//! - Wire 等级见仓库
//!   `.agents/ssot/types/canonical/plan/wire-commitment-matrix.md`；
//!   validation owner 见
//!   `.agents/ssot/types/canonical/plan/validation-owners.md`。
//! - 禁止在本 crate 做业务校验、codec、hash/sign。
//! - 形状辅助：[`shape`]；时间工具：[`proposed_time`]。

pub mod proposed_time;
pub mod shape;

use decimalx::{Decimal, Price, Qty};
use serde::{Deserialize, Serialize};

// 复用 decimalx 的 Money（ADR-007：唯一定义点在 decimalx，canonical 复用）
pub use decimalx::Money;
pub use proposed_time::{
    PROPOSED_TS_UNIT, TS_UNIT, dto_ts_from_unix_millis, ns_from_unix_millis,
    proposed_dto_ts_from_unix_millis, proposed_ns_from_unix_millis, proposed_unix_millis_from_ns,
    unix_millis_from_ns,
};
pub use shape::{
    cancel_request_shape_ok, is_nonempty_token, is_plausible_instrument_id,
    is_plausible_venue_slug, order_ref_payload_nonempty,
};

/// Venue identifier string alias（CAN-ID：adapter 用 shape 校验）。
pub type VenueId = String;
/// Instrument identifier string alias. Cross-venue normalization is **not** done here.
pub type InstrumentId = String;

/// An order reference with an explicit identifier namespace.
///
/// Preferred at adapter cancel/query boundaries (wire-commitment: Committed-candidate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderRef {
    Client(String),
    Exchange(String),
}

/// Structured cancellation request; OKX needs both instrument and order ID.
///
/// Wire: Committed-candidate — see `fixtures/market/order_cancel_okx.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub venue: VenueId,
    pub instrument: InstrumentId,
    pub id: OrderRef,
}

/// 订单状态（spec §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// 买卖方向（spec §4.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// 订单 DTO（spec §4.2，ADR-001）。
///
/// `id` 为 wire 字符串；结构化引用见 [`OrderRef`]。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
    pub status: OrderStatus,
}

/// 订单确认（spec §4.2）。
///
/// Wire: Committed-legacy JSON shape until migration completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderAck {
    /// Wire order id string.
    pub id: String,
    pub status: OrderStatus,
    /// Unix epoch **nanoseconds** (CAN-TIME-001). Adapters must convert exchange ms → ns.
    pub ts: i64,
}

/// 持仓 DTO（spec §4.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub qty: Qty,
    pub entry_price: Price,
}

/// 行情快照（spec §4.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: String,
    pub bid: Price,
    pub ask: Price,
    pub ts: i64,
}

/// 价格档位（spec §4.2，OrderBookSnapshot 内部结构）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Price,
    pub qty: Qty,
}

/// 订单簿快照（仅快照结构体，不含更新/diff 逻辑，ADR-001）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub ts: i64,
}

/// 成交 DTO（spec §4.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub price: Price,
    pub qty: Qty,
    pub ts: i64,
}

/// 标的元数据（spec §4.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMeta {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub tick_size: Decimal,
    pub min_qty: Qty,
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimalx::Money as DecimalxMoney;

    fn price(v: i128) -> Price {
        Price(Decimal::new(v, 0))
    }

    fn qty(v: i128) -> Qty {
        Qty(Decimal::new(v, 0))
    }

    fn assert_roundtrip<T>(value: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, &back);
    }

    #[test]
    fn order_serde_roundtrip() {
        let o = Order {
            id: "o1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: price(50_000),
            qty: qty(1),
            status: OrderStatus::Open,
        };
        assert_roundtrip(&o);
    }

    #[test]
    fn all_public_dtos_and_enums_serde_roundtrip() {
        assert_roundtrip(&Side::Buy);
        assert_roundtrip(&Side::Sell);
        assert_roundtrip(&OrderRef::Client("c-1".into()));
        assert_roundtrip(&OrderRef::Exchange("e-1".into()));
        assert_roundtrip(&CancelOrderRequest {
            venue: "binance".into(),
            instrument: "ETH-USDT".into(),
            id: OrderRef::Client("cid".into()),
        });
        assert_roundtrip(&Order {
            id: "o2".into(),
            symbol: "ETHUSDT".into(),
            side: Side::Sell,
            price: price(3_000),
            qty: qty(2),
            status: OrderStatus::Pending,
        });
        assert_roundtrip(&OrderAck { id: "ack-1".into(), status: OrderStatus::Filled, ts: 42 });
        assert_roundtrip(&Position {
            symbol: "BTCUSDT".into(),
            qty: qty(3),
            entry_price: price(40_000),
        });
        assert_roundtrip(&Tick { symbol: "BTCUSDT".into(), bid: price(1), ask: price(2), ts: 9 });
        assert_roundtrip(&PriceLevel { price: price(10), qty: qty(5) });
        assert_roundtrip(&OrderBookSnapshot {
            symbol: "BTCUSDT".into(),
            bids: vec![PriceLevel { price: price(99), qty: qty(1) }],
            asks: vec![PriceLevel { price: price(101), qty: qty(2) }],
            ts: 11,
        });
        assert_roundtrip(&Trade {
            symbol: "BTCUSDT".into(),
            price: price(100),
            qty: qty(1),
            ts: 12,
        });
        assert_roundtrip(&SymbolMeta {
            symbol: "BTCUSDT".into(),
            base: "BTC".into(),
            quote: "USDT".into(),
            tick_size: Decimal::new(1, 2),
            min_qty: qty(1),
        });
    }

    #[test]
    fn all_order_status_variants_serde_roundtrip() {
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
    }

    #[test]
    fn order_book_snapshot_default_empty() {
        let s = OrderBookSnapshot { symbol: "BTCUSDT".into(), bids: vec![], asks: vec![], ts: 0 };
        assert!(s.bids.is_empty());
        assert_roundtrip(&s);
    }

    #[test]
    fn order_ref_and_cancel_request_have_stable_wire_shape() {
        let exchange = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        let exchange_json = serde_json::to_string(&exchange).unwrap();
        assert_eq!(
            exchange_json,
            r#"{"venue":"okx","instrument":"BTC-USDT","id":{"Exchange":"987"}}"#
        );
        assert_eq!(serde_json::from_str::<CancelOrderRequest>(&exchange_json).unwrap(), exchange);

        let client = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Client("c-987".into()),
        };
        let client_json = serde_json::to_string(&client).unwrap();
        assert_eq!(
            client_json,
            r#"{"venue":"okx","instrument":"BTC-USDT","id":{"Client":"c-987"}}"#
        );
        assert_eq!(serde_json::from_str::<CancelOrderRequest>(&client_json).unwrap(), client);
    }

    #[test]
    fn legacy_order_ack_wire_shape_remains_unchanged() {
        let ack = OrderAck { id: "okx:987".into(), status: OrderStatus::Open, ts: 7 };
        assert_eq!(
            serde_json::to_string(&ack).unwrap(),
            r#"{"id":"okx:987","status":"Open","ts":7}"#
        );
        // ts unit remains OPEN: this only freezes the current JSON field shape.
        let back: OrderAck = serde_json::from_str(r#"{"id":"okx:987","status":"Open","ts":7}"#)
            .expect("legacy ack deserialize");
        assert_eq!(back, ack);
    }

    #[test]
    fn canonical_cancel_request_matches_protocol_fixture() {
        let request = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        // The fixture is the provider-neutral protocol wire shape.  Keep both
        // directions here so a serde change cannot silently break replay input
        // while still producing a superficially valid request.
        let expected = include_str!("../../../../fixtures/market/order_cancel_okx.json");
        let wire = expected.trim();
        assert_eq!(serde_json::to_string(&request).unwrap(), wire);
        assert_eq!(serde_json::from_str::<CancelOrderRequest>(wire).unwrap(), request);
    }

    #[test]
    fn money_is_decimalx_money_type_identity() {
        // Compile-time + runtime: re-export must be the same type, not a copy.
        let m: Money = DecimalxMoney {
            amount: Decimal::new(1, 0),
            currency: "USD".parse().expect("currency"),
        };
        let as_decimalx: DecimalxMoney = m;
        assert_eq!(m, as_decimalx);
        assert_roundtrip(&m);
    }

    #[test]
    fn no_business_methods_on_dto_surface() {
        // Structural guard: DTO modules expose data only. Behavior belongs in domain.
        // If someone adds inherent methods later, this inventory documents the baseline.
        let _ = std::mem::size_of::<Order>();
        let _ = std::mem::size_of::<OrderBookSnapshot>();
        let _ = std::mem::size_of::<CancelOrderRequest>();
    }

    /// Public DTO inventory used by production-readiness docs (must stay complete).
    fn public_dto_names() -> &'static [&'static str] {
        &[
            "Money",
            "VenueId",
            "InstrumentId",
            "OrderRef",
            "CancelOrderRequest",
            "OrderStatus",
            "Side",
            "Order",
            "OrderAck",
            "Position",
            "Tick",
            "PriceLevel",
            "OrderBookSnapshot",
            "Trade",
            "SymbolMeta",
        ]
    }

    #[test]
    fn validation_owners_table_covers_all_public_dtos() {
        let table =
            include_str!("../../../../.agents/ssot/types/canonical/plan/validation-owners.md");
        for name in public_dto_names() {
            assert!(
                table.contains(&format!("`{name}`")),
                "validation-owners.md missing public type {name}"
            );
        }
        assert!(
            table.contains("只表达形状") && table.contains("CAN-VALID"),
            "validation-owners.md must restate CAN-VALID principle"
        );
    }

    #[test]
    fn wire_commitment_matrix_covers_all_public_dtos() {
        let matrix =
            include_str!("../../../../.agents/ssot/types/canonical/plan/wire-commitment-matrix.md");
        for name in public_dto_names() {
            assert!(
                matrix.contains(&format!("`{name}`")),
                "wire-commitment-matrix.md missing public type {name}"
            );
        }
        for token in ["Committed-candidate", "Committed-legacy", "Uncommitted"] {
            assert!(matrix.contains(token), "wire matrix missing grade token {token}");
        }
        // Production honesty: most DTOs remain Uncommitted
        assert!(
            matrix.contains("| `Order` | Uncommitted"),
            "Order must remain Uncommitted until golden + human review"
        );
    }

    #[test]
    fn production_upgrade_plan_and_m1_approval_packet_exist() {
        let prod =
            include_str!("../../../../.agents/ssot/types/canonical/plan/production-upgrade.md");
        let appr = include_str!(
            "../../../../.agents/ssot/types/canonical/plan/approval-packet-prod-m1.md"
        );
        assert!(prod.contains("PLAN-TYPES-CANONICAL-PROD-001"));
        assert!(
            prod.contains("Production Ready"),
            "production-upgrade.md must mention Production Ready boundary"
        );
        assert!(appr.contains("APPR-TYPES-CANONICAL-PROD-M1"));
        assert!(
            appr.contains("liukongqiang5") && appr.contains("Approve ns"),
            "M1 packet must record human T1–T4 authorization"
        );
        assert!(appr.contains("纳秒"));
        assert!(
            appr.contains("Defer") && appr.contains("stable"),
            "M1 must still defer package stable"
        );
    }

    #[test]
    fn legacy_order_ack_fixture_file_roundtrip() {
        let ack = OrderAck { id: "okx:987".into(), status: OrderStatus::Open, ts: 7 };
        let fixture = include_str!("../../../../fixtures/market/order_ack_legacy.json").trim();
        assert_eq!(serde_json::to_string(&ack).unwrap(), fixture);
        assert_eq!(serde_json::from_str::<OrderAck>(fixture).unwrap(), ack);
    }

    #[test]
    fn ts_fields_remain_raw_i64_until_can_time_approved() {
        // Structural: production path must not silently introduce Timestamp deps here.
        let ack = OrderAck {
            id: "x".into(),
            status: OrderStatus::Pending,
            ts: -1, // negative allowed at shape layer; unit is Unix ns (CAN-TIME-001)
        };
        let tick = Tick { symbol: "S".into(), bid: price(1), ask: price(2), ts: 0 };
        assert_eq!(ack.ts, -1);
        assert_eq!(tick.ts, 0);
        assert_roundtrip(&ack);
        assert_roundtrip(&tick);
    }

    #[test]
    fn golden_v1_dir_committed_candidates_match() {
        let cancel = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        let cancel_wire =
            include_str!("../../../../fixtures/market/canonical/v1/cancel_order_request_okx.json")
                .trim();
        assert_eq!(serde_json::to_string(&cancel).unwrap(), cancel_wire);
        assert_eq!(serde_json::from_str::<CancelOrderRequest>(cancel_wire).unwrap(), cancel);
        assert!(cancel_request_shape_ok(&cancel));

        let ack = OrderAck { id: "okx:987".into(), status: OrderStatus::Open, ts: 7 };
        let ack_wire =
            include_str!("../../../../fixtures/market/canonical/v1/order_ack_legacy.json").trim();
        assert_eq!(serde_json::to_string(&ack).unwrap(), ack_wire);

        let ex =
            include_str!("../../../../fixtures/market/canonical/v1/order_ref_exchange.json").trim();
        let cl =
            include_str!("../../../../fixtures/market/canonical/v1/order_ref_client.json").trim();
        assert_eq!(serde_json::from_str::<OrderRef>(ex).unwrap(), OrderRef::Exchange("987".into()));
        assert_eq!(serde_json::from_str::<OrderRef>(cl).unwrap(), OrderRef::Client("c-987".into()));
    }

    #[test]
    fn proposed_time_helpers_match_exchange_ms_pattern() {
        // Mirrors adapter pattern: kernel ns / 1_000_000 => exchange ms, reverse for DTO proposal.
        let ns = 1_700_000_000_123_000_000_i64;
        let ms = proposed_unix_millis_from_ns(ns);
        assert_eq!(ms, 1_700_000_000_123);
        assert_eq!(proposed_ns_from_unix_millis(ms), Some(ns));
        assert_eq!(TS_UNIT, "unix_epoch_nanoseconds");
    }
}
