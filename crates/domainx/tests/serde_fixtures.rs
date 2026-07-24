//! DX-API-002：版本化 JSON fixture round-trip（camelCase / Decimal / TimeInForce）。

use domainx::{ExecutionReport, Order, TimeInForce, Trade, validate_order};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn fixture_order_limit_round_trip_and_validate() {
    let raw = include_str!("fixtures/order_limit.json");
    let order: Order = serde_json::from_str(raw).expect("deserialize order fixture");
    assert_eq!(order.order_id, "ord-001");
    assert_eq!(order.side, domainx::OrderSide::Buy);
    assert_eq!(order.order_type, domainx::OrderType::Limit);
    assert_eq!(order.price, Some(Decimal::from_str("50000.00").unwrap()));
    assert_eq!(order.quantity, Decimal::from_str("1.5000").unwrap());
    // 尾随零由 Decimal 保留 scale
    assert_eq!(order.quantity.scale(), 4);

    validate_order(&order).expect("fixture order must pass DX-VAL");

    let json = serde_json::to_value(&order).expect("serialize");
    // camelCase 字段名
    assert!(json.get("orderId").is_some());
    assert!(json.get("timeInForce").is_some());
    assert!(json.get("clientOrderId").is_some());
    assert!(json.get("order_id").is_none());

    let again: Order = serde_json::from_value(json).expect("round-trip");
    assert_eq!(order, again);
}

#[test]
fn fixture_time_in_force_gtd_adjacently_tagged() {
    let raw = include_str!("fixtures/time_in_force_gtd.json");
    let tif: TimeInForce = serde_json::from_str(raw).expect("deserialize Gtd");
    assert_eq!(tif, TimeInForce::Gtd(1_700_000_001_000));

    let value = serde_json::to_value(&tif).expect("serialize");
    assert_eq!(value["type"], "Gtd");
    assert_eq!(value["value"], 1_700_000_001_000_i64);

    // 其他 TIF 变体
    for (tif, type_name) in
        [(TimeInForce::Gtc, "Gtc"), (TimeInForce::Ioc, "Ioc"), (TimeInForce::Fok, "Fok")]
    {
        let v = serde_json::to_value(&tif).unwrap();
        assert_eq!(v["type"], type_name, "tif {type_name}");
        let back: TimeInForce = serde_json::from_value(v).unwrap();
        assert_eq!(back, tif);
    }
}

#[test]
fn fixture_trade_decimal_trailing_zeros_negative_large() {
    let raw = include_str!("fixtures/trade_decimal_edge.json");
    let trade: Trade = serde_json::from_str(raw).expect("deserialize trade");

    // 负数价格（fixture 覆盖 Decimal 精度路径；业务上是否允许由上层校验）
    assert_eq!(trade.price, Decimal::from_str("-0.000001").unwrap());
    assert_eq!(trade.quantity, Decimal::from_str("12345678901234567890.123456789").unwrap());
    let commission = trade.commission.as_ref().expect("commission");
    assert_eq!(commission.amount, Decimal::from_str("0.1000").unwrap());
    assert_eq!(commission.amount.scale(), 4);

    let json = serde_json::to_string(&trade).expect("serialize");
    // 不得落到 IEEE-754 浮点 JSON number 导致精度损失
    assert!(
        json.contains("12345678901234567890.123456789")
            || json.contains("\"12345678901234567890.123456789\""),
        "large decimal must survive as decimal string, got {json}"
    );
    let again: Trade = serde_json::from_str(&json).expect("round-trip");
    assert_eq!(trade, again);
}

#[test]
fn fixture_execution_report_camel_case() {
    let raw = include_str!("fixtures/execution_report.json");
    let report: ExecutionReport = serde_json::from_str(raw).expect("deserialize execution report");
    assert_eq!(report.exec_type, domainx::ExecType::Trade);
    assert_eq!(report.order_status, domainx::OrderStatus::PartiallyFilled);
    assert_eq!(report.last_filled_quantity, Some(Decimal::from_str("0.50").unwrap()));

    let value = serde_json::to_value(&report).unwrap();
    assert!(value.get("reportId").is_some());
    assert!(value.get("execType").is_some());
    assert!(value.get("lastFilledPrice").is_some());
    assert!(value.get("cumulativeFilledQuantity").is_some());
    assert!(value.get("occurredAt").is_some());

    let again: ExecutionReport = serde_json::from_value(value).unwrap();
    assert_eq!(report, again);
}

#[test]
fn enum_variants_use_camel_case_wire_names() {
    use domainx::{ExecType, OrderSide, OrderStatus, OrderType, PositionDirection};

    let cases: &[(&str, serde_json::Value)] = &[
        ("side-buy", serde_json::to_value(OrderSide::Buy).unwrap()),
        ("type-stopMarket", serde_json::to_value(OrderType::StopMarket).unwrap()),
        ("status-partiallyFilled", serde_json::to_value(OrderStatus::PartiallyFilled).unwrap()),
        ("dir-long", serde_json::to_value(PositionDirection::Long).unwrap()),
        ("exec-tradeCancel", serde_json::to_value(ExecType::TradeCancel).unwrap()),
    ];
    for (label, value) in cases {
        let s = value.as_str().unwrap_or_else(|| panic!("{label} must be string"));
        assert!(
            s.chars().next().is_some_and(|c| c.is_lowercase()) || s.contains(char::is_uppercase),
            "{label}: unexpected wire form {s}"
        );
        // 具体 camelCase 期望
        match *label {
            "side-buy" => assert_eq!(s, "buy"),
            "type-stopMarket" => assert_eq!(s, "stopMarket"),
            "status-partiallyFilled" => assert_eq!(s, "partiallyFilled"),
            "dir-long" => assert_eq!(s, "long"),
            "exec-tradeCancel" => assert_eq!(s, "tradeCancel"),
            _ => {}
        }
    }
}
