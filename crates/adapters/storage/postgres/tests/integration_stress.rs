//! 并发/压力集成测试（默认 `#[ignore]`）。
//!
//! ```bash
//! cargo test -p postgresx --test integration_stress -- --ignored --nocapture --test-threads=1
//! ```

use kernel::ErrorKind;
use postgresx::{PoolStats, PostgresConfig, PostgresPool, SslMode};
use std::sync::Arc;
use std::time::Duration;

fn set_env() {
    // SAFETY: 仅通过 --test-threads=1 运行，无并发 set_var 竞争
    if std::env::var("FOUNDATIONX_POSTGRESX_HOST").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_HOST", "127.0.0.1");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_PORT").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_PORT", "5432");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_DATABASE").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_DATABASE", "market_binance");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_USER").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_USER", "market_binance");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_PASSWORD").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_PASSWORD", "Kt63mWgbhBwSPWnrEnMkC");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_SSLMODE").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_SSLMODE", "disable");
        }
    }
}

fn stress_config(pool_size: usize) -> PostgresConfig {
    set_env();
    PostgresConfig::builder()
        .host("127.0.0.1")
        .port(5432)
        .database("market_binance")
        .user("market_binance")
        .password("Kt63mWgbhBwSPWnrEnMkC")
        .sslmode(SslMode::Disable)
        .max_pool_size(pool_size)
        .acquire_timeout(Duration::from_secs(10))
        .operation_timeout(Duration::from_secs(10))
        .build()
        .expect("stress config")
}

fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("s{}", t)
}

// ============================================================================
// STRESS-1: concurrent_acquire_and_query
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn stress_concurrent_acquire_and_query() {
    let pool = Arc::new(PostgresPool::connect(&stress_config(4)).await.expect("connect"));

    let mut handles = Vec::new();
    for i in 0..8 {
        let pool = Arc::clone(&pool);
        handles.push(tokio::spawn(async move {
            let mut conn = pool.acquire().await.expect("acquire");
            // 使用 pg_sleep 模拟短暂 I/O
            let row =
                conn.query_one("SELECT pg_sleep(0.01), $1::int4 AS n", &[&i]).await.expect("query");
            let n: i32 = row.get("n");
            assert_eq!(n, i);
            drop(conn); // 归还连接
            n
        }));
    }

    let results: Vec<i32> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("task succeeded"))
        .collect();

    assert_eq!(results.len(), 8);
    // 验证所有返回值存在（无需关心顺序）
    let mut expected: Vec<i32> = (0..8).collect();
    let mut got = results;
    got.sort_unstable();
    expected.sort_unstable();
    assert_eq!(got, expected);

    pool.close();
}

// ============================================================================
// STRESS-2: pool_saturation_deadline
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn stress_pool_saturation_deadline() {
    let pool = Arc::new(PostgresPool::connect(&stress_config(1)).await.expect("connect"));

    // 持有一条连接
    let _held = pool.acquire().await.expect("first acquire");

    // 尝试在短 deadline 内再获取连接
    let second = pool.acquire_with(Duration::from_millis(500)).await;
    assert!(second.is_err(), "池满时短 deadline acquire 应失败");

    if let Err(e) = second {
        assert_eq!(
            e.kind(),
            ErrorKind::DeadlineExceeded,
            "应为 DeadlineExceeded，实际: {:?}",
            e.kind()
        );
    }

    // 释放第一条连接
    drop(_held);

    // 重新获取应成功
    let reacquire = pool.acquire_with(Duration::from_secs(5)).await;
    assert!(reacquire.is_ok(), "释放后重新获取应成功");
    drop(reacquire);

    pool.close();
}

