//! taosx 热路径：配置构造 +（可选）REST ping。
//!
//! ```text
//! cargo run -p taosx --bench hot_path -- --quick
//! FOUNDATIONX_TAOSX_PASSWORD=... cargo run -p taosx --bench hot_path -- --live
//! ```

use std::hint::black_box;
use std::time::Instant;

use taosx::TaosConfig;

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 50_000 }
}

fn main() {
    let n = iters();
    let start = Instant::now();
    for _ in 0..n {
        let c = TaosConfig::default();
        black_box(c.rest_sql_url());
    }
    let elapsed = start.elapsed();
    println!("bench_taosx_config: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n);

    if std::env::args().any(|a| a == "--live") {
        let rt = tokio::runtime::Runtime::new().expect("rt");
        rt.block_on(async {
            let cfg = TaosConfig::from_env();
            match taosx::TaosPool::connect(cfg).await {
                Ok(pool) => {
                    let start = Instant::now();
                    let m = if std::env::args().any(|a| a == "--quick") { 20 } else { 200 };
                    for _ in 0..m {
                        pool.ping().await.expect("ping");
                    }
                    let elapsed = start.elapsed();
                    println!(
                        "bench_taosx_ping: iters={m} total={elapsed:?} per_iter={:?}",
                        elapsed / m
                    );
                    let _ = pool.close().await;
                }
                Err(e) => eprintln!("live bench skipped: {e}"),
            }
        });
    }
}
