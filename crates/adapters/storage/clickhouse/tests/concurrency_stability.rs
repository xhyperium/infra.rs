//! ClickHouse U+5E76U+53D1U+7A33U+5B9AU+6027U+6D4BU+8BD5U+3002
//!
//! ```text
//! cargo test -p clickhousex --test concurrency_stability -- --test-threads=1 --nocapture
//! ```

use std::sync::Arc;
use std::time::Duration;

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;
use serde_json::Value;

const TEST_PASSWORD: &str = "iCEOuptIx40EduvGOKX73rfY";
const TEST_DATABASE: &str = "infra_draft_concurrent_test";

fn default_config() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        password: TEST_PASSWORD.into(),
        database: "default".into(),
        ..ClickHouseConfig::default()
    }
}

async fn setup_concurrent_db() -> ClickHousePool {
    let mut cfg = default_config();
    cfg.database = "default".into();
    let pool = ClickHousePool::connect(cfg).await.expect("连接 default 数据库");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DATABASE}"))
        .await
        .expect("创建测试数据库");
    pool.close().await.expect("关闭 default 连接");
    let mut cfg = default_config();
    cfg.database = TEST_DATABASE.into();
    ClickHousePool::connect(cfg).await.expect("连接到测试数据库")
}

async fn create_simple_table(pool: &ClickHousePool, prefix: &str) -> String {
    let pid = std::process::id();
    let table = format!("{prefix}_{pid}");
    let ddl = format!(
        "CREATE TABLE IF NOT EXISTS {table} (\
           id UInt32,\
           marker String\
         ) ENGINE = MergeTree ORDER BY id"
    );
    pool.execute(&ddl).await.expect("创建测试表");
    pool.execute(&format!("TRUNCATE TABLE IF EXISTS {table}"))
        .await
        .expect("清理测试表");
    table
}

fn make_rows(start: u32, count: u32) -> Vec<Value> {
    (start..start + count)
        .map(|i| serde_json::json!({"id": i, "marker": format!("r{i}")}))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_writes_ten_tasks_insert_correct_row_count() {
    let pool = setup_concurrent_db().await;
    let table = create_simple_table(&pool, "ccw_table").await;
    let pool = Arc::new(pool);
    let table = Arc::new(table);
    let mut handles = Vec::new();
    for t in 0..10u32 {
        let pool = pool.clone();
        let table = table.clone();
        let rows = make_rows(t * 20, 20);
        handles.push(tokio::spawn(async move {
            pool.insert_json_each_row(&table, &rows).await
        }));
    }
    for handle in handles {
        handle.await.expect("任务 join").expect("各 task 插入成功");
    }
    let count_sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
    let count_text = pool.query_text(&count_sql).await.expect("count query");
    let count: u64 = count_text.trim().parse().expect("解析行数");
    assert_eq!(count, 200, "10 tasks x 20 rows = 200 total");
    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理测试表");
    pool.close().await.expect("关闭连接");
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_read_write_mixed_eventual_consistency() {
    let pool = setup_concurrent_db().await;
    let table = create_simple_table(&pool, "crw_table").await;
    let pool = Arc::new(pool);
    let table = Arc::new(table);
    let mut writer_handles = Vec::new();
    for t in 0..5u32 {
        let pool = pool.clone();
        let table = table.clone();
        let rows = make_rows(t * 20, 20);
        writer_handles.push(tokio::spawn(async move {
            pool.insert_json_each_row(&table, &rows).await
        }));
    }
    let mut reader_handles = Vec::new();
    for _ in 0..5 {
        let pool = pool.clone();
        let table = table.clone();
        reader_handles.push(tokio::spawn(async move {
            let sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
            pool.query_text(&sql).await.map(|t| t.trim().parse::<u64>().ok())
        }));
    }
    for handle in writer_handles {
        handle.await.expect("writer join").expect("writer success");
    }
    for handle in reader_handles {
        let _ = handle.await.expect("reader join");
    }
    let count_sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
    let count_text = pool.query_text(&count_sql).await.expect("count query");
    let count: u64 = count_text.trim().parse().expect("解析行数");
    assert_eq!(count, 100, "5 writers x 20 rows = 100 total");
    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理测试表");
    pool.close().await.expect("关闭连接");
}

#[tokio::test(flavor = "multi_thread")]
async fn pool_limit_max_in_flight_one_with_five_concurrent_requests() {
    let mut cfg = default_config();
    cfg.max_in_flight = 1;
    cfg.acquire_timeout = Duration::from_millis(500);
    let pool = ClickHousePool::connect(cfg).await.expect("连接");
    let hold_pool = pool.clone();
    let hold = tokio::spawn(async move {
        hold_pool.execute("SELECT sleep(3)").await
    });
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut ok_count = 0u32;
    let mut deadline_count = 0u32;
    let mut other_count = 0u32;
    let mut handles = Vec::new();
    for _ in 0..5 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            p.execute("SELECT 1").await.map_err(|e| e.kind())
        }));
    }
    for handle in handles {
        match handle.await.expect("join") {
            Ok(()) => ok_count += 1,
            Err(ErrorKind::DeadlineExceeded) => deadline_count += 1,
            Err(ErrorKind::Unavailable) => deadline_count += 1,
            Err(_) => other_count += 1,
        }
    }
    let _ = hold.await.expect("hold join");
    assert!(
        ok_count > 0 || deadline_count >= 4,
        "至少 1 个请求应成功或 4+ 超时: ok={ok_count} deadline={deadline_count}"
    );
    assert!(
        deadline_count + other_count >= 3,
        "多数并发应超时: ok={ok_count} deadline={deadline_count}"
    );
    pool.close().await.expect("关闭连接");
}

