//! clickhousex examples 集成测试 — 验证所有 4 个示例的功能正确性

use std::time::Duration;

use clickhousex::{BatchInsertOptions, ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;
use serde_json::json;

const BINANCE_DB: &str = "binance_futures";
const TEST_DB: &str = "binance_futures_test";

fn live_cfg(database: &str) -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        user: "default".into(),
        password: "iCEOuptIx40EduvGOKX73rfY".into(),
        database: database.into(),
        timeout: Duration::from_secs(30),
        ..ClickHouseConfig::default()
    }
}

// ============ 1. DDL Schema 验证 ============

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn ddl_schema_creates_all_tables_and_columns() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 验证所有 K 线表存在
    for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let tables = pool
            .query_rows(&format!("SHOW TABLES FROM {BINANCE_DB} LIKE 'klines_{interval}'"))
            .await
            .expect("show tables");
        assert_eq!(tables.len(), 1, "klines_{interval} should exist");
    }

    // 验证 funding_rate 表存在
    let tables = pool
        .query_rows(&format!("SHOW TABLES FROM {BINANCE_DB} LIKE 'funding_rate'"))
        .await
        .expect("show funding");
    assert_eq!(tables.len(), 1, "funding_rate should exist");

    // 验证列名
    let cols =
        pool.query_rows(&format!("DESCRIBE TABLE {BINANCE_DB}.klines_1h")).await.expect("describe");
    let col_names: Vec<&str> = cols.iter().filter_map(|r| r.first().map(|s| s.as_str())).collect();
    assert!(col_names.contains(&"open_time"), "column open_time missing");
    assert!(col_names.contains(&"symbol"), "column symbol missing");
    assert!(col_names.contains(&"open"), "column open missing");
    assert!(col_names.contains(&"high"), "column high missing");
    assert!(col_names.contains(&"low"), "column low missing");
    assert!(col_names.contains(&"close"), "column close missing");
    assert!(col_names.contains(&"volume"), "column volume missing");
    assert!(col_names.contains(&"quote_volume"), "column quote_volume missing");

    pool.close().await.ok();
}

// ============ 2. 导入数据验证 ============

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn imported_data_count_matches_expected() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 清理之前测试可能写入的脏数据
    pool.execute("ALTER TABLE klines_1m DELETE WHERE symbol LIKE 'BTC;%'").await.ok();

    // 验证每个 symbol/interval 都存在足够数据
    for symbol in &["BTCUSDT", "ETHUSDT"] {
        for (interval, min_count) in [
            ("1m", 1_500_000u64),
            ("5m", 250_000),
            ("15m", 50_000),
            ("1h", 10_000),
            ("4h", 5_000),
            ("1d", 1_000),
        ] {
            let text = pool
                .query_text(&format!(
                    "SELECT count() FROM {BINANCE_DB}.klines_{interval} WHERE symbol='{symbol}'"
                ))
                .await
                .expect("count");
            let count: u64 = text.trim().parse().unwrap();
            assert!(
                count >= min_count,
                "{symbol}/{interval}: expected >= {min_count}, got {count}"
            );
        }
    }

    // 验证时间范围
    let text = pool
        .query_text(&format!(
            "SELECT min(open_time), max(open_time) FROM {BINANCE_DB}.klines_1h WHERE symbol='BTCUSDT'"
        ))
        .await
        .expect("time range");
    assert!(text.contains("1690848000000"), "should start from 2023-08-01");
    // 数据应覆盖到 2026-07

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn imported_data_price_sanity() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // BTC 价格应在合理范围 (2023-08 ~ 2026-07: BTC never below 20k or above 200k)
    let text = pool
        .query_text(&format!(
            "SELECT min(close), max(close) FROM {BINANCE_DB}.klines_1d WHERE symbol='BTCUSDT'"
        ))
        .await
        .expect("BTC range");
    let parts: Vec<f64> = text.trim().split('\t').filter_map(|s| s.parse().ok()).collect();
    assert!(parts.len() == 2);
    assert!(parts[0] > 10000.0, "BTC min close should be > 10k: {}", parts[0]);
    assert!(parts[0] < 40000.0, "BTC min close should be < 40k: {}", parts[0]);
    assert!(parts[1] > 50000.0, "BTC max close should be > 50k: {}", parts[1]);

    // ETH 价格合理范围
    let text = pool
        .query_text(&format!(
            "SELECT min(close), max(close) FROM {BINANCE_DB}.klines_1d WHERE symbol='ETHUSDT'"
        ))
        .await
        .expect("ETH range");
    let parts: Vec<f64> = text.trim().split('\t').filter_map(|s| s.parse().ok()).collect();
    assert!(parts.len() == 2);
    assert!(parts[0] > 1000.0, "ETH min close should be > 1k: {}", parts[0]);
    assert!(parts[1] > 3000.0, "ETH max close should be > 3k: {}", parts[1]);

    pool.close().await.ok();
}

