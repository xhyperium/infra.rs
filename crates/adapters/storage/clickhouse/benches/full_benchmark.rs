//! clickhousex 综合基准测试。
//!
//! ```text
//! cargo bench -p clickhousex --bench full_benchmark
//! ```
use std::fs;
use std::time::{Duration, Instant};

use clickhousex::{ClickHouseConfig, ClickHousePool};
use serde_json::Value;

const QUICK: bool = true;

fn quick1(n: u32) -> u32 {
    if QUICK { n.min(20) } else { n }
}

fn parse_bench_args() -> ClickHouseConfig {
    let env = std::env::var("CLICKHOUSE_BENCH_PASSWORD").ok();
    let password = env.unwrap_or_default();
    if password.is_empty() {
        eprintln!("跳过 benchmark: 需要 CLICKHOUSE_BENCH_PASSWORD 环境变量");
        std::process::exit(0);
    }
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        password,
        database: "default".into(),
        ..ClickHouseConfig::default()
    }
}

fn report(label: &str, iters: u32, elapsed: Duration) {
    println!(
        "bench_{label}: iters={iters} total={elapsed:?} per_iter={:?}",
        elapsed / iters.max(1)
    );
}

// ── 基准 1：SELECT 1 延迟 ─────────────────────────────────────

async fn bench_select_one(pool: &ClickHousePool) {
    let n = quick1(1000u32);
    let start = Instant::now();
    for _ in 0..n {
        pool.execute("SELECT 1").await.expect("SELECT 1");
    }
    report("clickhousex_select1", n, start.elapsed());
}

// ── 基准 2：insert_json_each_row 吞吐 vs batch size ──────────

async fn bench_insert_json_each_row(pool: &ClickHousePool) {
    let pid = std::process::id();
    for &batch in &[1usize, 10, 100, 1000] {
        let table = format!("bench_insert_each_{batch}_{pid}");
        pool.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
        ))
        .await
        .expect("创建表");

        let n = quick1(20u32);
        let start = Instant::now();
        for _ in 0..n {
            let rows: Vec<Value> = (0..batch)
                .map(|i| serde_json::json!({"id": i, "marker": format!("r{i}")}))
                .collect();
            pool.insert_json_each_row(&table, &rows)
                .await
                .expect("insert each row");
        }
        let elapsed = start.elapsed();
        report(&format!("clickhousex_insert_each_row_batch{batch}"), n, elapsed);

        pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
            .await
            .expect("清理");
    }
}

// ── 基准 3：insert_batch 分块 ─────────────────────────────────

async fn bench_insert_batch(pool: &ClickHousePool) {
    let pid = std::process::id();
    let total = 5000usize;
    let rows: Vec<Value> = (0..total)
        .map(|i| serde_json::json!({"id": i, "marker": format!("r{i}")}))
        .collect();

    for &chunk in &[100, 500, 1000, 5000] {
        let table = format!("bench_insert_batch_{chunk}_{pid}");
        pool.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
        ))
        .await
        .expect("创建表");

        let options =
            clickhousex::BatchInsertOptions { max_rows_per_chunk: chunk };

        let n = quick1(5u32);
        let start = Instant::now();
        for _ in 0..n {
            pool.insert_batch(&table, &rows, options).await.expect("insert batch");
        }
        let elapsed = start.elapsed();
        report(&format!("clickhousex_insert_batch_chunk{chunk}"), n, elapsed);

        pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
            .await
            .expect("清理");
    }
}

// ── 基准 4：query_rows 延迟 vs 行数 ──────────────────────────

async fn bench_query_rows(pool: &ClickHousePool) {
    let pid = std::process::id();
    let table = format!("bench_query_rows_{pid}");

    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("创建查询表");

    // 插入 1000 行
    let rows: Vec<Value> = (0..1000)
        .map(|i| serde_json::json!({"id": i, "marker": format!("r{i}")}))
        .collect();
    pool.insert_json_each_row(&table, &rows)
        .await
        .expect("插入数据");

    for &limit in &[10, 100, 1000] {
        let sql = format!("SELECT * FROM {table} LIMIT {limit} FORMAT TabSeparated");
        let n = quick1(50u32);
        let start = Instant::now();
        for _ in 0..n {
            let _ = pool.query_rows(&sql).await.expect("查询");
        }
        report(&format!("clickhousex_query_rows_limit{limit}"), n, start.elapsed());
    }

    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理");
}

// ── 基准 5：并发 insert（不同 max_in_flight） ──────────────────

async fn bench_concurrent_insert(config: &ClickHouseConfig) {
    let total = 500u32;
    let rows: Vec<Value> = (0..total)
        .map(|i| serde_json::json!({"id": i, "marker": format!("r{i}")}))
        .collect();

    for &max_in_flight in &[1usize, 4, 16, 64] {
        let mut cfg = config.clone();
        cfg.max_in_flight = max_in_flight;
        let pool = ClickHousePool::connect(cfg)
            .await
            .expect(&format!("连接 max_in_flight={max_in_flight}"));
        let pid = std::process::id();
        let table = format!("bench_concurrent_{max_in_flight}_{pid}");

        pool.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
        ))
        .await
        .expect("创建表");

        let chunk_size = total as usize / max_in_flight.max(1);
        let n = quick1(3u32);
        let start = Instant::now();

        for _ in 0..n {
            let mut handles = Vec::new();
            for g in 0..max_in_flight {
                let pool = pool.clone();
                let table = table.clone();
                let start_idx = g * chunk_size;
                let end_idx = if g == max_in_flight - 1 { total as usize } else { (g + 1) * chunk_size };
                let chunk_rows = rows[start_idx..end_idx].to_vec();
                handles.push(tokio::spawn(async move {
                    pool.insert_json_each_row(&table, &chunk_rows).await
                }));
            }
            for h in handles {
                h.await.expect("join").expect("insert");
            }
        }

        report(
            &format!("clickhousex_concurrent_insert_mif{max_in_flight}"),
            n,
            start.elapsed(),
        );

        pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
            .await
            .expect("清理");
        pool.close().await.expect("关闭");
    }
}

// ── 基准 6：CPU/内存采样 ──────────────────────────────────────

fn sample_process_stats() {
    let pid = std::process::id();
    if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in status.lines() {
            if line.starts_with("VmRSS:") || line.starts_with("VmPeak:") || line.starts_with("VmSize:") {
                println!("bench_process_stats: {line}");
            }
        }
    } else {
        println!("bench_process_stats: /proc/self/status 不可读（非 Linux）");
    }
}

#[tokio::main]
async fn main() {
    let cfg = parse_bench_args();

    println!("=== ClickHouse 基准测试 ===");
    sample_process_stats();

    let pool = match tokio::time::timeout(Duration::from_secs(5), ClickHousePool::connect(cfg.clone())).await {
        Ok(Ok(p)) => p,
        Ok(Err(error)) => {
            eprintln!("连接失败: {error}");
            return;
        }
        Err(_) => {
            eprintln!("连接超时");
            return;
        }
    };

    println!("\n# SELECT 1 延迟");
    bench_select_one(&pool).await;

    println!("\n# insert_json_each_row 吞吐");
    bench_insert_json_each_row(&pool).await;

    println!("\n# insert_batch 分块");
    bench_insert_batch(&pool).await;

    println!("\n# query_rows 延迟");
    bench_query_rows(&pool).await;

    pool.close().await.expect("关闭 pool");

    println!("\n# 并发 insert");
    bench_concurrent_insert(&cfg).await;

    println!("\n=== 采样结束 ===");
    sample_process_stats();
}
