//! Committed wire 策略（生产边界）。
//!
//! 仅下列类型在本 crate 内声明 **Committed wire v1** 冻结面；其余 DTO 仍为
//! Uncommitted（可演进、无跨版本承诺）。
//!
//! # Committed 清单（v1）
//! - [`crate::CancelOrderRequest`]
//! - [`crate::OrderRef`]
//! - [`crate::OrderAck`]（legacy shape 冻结）
//! - [`crate::OrderStatus`]（variant 名 = wire 字符串）
//! - [`crate::Side`]（variant 名 = wire 字符串）
//!
//! # 冻结策略
//! - 字段名：JSON object 键（serde 默认 rename 无）
//! - 枚举：外部 tagging 的 Rust variant 名（`{"Exchange":"..."}` / `"Open"`）
//! - 未知字段：committed 类型 `deny_unknown_fields`
//! - 未知 variant：反序列化失败（拒绝样例覆盖）
//! - 缺省：无字段默认；缺字段反序列化失败
//! - 版本：无 envelope；N-1 兼容靠 fixture + 本模块测试；破坏性变更须新类型/显式迁移
//! - 时间：`ts: i64` = Unix epoch **纳秒**（CAN-TIME-001）
//!
//! Uncommitted DTO（`Order`、`Tick`、`Trade`、`Position`、`OrderBookSnapshot`、
//! `SymbolMeta`、`PriceLevel` 等）**不**承诺 wire，仍可能携带 decimal 校验边界
//!（见 decimalx）；adapter 边界应转换为已校验 domain 值后再进业务逻辑。

/// Wire 承诺等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireCommitment {
    /// 已冻结：字段/枚举/未知输入策略有测试与 fixture。
    CommittedV1,
    /// 未承诺：仅内部 DTO / 演进中。
    Uncommitted,
}

/// Committed wire v1 类型名清单（与源码类型对应）。
pub const COMMITTED_WIRE_V1: &[&str] =
    &["CancelOrderRequest", "OrderRef", "OrderAck", "OrderStatus", "Side"];

/// 查询类型名是否在 committed v1 清单中。
pub fn wire_commitment(type_name: &str) -> WireCommitment {
    if COMMITTED_WIRE_V1.contains(&type_name) {
        WireCommitment::CommittedV1
    } else {
        WireCommitment::Uncommitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CancelOrderRequest, OrderAck, OrderRef, OrderStatus, Side};
    use serde_json::json;

    #[test]
    fn committed_inventory_is_explicit() {
        assert_eq!(wire_commitment("CancelOrderRequest"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderRef"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderAck"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("OrderStatus"), WireCommitment::CommittedV1);
        assert_eq!(wire_commitment("Side"), WireCommitment::CommittedV1);
        // Uncommitted 明示
        assert_eq!(wire_commitment("Order"), WireCommitment::Uncommitted);
        assert_eq!(wire_commitment("Tick"), WireCommitment::Uncommitted);
        assert_eq!(wire_commitment("Trade"), WireCommitment::Uncommitted);
        assert_eq!(COMMITTED_WIRE_V1.len(), 5);
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
}