// ============================================================================
// STRESS-3: connection_recovery_after_timeout
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn stress_connection_recovery_after_timeout() {
    let pool = PostgresPool::connect(
        &PostgresConfig::builder()
            .host("127.0.0.1")
            .port(5432)
            .database("market_binance")
            .user("market_binance")
            .password("Kt63mWgbhBwSPWnrEnMkC")
            .sslmode(SslMode::Disable)
            .max_pool_size(1)
            .acquire_timeout(Duration::from_secs(10))
            .operation_timeout(Duration::from_millis(500)) // 短操作超时
            .build()
            .expect("config"),
    )
    .await
    .expect("connect");

    // 执行一个会超时的操作：pg_sleep 3 秒，operation_timeout 500ms
    let result = pool.query_one("SELECT pg_sleep(3), 1 AS n", &[]).await;

    assert!(result.is_err(), "pg_sleep(3) 在 500ms timeout 下应超时");

    // connection_recovery：连接应已脱池
    let stats: PoolStats = pool.stats();
    assert_eq!(stats.available, 0, "超时后连接应脱池，可用连接为 0");

    // 下一条 acquire 获取新连接，SELECT 1 应成功
    let row = pool.query_one("SELECT 1 AS n", &[]).await.expect("recovery SELECT 1");
    let n: i32 = row.get("n");
    assert_eq!(n, 1, "恢复后 SELECT 1 应成功");

    pool.close();
}

// ============================================================================
// STRESS-4: copy_large_dataset
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn stress_copy_large_dataset() {
    let pool = PostgresPool::connect(&stress_config(2)).await.expect("connect");

    let suffix = rand_id();
    let table = format!("stress_copy_{suffix}");

    let mut conn = pool.acquire().await.expect("acquire");

    // 创建表
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (id INT PRIMARY KEY, name TEXT NOT NULL, value NUMERIC)"
        ),
        &[],
    )
    .await
    .expect("create table");

    // 构造 1000 行 CSV
    let mut csv = String::with_capacity(64 * 1000);
    for i in 0..1000 {
        csv.push_str(&format!("{i}\titem_{i}\t{i}.5\n"));
    }
    let csv_bytes = csv.as_bytes();

    // COPY IN
    let rows = conn
        .copy_in_bytes(&format!("COPY {table} (id, name, value) FROM STDIN"), csv_bytes)
        .await
        .expect("copy in");
    assert_eq!(rows, 1000);

    // COPY OUT
    let out = conn
        .copy_out_bytes(&format!("COPY {table} (id, name, value) TO STDOUT"), 16 * 1024 * 1024)
        .await
        .expect("copy out");

    // 验证行数匹配：计算 \n 数量
    let out_text = String::from_utf8_lossy(&out);
    let line_count = out_text.lines().count();
    assert_eq!(line_count, 1000, "COPY OUT 应包含 1000 行");
    assert!(out_text.contains("item_0"));
    assert!(out_text.contains("item_999"));

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// STRESS-5: rapid_open_close_cycle
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn stress_rapid_open_close_cycle() {
    let cfg = stress_config(1);

    for i in 0..10 {
        let pool = PostgresPool::connect(&cfg).await.expect(&format!("connect cycle {i}"));

        pool.health().await.expect(&format!("health cycle {i}"));

        let row =
            pool.query_one("SELECT $1::int4 AS n", &[&i]).await.expect(&format!("query cycle {i}"));
        let n: i32 = row.get("n");
        assert_eq!(n, i);

        pool.close();
        let stats: PoolStats = pool.stats();
        assert!(stats.closed, "cycle {i}: 关闭后 closed 应为 true");
    }
}

// ============================================================================
// 单元测试（不依赖真实 Postgres）
// ============================================================================

#[test]
fn unit_stress_config_validate() {
    let cfg = stress_config(4);
    assert_eq!(cfg.max_pool_size, 4);
    cfg.validate().expect("valid");
}

#[test]
fn unit_pool_stats_defaults() {
    // PoolStats 无法直接构造（字段 pub 但从 pool::stats 获取）
    // 至少确保类型可用
    let _s: PoolStats;
}
