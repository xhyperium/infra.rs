//! ClickHouse 安全与可观测性集成测试。
//!
//! - Debug 脱敏、配置校验、标识符注入为离线测试。
#![allow(clippy::while_let_loop)] // mock server accept loops
//! - 池统计和关闭行为通过 mock HTTP 服务器离线验证。
//! - 错误分类测试（Missing/Conflict）需要本地 ClickHouse。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// ═══════════════════════════════════════════════════════════════
// Mock HTTP 服务器辅助函数
// ═══════════════════════════════════════════════════════════════

/// 启动一个单连接 mock 服务器，返回指定的 HTTP 状态行和正文。
async fn spawn_one_response(status: &str, body: String) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
    let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
    let status = status.to_owned();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("接受 HTTP 连接");
        let mut buf = [0u8; 4096];
        let _ = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await;
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.expect("写 HTTP 响应");
        stream.shutdown().await.expect("关闭 HTTP 流");
    });
    (port, server)
}

/// 启动一个持续监听的 mock 服务器：第一次请求快速返回 "1\n"（模拟 ping），
/// 后续请求挂起 `hold` 时长后再返回。
async fn spawn_fast_then_slow(hold: Duration) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
    let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
    let first = Arc::new(AtomicBool::new(true));

    let server = tokio::spawn({
        let first = first.clone();
        async move {
            loop {
                match tokio::time::timeout(Duration::from_secs(30), listener.accept()).await {
                    Ok(Ok((mut stream, _))) => {
                        let mut buf = [0u8; 4096];
                        let _ = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf))
                            .await;
                        if first.swap(false, Ordering::SeqCst) {
                            stream
                                .write_all(
                                    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\n1\n",
                                )
                                .await
                                .expect("写入 ping 响应");
                        } else {
                            tokio::time::sleep(hold).await;
                            stream
                                .write_all(
                                    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 1\r\nConnection: close\r\n\r\n\n",
                                )
                                .await
                                .expect("写入慢响应");
                        }
                        stream.shutdown().await.expect("关闭 HTTP 流");
                    }
                    _ => break,
                }
            }
        }
    });
    (port, server)
}

fn config_for(port: u16) -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_secs(5),
        acquire_timeout: Duration::from_secs(5),
        ..ClickHouseConfig::default()
    }
}

/// 本地 ClickHouse 连接配置。
fn live_ch_config() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        user: "default".into(),
        password: "iCEOuptIx40EduvGOK73XrfY".into(),
        ..ClickHouseConfig::default()
    }
}

/// 本地 ClickHouse 连接配置（指定测试数据库名）。
fn live_ch_config_with_db(db: &str) -> ClickHouseConfig {
    ClickHouseConfig { database: db.to_string(), ..live_ch_config() }
}

// ═══════════════════════════════════════════════════════════════
// 安全测试 — Debug 脱敏
// ═══════════════════════════════════════════════════════════════

#[test]
fn debug_redacts_password() {
    let cfg =
        ClickHouseConfig { password: "secret-value-xyz".into(), ..ClickHouseConfig::default() };
    let debug = format!("{cfg:?}");
    assert!(debug.contains("***"));
    assert!(!debug.contains("secret-value-xyz"));
}

#[test]
fn config_debug_does_not_contain_password() {
    let cfg = ClickHouseConfig { password: "my-secret-pw".into(), ..ClickHouseConfig::default() };
    let display = format!("{:?}", cfg);
    assert!(!display.contains("my-secret-pw"));
}

#[test]
fn clickhouseconfig_debug_fields_visible_except_password() {
    let cfg = ClickHouseConfig {
        host: "db1.internal".into(),
        http_port: 9440,
        user: "admin".into(),
        password: "hidden".into(),
        database: "metrics".into(),
        max_in_flight: 32,
        ..ClickHouseConfig::default()
    };
    let debug = format!("{cfg:?}");
    // 密码被脱敏
    assert!(debug.contains("***"));
    assert!(!debug.contains("hidden"));
    // 其余字段可见
    assert!(debug.contains("db1.internal"));
    assert!(debug.contains("9440"));
    assert!(debug.contains("admin"));
    assert!(debug.contains("metrics"));
    assert!(debug.contains("max_in_flight"));
}

