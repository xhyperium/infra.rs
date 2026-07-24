//! ClickHouse 全公开 API 覆盖测试（gap-zero）。
//!
//! 覆盖 client / module / config 的全部 `pub` 项目。
//!
//! # Note
//!
//! `pool.close().await` 是 async pool 资源释放的典型模式，
//! 锁持有横跨 await 是池生命周期管理的预期行为。
#![allow(clippy::await_holding_lock)]
//!
//! ```bash
//! cargo test -p clickhousex --test functional_api_coverage -- --nocapture
//! cargo test -p clickhousex --features scaffold --test functional_api_coverage -- --nocapture
//! ```

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bytes::Bytes;
use clickhousex::{
    ANALYTICS_TABLE, BatchInsertOptions, ClickHouseClient, ClickHouseConfig, ClickHousePool,
    ClickHousePoolStats, chunk_ranges, parse_tab_separated_rows,
};
use contracts::AnalyticsSink;
use kernel::{ErrorKind, XError};
use serde_json::{Value, json};

// ═══════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════

const FUNC_DB: &str = "infra_draft_func_test";

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 生成隔离表名，避免并行测试冲突。
fn utext(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}_{}_{id}", std::process::id())
}

fn base_cfg() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        user: "default".into(),
        password: "iCEOuptIx40EduvGOKX73rfY".into(),
        database: "default".into(),
        timeout: Duration::from_secs(30),
        ..ClickHouseConfig::default()
    }
}

fn func_cfg() -> ClickHouseConfig {
    ClickHouseConfig { database: FUNC_DB.into(), ..base_cfg() }
}

async fn setup_db() {
    let pool = ClickHousePool::connect(base_cfg()).await.expect("connect default");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {FUNC_DB}"))
        .await
        .expect("create test db");
    pool.close().await.ok();
}

async fn func_pool() -> ClickHousePool {
    setup_db().await;
    ClickHousePool::connect(func_cfg()).await.expect("connect func db")
}

async fn drop_tbl(pool: &ClickHousePool, table: &str) {
    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.ok();
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// 设置 ClickHouse 连接所需环境变量。调用方必须持有 ENV_LOCK。
fn set_clickhouse_env() {
    // SAFETY: 测试工具函数，单线程执行。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "8123");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_USER", "default");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_PASSWORD", "iCEOuptIx40EduvGOKX73rfY");
    }
}

fn clear_clickhouse_env() {
    // SAFETY: 测试工具函数，单线程执行。
    for var in &[
        "FOUNDATIONX_CLICKHOUSEX_HOST",
        "FOUNDATIONX_CLICKHOUSEX_HTTP_PORT",
        "FOUNDATIONX_CLICKHOUSEX_PORT",
        "FOUNDATIONX_CLICKHOUSEX_USER",
        "FOUNDATIONX_CLICKHOUSEX_PASSWORD",
        "FOUNDATIONX_CLICKHOUSEX_DATABASE",
        "FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS",
        "FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST",
        "FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT",
        "FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS",
        "FOUNDATIONX_CLICKHOUSEX_TLS",
        "FOUNDATIONX_CLICKHOUSEX_TLS_CA_FILE",
    ] {
        unsafe {
            std::env::remove_var(var);
        }
    }
}

/// 获取环境变量锁，防止并行测试互相干扰。
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().expect("env lock")
}

// ═══════════════════════════════════════════════════════════════
// 1. ClickHouseConfig - 默认值 / 环境变量 / 校验 / 调试
// ═══════════════════════════════════════════════════════════════

#[test]
fn config_default_values() {
    let c = ClickHouseConfig::default();
    assert_eq!(c.host, "127.0.0.1");
    assert_eq!(c.http_port, 8123);
    assert!(!c.tls);
    assert!(c.tls_ca_file.is_none());
    assert_eq!(c.user, "default");
    assert!(c.password.is_empty());
    assert_eq!(c.database, "default");
    assert_eq!(c.timeout, Duration::from_secs(10));
    assert_eq!(c.max_idle_per_host, 8);
    assert_eq!(c.max_in_flight, 64);
    assert_eq!(c.acquire_timeout, Duration::from_secs(5));
    assert_eq!(c.base_url(), "http://127.0.0.1:8123");
    c.validate().expect("default must be valid");
}

