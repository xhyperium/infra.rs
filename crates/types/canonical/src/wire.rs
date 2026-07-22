//! Committed wire 策略（生产边界）。
//!
//! 下列类型在本 crate 内声明 **Committed wire** 冻结面；按增量批次分版本常量：
//! - **v1**：执行路径基线（cancel / ref / ack / 枚举）
//! - **v1.1**：[`crate::Order`]
//! - **v1.2**：[`crate::Tick`] / [`crate::Trade`]
//! - **v1.3**：[`crate::Position`] / [`crate::OrderBookSnapshot`] /
//!   [`crate::PriceLevel`] / [`crate::SymbolMeta`]
//!
//! 未列入清单的类型仍为 Uncommitted（可演进、无跨版本承诺）。
//!
//! # Committed 清单
//!
//! ## v1
//! - [`crate::CancelOrderRequest`]
//! - [`crate::OrderRef`]
//! - [`crate::OrderAck`]（legacy shape 冻结）
//! - [`crate::OrderStatus`]（variant 名 = wire 字符串）
//! - [`crate::Side`]（variant 名 = wire 字符串）
//!
//! ## v1.1
//! - [`crate::Order`]
//!
//! ## v1.2
//! - [`crate::Tick`]
//! - [`crate::Trade`]
//!
//! ## v1.3
//! - [`crate::Position`]
//! - [`crate::OrderBookSnapshot`]
//! - [`crate::PriceLevel`]
//! - [`crate::SymbolMeta`]
//!
//! # 冻结策略
//! - 字段名：JSON object 键（serde 默认 rename 无）
//! - 枚举：外部 tagging 的 Rust variant 名（`{"Exchange":"..."}` / `"Open"`）
//! - 未知字段：committed 类型 `deny_unknown_fields`
//! - 未知 variant：反序列化失败（拒绝样例覆盖）
//! - 缺省：无字段默认；缺字段反序列化失败
//! - Decimal / Price / Qty：非法 `scale` 反序列化失败（走 decimalx 校验）
//! - 版本：无 envelope；N-1 兼容靠 fixture + 本模块测试；破坏性变更须新类型/显式迁移
//! - 时间：`ts: i64` = Unix epoch **纳秒**（CAN-TIME-001）
//!
//! `Money` 的 wire SSOT 在 decimalx；本 crate 仅 re-export，不单独承诺。

/// Wire 承诺等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireCommitment {
    /// 已冻结：字段/枚举/未知输入策略有测试与 fixture（含 v1 / v1.1 / v1.2 / v1.3）。
    CommittedV1,
    /// 未承诺：仅内部 DTO / 演进中。
    Uncommitted,
}

/// 已承诺 DTO shape 的精确版本标签。
///
/// 该标签描述本 crate 内的 serde JSON shape 批次，不代表 canonical bytes、通用 codec
/// 或跨语言协议版本。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireVersion {
    major: u16,
    minor: u16,
}

impl WireVersion {
    /// v1 基线。
    pub const V1: Self = Self { major: 1, minor: 0 };
    /// v1.1 批次。
    pub const V1_1: Self = Self { major: 1, minor: 1 };
    /// v1.2 批次。
    pub const V1_2: Self = Self { major: 1, minor: 2 };
    /// v1.3 批次。
    pub const V1_3: Self = Self { major: 1, minor: 3 };

    /// 主版本号。
    pub const fn major(self) -> u16 {
        self.major
    }

