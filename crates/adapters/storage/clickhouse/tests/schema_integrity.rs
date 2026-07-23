//! Schema 完整性验证 — DDL 约束、Config::validate、validate_ident、端口别名。

use clickhousex::{ClickHouseConfig, ClickHousePool, validate_ident};
use kernel::ErrorKind;
use std::time::Duration;

const BINANCE_DB: &str = "binance_futures";

// env_helpers: wrap unsafe set_var/remove_var for Rust 2024
fn set_env(k: &str, v: &str) {
    unsafe {
        std::env::set_var(k, v);
    }
}
fn remove_env(k: &str) {
    unsafe {
        std::env::remove_var(k);
    }
}

fn live_cfg() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        user: "default".into(),
        password: "iCEOuptIx40EduvGOKX73rfY".into(),
        database: BINANCE_DB.into(),
        timeout: Duration::from_secs(10),
        ..ClickHouseConfig::default()
    }
}

// ═══════════════════════════════════════════════════════════════
// DDL 约束 — 引擎 / 分区 / 排序键
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn all_tables_use_mergetree_engine() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");
    let tables = &[
        "klines_1m",
        "klines_5m",
        "klines_15m",
        "klines_1h",
        "klines_4h",
        "klines_1d",
        "funding_rate",
    ];
    for name in tables {
        let ddl = pool
            .query_text(&format!("SHOW CREATE TABLE {BINANCE_DB}.{name}"))
            .await
            .unwrap_or_else(|_| panic!("show create {name}"));
        assert!(ddl.contains("MergeTree"), "{name}: expected MergeTree engine");
    }
    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn klines_tables_use_monthly_partition() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");
    for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let ddl = pool
            .query_text(&format!("SHOW CREATE TABLE {BINANCE_DB}.klines_{interval}"))
            .await
            .unwrap_or_else(|_| panic!("show create klines_{interval}"));
        assert!(
            ddl.contains("PARTITION BY") && ddl.contains("toYYYYMM"),
            "klines_{interval}: expected PARTITION BY toYYYYMM"
        );
    }
    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn klines_order_by_symbol_and_open_time() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");
    for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let ddl = pool
            .query_text(&format!("SHOW CREATE TABLE {BINANCE_DB}.klines_{interval}"))
            .await
            .unwrap_or_else(|_| panic!("show create klines_{interval}"));
        assert!(
            ddl.contains("ORDER BY") && ddl.contains("symbol") && ddl.contains("open_time"),
            "klines_{interval}: expected ORDER BY (symbol, open_time)"
        );
    }
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 列名与类型
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn klines_columns_have_correct_names_and_types() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");

    let rows =
        pool.query_rows(&format!("DESCRIBE TABLE {BINANCE_DB}.klines_1h")).await.expect("describe");

    let mut cols = std::collections::HashMap::new();
    for r in &rows {
        if r.len() >= 2 {
            cols.insert(r[0].clone(), r[1].clone());
        }
    }

    for required in &[
        "open_time",
        "symbol",
        "open",
        "high",
        "low",
        "close",
        "volume",
        "close_time",
        "quote_volume",
        "count",
        "taker_buy_volume",
        "taker_buy_quote_volume",
    ] {
        assert!(cols.contains_key(*required), "missing column: {required}");
    }

    assert!(cols["symbol"].starts_with("String"), "symbol type");
    assert!(cols["open"].starts_with("Float64"), "open type");
    assert!(cols["volume"].starts_with("Float64"), "volume type");
    assert!(cols["count"].starts_with("UInt32"), "count type: {}", cols["count"]);

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn all_klines_tables_have_same_column_count() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");

    let mut counts = Vec::new();
    for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let rows = pool
            .query_rows(&format!("DESCRIBE TABLE {BINANCE_DB}.klines_{interval}"))
            .await
            .expect("describe");
        counts.push((*interval, rows.len()));
    }

    let expected = counts[0].1;
    for (name, count) in &counts {
        assert_eq!(*count, expected, "klines_{name}: {count} cols vs expected {expected}");
    }

    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 数据级约束 — 非空、无非法值
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn ordering_columns_are_non_null() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");

    for interval in &["1m", "5m", "15m", "1h", "4h", "1d"] {
        let bad = pool
            .query_text(&format!(
                "SELECT count() FROM {BINANCE_DB}.klines_{interval} WHERE open_time <= 0 OR symbol = ''"
            ))
            .await
            .expect("count bad");
        assert_eq!(bad.trim(), "0", "klines_{interval}: invalid rows found");
    }

    pool.close().await.ok();
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn ohlc_values_are_non_negative() {
    let pool = ClickHousePool::connect(live_cfg()).await.expect("connect");

    let text = pool
        .query_text(&format!(
            "SELECT count() FROM {BINANCE_DB}.klines_1d WHERE open < 0 OR high < 0 OR low < 0 OR close < 0 OR volume < 0"
        ))
        .await
        .expect("count negative");
    assert_eq!(text.trim(), "0", "found negative OHLCV values");

    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// Config::validate — 全部失败路径
// ═══════════════════════════════════════════════════════════════

#[test]
fn config_rejects_zero_max_in_flight() {
    let cfg = ClickHouseConfig { max_in_flight: 0, ..Default::default() };
    let err = cfg.validate().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.to_string().contains("max_in_flight"));
}