#[test]
fn config_base_url_http_and_https() {
    let http_cfg = ClickHouseConfig::default();
    assert_eq!(http_cfg.base_url(), "http://127.0.0.1:8123");

    let https_cfg = ClickHouseConfig { tls: true, http_port: 8443, ..Default::default() };
    assert_eq!(https_cfg.base_url(), "https://127.0.0.1:8443");
}

#[test]
fn config_debug_redacts_password() {
    let c = ClickHouseConfig { password: "secret-value".into(), ..Default::default() };
    let s = format!("{c:?}");
    assert!(s.contains("***"), "Debug 必须用 *** 替换密码");
    assert!(!s.contains("secret-value"), "Debug 不得暴露真实密码");
}

#[test]
fn config_from_env_reads_all_variables() {
    let _lock = env_lock();
    clear_clickhouse_env();
    set_clickhouse_env();
    // SAFETY: 测试环境，单线程。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_DATABASE", "test_db");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS", "30000");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST", "16");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT", "32");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS", "10000");
    }

    let c = ClickHouseConfig::from_env().expect("from_env");
    assert_eq!(c.host, "127.0.0.1");
    assert_eq!(c.http_port, 8123);
    assert_eq!(c.user, "default");
    assert_eq!(c.password, "iCEOuptIx40EduvGOKX73rfY");
    assert_eq!(c.database, "test_db");
    assert_eq!(c.timeout, Duration::from_millis(30000));
    assert_eq!(c.max_idle_per_host, 16);
    assert_eq!(c.max_in_flight, 32);
    assert_eq!(c.acquire_timeout, Duration::from_millis(10000));
}

#[test]
fn config_from_env_uses_defaults_for_missing() {
    let _lock = env_lock();
    clear_clickhouse_env();
    set_clickhouse_env();
    let c = ClickHouseConfig::from_env().expect("from_env");
    assert_eq!(c.database, "default"); // 未设置 → 默认
    assert_eq!(c.timeout, Duration::from_secs(10)); // 未设置 → 默认
    assert_eq!(c.max_in_flight, 64); // 未设置 → 默认
}

#[test]
fn config_from_env_tls_parsing() {
    let _lock = env_lock();
    clear_clickhouse_env();
    // SAFETY: 测试环境，单线程。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "8123");
    }

    for (val, expected) in &[
        ("1", true),
        ("true", true),
        ("yes", true),
        ("on", true),
        ("TRUE", true),
        ("0", false),
        ("false", false),
        ("no", false),
        ("off", false),
    ] {
        // SAFETY: 测试环境，单线程。
        unsafe {
            std::env::set_var("FOUNDATIONX_CLICKHOUSEX_TLS", val);
        }
        let c = ClickHouseConfig::from_env().expect("from_env");
        assert_eq!(c.tls, *expected, "TLS={val} → {expected}");
    }
}

