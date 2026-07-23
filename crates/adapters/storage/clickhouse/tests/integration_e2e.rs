//! ClickHouse E2E 集成测试（gap-zero）。
//!
//! 覆盖端到端流程：多类型往返、batch 分块、analytics 全路径、
#![allow(clippy::while_let_loop)] // mock server accept loops
//! 跨重连持久化、大批量、错误恢复、close 拒绝。
//!
//! ```bash
//! cargo test -p clickhousex --test integration_e2e -- --nocapture
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bytes::Bytes;
use clickhousex::{ANALYTICS_TABLE, BatchInsertOptions, ClickHouseConfig, ClickHousePool};
use contracts::AnalyticsSink;
use kernel::ErrorKind;
use serde_json::{Value, json};

// ═══════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════

const INTEGRATION_DB: &str = "infra_draft_integration_test";

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn runique_table(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}_{}_{id}", std::process::id())
}

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

async fn connect_integration() -> ClickHousePool {
    // 先用 default 创建测试库，再切换
    let pool = ClickHousePool::connect(live_cfg("default")).await.expect("connect default");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {INTEGRATION_DB}"))
        .await
        .expect("create integration db");
    pool.close().await.ok();
    ClickHousePool::connect(live_cfg(INTEGRATION_DB)).await.expect("connect integration db")
}

async fn drop_table_silently(pool: &ClickHousePool, table: &str) {
    pool.execute(&format!("DROP TABLE IF EXISTS {table}")).await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 1. 多种 JSON 类型往返
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn roundtrip_insert_query_multiple_types() {
    let pool = connect_integration().await;
    let tbl = runique_table("rtt");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (\
           simple String,\
           nested String,\
           arr String,\
           null_val Nullable(String),\
           bool_val UInt8\
         ) ENGINE = MergeTree ORDER BY simple"
    ))
    .await
    .expect("create");

    let rows = vec![
        json!({
            "simple": "hello",
            "nested": "{\"key\":\"value\"}",
            "arr": "[1,2,3]",
            "null_val": null,
            "bool_val": 1
        }),
        json!({
            "simple": "world",
            "nested": "{\"a\":1}",
            "arr": "[]",
            "null_val": "not-null",
            "bool_val": 0
        }),
    ];
    pool.insert_json_each_row(&tbl, &rows).await.expect("insert");

    let got = pool
        .query_rows(&format!(
            "SELECT simple, nested, arr, null_val, bool_val FROM {tbl} ORDER BY simple"
        ))
        .await
        .expect("select");
    assert_eq!(got.len(), 2);
    assert_eq!(got[0][0], "hello");
    assert_eq!(got[0][1], "{\"key\":\"value\"}");
    assert_eq!(got[0][2], "[1,2,3]");
    assert_eq!(got[0][3], "\\N"); // ClickHouse NULL → \N in TabSeparated
    assert_eq!(got[0][4], "1");
    assert_eq!(got[1][0], "world");
    assert_eq!(got[1][3], "not-null");
    assert_eq!(got[1][4], "0");

    drop_table_silently(&pool, &tbl).await;
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 2. Batch 分块验证 — 50 行每 chunk=10 → 5 次独立 POST
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn batch_insert_chunk_verification() {
    // 一个接受恰好 5 次 POST 的计数服务端
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时端口");
    let port = listener.local_addr().expect("端口").port();
    let server = tokio::spawn(async move {
        let mut count = 0usize;
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let mut buf = vec![0u8; 32768];
                    let _ = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf))
                        .await
                        .ok();
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
                Err(_) => break,
            }
        }
        count
    });

    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_secs(10),
        acquire_timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let pool = ClickHousePool::connect_without_ping(cfg).expect("build without ping");

    let rows: Vec<Value> = (0..50).map(|i| json!({"n": i})).collect();
    pool.insert_batch("valid_table", &rows, BatchInsertOptions { max_rows_per_chunk: 10 })
        .await
        .expect("insert_batch");

    pool.close().await.ok();
    let count = server.await.expect("server");
    assert_eq!(count, 5, "50行每chunk=10应产生5次独立HTTP POST");
}

