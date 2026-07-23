//! taosx BTC/ETH kline 完整 CRUD 示例
//!
//! 功能：
//!   - 从 CSV 读取 Binance 合约 BTC/ETH K 线数据
//!   - 批量导入到 TDengine
//!   - 查询、更新、删除操作
//!
//! 数据来源: /home/workspace/data/binance_futures/merged/
//! 配置:     FOUNDATIONX_TAOSX_* 环境变量
//!
//! 运行:
//!   cargo run --example btc_eth_crud -p taosx
//!   cargo run --example btc_eth_crud -p taosx -- --quick

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::{TaosPool, build_insert_sql_chunks};

// ---------------------------------------------------------------------------
// Kline 数据结构
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct KlineRow {
    open_time_ms: i64,
    close: f64,
}

impl KlineRow {
    fn to_tick(&self, symbol: &str, prec: taosx::TsPrecision) -> Tick {
        let ts = prec.to_nanos(prec.from_nanos(self.open_time_ms * 1_000_000));
        let scaled = (self.close * 100.0) as i128;
        let price = Decimal::try_new(scaled, 2).unwrap_or_else(|_| Decimal::try_new(0, 2).unwrap());
        Tick { symbol: symbol.into(), bid: Price::new(price), ask: Price::new(price), ts }
    }
}

