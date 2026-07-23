//! taosx **集成测试**：公开 API 面（需 live TDengine）。
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo test -p taosx --test integration_all_api -- --ignored --nocapture
//! ```

use std::time::Duration;

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use futures_util::StreamExt;
use taosx::selfcheck::{CheckLevel, CheckStatus, TaosValidator};
use taosx::{
    RetryPolicy, SoakConfig, TaosConfig, TaosPool, TmqConsumer, TransportMode, WriteBatcher,
    WriteBatcherConfig, connect_native_ws, exec_sql_ws, probe_native_tcp, run_soak,
    ws_probe_totals,
};

fn sample(symbol: &str, ts: i64) -> Tick {
    Tick {
        symbol: symbol.into(),
        bid: Price::new(Decimal::try_new(10_050, 2).expect("bid")),
        ask: Price::new(Decimal::try_new(10_060, 2).expect("ask")),
        ts,
    }
}

fn table(suffix: &str) -> String {
    format!("_sc_it_{}_{suffix}", std::process::id())
}

#[tokio::test]
#[ignore = "requires live TDengine via FOUNDATIONX_TAOSX_*"]
async fn it_config_pool_health_metrics_lifecycle() {
    let cfg = TaosConfig::from_env();
    cfg.validate().expect("validate");
    assert!(!cfg.endpoint_hosts().is_empty());
    assert!(cfg.rest_sql_url().contains("/rest/sql"));
    assert!(format!("{cfg:?}").contains("***") || cfg.password.is_empty());

    let pool = TaosPool::connect(cfg).await.expect("connect");
    assert!(pool.liveness());
    pool.ping().await.expect("ping");
    let health = pool.health().await.expect("health");
    assert!(health.is_ready(), "{health:?}");
    let m = pool.metrics();
    assert!(m.ping_ok >= 1 || m.health_ready >= 1);
    let prom = pool.metrics_prometheus();
    assert!(prom.contains("taosx_ops_total"));
    let _ = pool.stats();
    let _ = pool.precision();
    let _ = pool.config();
    let client = pool.client();
    assert_eq!(client.config().host, pool.config().host);

    pool.close().await.expect("close");
    assert!(pool.is_closed());
    assert!(!pool.liveness());
}

#[tokio::test]
#[ignore = "requires live TDengine"]
async fn it_write_query_stream_batcher_retry_tmq() {
    let pool = TaosPool::connect_from_env().await.expect("connect");
    let t = table("wq");
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        as i64;
    let prec = pool.precision();
    let ts0 = prec.to_nanos(prec.from_nanos(now));
    let ts1 = prec.to_nanos(prec.from_nanos(now + 2_000_000));

    pool.write_batch(&t, &[sample("S1", ts0), sample("S2", ts1)]).await.expect("write_batch");
    let report = pool.write_batch_idempotent(&t, &[sample("S1", ts0)]).await.expect("idempotent");
    assert!(report.is_complete() || report.accepted >= 1);

    let rows = pool.query_series(&t, ts0, ts1).await.expect("query");
    assert!(!rows.is_empty());

    let mut stream = pool.query_series_stream(&t, ts0, ts1).await.expect("stream");
    let mut n = 0;
    while let Some(item) = stream.next().await {
        item.expect("row");
        n += 1;
    }
    assert!(n >= 1);

    let batcher = WriteBatcher::new(
        pool.clone(),
        format!("{t}_b"),
        WriteBatcherConfig {
            max_rows: 2,
            flush_interval: Duration::from_secs(60),
            ..Default::default()
        },
    );
    batcher.push(sample("B", ts0)).await.expect("push1");
    batcher.push(sample("B", ts1)).await.expect("push2"); // auto flush
    let r = batcher.close().await.expect("batcher close");
    assert!(r.accepted >= 1 || batcher.totals().await.0 >= 1);

    let policy = RetryPolicy::for_read();
    policy.run(|| async { pool.ping().await }).await.expect("retry ping");

    let topic = format!("_sct_it_{}", std::process::id());
    // 订阅后写入新点，确保水位轮询可见（非恒真断言）
    let mut tmq = TmqConsumer::subscribe(pool.clone(), &topic, &t).await.expect("tmq");
    let ts_new = prec.to_nanos(prec.from_nanos(now + 5_000_000));
    pool.write_series(&t, vec![sample("TMQ_NEW", ts_new)]).await.expect("tmq seed");
    let polled = tmq.poll(50).await.expect("poll");
    assert!(!polled.is_empty(), "tmq poll 在写入后必须非空; watermark 推进后应读到新行");
    assert!(polled.iter().any(|r| r.symbol == "TMQ_NEW" || r.ts >= ts0));
    tmq.close().await.expect("tmq close");

    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{t}`")).await;
    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{t}_b`")).await;
}

#[tokio::test]
#[ignore = "requires live TDengine"]
async fn it_native_ws_sql_and_selfcheck_full() {
    let mut cfg = TaosConfig::from_env();
    cfg.transport = TransportMode::NativeWs;
    // connect 会先 WS 握手再 REST SQL
    let pool = TaosPool::connect(cfg.clone()).await.expect("connect native mode");
    let body = pool.exec_sql_ws("SELECT SERVER_VERSION()").await.expect("ws sql");
    assert!(!body.is_empty(), "ws sql body 不得为空");
    // 自由函数路径：必须得到非空 Ok，或结构化错误（禁止恒真 is_ok||is_err）
    match exec_sql_ws(&cfg, "SELECT 1").await {
        Ok(b) => assert!(!b.is_empty(), "exec_sql_ws free Ok 体不得为空"),
        Err(e) => assert!(
            matches!(
                e.kind(),
                kernel::ErrorKind::Unavailable
                    | kernel::ErrorKind::DeadlineExceeded
                    | kernel::ErrorKind::Transient
                    | kernel::ErrorKind::Invalid
            ),
            "unexpected kind {:?}",
            e.kind()
        ),
    }
    connect_native_ws(&cfg).await.expect("native handshake after sql");
    let (ok, err) = ws_probe_totals();
    assert!(ok + err >= 1);

    // native tcp 探测：6041 通常可连；6030 可能未开
    let _ = probe_native_tcp(pool.config(), pool.config().port).await;

    let report = TaosValidator::new(pool.clone()).run(CheckLevel::Full).await;
    assert!(report.passed, "items={:?}", report.items);
    for item in &report.items {
        assert!(matches!(item.status, CheckStatus::Passed | CheckStatus::Degraded), "{item:?}");
    }
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "requires live TDengine"]
async fn it_soak_short() {
    let pool = TaosPool::connect_from_env().await.expect("connect");
    let cfg = SoakConfig {
        duration: Duration::from_secs(2),
        interval: Duration::from_millis(100),
        table: table("soak"),
        artifact_dir: std::path::PathBuf::from("/home/workspace/data/taosx/soak"),
    };
    let report = run_soak(&pool, cfg).await.expect("soak");
    assert!(report.iterations >= 1);
    assert!(!report.artifact.is_empty());
    assert!(std::path::Path::new(&report.artifact).is_file());
    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{}`", table("soak"))).await;
}
