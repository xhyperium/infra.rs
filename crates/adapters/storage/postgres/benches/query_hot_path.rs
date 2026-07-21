//! 热路径微基准：参数化 `SELECT $1::int`（harness = false）。
//!
//! 需要可连接的 Postgres（同 live 环境变量）。无环境时以 exit 0 跳过。
//!
//! ```bash
//! cargo bench -p postgresx --bench query_hot_path
//! ```

use std::time::Instant;

use postgresx::{PostgresConfig, PostgresPool};

#[tokio::main]
async fn main() {
    let cfg = match PostgresConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip query_hot_path bench: no config ({e})");
            return;
        }
    };

    let pool = match PostgresPool::connect(&cfg).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skip query_hot_path bench: connect failed ({e})");
            return;
        }
    };

    // warmup
    for i in 0..50 {
        let row = pool.query_one("SELECT $1::int AS n", &[&i]).await.expect("warmup");
        let _: i32 = row.get(0);
    }

    let iters = std::env::var("POSTGRESX_BENCH_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000usize);

    let start = Instant::now();
    for i in 0..iters {
        let n = (i % 10_000) as i32;
        let row = pool.query_one("SELECT $1::int AS n", &[&n]).await.expect("query");
        let got: i32 = row.get(0);
        assert_eq!(got, n);
    }
    let elapsed = start.elapsed();
    let per = elapsed / iters as u32;
    println!(
        "postgresx query_hot_path: iters={iters} total={elapsed:?} avg={per:?} pool={:?}",
        pool.stats()
    );
    pool.close();
}