// ---------------------------------------------------------------------------
// CSV 读取
// ---------------------------------------------------------------------------
fn read_kline_csv(path: &str, max_rows: Option<usize>) -> Vec<KlineRow> {
    let file = File::open(path).expect("open CSV");
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 {
            continue;
        }

        rows.push(KlineRow {
            open_time_ms: fields[0].parse().unwrap_or(0),
            close: fields[4].parse().unwrap_or(0.0),
        });

        if let Some(max) = max_rows {
            if rows.len() >= max {
                break;
            }
        }
    }
    rows
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let max_rows: Option<usize> = if quick { Some(1000) } else { None };
    let batch_size: usize = if quick { 500 } else { 2000 };

    println!("=== taosx BTC/ETH Kline CRUD Example ===");
    println!("mode: {}", if quick { "fast (max 1000 rows per symbol)" } else { "full (all data)" });

    // --- 1. Connect ---
    let start = Instant::now();
    let pool = TaosPool::connect_from_env()
        .await
        .expect("connect to TDengine (set FOUNDATIONX_TAOSX_* env)");
    let prec = pool.precision();
    let db = "market_binance";
    pool.exec_sql(&format!("CREATE DATABASE IF NOT EXISTS `{db}` PRECISION 'ns'")).await.unwrap();
    pool.exec_sql(&format!("USE `{db}`")).await.unwrap();
    println!("\n1. CONNECT: ok ({:?}) precision={prec:?}", start.elapsed());

    // --- 2. Define datasets ---
    let data_root = "/home/workspace/data/binance_futures/merged";
    let btc_csv = format!("{data_root}/BTCUSDT/1h.csv");
    let eth_csv = format!("{data_root}/ETHUSDT/1h.csv");
    let datasets: Vec<(&str, &str, &str)> =
        vec![("BTCUSDT", "1h", &btc_csv), ("ETHUSDT", "1h", &eth_csv)];

    let total_start = Instant::now();
    let mut total_inserts = 0u64;
    let mut total_queries = 0u64;
    let mut total_updates = 0u64;
    let mut total_deletes = 0u64;
    let mut total_errors = 0u64;

    for (symbol, interval, path) in &datasets {
        println!("\n=== {symbol}/{interval} ===");
        let table = format!("st_kline_{symbol}_{interval}").to_lowercase();

        // --- 2a. CREATE TABLE ---
        let s = Instant::now();
        let ddl = format!(
            "CREATE STABLE IF NOT EXISTS `{table}` \
             (ts TIMESTAMP, bid NCHAR(64), ask NCHAR(64)) \
             TAGS (symbol NCHAR(16))"
        );
        pool.exec_sql(&ddl).await.expect("create stable");
        println!("  CREATE: ok ({:?})", s.elapsed());

        // --- 2b. READ CSV ---
        let s = Instant::now();
        let rows = read_kline_csv(path, max_rows);
        let count = rows.len();
        println!("  READ:  {} rows from CSV ({:?})", count, s.elapsed());

        // --- 2c. INSERT ---
        let s = Instant::now();
        let ticks: Vec<Tick> = rows.iter().map(|r| r.to_tick(symbol, prec)).collect();
        let mut chunk_count = 0u64;
        let mut offset = 0;
        while offset < ticks.len() {
            let batch = &ticks[offset..(offset + batch_size).min(ticks.len())];
            match build_insert_sql_chunks(&table, batch, prec, batch_size) {
                Ok(chunks) => {
                    for sql in &chunks {
                        if pool.exec_sql(sql).await.is_err() {
                            total_errors += 1;
                        } else {
                            chunk_count += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  chunk error: {e}");
                    total_errors += 1;
                }
            }
            offset += batch_size;
        }
        total_inserts += chunk_count;
        println!("  INSERT: {} rows in {} chunks ({:?})", count, chunk_count, s.elapsed());

        // --- 2d. QUERY count ---
        let s = Instant::now();
        if let Ok(r) = pool.exec_sql(&format!("SELECT COUNT(*) FROM `{table}`")).await {
            total_queries += 1;
            let row_count = r.rows.len();
            println!(
                "  QUERY: SELECT COUNT(*) -> {} ({} rows) ({:?})",
                row_count,
                if row_count > 0 { "ok" } else { "empty" },
                s.elapsed()
            );
        }

        // --- 2e. QUERY range summary ---
        let s = Instant::now();
        let minmax_sql = format!("SELECT MIN(ts), MAX(ts), COUNT(*) FROM `{table}`");
        if let Ok(r) = pool.exec_sql(&minmax_sql).await {
            total_queries += 1;
            println!("  QUERY: MIN/MAX/COUNT -> {} result rows ({:?})", r.rows.len(), s.elapsed());
        }

        // --- 2f. QUERY latest 10 ---
        let s = Instant::now();
        let latest_sql = format!("SELECT * FROM `{table}` ORDER BY ts DESC LIMIT 10");
        if let Ok(r) = pool.exec_sql(&latest_sql).await {
            total_queries += 1;
            println!("  QUERY: latest 10 -> {} rows ({:?})", r.rows.len(), s.elapsed());
        }

        // --- 2g. UPDATE (upsert last row) ---
        if let Some(last) = rows.last() {
            let s = Instant::now();
            let tick = last.to_tick(symbol, prec);
            if pool.write_series(&table, vec![tick]).await.is_ok() {
                total_updates += 1;
            }
            println!("  UPDATE: upsert last row ({:?})", s.elapsed());
        }

        // --- 2h. DELETE test row ---
        let s = Instant::now();
        // Insert a marker row with future timestamp, then delete it
        let future_ts = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0))
            + 3_600_000_000_000; // +1h
        let marker_ts = prec.to_nanos(prec.from_nanos(future_ts));
        let marker_tick = Tick {
            symbol: "DELETE_TEST".into(),
            bid: Price::new(Decimal::try_new(0, 2).unwrap()),
            ask: Price::new(Decimal::try_new(0, 2).unwrap()),
            ts: marker_ts,
        };
        if pool.write_series(&table, vec![marker_tick]).await.is_ok() {
            let del_sql = format!("DELETE FROM `{table}` WHERE ts = {marker_ts}");
            if pool.exec_sql(&del_sql).await.is_ok() {
                total_deletes += 1;
            }
        }
        println!("  DELETE: marker row insert+delete ({:?})", s.elapsed());
    }

    // --- 3. Pool stats ---
    println!("\n=== POOL STATS ===");
    let ps = pool.stats();
    println!("  in_flight={} closed={}", ps.in_flight, pool.is_closed());
    if let Ok(h) = pool.health().await {
        println!("  health: ready={} detail={}", h.ready, h.detail);
    }

    // --- 4. Metrics ---
    println!("\n=== METRICS (Prometheus) ===");
    let metrics = pool.metrics_prometheus();
    for line in metrics.lines().take(15) {
        println!("  {}", line);
    }
    if metrics.lines().count() > 15 {
        println!("  ... ({} total lines)", metrics.lines().count());
    }

    // --- 5. Final summary ---
    let elapsed = total_start.elapsed();
    println!("\n=== SUMMARY ===");
    println!(
        "  inserts={total_inserts} queries={total_queries} updates={total_updates} deletes={total_deletes} errors={total_errors}"
    );
    println!("  total elapsed={elapsed:?}");

    // --- 6. Cleanup ---
    let s = Instant::now();
    let _ = pool.close().await;
    println!("\nCLOSE: done ({:?})", s.elapsed());
    println!("=== DONE ===");
}