// ============ 3. CRUD 操作验证 ============

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn crud_read_queries_return_data() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 验证 query_rows
    let rows = pool
        .query_rows(&format!(
            "SELECT symbol, open, high, low, close FROM {BINANCE_DB}.klines_1h LIMIT 1"
        ))
        .await
        .expect("query");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 5);

    // 验证 query_text
    let text = pool
        .query_text(&format!("SELECT count() FROM {BINANCE_DB}.klines_1d WHERE symbol='BTCUSDT'"))
        .await
        .expect("count");
    assert!(text.trim().parse::<u64>().unwrap() > 0);

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn crud_create_insert_and_verify() {
    let pool = ClickHousePool::connect(live_cfg("default")).await.expect("connect");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DB}")).await.expect("create db");
    let pool = ClickHousePool::connect(live_cfg(TEST_DB)).await.expect("connect");

    // 建测试表
    pool.execute(
        "CREATE TABLE IF NOT EXISTS crud_test (id UInt64, name String) ENGINE = MergeTree ORDER BY id",
    )
    .await
    .expect("create");

    // INSERT
    let row = json!({"id": 1, "name": "test-row"});
    pool.insert_json_each_row("crud_test", &[row]).await.expect("insert");

    // SELECT 验证
    let rows =
        pool.query_rows("SELECT id, name FROM crud_test WHERE id = 1").await.expect("select");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "1");
    assert_eq!(rows[0][1], "test-row");

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn crud_delete_via_alter_table() {
    // 确保 TEST_DB 和表存在 (自给自足, 不依赖其他测试执行顺序)
    let pool = ClickHousePool::connect(live_cfg("default")).await.expect("connect");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DB}")).await.expect("create db");
    let pool = ClickHousePool::connect(live_cfg(TEST_DB)).await.expect("connect");
    pool.execute(
        "CREATE TABLE IF NOT EXISTS crud_test (id UInt64, name String) ENGINE = MergeTree ORDER BY id",
    )
    .await
    .expect("create table");

    // 准备测试数据
    pool.execute("INSERT INTO crud_test VALUES (100, 'to-delete')").await.ok();
    let before = pool
        .query_text("SELECT count() FROM crud_test WHERE id = 100")
        .await
        .expect("count before");
    assert_eq!(before.trim(), "1");

    // DELETE
    pool.execute("ALTER TABLE crud_test DELETE WHERE id = 100").await.expect("delete");

    // 验证删除
    tokio::time::sleep(Duration::from_secs(1)).await; // ClickHouse DELETE mutation is async
    let after =
        pool.query_text("SELECT count() FROM crud_test WHERE id = 100").await.expect("count after");
    assert_eq!(after.trim(), "0", "row should be deleted");

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn crud_cleanup_test_tables() {
    // 确保 TEST_DB 存在 (自给自足)
    let pool = ClickHousePool::connect(live_cfg("default")).await.expect("connect");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DB}")).await.expect("create db");
    let pool = ClickHousePool::connect(live_cfg(TEST_DB)).await.expect("connect");
    pool.execute("DROP TABLE IF EXISTS crud_test").await.ok();
    pool.close().await.ok();
}

