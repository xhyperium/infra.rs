//! 有界 API 矩阵 bench：ping / write+query / metrics。
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo bench -p taosx --bench api_matrix -- --quick
//! ```

use std::time::{Duration, Instant};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::TaosPool;

#[tokio::main]
async fn main() {
    let deadline = Instant::now() + Duration::from_secs(5);
    let Ok(pool) = TaosPool::connect_from_env().await else {
        eprintln!("bench_api_matrix: skipped (no taos / connect failed)");
        return;
    };

    let mut ping_n = 0u32;
    let mut ping_ns = 0u128;
    while Instant::now() < deadline && ping_n < 50 {
        let t0 = Instant::now();
        if pool.ping().await.is_ok() {
            ping_ns += t0.elapsed().as_nanos();
            ping_n += 1;
        } else {
            break;
        }
    }
    if ping_n > 0 {
        eprintln!("bench_ping: n={ping_n} avg_us={}", (ping_ns / u128::from(ping_n)) / 1_000);
    }

    let table = format!("_sc_bench_{}", std::process::id());
    let prec = pool.precision();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    let ts = prec.to_nanos(prec.from_nanos(now));
    let tick = Tick {
        symbol: "BN".into(),
        bid: Price::new(Decimal::try_new(1, 2).unwrap()),
        ask: Price::new(Decimal::try_new(2, 2).unwrap()),
        ts,
    };
    let t0 = Instant::now();
    if pool.write_series(&table, vec![tick]).await.is_ok() {
        let _ = pool.query_series(&table, ts, ts).await;
        eprintln!("bench_write_query_us={}", t0.elapsed().as_micros());
    }
    let _ = pool.metrics_prometheus();
    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{table}`")).await;
    let _ = pool.close().await;
}