    /// 次版本号。
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Committed wire v1 类型名清单（执行路径基线）。
pub const COMMITTED_WIRE_V1: &[&str] =
    &["CancelOrderRequest", "OrderRef", "OrderAck", "OrderStatus", "Side"];

/// Committed wire v1.1：订单全量 DTO。
pub const COMMITTED_WIRE_V1_1: &[&str] = &["Order"];

/// Committed wire v1.2：行情与成交。
pub const COMMITTED_WIRE_V1_2: &[&str] = &["Tick", "Trade"];

/// Committed wire v1.3：持仓、订单簿与标的元数据。
pub const COMMITTED_WIRE_V1_3: &[&str] =
    &["Position", "OrderBookSnapshot", "PriceLevel", "SymbolMeta"];

/// 查询类型名对应的精确 committed shape 版本。
#[must_use]
pub fn committed_wire_version(type_name: &str) -> Option<WireVersion> {
    if COMMITTED_WIRE_V1.contains(&type_name) {
        Some(WireVersion::V1)
    } else if COMMITTED_WIRE_V1_1.contains(&type_name) {
        Some(WireVersion::V1_1)
    } else if COMMITTED_WIRE_V1_2.contains(&type_name) {
        Some(WireVersion::V1_2)
    } else if COMMITTED_WIRE_V1_3.contains(&type_name) {
        Some(WireVersion::V1_3)
    } else {
        None
    }
}

/// 查询类型名是否在任一 committed 清单中。
pub fn wire_commitment(type_name: &str) -> WireCommitment {
    if committed_wire_version(type_name).is_some() {
        WireCommitment::CommittedV1
    } else {
        WireCommitment::Uncommitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CancelOrderRequest, Order, OrderAck, OrderBookSnapshot, OrderRef, OrderStatus, Position,
        PriceLevel, Side, SymbolMeta, Tick, Trade,
    };
    use decimalx::{Decimal, MAX_SCALE, Price, Qty};
    use serde_json::json;

    fn price(v: i128) -> Price {
        Price::new(Decimal::new(v, 0))
    }

    fn qty(v: i128) -> Qty {
        Qty::new(Decimal::new(v, 0))
    }

    fn illegal_scale_price_json() -> String {
        format!(r#"{{"mantissa":1,"scale":{}}}"#, MAX_SCALE + 1)
    }

    #[test]
    fn committed_inventory_is_explicit() {
        // v1 基线
        assert_eq!(wire_commitment("CancelOrderRequest"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderRef"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderAck"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderStatus"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("Side"), WireCommitment::CommittedV1);
        // v1.1
        assert_eq!(wire_commitment("Order"), WireCommitment::CommittedV1);
        // v1.2
        assert_eq!(wire_commitment("Tick"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("Trade"), WireCommitment::CommittedV1);
        // v1.3
        assert_eq!(wire_commitment("Position"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderBookSnapshot"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("PriceLevel"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("SymbolMeta"), WireCommitment::CommittedV1);
        // 非本 crate DTO / 未知名
        assert_eq!(wire_commitment("Money"), WireCommitment::Uncommitted);
        assert_eq!(wire_commitment("NotAType"), WireCommitment::Uncommitted);
        assert_eq!(COMMITTED_WIRE_V1.len(), 5);
        assert_eq!(COMMITTED_WIRE_V1_1.len(), 1);
        assert_eq!(COMMITTED_WIRE_V1_2.len(), 2);
        assert_eq!(COMMITTED_WIRE_V1_3.len(), 4);
        assert_eq!(committed_wire_version("CancelOrderRequest"), Some(WireVersion::V1));
        assert_eq!(committed_wire_version("Order"), Some(WireVersion::V1_1));
        assert_eq!(committed_wire_version("Tick"), Some(WireVersion::V1_2));
        assert_eq!(committed_wire_version("OrderBookSnapshot"), Some(WireVersion::V1_3));
        assert_eq!(committed_wire_version("NotAType"), None);
        assert_eq!(WireVersion::V1_3.major(), 1);
        assert_eq!(WireVersion::V1_3.minor(), 3);
    }

    #[test]
    fn cancel_request_rejects_unknown_fields() {
        let bad = r#"{"venue":"okx","instrument":"BTC-USDT","id":{"Exchange":"1"},"extra":true}"#;
        let err = serde_json::from_str::<CancelOrderRequest>(bad).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown field") || msg.contains("extra"),
            "must reject unknown field: {msg}"
        );
    }

    #[test]
    fn order_ref_rejects_unknown_variant() {
        let bad = r#"{"Weird":"x"}"#;
        assert!(serde_json::from_str::<OrderRef>(bad).is_err());
    }

    #[test]
    fn order_status_rejects_unknown_variant() {
        assert!(serde_json::from_str::<OrderStatus>(r#""Flying""#).is_err());
        // 合法 variant 双向 golden
        for (wire, status) in [
            ("Pending", OrderStatus::Pending),
            ("Open", OrderStatus::Open),
            ("PartiallyFilled", OrderStatus::PartiallyFilled),
            ("Filled", OrderStatus::Filled),
            ("Cancelled", OrderStatus::Cancelled),
            ("Rejected", OrderStatus::Rejected),
        ] {
            let s = serde_json::to_string(&status).unwrap();
            assert_eq!(s, format!("\"{wire}\""));
            assert_eq!(serde_json::from_str::<OrderStatus>(&s).unwrap(), status);
        }
    }

    #[test]
    fn side_rejects_unknown_variant() {
        assert!(serde_json::from_str::<Side>(r#""Hold""#).is_err());
        assert_eq!(serde_json::to_string(&Side::Buy).unwrap(), r#""Buy""#);
        assert_eq!(serde_json::to_string(&Side::Sell).unwrap(), r#""Sell""#);
    }

    #[test]
    fn order_ack_n1_legacy_fixture_and_missing_field() {
        // N-1：仅有已冻结字段的旧 fixture
        let legacy = r#"{"id":"okx:987","status":"Open","ts":7}"#;
        let ack: OrderAck = serde_json::from_str(legacy).unwrap();
        assert_eq!(ack.id, "okx:987");
        assert_eq!(ack.status, OrderStatus::Open);
        assert_eq!(ack.ts, 7);
        // 缺字段拒绝
        assert!(serde_json::from_str::<OrderAck>(r#"{"id":"x","status":"Open"}"#).is_err());
        // 未知字段拒绝
        let with_extra = r#"{"id":"x","status":"Open","ts":1,"meta":{}}"#;
        assert!(serde_json::from_str::<OrderAck>(with_extra).is_err());
    }

    #[test]
    fn cancel_request_bidirectional_golden() {
        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        let wire = r#"{"venue":"okx","instrument":"BTC-USDT","id":{"Exchange":"987"}}"#;
        assert_eq!(serde_json::to_string(&req).unwrap(), wire);
        assert_eq!(serde_json::from_str::<CancelOrderRequest>(wire).unwrap(), req);
        // 缺 id 拒绝
        assert!(
            serde_json::from_str::<CancelOrderRequest>(
                r#"{"venue":"okx","instrument":"BTC-USDT"}"#
            )
            .is_err()
        );
        let _ = json!({"unused": true});
    }

    // ── v1.1 Order ──────────────────────────────────────────────────

    #[test]
    fn order_v1_1_bidirectional_golden_and_rejects() {
        let order = Order {
            id: "hist-ord-1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: price(50_000),
            qty: qty(1),
            status: OrderStatus::Open,
        };
        let wire = r#"{"id":"hist-ord-1","symbol":"BTCUSDT","side":"Buy","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":1,"scale":0},"status":"Open"}"#;
        assert_eq!(serde_json::to_string(&order).unwrap(), wire);
        assert_eq!(serde_json::from_str::<Order>(wire).unwrap(), order);

        // N-1 historical-looking fixture（同字段集；首次冻结）
        let n1 =
            include_str!("../../../../fixtures/market/canonical/v1.1/order_legacy.json").trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<Order>(n1).unwrap(), order);

        // 未知字段
        let with_extra = r#"{"id":"hist-ord-1","symbol":"BTCUSDT","side":"Buy","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":1,"scale":0},"status":"Open","extra":1}"#;
        assert!(serde_json::from_str::<Order>(with_extra).is_err());
        // 未知枚举 variant
        let bad_side = r#"{"id":"hist-ord-1","symbol":"BTCUSDT","side":"Hold","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":1,"scale":0},"status":"Open"}"#;
        assert!(serde_json::from_str::<Order>(bad_side).is_err());
        let bad_status = r#"{"id":"hist-ord-1","symbol":"BTCUSDT","side":"Buy","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":1,"scale":0},"status":"Flying"}"#;
        assert!(serde_json::from_str::<Order>(bad_status).is_err());
        // 缺必填字段
        assert!(serde_json::from_str::<Order>(
            r#"{"id":"hist-ord-1","symbol":"BTCUSDT","side":"Buy","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":1,"scale":0}}"#
        )
        .is_err());
        // 非法 Decimal scale
        let bad_scale = format!(
            r#"{{"id":"x","symbol":"S","side":"Buy","price":{},"qty":{{"mantissa":1,"scale":0}},"status":"Open"}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<Order>(&bad_scale).is_err());
    }

    // ── v1.2 Tick / Trade ───────────────────────────────────────────

    #[test]
    fn tick_v1_2_bidirectional_golden_and_rejects() {
        let tick = Tick {
            symbol: "BTCUSDT".into(),
            bid: price(49_999),
            ask: price(50_001),
            ts: 1_700_000_000_123_000_000,
        };
        let wire = r#"{"symbol":"BTCUSDT","bid":{"mantissa":49999,"scale":0},"ask":{"mantissa":50001,"scale":0},"ts":1700000000123000000}"#;
        assert_eq!(serde_json::to_string(&tick).unwrap(), wire);
        assert_eq!(serde_json::from_str::<Tick>(wire).unwrap(), tick);

        let n1 = include_str!("../../../../fixtures/market/canonical/v1.2/tick_legacy.json").trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<Tick>(n1).unwrap(), tick);

        assert!(serde_json::from_str::<Tick>(
            r#"{"symbol":"BTCUSDT","bid":{"mantissa":1,"scale":0},"ask":{"mantissa":2,"scale":0},"ts":1,"meta":{}}"#
        )
        .is_err());
        assert!(serde_json::from_str::<Tick>(
            r#"{"symbol":"BTCUSDT","bid":{"mantissa":1,"scale":0},"ask":{"mantissa":2,"scale":0}}"#
        )
        .is_err());
        let bad_scale = format!(
            r#"{{"symbol":"S","bid":{},"ask":{{"mantissa":2,"scale":0}},"ts":1}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<Tick>(&bad_scale).is_err());
    }

    #[test]
    fn trade_v1_2_bidirectional_golden_and_rejects() {
        let trade = Trade {
            symbol: "BTCUSDT".into(),
            price: price(50_000),
            qty: qty(2),
            ts: 1_700_000_000_123_000_000,
        };
        let wire = r#"{"symbol":"BTCUSDT","price":{"mantissa":50000,"scale":0},"qty":{"mantissa":2,"scale":0},"ts":1700000000123000000}"#;
        assert_eq!(serde_json::to_string(&trade).unwrap(), wire);
        assert_eq!(serde_json::from_str::<Trade>(wire).unwrap(), trade);

        let n1 =
            include_str!("../../../../fixtures/market/canonical/v1.2/trade_legacy.json").trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<Trade>(n1).unwrap(), trade);

        assert!(serde_json::from_str::<Trade>(
            r#"{"symbol":"BTCUSDT","price":{"mantissa":1,"scale":0},"qty":{"mantissa":1,"scale":0},"ts":1,"extra":true}"#
        )
        .is_err());
        assert!(serde_json::from_str::<Trade>(
            r#"{"symbol":"BTCUSDT","price":{"mantissa":1,"scale":0},"qty":{"mantissa":1,"scale":0}}"#
        )
        .is_err());
        let bad_scale = format!(
            r#"{{"symbol":"S","price":{},"qty":{{"mantissa":1,"scale":0}},"ts":1}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<Trade>(&bad_scale).is_err());
    }

    // ── v1.3 Position / OrderBook / PriceLevel / SymbolMeta ─────────

    #[test]
    fn position_v1_3_bidirectional_golden_and_rejects() {
        let pos = Position { symbol: "BTCUSDT".into(), qty: qty(3), entry_price: price(40_000) };
        let wire = r#"{"symbol":"BTCUSDT","qty":{"mantissa":3,"scale":0},"entry_price":{"mantissa":40000,"scale":0}}"#;
        assert_eq!(serde_json::to_string(&pos).unwrap(), wire);
        assert_eq!(serde_json::from_str::<Position>(wire).unwrap(), pos);

        let n1 =
            include_str!("../../../../fixtures/market/canonical/v1.3/position_legacy.json").trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<Position>(n1).unwrap(), pos);

        assert!(serde_json::from_str::<Position>(
            r#"{"symbol":"BTCUSDT","qty":{"mantissa":3,"scale":0},"entry_price":{"mantissa":40000,"scale":0},"extra":1}"#
        )
        .is_err());
        assert!(
            serde_json::from_str::<Position>(
                r#"{"symbol":"BTCUSDT","qty":{"mantissa":3,"scale":0}}"#
            )
            .is_err()
        );
        let bad_scale = format!(
            r#"{{"symbol":"S","qty":{{"mantissa":1,"scale":0}},"entry_price":{}}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<Position>(&bad_scale).is_err());
    }

    #[test]
    fn price_level_v1_3_bidirectional_golden_and_rejects() {
        let level = PriceLevel { price: price(99), qty: qty(1) };
        let wire = r#"{"price":{"mantissa":99,"scale":0},"qty":{"mantissa":1,"scale":0}}"#;
        assert_eq!(serde_json::to_string(&level).unwrap(), wire);
        assert_eq!(serde_json::from_str::<PriceLevel>(wire).unwrap(), level);

        let n1 = include_str!("../../../../fixtures/market/canonical/v1.3/price_level_legacy.json")
            .trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<PriceLevel>(n1).unwrap(), level);

        assert!(
            serde_json::from_str::<PriceLevel>(
                r#"{"price":{"mantissa":99,"scale":0},"qty":{"mantissa":1,"scale":0},"extra":0}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<PriceLevel>(r#"{"price":{"mantissa":99,"scale":0}}"#).is_err()
        );
        let bad_scale = format!(
            r#"{{"price":{},"qty":{{"mantissa":1,"scale":0}}}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<PriceLevel>(&bad_scale).is_err());
    }

    #[test]
    fn order_book_snapshot_v1_3_bidirectional_golden_and_rejects() {
        let book = OrderBookSnapshot {
            symbol: "BTCUSDT".into(),
            bids: vec![PriceLevel { price: price(99), qty: qty(1) }],
            asks: vec![PriceLevel { price: price(101), qty: qty(2) }],
            ts: 11,
        };
        let wire = r#"{"symbol":"BTCUSDT","bids":[{"price":{"mantissa":99,"scale":0},"qty":{"mantissa":1,"scale":0}}],"asks":[{"price":{"mantissa":101,"scale":0},"qty":{"mantissa":2,"scale":0}}],"ts":11}"#;
        assert_eq!(serde_json::to_string(&book).unwrap(), wire);
        assert_eq!(serde_json::from_str::<OrderBookSnapshot>(wire).unwrap(), book);

        let n1 = include_str!(
            "../../../../fixtures/market/canonical/v1.3/order_book_snapshot_legacy.json"
        )
        .trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<OrderBookSnapshot>(n1).unwrap(), book);

        assert!(
            serde_json::from_str::<OrderBookSnapshot>(
                r#"{"symbol":"BTCUSDT","bids":[],"asks":[],"ts":0,"extra":true}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<OrderBookSnapshot>(
                r#"{"symbol":"BTCUSDT","bids":[],"asks":[]}"#
            )
            .is_err()
        );
        // 嵌套 PriceLevel 非法 scale
        let bad_nested = format!(
            r#"{{"symbol":"S","bids":[{{"price":{},"qty":{{"mantissa":1,"scale":0}}}}],"asks":[],"ts":0}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<OrderBookSnapshot>(&bad_nested).is_err());
    }

    #[test]
    fn symbol_meta_v1_3_bidirectional_golden_and_rejects() {
        let meta = SymbolMeta {
            symbol: "BTCUSDT".into(),
            base: "BTC".into(),
            quote: "USDT".into(),
            tick_size: Decimal::new(1, 2),
            min_qty: qty(1),
        };
        let wire = r#"{"symbol":"BTCUSDT","base":"BTC","quote":"USDT","tick_size":{"mantissa":1,"scale":2},"min_qty":{"mantissa":1,"scale":0}}"#;
        assert_eq!(serde_json::to_string(&meta).unwrap(), wire);
        assert_eq!(serde_json::from_str::<SymbolMeta>(wire).unwrap(), meta);

        let n1 = include_str!("../../../../fixtures/market/canonical/v1.3/symbol_meta_legacy.json")
            .trim();
        assert_eq!(n1, wire);
        assert_eq!(serde_json::from_str::<SymbolMeta>(n1).unwrap(), meta);

        assert!(serde_json::from_str::<SymbolMeta>(
            r#"{"symbol":"BTCUSDT","base":"BTC","quote":"USDT","tick_size":{"mantissa":1,"scale":2},"min_qty":{"mantissa":1,"scale":0},"extra":1}"#
        )
        .is_err());
        assert!(serde_json::from_str::<SymbolMeta>(
            r#"{"symbol":"BTCUSDT","base":"BTC","quote":"USDT","tick_size":{"mantissa":1,"scale":2}}"#
        )
        .is_err());
        let bad_scale = format!(
            r#"{{"symbol":"S","base":"B","quote":"Q","tick_size":{},"min_qty":{{"mantissa":1,"scale":0}}}}"#,
            illegal_scale_price_json()
        );
        assert!(serde_json::from_str::<SymbolMeta>(&bad_scale).is_err());
    }
}
