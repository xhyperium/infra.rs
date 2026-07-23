//! `taosx` 边界对抗性 conformance（R2）。
//!
//! 全部离线可跑：本地 `TcpListener` mock TDengine REST，
//! 不依赖真实 TDengine 集群，不使用 `#[ignore]`。
//!
//! 覆盖：鉴权降级拒绝、无界批量/响应/查询拒绝、schema 冲突拒绝、
//! close 排空与重复 close、in-flight 背压超时、连接拒绝语义。

use std::time::Duration;

use canonical::Tick;
use decimalx::{Decimal, Price};
use kernel::ErrorKind;
use taosx::{TaosConfig, TaosPool};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn sample_tick(symbol: &str, ts_ns: i64) -> Tick {
    Tick {
        symbol: symbol.into(),
        bid: Price::new(Decimal::try_new(10_050, 2).expect("bid")),
        ask: Price::new(Decimal::try_new(10_060, 2).expect("ask")),
        ts: ts_ns,
    }
}

/// 依序为多个连续请求（各自独立连接，`Connection: close`）返回预设 JSON body。
async fn serve_sequence(bodies: Vec<String>) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        for body in bodies {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut request = [0u8; 4096];
            let _ = stream.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    port
}

/// 声明响应体的 `Content-Length` 大于实际写入字节数，
/// 模拟远端谎报体积/流式截断场景；驱动必须按声明长度 fail-closed。
async fn serve_oversized_content_length(declared_len: usize, actual_body: &'static str) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let Ok((mut stream, _)) = listener.accept().await else {
            return;
        };
        let mut request = [0u8; 4096];
        let _ = stream.read(&mut request).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {declared_len}\r\nConnection: close\r\n\r\n{actual_body}"
        );
        let _ = stream.write_all(response.as_bytes()).await;
    });
    port
}

fn loopback_config(port: u16) -> TaosConfig {
    TaosConfig {
        port,
        database: String::new(),
        timeout: Duration::from_secs(2),
        ..TaosConfig::default()
    }
}

fn offline_pool(cfg: TaosConfig) -> TaosPool {
    TaosPool::connect_without_ping(cfg).expect("offline pool")
}

// ---------------------------------------------------------------------------
// 鉴权降级拒绝：非 loopback host 必须拒绝明文/空密码，且不发出任何网络请求。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn remote_plaintext_connect_is_rejected_before_any_network_call() {
    let cfg = TaosConfig {
        host: "taos.example.com".into(),
        tls: false,
        password: "irrelevant".into(),
        ..TaosConfig::default()
    };
    match TaosPool::connect(cfg).await {
        Ok(_) => panic!("remote plaintext must fail-closed"),
        Err(error) => assert_eq!(error.kind(), ErrorKind::Invalid),
    }
}

#[tokio::test]
async fn remote_tls_without_password_is_rejected() {
    let cfg = TaosConfig {
        host: "taos.example.com".into(),
        tls: true,
        password: String::new(),
        ..TaosConfig::default()
    };
    match TaosPool::connect(cfg).await {
        Ok(_) => panic!("remote with empty password must fail-closed"),
        Err(error) => assert_eq!(error.kind(), ErrorKind::Invalid),
    }
}

// ---------------------------------------------------------------------------
// 连接拒绝语义：连不上的端口必须映射为 Unavailable。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn connect_to_closed_port_maps_to_unavailable() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind ephemeral");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);

    let cfg = loopback_config(port);
    match TaosPool::connect(cfg).await {
        Ok(_) => panic!("connect to closed port must fail"),
        Err(error) => assert_eq!(error.kind(), ErrorKind::Unavailable),
    }
}

// ---------------------------------------------------------------------------
// 无界批量写入拒绝：超过 batch_max_rows / batch_max_bytes 必须在本地被拒绝。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_batch_chunked_rejects_custom_max_rows_above_config_limit() {
    // 不监听端口：断言必须在网络 I/O 之前触发。
    let cfg = TaosConfig {
        batch_max_rows: 10,
        host: "127.0.0.1".into(),
        port: 1,
        database: String::new(),
        ..TaosConfig::default()
    };
    let pool = offline_pool(cfg);

    let error = pool
        .write_batch_chunked("ticks", &[sample_tick("BTC", 1)], 11)
        .await
        .expect_err("custom max_rows above config limit must be rejected locally");
    assert_eq!(error.kind(), ErrorKind::Invalid);
}

#[tokio::test]
async fn write_batch_chunked_rejects_zero_max_rows() {
    let cfg = TaosConfig {
        host: "127.0.0.1".into(),
        port: 1,
        database: String::new(),
        ..TaosConfig::default()
    };
    let pool = offline_pool(cfg);

    let error = pool
        .write_batch_chunked("ticks", &[sample_tick("BTC", 1)], 0)
        .await
        .expect_err("zero max_rows must be rejected");
    assert_eq!(error.kind(), ErrorKind::Invalid);
}