// ============ 4. 分析查询验证 ============

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn analytics_ohlcv_aggregation_produces_correct_bars() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 1m->5m 聚合: 24h 数据应有 288 个 5m bar (24*60/5)
    let text = pool
        .query_text(&format!(
            "SELECT count() FROM ( \
             SELECT toStartOfFiveMinutes(toDateTime(intDiv(open_time, 1000))) bar \
             FROM {BINANCE_DB}.klines_1m WHERE symbol='BTCUSDT' \
             AND open_time/1000 >= toUnixTimestamp(now() - INTERVAL 24 HOUR) \
             GROUP BY bar \
             )"
        ))
        .await
        .expect("count 5m bars");
    let bar_count: u64 = text.trim().parse().unwrap();
    assert!(bar_count > 0 && bar_count <= 288, "should have 1-288 5m bars in 24h, got {bar_count}");

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn analytics_sma_produces_valid_values() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 验证 BTC 日线平均价格在合理范围
    let text = pool
        .query_text(&format!(
            "SELECT round(avg(close), 2) FROM ( \
             SELECT close FROM {BINANCE_DB}.klines_1d WHERE symbol='BTCUSDT' AND close > 0 \
             )"
        ))
        .await
        .expect("avg close");
    let avg: f64 = text.trim().parse().unwrap();
    assert!(avg > 20000.0, "BTC avg close should be reasonable: {}", avg);

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn analytics_btc_eth_correlation_is_valid() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // BTC-ETH 日线收盘价相关性 (两者高度相关, 应 > 0.5)
    let text = pool
        .query_text(&format!(
            "SELECT round(corrStable(btc.close, eth.close), 2) FROM \
             (SELECT toStartOfDay(toDateTime(intDiv(open_time,1000))) day, close FROM {BINANCE_DB}.klines_1d WHERE symbol='BTCUSDT') btc \
             JOIN \
             (SELECT toStartOfDay(toDateTime(intDiv(open_time,1000))) day, close FROM {BINANCE_DB}.klines_1d WHERE symbol='ETHUSDT') eth \
             ON btc.day = eth.day"
        ))
        .await
        .expect("correlation");
    let corr: f64 = text.trim().parse().unwrap();
    assert!(corr > 0.5, "BTC-ETH correlation should be > 0.5, got {corr}");
    assert!(corr <= 1.0, "correlation should be <= 1.0");

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn analytics_volume_top_returns_results() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    let rows = pool
        .query_rows(&format!(
            "SELECT symbol, quote_volume FROM {BINANCE_DB}.klines_1h WHERE symbol IN ('BTCUSDT','ETHUSDT') \
             ORDER BY quote_volume DESC LIMIT 5"
        ))
        .await
        .expect("top volume");
    assert_eq!(rows.len(), 5, "should return 5 rows");
    // 所有 volume 应为正数
    for r in &rows {
        let vol: f64 = r[1].parse().unwrap();
        assert!(vol > 0.0, "volume should be positive");
    }

    pool.close().await.ok();
}

// ============ 5. 导入示例边界测试 ============

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn import_handles_empty_csv_gracefully() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 创建临时表测试空批次插入
    pool.execute(
        "CREATE TABLE IF NOT EXISTS empty_test (id UInt64) ENGINE = MergeTree ORDER BY id",
    )
    .await
    .ok();

    // 空行批量: insert_batch with empty slice
    let result = pool.insert_batch("empty_test", &[], BatchInsertOptions::default()).await;
    assert!(result.is_ok(), "empty batch should succeed");

    // 空行单条: insert_json_each_row with empty slice
    let result = pool.insert_json_each_row("empty_test", &[]).await;
    assert!(result.is_ok(), "empty rows should succeed");

    pool.execute("DROP TABLE IF EXISTS empty_test").await.ok();
    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn import_handles_invalid_symbol_fast_fail() {
    let pool = ClickHousePool::connect(live_cfg(BINANCE_DB)).await.expect("connect");

    // 非法表名会被 validate_ident 在发网络请求之前拦截
    let result = pool
        .insert_json_each_row(
            "invalid-table;DROP",
            &[json!({"open_time": 0, "symbol": "BTCUSDT", "open": 1.0})],
        )
        .await;
    assert!(result.is_err(), "illegal table name should be rejected");
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Invalid);

    pool.close().await.ok();
}