// ═══════════════════════════════════════════════════════════════
// 3. analytics_sink 完整路径
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn analytics_sink_full_path() {
    let pool = connect_integration().await;
    drop_table_silently(&pool, ANALYTICS_TABLE).await;

    // 幂等建表
    pool.ensure_analytics_table().await.expect("ensure");
    pool.ensure_analytics_table().await.expect("re-ensure idempotent");

    // sink 多个事件
    for i in 0..3 {
        pool.sink("e2e_analytics_event", Bytes::from(format!("payload-{}", i)))
            .await
            .expect("sink");
    }

    // 验证
    let rows = pool
        .query_rows(&format!(
            "SELECT event, payload FROM {ANALYTICS_TABLE} WHERE event = 'e2e_analytics_event' ORDER BY ts"
        ))
        .await
        .expect("select");
    assert_eq!(rows.len(), 3, "应有 3 条事件");
    assert_eq!(rows[0][1], "payload-0");
    assert_eq!(rows[1][1], "payload-1");
    assert_eq!(rows[2][1], "payload-2");

    // 幂等重复：再 sink 一条后总数应为 4
    pool.sink("e2e_analytics_event", Bytes::from_static(b"payload-3")).await.expect("sink again");
    let rows = pool
        .query_rows(&format!(
            "SELECT event, payload FROM {ANALYTICS_TABLE} WHERE event = 'e2e_analytics_event' ORDER BY ts"
        ))
        .await
        .expect("select");
    assert_eq!(rows.len(), 4);

    drop_table_silently(&pool, ANALYTICS_TABLE).await;
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 4. 双表 + close + reconnect 数据持久化
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn multi_table_and_persistence_across_reconnect() {
    let pool = connect_integration().await;
    let tbl_a = runique_table("persist_a");
    let tbl_b = runique_table("persist_b");

    // 双表
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl_a} (id UInt64, val String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("create a");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl_b} (id UInt64, val String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("create b");

    // 插入数据
    let rows = (1..=3).map(|i| json!({"id": i, "val": format!("val-{i}")})).collect::<Vec<_>>();
    pool.insert_json_each_row(&tbl_a, &rows).await.expect("insert a");
    pool.insert_json_each_row(&tbl_b, &rows).await.expect("insert b");

    // close
    pool.close().await.ok();

    // reconnect
    let pool = connect_integration().await;

    // 验证数据持久化
    let got_a = pool
        .query_rows(&format!("SELECT id, val FROM {tbl_a} ORDER BY id"))
        .await
        .expect("select a");
    assert_eq!(got_a.len(), 3, "表 A 数据应在 reconnect 后保持");
    assert_eq!(got_a[0][1], "val-1");

    let got_b = pool
        .query_rows(&format!("SELECT id, val FROM {tbl_b} ORDER BY id"))
        .await
        .expect("select b");
    assert_eq!(got_b.len(), 3, "表 B 数据应在 reconnect 后保持");
    assert_eq!(got_b[2][1], "val-3");

    drop_table_silently(&pool, &tbl_a).await;
    drop_table_silently(&pool, &tbl_b).await;
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 5. 大批量插入 (1000+ 行) 抽样校验
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn large_batch_insert_and_verify() {
    let pool = connect_integration().await;
    let tbl = runique_table("large_batch");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (idx UInt64, label String) ENGINE = MergeTree ORDER BY idx"
    ))
    .await
    .expect("create");

    let total = 1001;
    let rows: Vec<Value> =
        (0..total).map(|i| json!({"idx": i, "label": format!("row-{}", i)})).collect();

    pool.insert_batch(&tbl, &rows, BatchInsertOptions { max_rows_per_chunk: 200 })
        .await
        .expect("insert_batch");

    // 总数验证
    let count_text = pool.query_text(&format!("SELECT count() FROM {tbl}")).await.expect("count");
    let count: u64 = count_text.trim().parse().unwrap();
    assert_eq!(count, total as u64);

    // 抽样校验：首 / 中 / 尾
    for idx in [0, 500, 1000u64] {
        let got = pool
            .query_rows(&format!("SELECT idx, label FROM {tbl} WHERE idx = {idx}"))
            .await
            .expect("select");
        assert_eq!(got.len(), 1, "idx={idx} 应恰好一行");
        assert_eq!(got[0][0], idx.to_string());
        assert_eq!(got[0][1], format!("row-{idx}"));
    }

    drop_table_silently(&pool, &tbl).await;
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 6. 错误恢复：合法插入后非法插入，已有数据不受影响
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn error_recovery_partial_success() {
    let pool = connect_integration().await;
    let tbl = runique_table("err_recovery");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (id UInt64, val String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("create");

    // 合法插入
    let good = json!({"id": 1, "val": "good"});
    pool.insert_json_each_row(&tbl, &[good]).await.expect("good insert");

    // 非法插入（schema 不匹配：缺少列）
    let bad = json!({"unknown_column": "bad"});
    let _err = pool.insert_json_each_row(&tbl, &[bad]).await.expect_err("bad insert must fail");

    // 已有数据不受影响
    let got = pool.query_rows(&format!("SELECT id, val FROM {tbl}")).await.expect("select");
    assert_eq!(got.len(), 1, "legal data survives error");
    assert_eq!(got[0][0], "1");
    assert_eq!(got[0][1], "good");

    // 继续合法插入
    let good2 = json!({"id": 2, "val": "after-error"});
    pool.insert_json_each_row(&tbl, &[good2]).await.expect("good insert after error");
    let got =
        pool.query_rows(&format!("SELECT id, val FROM {tbl} ORDER BY id")).await.expect("select");
    assert_eq!(got.len(), 2);
    assert_eq!(got[1][1], "after-error");

    drop_table_silently(&pool, &tbl).await;
    pool.close().await.ok();
}

// ═══════════════════════════════════════════════════════════════
// 7. close 后拒绝所有新请求 + stats 反映已关闭
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn close_rejects_new_requests_and_stats_closed() {
    let pool = connect_integration().await;
    let tbl = runique_table("close_test");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {tbl} (id UInt64) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("create");

    // close 前一切正常
    assert!(!pool.stats().closed);
    assert!(!pool.is_closed());
    pool.query_text("SELECT 1").await.expect("query before close");

    // close
    pool.close().await.expect("close");

    // stats 反映已关闭
    assert!(pool.stats().closed, "stats.closed must be true");
    assert!(pool.is_closed(), "is_closed() must be true");

    // execute 拒绝
    let err = pool
        .execute(&format!("INSERT INTO {tbl} VALUES (1)"))
        .await
        .expect_err("execute after close must fail");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    // query_text 拒绝
    let err = pool.query_text("SELECT 1").await.expect_err("query after close must fail");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    // insert_json_each_row 拒绝
    let err = pool
        .insert_json_each_row(&tbl, &[json!({"id": 1})])
        .await
        .expect_err("insert after close must fail");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    // 幂等 close 不报错
    pool.close().await.expect("close again idempotent");

    drop_table_silently(&pool, &tbl).await;
}