#[tokio::test(flavor = "multi_thread")]
async fn long_running_stability_thirty_seconds_no_leak() {
    let pool = setup_concurrent_db().await;
    let table = create_simple_table(&pool, "lrs_table").await;
    let pool = Arc::new(pool);
    let table = Arc::new(table);
    let start = tokio::time::Instant::now();
    let mut insert_count = 0u64;
    while start.elapsed() < Duration::from_secs(30) {
        let rows = make_rows((insert_count % 1000) as u32 * 100, 10);
        pool.insert_json_each_row(&table, &rows)
            .await
            .expect("insert");
        let count_sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
        let _ = pool.query_text(&count_sql).await.expect("count query");
        insert_count += 1;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    let stats = pool.stats();
    assert_eq!(stats.in_flight, 0, "30s 后 in_flight 应归零");
    if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", std::process::id())) {
        let vm_rss: Option<u64> = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<u64>().ok())
            });
        if let Some(rss) = vm_rss {
            assert!(rss < 500_000, "VmRSS 异常偏高: {rss} kB");
        }
    }
    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理测试表");
    pool.close().await.expect("关闭连接");
}

#[tokio::test(flavor = "multi_thread")]
async fn backpressure_behavior_with_different_max_in_flight() {
    for &max_in_flight in &[2usize, 4] {
        let mut cfg = default_config();
        cfg.max_in_flight = max_in_flight;
        cfg.acquire_timeout = Duration::from_millis(800);
        let pool = ClickHousePool::connect(cfg).await.expect("连接");
        let hold_count = max_in_flight - 1;
        let mut hold_handles = Vec::new();
        for _ in 0..hold_count {
            let p = pool.clone();
            hold_handles.push(tokio::spawn(async move {
                p.execute("SELECT sleep(3)").await
            }));
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let extra_pool = pool.clone();
        let extra = tokio::spawn(async move {
            extra_pool.execute("SELECT 1").await
        });
        let p2 = pool.clone();
        let overflow = tokio::spawn(async move {
            p2.execute("SELECT 1").await
        });
        let overflow_result = overflow.await.expect("overflow join");
        match overflow_result {
            Ok(()) => {}
            Err(e) => {
                let kind = e.kind();
                assert!(
                    matches!(kind, ErrorKind::DeadlineExceeded | ErrorKind::Unavailable),
                    "背压错误应为 DeadlineExceeded 或 Unavailable, 实际: {kind:?}"
                );
            }
        }
        let _ = extra.await.expect("extra join");
        for h in hold_handles {
            let _ = h.await.expect("hold join");
        }
        pool.close().await.expect("关闭连接");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn connection_leak_detection_pool_stats_sane_after_operations() {
    let pool = setup_concurrent_db().await;
    let table = create_simple_table(&pool, "cld_table").await;
    for i in 0..100u32 {
        let rows = make_rows(i * 10, 10);
        pool.insert_json_each_row(&table, &rows)
            .await
            .expect("insert");
        let count_sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
        let _ = pool.query_text(&count_sql).await.expect("count query");
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let stats = pool.stats();
    assert_eq!(stats.in_flight, 0, "100 次操作后 in_flight 应归零: actual={}", stats.in_flight);
    assert!(!stats.closed, "池不应被关闭");
    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理测试表");
    pool.close().await.expect("关闭连接");
}

#[tokio::test(flavor = "multi_thread")]
async fn closed_pool_all_concurrent_requests_return_unavailable() {
    let pool = setup_concurrent_db().await;
    pool.close().await.expect("close");
    let pool = Arc::new(pool);
    let mut handles = Vec::new();
    for _ in 0..10 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            p.execute("SELECT 1").await.map_err(|e| e.kind())
        }));
    }
    for handle in handles {
        let result = handle.await.expect("join");
        match result {
            Ok(()) => panic!("关闭后请求不应成功"),
            Err(kind) => {
                assert_eq!(
                    kind,
                    ErrorKind::Unavailable,
                    "关闭后错误应为 Unavailable, 实际: {kind:?}"
                );
            }
        }
    }
}
