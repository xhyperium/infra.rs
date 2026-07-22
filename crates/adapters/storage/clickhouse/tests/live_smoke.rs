//! ClickHouse 真实 HTTP 烟测。
//!
//! ```text
//! export FOUNDATIONX_CLICKHOUSEX_PASSWORD=...
//! cargo test -p clickhousex --test live_smoke -- --ignored --nocapture
//! ```

use bytes::Bytes;
use clickhousex::{ANALYTICS_TABLE, ClickHouseConfig, ClickHousePool};
use contract_testkit::assert_analytics_sink;

fn live_config() -> Option<ClickHouseConfig> {
    let password = std::env::var("FOUNDATIONX_CLICKHOUSEX_PASSWORD").ok()?;
    if password.is_empty() {
        return None;
    }
    let mut cfg = ClickHouseConfig::from_env().expect("clickhouse 配置");
    if cfg.password.is_empty() {
        cfg.password = password;
    }
    if cfg.database == "default" {
        // 烟测使用独立库，避免污染默认库
        cfg.database = std::env::var("FOUNDATIONX_CLICKHOUSEX_DATABASE")
            .unwrap_or_else(|_| "infra_draft".into());
    }
    Some(cfg)
}

#[tokio::test]
#[ignore = "requires live ClickHouse; set FOUNDATIONX_CLICKHOUSEX_PASSWORD"]
async fn live_create_insert_select() {
    let Some(cfg) = live_config() else {
        panic!("FOUNDATIONX_CLICKHOUSEX_PASSWORD required for live test");
    };
    let pool = ClickHousePool::connect(cfg).await.expect("connect");

    pool.execute("CREATE DATABASE IF NOT EXISTS infra_draft").await.expect("create db");
    // 切到 infra_draft：重建连接更简单
    let mut cfg = pool.config().clone();
    cfg.database = "infra_draft".into();
    let pool = ClickHousePool::connect(cfg).await.expect("reconnect");

    pool.execute(
        "CREATE TABLE IF NOT EXISTS infra_draft_smoke (\
           ts DateTime64(3) DEFAULT now64(3),\
           event String,\
           payload String\
         ) ENGINE = MergeTree ORDER BY (event, ts)",
    )
    .await
    .expect("create table");

    let marker = format!("smoke-{}", std::process::id());
    let row = serde_json::json!({
        "event": marker,
        "payload": "hello-clickhousex",
    });
    pool.insert_json_each_row("infra_draft_smoke", &[row]).await.expect("insert");

    let sql = format!(
        "SELECT event, payload FROM infra_draft_smoke WHERE event = '{}' FORMAT TabSeparated",
        marker.replace('\'', "\\'")
    );
    let text = pool.query_text(&sql).await.expect("select");
    assert!(text.contains("hello-clickhousex"), "unexpected select body: {text:?}");

    // AnalyticsSink 路径
    pool.ensure_analytics_table().await.expect("analytics ddl");
    let sink_event = format!("sink-{}", std::process::id());
    assert_analytics_sink(&pool, &sink_event, Bytes::from_static(b"payload-bytes"))
        .await
        .expect("可移植 AnalyticsSink suite");
    let check = format!(
        "SELECT payload FROM {ANALYTICS_TABLE} WHERE event = '{}' FORMAT TabSeparated",
        sink_event.replace('\'', "\\'")
    );
    let got = pool.query_text(&check).await.expect("sink select");
    assert!(got.contains("payload-bytes"), "sink select: {got:?}");

    pool.close().await.expect("close");
}

#[tokio::test]
#[ignore = "requires live ClickHouse"]
async fn live_ping() {
    let Some(cfg) = live_config() else {
        panic!("FOUNDATIONX_CLICKHOUSEX_PASSWORD required");
    };
    let pool = ClickHousePool::connect(cfg).await.expect("connect");
    pool.ping().await.expect("ping");
    pool.close().await.expect("close");
}
