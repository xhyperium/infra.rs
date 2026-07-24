//! DX-COMP-001：Position.status 与多资产手续费 fixture。

use domainx::{Commission, Portfolio, PositionStatus, aggregate_commissions_by_asset};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn portfolio_multi_asset_commissions_round_trip() {
    let raw = include_str!("fixtures/portfolio_multi_commission.json");
    let pf: Portfolio = serde_json::from_str(raw).expect("portfolio");
    assert_eq!(pf.positions.len(), 1);
    assert_eq!(pf.positions[0].status, Some(PositionStatus::Open));
    assert_eq!(pf.commissions.len(), 3);

    let aggregated = aggregate_commissions_by_asset(&pf.commissions);
    assert_eq!(aggregated.len(), 2);
    let bnb = aggregated.iter().find(|c| c.asset == "BNB").unwrap();
    assert_eq!(bnb.amount, Decimal::from_str("0.0015").unwrap());
    let usdt = aggregated.iter().find(|c| c.asset == "USDT").unwrap();
    assert_eq!(usdt.amount, Decimal::from_str("1.25").unwrap());

    let value = serde_json::to_value(&pf).unwrap();
    assert!(value.get("commissions").is_some());
    assert!(value["positions"][0].get("status").is_some());
    let again: Portfolio = serde_json::from_value(value).unwrap();
    assert_eq!(pf, again);
}

#[test]
fn legacy_position_without_status_deserializes() {
    let json = r#"{
      "positionId":"p","instrument":"ETHUSDT","direction":"short",
      "quantity":"2","entryPrice":"3000","currentPrice":"2900",
      "unrealizedPnl":"200","realizedPnl":"0",
      "createdAt":1700000000000,"updatedAt":1700000000000
    }"#;
    let pos: domainx::Position = serde_json::from_str(json).unwrap();
    assert_eq!(pos.status, None);
    assert_eq!(pos.direction, domainx::PositionDirection::Short);
}

#[test]
fn aggregate_commissions_empty() {
    assert!(aggregate_commissions_by_asset(&[]).is_empty());
    let one = [Commission { amount: Decimal::new(1, 0), asset: "BTC".into() }];
    assert_eq!(aggregate_commissions_by_asset(&one).len(), 1);
}