// ═══════════════════════════════════════════════════════════════
// 安全测试 — 标识符注入防护（通过 insert_json_each_row 间接测试）
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn insert_rejects_injection_patterns() {
    let (port, server) = spawn_one_response("200 OK", "1\n".into()).await;
    let pool = ClickHousePool::connect(config_for(port)).await.expect("connect");
    let row = serde_json::json!({"a": 1});

    let patterns = ["a;drop", "1bad", "", "table'; DROP TABLE", "bad-table", "a b"];
    for pattern in patterns {
        let err = pool
            .insert_json_each_row(pattern, std::slice::from_ref(&row))
            .await
            .expect_err(&format!("标识符 {pattern:?} 必须被拒绝"));
        assert_eq!(err.kind(), ErrorKind::Invalid, "标识符 {pattern:?} 拒绝类型错误");
    }
    server.await.expect("mock server task");
}

// ═══════════════════════════════════════════════════════════════
// 安全测试 — 配置校验
// ═══════════════════════════════════════════════════════════════

#[test]
fn remote_http_without_tls_is_rejected() {
    let cfg =
        ClickHouseConfig { host: "clickhouse.example.com".into(), ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("远程 HTTP 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn remote_https_passes_validation() {
    let cfg = ClickHouseConfig {
        host: "clickhouse.example.com".into(),
        tls: true,
        ..ClickHouseConfig::default()
    };
    cfg.validate().expect("远程 HTTPS 应通过校验");
}

#[test]
fn ca_file_without_tls_is_rejected() {
    let cfg =
        ClickHouseConfig { tls_ca_file: Some("/tmp/ca.pem".into()), ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("CA 无 TLS 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn zero_timeout_is_rejected() {
    let cfg = ClickHouseConfig { timeout: Duration::ZERO, ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("零 timeout 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn zero_acquire_timeout_is_rejected() {
    let cfg = ClickHouseConfig { acquire_timeout: Duration::ZERO, ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("零 acquire_timeout 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn zero_max_in_flight_is_rejected() {
    let cfg = ClickHouseConfig { max_in_flight: 0, ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("零 max_in_flight 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn base_url_does_not_contain_password() {
    let cfg = ClickHouseConfig {
        host: "127.0.0.2".into(),
        http_port: 9123,
        password: "hidden-pw".into(),
        ..ClickHouseConfig::default()
    };
    let url = cfg.base_url();
    assert!(!url.contains("hidden-pw"));
    assert_eq!(url, "http://127.0.0.2:9123");
}

#[test]
fn empty_host_is_invalid() {
    let cfg = ClickHouseConfig { host: String::new(), ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("空 host 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn port_zero_is_invalid() {
    let cfg = ClickHouseConfig { http_port: 0, ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("端口 0 必须被拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

// ═══════════════════════════════════════════════════════════════
// 可观测性测试 — 池统计
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn pool_stats_reflect_initial_state() {
    let (port, server) = spawn_one_response("200 OK", "1\n".into()).await;
    let pool = ClickHousePool::connect(config_for(port)).await.expect("connect");
    assert_eq!(pool.stats().in_flight, 0);
    assert!(!pool.stats().closed);
    server.await.expect("mock server task");
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn pool_stats_after_close() {
    let (port, server) = spawn_one_response("200 OK", "1\n".into()).await;
    let pool = ClickHousePool::connect(config_for(port)).await.expect("connect");

    pool.close().await.expect("close");
    let stats = pool.stats();
    assert!(stats.closed);
    assert!(pool.is_closed());

    // 关闭后请求被拒绝
    let err = pool.execute("SELECT 1").await.expect_err("关闭后必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
    server.await.expect("mock server task");
}

// ═══════════════════════════════════════════════════════════════
// 可观测性测试 — 错误类型
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_invalid_on_bad_table_name() {
    let (port, server) = spawn_one_response("200 OK", "1\n".into()).await;
    let pool = ClickHousePool::connect(config_for(port)).await.expect("connect");
    let err = pool
        .insert_json_each_row("bad!", &[serde_json::json!({"a": 1})])
        .await
        .expect_err("非表名必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.context().contains("bad!"));
    server.await.expect("mock server task");
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_unavailable_on_closed_pool() {
    let (port, server) = spawn_one_response("200 OK", "1\n".into()).await;
    let pool = ClickHousePool::connect(config_for(port)).await.expect("connect");
    pool.close().await.expect("close");

    let err = pool.execute("SELECT 1").await.expect_err("关闭池必须拒绝");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
    server.await.expect("mock server task");
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_deadline_exceeded_on_short_timeout() {
    let hold = Duration::from_secs(3);
    let (port, server) = spawn_fast_then_slow(hold).await;
    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_millis(500),
        acquire_timeout: Duration::from_secs(5),
        ..ClickHouseConfig::default()
    };
    let pool = ClickHousePool::connect(cfg).await.expect("connect");

    let err = pool.execute("SELECT 1").await.expect_err("短 timeout 必须超时");
    assert_eq!(err.kind(), ErrorKind::DeadlineExceeded);
    server.await.expect("mock server task");
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_context_for_max_in_flight() {
    // max_in_flight=1，第一次请求占用许可，第二次请求在 acquire 超时时返回
    // 错误，上下文应包含 "max_in_flight" 和 "max=1"。
    let hold = Duration::from_secs(4);
    let (port, server) = spawn_fast_then_slow(hold).await;
    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_secs(10),
        acquire_timeout: Duration::from_millis(300),
        max_in_flight: 1,
        ..ClickHouseConfig::default()
    };
    let pool = ClickHousePool::connect(cfg).await.expect("connect");

    let first = {
        let pool = pool.clone();
        tokio::spawn(async move { pool.execute("SELECT 1").await })
    };
    // 给第一个请求足够时间先拿到唯一许可
    tokio::time::sleep(Duration::from_millis(100)).await;

    let second_err = pool.execute("SELECT 1").await.expect_err("第二个请求必须超时");
    assert_eq!(second_err.kind(), ErrorKind::DeadlineExceeded);
    assert!(second_err.context().contains("max_in_flight"));

    let _ = first.await;
    server.await.expect("mock server task");
}

// ═══════════════════════════════════════════════════════════════
// 可观测性测试 — 错误分类（需要 ClickHouse）
// ══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_classification_missing() {
    let pool = ClickHousePool::connect(live_ch_config()).await.expect("connect");

    // 先确保测试库存在，再尝试创建 → 预期 Missing 或 Conflict
    let _ = pool.execute("CREATE DATABASE IF NOT EXISTS gap_zero_missing_test").await;
    let err = pool
        .execute("CREATE DATABASE gap_zero_missing_test")
        .await
        .expect_err("重复创建数据库必须失败");

    // server_code=81 → Missing; 其他可能返回 Conflict
    let kind = err.kind();
    assert!(
        matches!(kind, ErrorKind::Missing | ErrorKind::Conflict),
        "预期 Missing|Conflict，实际 {kind:?}: {}",
        err.context()
    );
    // 清理
    let _ = pool.execute("DROP DATABASE IF EXISTS gap_zero_missing_test").await;
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_classification_conflict_duplicate_table() {
    let db = "gap_zero_conflict";
    let pool = ClickHousePool::connect(live_ch_config_with_db(db)).await.expect("connect");

    let _ = pool.execute("CREATE DATABASE IF NOT EXISTS gap_zero_conflict").await;

    pool.execute("CREATE TABLE IF NOT EXISTS ct_test (x UInt8) ENGINE = MergeTree ORDER BY x")
        .await
        .expect("建表");

    let err = pool
        .execute("CREATE TABLE ct_test (x UInt8) ENGINE = MergeTree ORDER BY x")
        .await
        .expect_err("重复建表必须失败");

    // TABLE_ALREADY_EXISTS → Conflict（server_code=57）
    let kind = err.kind();
    assert!(
        matches!(kind, ErrorKind::Conflict | ErrorKind::Transient),
        "预期 Conflict|Transient，实际 {kind:?}: {}",
        err.context()
    );

    // 清理
    let _ = pool.execute("DROP TABLE IF EXISTS ct_test").await;
    let _ = pool.execute("DROP DATABASE IF EXISTS gap_zero_conflict").await;
}

// ═══════════════════════════════════════════════════════════════
// 可观测性测试 — 错误分类（离线 mock）
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_classification_transient_5xx() {
    let (port, server) = spawn_one_response("500 Internal Server Error", "boom".into()).await;
    let err = match ClickHousePool::connect(config_for(port)).await {
        Err(err) => err,
        Ok(_) => panic!("500 响应必须失败"),
    };
    assert_eq!(err.kind(), ErrorKind::Transient);
    server.await.expect("mock server task");
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn error_classification_unavailable_403() {
    let (port, server) = spawn_one_response("403 Forbidden", "denied".into()).await;
    let err = match ClickHousePool::connect(config_for(port)).await {
        Err(err) => err,
        Ok(_) => panic!("403 响应必须失败"),
    };
    assert_eq!(err.kind(), ErrorKind::Unavailable);
    server.await.expect("mock server task");
}