#[tokio::test]
async fn write_batch_rejects_oversized_single_row_before_network_send() {
    // batch_max_bytes 小于单行 INSERT 语句体积：必须在拼 SQL 阶段本地拒绝。
    // write_batch 会先 ensure_stable（需网络），因此直接走 build 路径：
    // max_rows=1 且 bytes 极小 → chunk 构建失败。
    // ensure_stable 会尝试网络；对 bytes 限制需在 ensure 之后。
    // 因此提供一次 CREATE + DESCRIBE 合法 schema 再拒绝写入。
    let create_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#.to_string();
    let describe_ok = concat!(
        r#"{"code":0,"column_meta":[["field","VARCHAR",16],["type","VARCHAR",16],["length","VARCHAR",8]],"#,
        r#""data":[["ts","TIMESTAMP","8"],["bid","NCHAR","64"],["ask","NCHAR","64"]],"rows":3}"#
    )
    .to_string();
    let port = serve_sequence(vec![create_ok, describe_ok]).await;
    let cfg = TaosConfig {
        host: "127.0.0.1".into(),
        port,
        database: String::new(),
        batch_max_bytes: 32,
        batch_max_rows: 1,
        timeout: Duration::from_secs(2),
        ..TaosConfig::default()
    };
    let pool = offline_pool(cfg);

    let error = pool
        .write_batch("ticks_narrow", &[sample_tick("VERY_LONG_SYMBOL_NAME", 1)])
        .await
        .expect_err("oversized row must be rejected locally after schema ok");
    assert_eq!(error.kind(), ErrorKind::Invalid);
}

// ---------------------------------------------------------------------------
// schema 冲突拒绝：ensure_stable 探测到旧 DOUBLE schema 时必须 Conflict。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ensure_stable_rejects_legacy_double_schema() {
    let create_stable_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#.to_string();
    let describe_double_schema = concat!(
        r#"{"code":0,"column_meta":[["field","VARCHAR",16],["type","VARCHAR",16],["length","VARCHAR",8]],"#,
        r#""data":[["ts","TIMESTAMP","8"],["bid","DOUBLE","8"],["ask","DOUBLE","8"]],"rows":3}"#
    )
    .to_string();
    let port = serve_sequence(vec![create_stable_ok, describe_double_schema]).await;
    let pool = offline_pool(loopback_config(port));

    let error = pool
        .ensure_stable("legacy_ticks")
        .await
        .expect_err("legacy DOUBLE schema must be rejected");
    assert_eq!(error.kind(), ErrorKind::Conflict);
}

// ---------------------------------------------------------------------------
// 响应体大小上限：声明 Content-Length 超过 max_response_bytes 必须 fail-closed。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn oversized_declared_content_length_is_rejected_without_buffering() {
    let port = serve_oversized_content_length(1024, "{\"code\":0}").await;
    let cfg = TaosConfig { max_response_bytes: 16, ..loopback_config(port) };
    let pool = offline_pool(cfg);

    let error =
        pool.exec_sql("SELECT 1").await.expect_err("declared oversized body must be rejected");
    assert_eq!(error.kind(), ErrorKind::Unavailable);
}

// ---------------------------------------------------------------------------
// 查询行数上限：结果行数超过 max_query_rows 必须拒绝。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn query_row_limit_rejects_oversized_result_sets() {
    let rows_json = (0..5).map(|i| format!(r#"["{i}"]"#)).collect::<Vec<_>>().join(",");
    let body =
        format!(r#"{{"code":0,"column_meta":[["n","VARCHAR",8]],"data":[{rows_json}],"rows":5}}"#);
    let port = serve_sequence(vec![body]).await;
    let cfg = TaosConfig { max_query_rows: 3, ..loopback_config(port) };
    let pool = offline_pool(cfg);

    let error = pool
        .exec_sql("SELECT n FROM t")
        .await
        .expect_err("result exceeding max_query_rows must fail");
    assert_eq!(error.kind(), ErrorKind::Unavailable);
}

// ---------------------------------------------------------------------------
// close 排空与重复 close。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn closed_pool_rejects_new_requests_and_close_is_repeatable() {
    let pool = offline_pool(TaosConfig {
        host: "127.0.0.1".into(),
        port: 1,
        database: String::new(),
        ..TaosConfig::default()
    });

    pool.close().await.expect("first close must succeed with no in-flight work");
    assert!(pool.is_closed());

    let error = pool.exec_sql("SELECT 1").await.expect_err("closed pool must reject new requests");
    assert_eq!(error.kind(), ErrorKind::Unavailable);

    pool.close().await.expect("repeated close on drained pool must stay ok");
    assert!(pool.is_closed());
}

// ---------------------------------------------------------------------------
// in-flight 背压超时：max_in_flight=1 时第二个请求必须 DeadlineExceeded。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn in_flight_backpressure_times_out_when_saturated() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            // 保持连接打开但不响应，直到测试结束。
            std::mem::forget(stream);
        }
    });
    let cfg = TaosConfig {
        max_in_flight: 1,
        acquire_timeout: Duration::from_millis(200),
        timeout: Duration::from_secs(5),
        ..loopback_config(port)
    };
    let pool = offline_pool(cfg);

    let first = pool.clone();
    let blocking = tokio::spawn(async move { first.exec_sql("SELECT slow()").await });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let error = pool
        .exec_sql("SELECT 1")
        .await
        .expect_err("second request must time out acquiring in-flight permit");
    assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);

    blocking.abort();
}
