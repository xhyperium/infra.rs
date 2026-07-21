//! TDengine REST 真实烟测。
//!
//! ```text
//! export FOUNDATIONX_TAOSX_PASSWORD=...
//! cargo test -p taosx --test live_smoke -- --ignored --nocapture
//! ```

use canonical::Tick;
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

    let points = vec![
        sample_tick("BTCUSDT", t0, 10050, 10060),
        sample_tick("BTCUSDT", t1, 10055, 10065),
        sample_tick("ETHUSDT", t2, 350000, 350100),
    ];

    pool.write_series(&table, points).await.expect("write");

    let rows = pool.query_series(&table, t0, t2).await.expect("query");
    assert!(rows.len() >= 3, "expected >=3 ticks, got {} ({rows:?})", rows.len());
    assert!(rows.iter().any(|t| t.symbol == "BTCUSDT"));
    assert!(rows.iter().any(|t| t.symbol == "ETHUSDT"));

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
