//! DM-ENV-001：typed MarketFact / MarketSubject。

use domain_market::{MarketFact, MarketFactEnvelope, MarketSubject, OpenInterestPoint};

#[test]
fn typed_tick_fact_subject_and_round_trip() {
    let raw = include_str!("fixtures/market_fact_tick.json");
    let fact: MarketFact = serde_json::from_str(raw).expect("tick fact");
    match &fact {
        MarketFact::Tick(t) => assert_eq!(t.instrument.symbol, "BTCUSDT"),
        other => panic!("expected Tick, got {other:?}"),
    }
    assert!(matches!(
        fact.subject(),
        MarketSubject::Instrument(ref k) if k.exchange == "binance"
    ));
    let again: MarketFact = serde_json::from_value(serde_json::to_value(&fact).unwrap()).unwrap();
    assert_eq!(fact, again);
}

#[test]
fn typed_aggregate_oi_does_not_fake_instrument_key() {
    let raw = include_str!("fixtures/market_fact_oi.json");
    let fact: MarketFact = serde_json::from_str(raw).expect("oi fact");
    match fact.subject() {
        MarketSubject::Aggregate { coin, exchange } => {
            assert_eq!(coin, "BTC");
            assert!(exchange.is_some());
        }
        MarketSubject::Instrument(_) => panic!("aggregate must not be Instrument subject"),
        _ => panic!("unexpected subject variant"),
    }
}

#[test]
fn envelope_optional_sequence_compat() {
    // 无 sequence 的历史 JSON
    let json = r#"{
      "instrument":{"exchange":"okx","symbol":"BTC-USDT"},
      "source":"okx",
      "factType":"quote",
      "data":{"bid":"1"},
      "timestamp":1700000000000
    }"#;
    let env: MarketFactEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.sequence, None);

    let mut env2 = env.clone();
    env2.sequence = Some(42);
    let v = serde_json::to_value(&env2).unwrap();
    assert_eq!(v["sequence"], 42);
}

#[test]
fn open_interest_point_still_independent() {
    let raw = include_str!("fixtures/market_fact_oi.json");
    let fact: MarketFact = serde_json::from_str(raw).unwrap();
    if let MarketFact::OpenInterest(oi) = fact {
        let _: OpenInterestPoint = oi;
    } else {
        panic!("expected OI");
    }
}