#[test]
fn config_validate_max_in_flight_at_least_1() {
    let c = ClickHouseConfig { max_in_flight: 0, ..Default::default() };
    let err = c.validate().expect_err("max_in_flight=0 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.to_string().contains("max_in_flight"));
}

#[test]
fn config_validate_timeout_must_be_positive() {
    let err = ClickHouseConfig { timeout: Duration::ZERO, ..Default::default() }
        .validate()
        .expect_err("零 timeout 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    let err = ClickHouseConfig { acquire_timeout: Duration::ZERO, ..Default::default() }
        .validate()
        .expect_err("零 acquire_timeout 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn config_validate_host_and_port_required() {
    let err = ClickHouseConfig { host: "".into(), ..Default::default() }
        .validate()
        .expect_err("空 host 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    let err = ClickHouseConfig { host: "   ".into(), ..Default::default() }
        .validate()
        .expect_err("空白 host 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    let err = ClickHouseConfig { http_port: 0, ..Default::default() }
        .validate()
        .expect_err("port=0 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn config_validate_remote_requires_tls() {
    let c = ClickHouseConfig { host: "clickhouse.example.com".into(), ..Default::default() };
    let err = c.validate().expect_err("远程 HTTP 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    // localhost 可以不用 TLS
    ClickHouseConfig::default().validate().expect("本地回环应通过");
}

#[test]
fn config_validate_ca_requires_tls() {
    let c = ClickHouseConfig { tls_ca_file: Some("/tmp/ca.pem".into()), ..Default::default() };
    let err = c.validate().expect_err("CA 不含 TLS 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn config_port_alias_http_port_priority() {
    let _lock = env_lock();
    // 仅 HTTP_PORT 设置
    let result = resolve_http_port_test(Some("9440"), None);
    assert_eq!(result, 9440);

    // 仅 PORT 兼容
    let result = resolve_http_port_test(None, Some("8443"));
    assert_eq!(result, 8443);

    // 双设相同值 → Ok
    let result = resolve_http_port_test(Some("9440"), Some("9440"));
    assert_eq!(result, 9440);

    // 都未设置 → 使用默认值 8123
    let result = resolve_http_port_test(None, None);
    assert_eq!(result, 8123);

    // 双设不同值 → 冲突拒绝
    let err = resolve_http_port_err(Some("8123"), Some("8443"));
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.to_string().contains("冲突"));
}

/// 通过环境变量测试端口别名，返回解析后的 http_port。
fn resolve_http_port_test(http_port: Option<&str>, port_alias: Option<&str>) -> u16 {
    clear_clickhouse_env();
    // SAFETY: 测试环境，单线程。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
        if let Some(v) = http_port {
            std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", v);
        }
        if let Some(v) = port_alias {
            std::env::set_var("FOUNDATIONX_CLICKHOUSEX_PORT", v);
        }
    }
    ClickHouseConfig::from_env().expect("from_env").http_port
}

fn resolve_http_port_err(http_port: Option<&str>, port_alias: Option<&str>) -> XError {
    clear_clickhouse_env();
    // SAFETY: 测试环境，单线程。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
        if let Some(v) = http_port {
            std::env::set_var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", v);
        }
        if let Some(v) = port_alias {
            std::env::set_var("FOUNDATIONX_CLICKHOUSEX_PORT", v);
        }
    }
    ClickHouseConfig::from_env().expect_err("must error")
}

// ═══════════════════════════════════════════════════════════════
// 2. ClickHousePool / ClickHouseClient — 真实连接
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn pool_connect_and_ping() {
    let pool = func_pool().await;
    pool.ping().await.expect("ping must succeed");
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_connect_from_env() {
    let _lock = env_lock();
    clear_clickhouse_env();
    set_clickhouse_env();
    // SAFETY: 测试环境，单线程。
    unsafe {
        std::env::set_var("FOUNDATIONX_CLICKHOUSEX_DATABASE", "default");
    }
    let pool = ClickHousePool::connect_from_env().await.expect("connect_from_env");
    pool.ping().await.expect("ping");
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_execute_ddl_create_database_table_drop() {
    let pool = func_pool().await;
    let tbl = utext("ddl_test");

    // CREATE TABLE
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (\
           id UInt64, name String\
         ) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("create table");
    drop_tbl(&pool, &tbl).await;

    // 幂等 CREATE IF NOT EXISTS
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (id UInt64) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("if not exists");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (id UInt64) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("idempotent if not exists");

    drop_tbl(&pool, &tbl).await;
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_query_text_select_1_and_literal() {
    let pool = func_pool().await;
    let body = pool.query_text("SELECT 1").await.expect("SELECT 1");
    assert_eq!(body.trim(), "1");

    let body = pool.query_text("SELECT 'hello' AS greeting").await.expect("select literal");
    assert_eq!(body.trim(), "hello");

    pool.close().await.ok();
}

#[tokio::test]
async fn pool_query_rows_multi_column() {
    let pool = func_pool().await;
    let tbl = utext("qr_test");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (n UInt64, s String) ENGINE = MergeTree ORDER BY n"
    ))
    .await
    .expect("create");

    pool.execute(&format!("INSERT INTO {tbl} VALUES (1, 'a'), (2, 'b'), (3, 'c')"))
        .await
        .expect("insert");

    let rows =
        pool.query_rows(&format!("SELECT n, s FROM {tbl} ORDER BY n")).await.expect("query_rows");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], vec!["1", "a"]);
    assert_eq!(rows[1], vec!["2", "b"]);
    assert_eq!(rows[2], vec!["3", "c"]);

    drop_tbl(&pool, &tbl).await;
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_insert_json_each_row_and_verify() {
    let pool = func_pool().await;
    let tbl = utext("ijr_test");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (n UInt64, s String) ENGINE = MergeTree ORDER BY n"
    ))
    .await
    .expect("create");

    let rows = vec![json!({"n": 10, "s": "ten"}), json!({"n": 20, "s": "twenty"})];
    pool.insert_json_each_row(&tbl, &rows).await.expect("insert");

    let got = pool.query_rows(&format!("SELECT n, s FROM {tbl} ORDER BY n")).await.expect("select");
    assert_eq!(got.len(), 2);
    assert_eq!(got[0], vec!["10", "ten"]);
    assert_eq!(got[1], vec!["20", "twenty"]);

    drop_tbl(&pool, &tbl).await;
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_insert_json_each_row_empty_short_circuits() {
    let pool = func_pool().await;
    pool.insert_json_each_row("valid_table_name", &[]).await.expect("空 rows 必须直接成功");
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_insert_json_each_row_rejects_non_object() {
    let pool = func_pool().await;
    let err = pool
        .insert_json_each_row("valid_table", &[json!(["not", "object"])])
        .await
        .expect_err("非 object 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[tokio::test]
async fn pool_insert_json_each_row_rejects_invalid_table_name() {
    let pool = func_pool().await;
    let err =
        pool.insert_json_each_row("1bad", &[json!({"a": 1})]).await.expect_err("非法表名必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[tokio::test]
async fn pool_insert_batch_with_chunks_and_verify() {
    let pool = func_pool().await;
    let tbl = utext("batch_test");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (n UInt64) ENGINE = MergeTree ORDER BY n"
    ))
    .await
    .expect("create");

    let rows: Vec<Value> = (1..=5).map(|i| json!({"n": i})).collect();
    let options = BatchInsertOptions { max_rows_per_chunk: 2 };
    pool.insert_batch(&tbl, &rows, options).await.expect("insert_batch");

    let got = pool.query_rows(&format!("SELECT n FROM {tbl} ORDER BY n")).await.expect("select");
    assert_eq!(got.len(), 5);
    for (i, r) in got.iter().enumerate() {
        assert_eq!(r[0], (i + 1).to_string());
    }

    drop_tbl(&pool, &tbl).await;
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_insert_batch_empty_short_circuits() {
    let pool = func_pool().await;
    pool.insert_batch("valid_table", &[], BatchInsertOptions::default())
        .await
        .expect("空 rows 必须直接成功");
    pool.close().await.ok();
}

#[tokio::test]
async fn pool_insert_batch_rejects_invalid_table_name() {
    let pool = func_pool().await;
    let err = pool
        .insert_batch("a;drop", &[json!({"a": 1})], BatchInsertOptions::default())
        .await
        .expect_err("非法表名必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

/// 发起一个记录独立 HTTP 请求数的计数服务器，证明
/// `insert_batch` 为每个 chunk 发出独立 POST（而非合并）。
#[tokio::test]
async fn pool_insert_batch_sends_one_http_request_per_chunk() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时端口");
    let port = listener.local_addr().expect("端口").port();
    let expected = 3; // 5行每chunk=2 → 3次HTTP POST
    let server = tokio::spawn(async move {
        let mut count = 0usize;
        for _ in 0..expected {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 16384];
            let _ = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await.ok();
            count += 1;
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\
                      Content-Length: 1\r\nConnection: close\r\n\r\n\n",
                )
                .await
                .ok();
            stream.shutdown().await.ok();
        }
        count
    });

    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(5),
        ..Default::default()
    };
    let pool = ClickHousePool::connect_without_ping(cfg).expect("build");

    let rows: Vec<Value> = (0..5).map(|i| json!({"n": i})).collect();
    pool.insert_batch("valid_table", &rows, BatchInsertOptions { max_rows_per_chunk: 2 })
        .await
        .expect("insert_batch");

    pool.close().await.ok();
    let count = server.await.expect("server task");
    assert_eq!(count, expected, "5行每chunk=2应产生3次独立HTTP POST");
}

#[tokio::test]
async fn pool_ensure_analytics_table_idempotent() {
    let pool = func_pool().await;

    pool.ensure_analytics_table().await.expect("first");
    pool.ensure_analytics_table().await.expect("second idempotent");

    // 验证表存在且有预期列
    let cols =
        pool.query_rows(&format!("DESCRIBE TABLE {ANALYTICS_TABLE}")).await.expect("describe");
    let names: Vec<&str> = cols.iter().filter_map(|r| r.first().map(|s| s.as_str())).collect();
    assert!(names.contains(&"ts"), "缺少 ts 列");
    assert!(names.contains(&"event"), "缺少 event 列");
    assert!(names.contains(&"payload"), "缺少 payload 列");

    pool.close().await.ok();
}

#[tokio::test]
async fn pool_close_idempotent_and_rejects_after_close() {
    let pool = func_pool().await;
    pool.close().await.expect("close");
    pool.close().await.expect("close again idempotent");

    let err = pool.execute("SELECT 1").await.expect_err("closed must reject execute");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    let err = pool.query_text("SELECT 1").await.expect_err("closed must reject query");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    let err = pool
        .insert_json_each_row("valid_table", &[json!({"a": 1})])
        .await
        .expect_err("closed must reject insert");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
}

#[tokio::test]
async fn pool_accessors_client_config_stats_is_closed() {
    let pool = func_pool().await;

    // client() 返回同类型克隆
    let client: ClickHouseClient = pool.client();
    assert!(!client.is_closed());

    // config()
    let cfg = pool.config();
    assert_eq!(cfg.database, FUNC_DB);

    // stats()
    let s = pool.stats();
    assert_eq!(s.in_flight, 0);
    assert!(!s.closed);

    // is_closed()
    assert!(!pool.is_closed());

    pool.close().await.ok();
    assert!(pool.is_closed());
    let s = pool.stats();
    assert!(s.closed);
}

#[tokio::test]
async fn pool_connect_without_ping_builds_without_network() {
    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 1,
        timeout: Duration::from_millis(200),
        acquire_timeout: Duration::from_millis(200),
        max_in_flight: 1,
        ..Default::default()
    };
    let pool = ClickHousePool::connect_without_ping(cfg).expect("build no ping");
    assert!(!pool.is_closed());
    assert_eq!(pool.stats().in_flight, 0);
    pool.close().await.ok();
    assert!(pool.is_closed());
}

// ═══════════════════════════════════════════════════════════════
// 3. AnalyticsSink impl for ClickHousePool
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn analytics_sink_writes_and_verifies() {
    let pool = func_pool().await;
    pool.ensure_analytics_table().await.expect("ensure analytics");

    // 使用唯一事件名避免并行冲突
    let event = utext("sink_write");

    // 先清理已有数据
    pool.execute(&format!("ALTER TABLE {ANALYTICS_TABLE} DELETE WHERE event = '{event}'"))
        .await
        .ok();
    tokio::time::sleep(Duration::from_millis(200)).await;

    pool.sink(&event, Bytes::from_static(b"payload-data")).await.expect("sink");
    pool.sink(&event, Bytes::from_static(b"payload-2")).await.expect("sink 2");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let rows = pool
        .query_rows(&format!(
            "SELECT event, payload FROM {ANALYTICS_TABLE} WHERE event = '{event}' ORDER BY ts"
        ))
        .await
        .expect("query");
    assert!(rows.len() >= 2, "至少应有 2 条记录");
    assert!(rows.iter().any(|r| r[1] == "payload-data"));
    assert!(rows.iter().any(|r| r[1] == "payload-2"));

    pool.execute(&format!("ALTER TABLE {ANALYTICS_TABLE} DELETE WHERE event = '{event}'"))
        .await
        .ok();
    pool.close().await.ok();
}

#[tokio::test]
async fn analytics_sink_rejects_empty_event() {
    let pool = func_pool().await;
    pool.ensure_analytics_table().await.expect("ensure");

    let err = pool.sink("", Bytes::from_static(b"data")).await.expect_err("空 event 必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 4. ClickHouseAdapter（feature = "scaffold"）
// ═══════════════════════════════════════════════════════════════

#[cfg(feature = "scaffold")]
mod scaffold_tests {
    use super::*;
    use clickhousex::ClickHouseAdapter;

    #[tokio::test]
    async fn adapter_new_and_accessors() {
        let a = ClickHouseAdapter::new("my-adapter", "http://host:8123");
        assert_eq!(a.name(), "my-adapter");
        assert_eq!(a.endpoint(), "http://host:8123");
        assert_eq!(a.event_count().expect("count"), 0);
    }

    #[tokio::test]
    async fn adapter_local_defaults() {
        let a = ClickHouseAdapter::local();
        assert_eq!(a.name(), "clickhouse-local");
        assert_eq!(a.endpoint(), "http://127.0.0.1:8123");
        assert_eq!(a.event_count().expect("count"), 0);
    }

    #[tokio::test]
    async fn adapter_sink_single_event() {
        let a = ClickHouseAdapter::local();
        a.sink("ev", Bytes::from_static(b"payload")).await.expect("sink");
        assert_eq!(a.event_count().expect("count"), 1);
    }

    #[tokio::test]
    async fn adapter_sink_accumulates_multiple_events() {
        let a = ClickHouseAdapter::new("multi", "http://example.invalid:8123");
        for i in 0..5 {
            a.sink(&format!("event-{i}"), Bytes::from(format!("payload-{i}"))).await.expect("sink");
        }
        assert_eq!(a.event_count().expect("count"), 5);
    }
}

// ═══════════════════════════════════════════════════════════════
// 5. 工具函数 — parse_tab_separated_rows / chunk_ranges /
//    BatchInsertOptions / ClickHousePoolStats
// ═══════════════════════════════════════════════════════════════

#[test]
fn utils_parse_tab_separated_rows_single_line() {
    let rows = parse_tab_separated_rows("a\tb\n");
    assert_eq!(rows, vec![vec!["a".to_string(), "b".to_string()]]);
}

#[test]
fn utils_parse_tab_separated_rows_multi_line() {
    let rows = parse_tab_separated_rows("a\tb\nc\td\te\n");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["a".to_string(), "b".to_string()]);
    assert_eq!(rows[1], vec!["c".to_string(), "d".to_string(), "e".to_string()]);
}

#[test]
fn utils_parse_tab_separated_rows_empty() {
    assert!(parse_tab_separated_rows("").is_empty());
}

#[test]
fn utils_parse_tab_separated_rows_all_blank_lines() {
    assert!(parse_tab_separated_rows("\n\n\n").is_empty());
}

#[test]
fn utils_parse_tab_separated_rows_skips_empty_lines() {
    let rows = parse_tab_separated_rows("a\n\nb\n");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["a".to_string()]);
    assert_eq!(rows[1], vec!["b".to_string()]);
}

#[test]
fn utils_parse_tab_separated_rows_no_tab() {
    let rows = parse_tab_separated_rows("single\n");
    assert_eq!(rows, vec![vec!["single".to_string()]]);
}

#[test]
fn utils_chunk_ranges_zero() {
    assert!(chunk_ranges(0, 10).is_empty());
}

#[test]
fn utils_chunk_ranges_single_chunk() {
    assert_eq!(chunk_ranges(5, 10), vec![(0, 5)]);
}

#[test]
fn utils_chunk_ranges_multiple_chunks() {
    assert_eq!(chunk_ranges(5, 2), vec![(0, 2), (2, 4), (4, 5)]);
}

#[test]
fn utils_chunk_ranges_remainder() {
    assert_eq!(chunk_ranges(7, 3), vec![(0, 3), (3, 6), (6, 7)]);
}

#[test]
fn utils_chunk_ranges_size_one() {
    assert_eq!(chunk_ranges(3, 1), vec![(0, 1), (1, 2), (2, 3)]);
}

#[test]
fn utils_chunk_ranges_zero_lifts_to_one() {
    assert_eq!(chunk_ranges(2, 0), vec![(0, 1), (1, 2)]);
}

#[test]
fn utils_chunk_ranges_exact_division() {
    assert_eq!(chunk_ranges(6, 2), vec![(0, 2), (2, 4), (4, 6)]);
}

#[test]
fn utils_batch_insert_options_default() {
    let o = BatchInsertOptions::default();
    assert_eq!(o.max_rows_per_chunk, 1000);
}

#[test]
fn utils_clickhouse_pool_stats_field_access() {
    let s = ClickHousePoolStats { in_flight: 3, closed: false };
    assert_eq!(s.in_flight, 3);
    assert!(!s.closed);
}

#[test]
fn utils_clickhouse_pool_stats_closed() {
    let s = ClickHousePoolStats { in_flight: 0, closed: true };
    assert_eq!(s.in_flight, 0);
    assert!(s.closed);
}
