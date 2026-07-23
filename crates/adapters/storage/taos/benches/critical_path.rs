//! taosx critical-path latency benchmark
//!
//! Measures latency of all critical data paths:
//!   connect, ping, health, write_series, query_series, exec_sql,
//!   write_batch_chunked, write_batch_idempotent, metrics, close
//!
//! Usage:
//!   cargo bench -p taosx --bench critical_path -- --quick

use std::time::{Duration, Instant};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::{TaosConfig, TaosPool, TsPrecision, build_insert_sql_chunks};

fn now_ts(prec: TsPrecision) -> i64 {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);
    prec.to_nanos(prec.from_nanos(ns))
}

fn tick_1ms(ts: i64) -> Tick {
    Tick {
        symbol: "BT".into(),
        bid: Price::new(Decimal::try_new(10000, 2).unwrap()),
        ask: Price::new(Decimal::try_new(10001, 2).unwrap()),
        ts,
    }
}

fn read_rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok().and_then(|s| {
        s.lines()
            .find(|l| l.starts_with("VmRSS:"))
            .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
    })
}

#[tokio::main]
async fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let n: usize = if quick { 20 } else { 50 };
    let bn: usize = if quick { 10 } else { 20 };

    let mem_before = read_rss_kb();

    // --- connect ---
    let start = Instant::now();
    let cfg = TaosConfig::from_env();
    let pool = match tokio::time::timeout(Duration::from_secs(5), TaosPool::connect(cfg)).await {
        Ok(Ok(p)) => p,
        _ => {
            eprintln!("CRITICAL_BENCH: skipped (no taos)");
            return;
        }
    };
    eprintln!("CRITICAL_BENCH: connect={:?}", start.elapsed());

    let mem_after_connect = read_rss_kb();

    let prec = pool.precision();
    let ts = now_ts(prec);
    let table = format!("_sc_crit_{}", std::process::id());

    // --- 1. ping (warmest/fastest path) ---
    eprintln!("-- 1. ping");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            if pool.ping().await.is_ok() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    ping: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 2. health ---
    eprintln!("-- 2. health");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            if let Ok(h) = pool.health().await {
                if h.ready {
                    ok += 1;
                }
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    health: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 3. write_series (single tick) ---
    eprintln!("-- 3. write_series");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            let t = tick_1ms(ts);
            if pool.write_series(&table, vec![t]).await.is_ok() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    write_series: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 4. query_series ---
    eprintln!("-- 4. query_series");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            if pool.query_series(&table, ts, ts + 1000).await.is_ok() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    query_series: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 5. exec_sql (raw) ---
    eprintln!("-- 5. exec_sql");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            if pool.exec_sql("SELECT SERVER_VERSION()").await.is_ok() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    exec_sql: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 6. write_batch (chunked, multi-symbol) ---
    eprintln!("-- 6. write_batch_chunked ({bn} symbols)");
    let symbols: Vec<String> = (0..bn).map(|i| format!("S{}", i)).collect();
    let ticks: Vec<Tick> = symbols
        .iter()
        .map(|s| {
            let mut t = tick_1ms(ts);
            t.symbol = s.clone();
            t
        })
        .collect();
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            let chunks = build_insert_sql_chunks(&table, &ticks, prec, 500).unwrap_or_default();
            let mut all_ok = true;
            for sql in &chunks {
                if pool.exec_sql(sql).await.is_err() {
                    all_ok = false;
                    break;
                }
            }
            if all_ok {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    write_batch_chunked: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 7. write_batch_idempotent ---
    eprintln!("-- 7. write_batch_idempotent");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            if pool.write_batch_idempotent(&table, &ticks).await.is_ok() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    write_batch_idempotent: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 8. metrics_prometheus ---
    eprintln!("-- 8. metrics_prometheus");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            let s = pool.metrics_prometheus();
            if !s.is_empty() {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    metrics: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 9. write+query roundtrip ---
    eprintln!("-- 9. write+query roundtrip");
    {
        let start = Instant::now();
        let mut ok = 0;
        for _ in 0..n {
            let t = tick_1ms(ts);
            if pool.write_series(&table, vec![t]).await.is_ok()
                && pool.query_series(&table, ts, ts + 1000).await.is_ok()
            {
                ok += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!(
            "    write+query: iters={n} ok={ok} total={elapsed:?} per_iter={:?}",
            elapsed / n.max(1) as u32
        );
    }

    // --- 10. close ---
    eprintln!("-- 10. close");
    {
        let start = Instant::now();
        let _ = pool.clone().close().await;
        eprintln!("    close: total={:?}", start.elapsed());
    }

    // --- cleanup ---
    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{table}`")).await;
    let _ = pool.close().await;

    let mem_after = read_rss_kb();
    if let (Some(before), Some(after_conn), Some(after)) =
        (mem_before, mem_after_connect, mem_after)
    {
        eprintln!(
            "CRITICAL_BENCH: mem VmRSS before={before}kB after_connect={after_conn}kB after_all={after}kB delta={}kB",
            after.saturating_sub(before)
        );
    }
    eprintln!("CRITICAL_BENCH: done");
}
