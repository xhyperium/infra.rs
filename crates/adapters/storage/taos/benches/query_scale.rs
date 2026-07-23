//! taosx query scalability benchmark
//!
//! Measures query_series latency scaling with result set sizes:
//!   0 rows (empty), 10, 100, 1_000, 10_000
//!
//! Each size is run 3 times, reporting min/avg/max.
//!
//! Usage:
//!   cargo bench -p taosx --bench query_scale
//!   cargo bench -p taosx --bench query_scale -- --quick

use std::time::{Duration, Instant};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::{TaosPool, TaosConfig};

fn tick(ts: i64, sym: &str) -> Tick {
    Tick {
        symbol: sym.into(),
        bid: Price::new(Decimal::try_new(10000, 2).unwrap()),
        ask: Price::new(Decimal::try_new(10001, 2).unwrap()),
        ts,
    }
}

#[tokio::main]
async fn main() {
    let quick = std::env::args().any(|a| a == "--quick");

    let prec;
    let pool = {
        let cfg = TaosConfig::from_env();
        match tokio::time::timeout(Duration::from_secs(5), TaosPool::connect(cfg)).await {
            Ok(Ok(p)) => { prec = p.precision(); p }
            _ => { eprintln!("QUERY_SCALE: skipped (no taos)"); return; }
        }
    };

    let table = format!("_qs_{}", std::process::id());
    let base_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    let base_ts = prec.to_nanos(prec.from_nanos(base_ts));

    // Sizes to test: 0 (empty), 10, 100, 1K, 10K
    let sizes: Vec<usize> = if quick { vec![0, 10, 100, 1_000] } else { vec![0, 10, 100, 1_000, 10_000] };
    let runs = if quick { 2usize } else { 3usize };

    eprintln!("QUERY_SCALE: table={table} precision={prec:?} sizes={sizes:?}");

    // Pre-populate max-size data so query ranges grow incrementally without re-inserts
    let max_size = *sizes.last().unwrap();
    let symbols: Vec<String> = (0..max_size).map(|i| format!("S{i:05}")).collect();

    // Insert data in chunks
    let chunk_size = 1000;
    let mut inserted = 0;
    while inserted < max_size {
        let batch: Vec<Tick> = symbols[inserted..(inserted + chunk_size).min(max_size)]
            .iter()
            .map(|s| tick(base_ts + inserted as i64, s))
            .collect();
        let _ = pool.write_series(&table, batch).await;
        inserted += chunk_size;
        eprintln!("    inserted {inserted}/{max_size} rows");
    }

    eprintln!("{:-^66}", "");
    eprintln!("| {:<8} | {:>8} | {:>8} | {:>8} | {:>12} |", "rows", "min(ms)", "avg(ms)", "max(ms)", "per-row(us)");
    eprintln!("|{:-<10}|{:-<10}|{:-<10}|{:-<10}|{:-<14}|", "", "", "", "", "");

    for &size in &sizes {
        let mut latencies = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            let end_ts = if size == 0 { base_ts } else { base_ts + size as i64 };
            let _ = pool.query_series(&table, base_ts, end_ts).await;
            latencies.push(start.elapsed());
        }
        latencies.sort_by_key(|d| d.as_nanos());
        let min = latencies.first().unwrap();
        let max = latencies.last().unwrap();
        let avg = latencies.iter().sum::<Duration>() / runs as u32;
        let per_row_us = if size > 0 {
            avg.as_micros() as f64 / size as f64
        } else {
            0.0
        };
        eprintln!("| {:<8} | {:>8.2} | {:>8.2} | {:>8.2} | {:>12.1} |",
            size, min.as_secs_f64() * 1000.0, avg.as_secs_f64() * 1000.0,
            max.as_secs_f64() * 1000.0, per_row_us);
    }

    eprintln!("{:-^66}", "");

    // Cleanup
    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{table}`")).await;
    let _ = pool.close().await;
    eprintln!("QUERY_SCALE: done");
}
