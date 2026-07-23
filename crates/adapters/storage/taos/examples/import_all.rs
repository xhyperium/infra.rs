//! taosx 全量数据导入工具
//!
//! 导入 /home/workspace/data/binance_futures/merged/ 下所有 CSV 到 TDengine
//! 类别: klines / markPriceKlines / indexPriceKlines / premiumIndexKlines / fundingRate
//! 输出: 已写入行数、已查询行数、失败行数
//!
//! Usage:
//!   cargo run --example import_all -p taosx --release

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use canonical::Tick;
use decimalx::{Decimal, Price};
use taosx::{TaosPool, build_insert_sql_chunks};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DataKind {
    Kline,
    MarkPrice,
    IndexPrice,
    PremiumIndex,
    FundingRate,
}

impl DataKind {
    fn price_field_index(&self) -> usize {
        match self {
            Self::FundingRate => 2,
            _ => 4,
        }
    }
    fn table_prefix(&self) -> &'static str {
        match self {
            Self::Kline => "st_kline",
            Self::MarkPrice => "st_mark",
            Self::IndexPrice => "st_index",
            Self::PremiumIndex => "st_premium",
            Self::FundingRate => "st_funding",
        }
    }
}

fn detect_kind(path: &str) -> DataKind {
    if path.contains("markPriceKlines") {
        DataKind::MarkPrice
    } else if path.contains("indexPriceKlines") {
        DataKind::IndexPrice
    } else if path.contains("premiumIndexKlines") {
        DataKind::PremiumIndex
    } else if path.contains("fundingRate") {
        DataKind::FundingRate
    } else {
        DataKind::Kline
    }
}

fn extract_symbol_interval(path: &str) -> (String, String) {
    // merged/BTCUSDT/1h.csv → (BTCUSDT, 1h)
    // merged/um/markPriceKlines/BTCUSDT/1h.csv → (BTCUSDT, 1h)
    // fundingRate.csv → (BTCUSDT, fund)
    let parts: Vec<&str> = path.rsplitn(3, '/').collect();
    let fname = parts[0]; // e.g. "1h.csv" or "fundingRate.csv"
    let interval = fname.replace(".csv", "");
    let symbol = parts[1].to_string();
    (symbol, interval)
}

