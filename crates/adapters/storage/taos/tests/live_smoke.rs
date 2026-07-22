//! TDengine REST 真实烟测。
//!
//! ```text
//! export FOUNDATIONX_TAOSX_PASSWORD=...
//! cargo test -p taosx --test live_smoke -- --ignored --nocapture
//! ```

use canonical::Tick;
use contract_testkit::assert_time_series_store;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::{TaosConfig, TaosPool};

fn live_config() -> Option<TaosConfig> {
    let password = std::env::var("FOUNDATIONX_TAOSX_PASSWORD").ok()?;
    if password.is_empty() {
        return None;
    }
    let mut cfg = TaosConfig::from_env();
    if cfg.password.is_empty() {
        cfg.password = password;
    }
    Some(cfg)
}

fn sample_tick(symbol: &str, ts_ns: i64, bid: i128, ask: i128) -> Tick {
    Tick {
        symbol: symbol.into(),
        bid: Price::new(Decimal::try_new(bid, 2).expect("bid")),
        ask: Price::new(Decimal::try_new(ask, 2).expect("ask")),
        ts: ts_ns,
    }
}

#[tokio::test]
#[ignore = "requires live TDengine REST; set FOUNDATIONX_TAOSX_PASSWORD"]
async fn live_write_query_ticks() {
    let Some(cfg) = live_config() else {
        panic!("FOUNDATIONX_TAOSX_PASSWORD required for live test");
    };
    let pool = TaosPool::connect(cfg).await.expect("connect");
    let table = format!("infra_draft_ticks_{}", std::process::id());

    // 使用当前时间附近的纳秒（库精度可能是 ms/ns）
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos() as i64;
    let t0 = now_ns - 2_000_000_000; // -2s
    let t1 = now_ns - 1_000_000_000;
    let t2 = now_ns;
    let exact_bid =
        Decimal::try_new(123_456_789_012_345_678_901_234_567_890_123_456, 18).expect("exact bid");
    let exact_ask =
        Decimal::try_new(-123_456_789_012_345_678_901_234_567_890_123_455, 18).expect("exact ask");
    let exact_ts = now_ns + 1_000_000_000;

    let mut points = vec![
        sample_tick("BTCUSDT", t0, 10050, 10060),
        sample_tick("BTCUSDT", t1, 10055, 10065),
        sample_tick("ETHUSDT", t2, 350000, 350100),
    ];
    points.push(Tick {
        symbol: "DECIMAL_EXACT".into(),
        bid: Price::new(exact_bid),
        ask: Price::new(exact_ask),
        ts: exact_ts,
    });

    let suite_table = format!("{table}_contract");
    let mut suite_tick = points[0].clone();
    let precision = pool.precision();
    suite_tick.ts = precision.to_nanos(precision.from_nanos(suite_tick.ts));
    assert_time_series_store(&pool, &suite_table, suite_tick)
        .await
        .expect("可移植 TimeSeriesStore suite");

    pool.write_series(&table, points).await.expect("write");

    let rows = pool.query_series(&table, t0, exact_ts).await.expect("query");
    assert!(rows.len() >= 4, "expected >=4 ticks, got {} ({rows:?})", rows.len());
    assert!(rows.iter().any(|t| t.symbol == "BTCUSDT"));
    assert!(rows.iter().any(|t| t.symbol == "ETHUSDT"));
    let exact = rows.iter().find(|tick| tick.symbol == "DECIMAL_EXACT").expect("exact decimal row");
    assert_eq!(exact.bid.as_decimal(), exact_bid);
    assert_eq!(exact.ask.as_decimal(), exact_ask);

    // 窄范围只命中中间点附近
    let mid = pool.query_series(&table, t1 - 100, t1 + 100).await.expect("mid");
    assert!(mid.iter().any(|t| t.symbol == "BTCUSDT"), "mid range should include BTCUSDT: {mid:?}");

    pool.close().await.expect("close");
}

#[tokio::test]
#[ignore = "requires live TDengine REST"]
async fn live_ping() {
    let Some(cfg) = live_config() else {
        panic!("FOUNDATIONX_TAOSX_PASSWORD required");
    };
    let pool = TaosPool::connect(cfg).await.expect("connect");
    pool.ping().await.expect("ping");
}