#[test]
fn config_rejects_zero_timeout() {
    let cfg = ClickHouseConfig { timeout: Duration::ZERO, ..Default::default() };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_rejects_zero_acquire_timeout() {
    let cfg = ClickHouseConfig { acquire_timeout: Duration::ZERO, ..Default::default() };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_rejects_empty_host() {
    let cfg = ClickHouseConfig { host: String::new(), ..Default::default() };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_rejects_whitespace_host() {
    let cfg = ClickHouseConfig { host: "   ".into(), ..Default::default() };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_rejects_port_zero() {
    let cfg = ClickHouseConfig { http_port: 0, ..Default::default() };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_rejects_remote_plaintext() {
    let cfg = ClickHouseConfig { host: "ch.example.com".into(), tls: false, ..Default::default() };
    let err = cfg.validate().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.to_string().contains("HTTPS") || err.to_string().contains("TLS"));
}

#[test]
fn config_rejects_ca_without_tls() {
    let cfg = ClickHouseConfig {
        tls: false,
        tls_ca_file: Some("/tmp/ca.pem".into()),
        ..Default::default()
    };
    assert_eq!(cfg.validate().unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn config_accepts_valid_default() {
    ClickHouseConfig::default().validate().expect("default must pass");
}

#[test]
fn config_accepts_remote_https() {
    ClickHouseConfig { host: "ch.example.com".into(), tls: true, ..Default::default() }
        .validate()
        .expect("remote HTTPS must pass");
}

// ═══════════════════════════════════════════════════════════════
// validate_ident — 标识符约束
// ═══════════════════════════════════════════════════════════════

#[test]
fn ident_rejects_empty() {
    assert_eq!(validate_ident("").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_numeric_prefix() {
    assert_eq!(validate_ident("1bad").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_semicolon() {
    assert_eq!(validate_ident("a;drop").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_dash() {
    assert_eq!(validate_ident("bad-table").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_space() {
    assert_eq!(validate_ident("a b").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_slash() {
    assert_eq!(validate_ident("bad/name").unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_rejects_overlong() {
    let long = "a".repeat(193);
    assert_eq!(validate_ident(&long).unwrap_err().kind(), ErrorKind::Invalid);
}

#[test]
fn ident_accepts_valid() {
    for id in &["valid_table", "_underscore_prefix", "TABLE_name_123", "xyz"] {
        validate_ident(id).unwrap_or_else(|_| panic!("{id} should be valid"));
    }
}

// ═══════════════════════════════════════════════════════════════
// 端口别名 — 4 种组合
// ═══════════════════════════════════════════════════════════════

fn set_envs(fqdn: &str, http: &str, port: &str) {
    let _ = std::env::var("FOUNDATIONX_CLICKHOUSEX_HOST"); // suppress unused
    set_env("FOUNDATIONX_CLICKHOUSEX_HOST", fqdn);
    set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", http);
    set_env("FOUNDATIONX_CLICKHOUSEX_PORT", port);
}

unsafe fn remove_port_env() {
    remove_env("FOUNDATIONX_CLICKHOUSEX_PORT");
}

unsafe fn remove_http_port_env() {
    remove_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT");
}

#[test]
fn port_alias_conflict_rejected() {
    set_envs("127.0.0.1", "8123", "8443");
    let err = ClickHouseConfig::from_env().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.to_string().contains("冲突") || err.to_string().contains("conflict"));
}

#[test]
fn port_alias_both_same_passes() {
    set_envs("127.0.0.1", "9440", "9440");
    let cfg = ClickHouseConfig::from_env().expect("matching ports");
    assert_eq!(cfg.http_port, 9440);
}

#[test]
fn port_alias_http_port_only() {
    set_envs("127.0.0.1", "9999", "0");
    unsafe { remove_port_env() };
    let cfg = ClickHouseConfig::from_env().expect("HTTP_PORT only");
    assert_eq!(cfg.http_port, 9999);
}

#[test]
fn port_alias_port_fallback() {
    set_envs("127.0.0.1", "0", "7777");
    unsafe { remove_http_port_env() };
    let cfg = ClickHouseConfig::from_env().expect("PORT alias");
    assert_eq!(cfg.http_port, 7777);
}