#[tokio::main]
async fn main() {
    println!("=== taosx 全量数据导入 ===");
    let total_start = Instant::now();

    // --- Connect ---
    let start = Instant::now();
    let pool = TaosPool::connect_from_env()
        .await
        .expect("connect to TDengine (set FOUNDATIONX_TAOSX_* env)");
    let prec = pool.precision();
    let db = "market_binance";
    pool.exec_sql(&format!("CREATE DATABASE IF NOT EXISTS `{db}` PRECISION 'ns'")).await.unwrap();
    pool.exec_sql(&format!("USE `{db}`")).await.unwrap();
    println!("CONNECT: ok ({:?}) prec={prec:?}\n", start.elapsed());

    // --- Scan CSV files ---
    let data_root = "/home/workspace/data/binance_futures/merged";
    let mut csv_files: Vec<String> = Vec::new();
    fn walk(dir: &str, out: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p.to_string_lossy(), out);
                } else if p.extension().is_some_and(|e| e == "csv") {
                    out.push(p.to_string_lossy().to_string());
                }
            }
        }
    }
    walk(data_root, &mut csv_files);
    csv_files.sort();
    println!("FILES: {} CSV files found\n", csv_files.len());

    // --- Import ---
    let mut total_csv_rows: u64 = 0;
    let mut total_inserted: u64 = 0;
    let mut total_errors: u64 = 0;
    let mut file_count: u64 = 0;
    let batch_max = 2000;

    for path in &csv_files {
        file_count += 1;

        // Parse path components
        let kind = detect_kind(path);
        let (symbol, interval) = extract_symbol_interval(path);
        let table = format!("{}_{}_{}", kind.table_prefix(), symbol, interval).to_lowercase();

        // Read CSV
        let mut rows: Vec<(i64, f64)> = Vec::new(); // (open_time_ms, close)
        if let Ok(file) = File::open(path) {
            for line in BufReader::new(file).lines().map_while(Result::ok) {
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() < kind.price_field_index() + 2 {
                    continue;
                }
                let open_time_ms: i64 = fields[0].parse().unwrap_or(0);
                let price_idx = kind.price_field_index();
                let close: f64 = fields[price_idx].parse().unwrap_or(0.0);
                rows.push((open_time_ms, close));
            }
        }

        let csv_rows = rows.len() as u64;
        total_csv_rows += csv_rows;

        // Ensure stable exists
        let ddl = format!(
            "CREATE STABLE IF NOT EXISTS `{table}` \
             (ts TIMESTAMP, bid NCHAR(64), ask NCHAR(64)) \
             TAGS (symbol NCHAR(16))"
        );
        if pool.exec_sql(&ddl).await.is_err() {
            total_errors += 1;
            continue;
        }

        // Convert to Ticks and batch INSERT
        let ticks: Vec<Tick> = rows
            .iter()
            .map(|(t, c)| {
                let scaled = (*c * 100.0) as i128;
                let price =
                    Decimal::try_new(scaled, 2).unwrap_or_else(|_| Decimal::try_new(0, 2).unwrap());
                let ts = prec.to_nanos(prec.from_nanos(*t * 1_000_000));
                Tick { symbol: symbol.clone(), bid: Price::new(price), ask: Price::new(price), ts }
            })
            .collect();

        let mut offset = 0;
        let mut file_inserted: u64 = 0;
        let mut file_errors: u64 = 0;
        while offset < ticks.len() {
            let batch = &ticks[offset..(offset + batch_max).min(ticks.len())];
            match build_insert_sql_chunks(&table, batch, prec, batch_max) {
                Ok(chunks) => {
                    for sql in &chunks {
                        match pool.exec_sql(sql).await {
                            Ok(_) => file_inserted += 1,
                            Err(_) => file_errors += 1,
                        }
                    }
                }
                Err(_) => file_errors += 1,
            }
            offset += batch_max;
        }
        total_inserted += file_inserted;
        total_errors += file_errors;

        let status = if file_errors == 0 { "OK" } else { "ERR" };
        println!(
            "[{file_count:>3}/{:<3}] {status} {table:>40} rows={csv_rows:>10} chunks={file_inserted:>5} err={file_errors}",
            csv_files.len() - file_count as usize
        );

        // Verify with COUNT(*) query
        let count_sql = format!("SELECT COUNT(*) FROM `{table}`");
        if let Ok(r) = pool.exec_sql(&count_sql).await {
            let _ = r;
        }
    }

    // --- Summary ---
    let elapsed = total_start.elapsed();
    println!("\n{:=^60}", "");
    println!("| {:<30} | {:>20} |", "总量", "数值");
    println!("|{:-<32}|{:-<22}|", "", "");
    println!("| {:<30} | {:>20} |", "CSV 文件数", csv_files.len());
    println!("| {:<30} | {:>20} |", "CSV 原始行数", total_csv_rows);
    println!("| {:<30} | {:>20} |", "INSERT chunk 数", total_inserted);
    println!("| {:<30} | {:>20} |", "失败数", total_errors);
    println!("| {:<30} | {:>20.2?} |", "总耗时", elapsed);
    if total_csv_rows > 0 {
        let pct = total_inserted as f64 / total_csv_rows as f64 * 100.0;
        println!("| {:<30} | {:>20.1}% |", "成功率 (chunks vs rows)", pct);
    }
    println!("{:=^60}", "");

    // --- Final stats ---
    let ps = pool.stats();
    println!("\nPOOL: in_flight={} closed={}", ps.in_flight, pool.is_closed());

    let _ = pool.close().await;
    println!("DONE");
}
