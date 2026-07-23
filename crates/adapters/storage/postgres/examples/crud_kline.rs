//! postgresx CRUD + BTC/ETH K 线数据导入示例
//!
//! 用法：
//!   FOUNDATIONX_POSTGRESX_HOST=127.0.0.1 \
//!   FOUNDATIONX_POSTGRESX_PORT=5432 \
//!   FOUNDATIONX_POSTGRESX_DATABASE=market_binance \
//!   FOUNDATIONX_POSTGRESX_USER=market_binance \
//!   FOUNDATIONX_POSTGRESX_PASSWORD=... \
//!   FOUNDATIONX_POSTGRESX_SSLMODE=disable \
//!   cargo run --example crud_kline

use kernel::{XError, XResult};
use postgresx::*;
use std::fs;
use std::path::Path;

const DATA_DIR: &str = "/home/workspace/data/binance_futures/merged";

#[tokio::main]
async fn main() -> XResult<()> {
    let cfg = PostgresConfig::from_env()?;
    let pool = PostgresPool::connect(&cfg).await?;
    println!("=== postgresx CRUD 示例 ===\n");
    println!("✓ 连接: {}", pool.summary());

    // 1. 建表
    println!("\n--- 1. 建表 ---");
    create_table(&pool).await?;
    println!("✓ 表 postgresx_kline 就绪");

    // 2. 导入 BTC/ETH 数据（先清空旧数据）
    println!("\n--- 2. 数据导入 ---");
    pool.execute("DELETE FROM postgresx_kline", &[]).await?;
    let btc_count = import_csv(&pool, "BTCUSDT", &format!("{DATA_DIR}/BTCUSDT/1d.csv")).await?;
    let eth_count = import_csv(&pool, "ETHUSDT", &format!("{DATA_DIR}/ETHUSDT/1d.csv")).await?;
    println!("✓ BTCUSDT: {btc_count} 条  |  ETHUSDT: {eth_count} 条");

    // 3. CRUD 操作
    println!("\n--- 3. CRUD ---");
    crud_operations(&pool).await?;

    // 4. 聚合查询
    println!("\n--- 4. 聚合查询 ---");
    aggregate_queries(&pool).await?;

    // 5. 统计
    let s = pool.stats();
    println!("\n--- 5. 池统计 ---");
    println!("最大={}  当前={}  可用={}  关闭={}", s.max_size, s.size, s.available, s.closed);

    pool.close();
    println!("\n✓ 完成");
    Ok(())
}

// ═══════════════════════════════

async fn create_table(pool: &PostgresPool) -> XResult<()> {
    pool.execute(
        "CREATE TABLE IF NOT EXISTS postgresx_kline (\
           id BIGSERIAL PRIMARY KEY, \
           symbol TEXT NOT NULL, \
           interval TEXT NOT NULL, \
           open_time BIGINT NOT NULL, \
           open DECIMAL NOT NULL, \
           high DECIMAL NOT NULL, \
           low DECIMAL NOT NULL, \
           close DECIMAL NOT NULL, \
           volume DECIMAL NOT NULL, \
           close_time BIGINT NOT NULL, \
           quote_volume DECIMAL NOT NULL, \
           count BIGINT NOT NULL, \
           taker_buy_volume DECIMAL NOT NULL, \
           taker_buy_quote_volume DECIMAL NOT NULL, \
           UNIQUE(symbol, interval, open_time)\
         )",
        &[],
    )
    .await?;
    Ok(())
}

async fn import_csv(pool: &PostgresPool, symbol: &str, path: &str) -> XResult<usize> {
    let content =
        fs::read_to_string(path).map_err(|e| XError::invalid(format!("无法读取 {path}: {e}")))?;

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Ok(0);
    }

    // 通过 COPY 批量导入
    let mut csv_lines: Vec<String> = Vec::new();
    let interval = Path::new(path).file_stem().unwrap().to_string_lossy().to_string();

    for line in &lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 11 {
            continue;
        }
        csv_lines.push(format!(
            "{symbol},{interval},{},{},{},{},{},{},{},{},{},{},{}",
            fields[0],
            fields[1],
            fields[2],
            fields[3],
            fields[4],
            fields[5],
            fields[6],
            fields[7],
            fields[8],
            fields[9],
            fields[10]
        ));
    }

    let csv_data = csv_lines.join("\n") + "\n";
    let rows = pool.copy_in_bytes(
        "COPY postgresx_kline (symbol, interval, open_time, open, high, low, close, volume, close_time, quote_volume, count, taker_buy_volume, taker_buy_quote_volume) FROM STDIN CSV",
        csv_data.as_bytes(),
    ).await?;

    Ok(rows as usize)
}

async fn crud_operations(pool: &PostgresPool) -> XResult<()> {
    // CREATE — 单条 INSERT 演示
    let _ = pool.execute(
        "INSERT INTO postgresx_kline (symbol, interval, open_time, open, high, low, close, volume, close_time, quote_volume, count, taker_buy_volume, taker_buy_quote_volume) \
         VALUES ('MANUAL','1d',1,0.0,0.0,0.0,0.0,0.0,1,0.0,1,0.0,0.0) \
         ON CONFLICT (symbol, interval, open_time) DO NOTHING",
        &[],
    ).await;
    println!("  CREATE: 硬编码 INSERT 测试行");

    // READ — query SYMBOL column and CAST close to FLOAT8 for Get by index
    let row = pool
        .query_one(
            "SELECT symbol, close::FLOAT8 FROM postgresx_kline ORDER BY open_time DESC LIMIT 1",
            &[],
        )
        .await?;
    let sym: String = row.get("symbol");
    let cls: f64 = row.get(1usize);
    println!("  READ:   最新 {sym} close={cls:.2}");

    // READ by symbol
    let count: i64 = pool
        .query_one("SELECT COUNT(*) FROM postgresx_kline WHERE symbol='BTCUSDT'", &[])
        .await?
        .get(0);
    println!("  READ:   BTCUSDT 共 {count} 条");

    // UPDATE  — update a column that exists
    let n = pool
        .execute(
            "UPDATE postgresx_kline SET count=count+1 WHERE symbol='BTCUSDT' AND count<10",
            &[],
        )
        .await?;
    println!("  UPDATE: 更新 {n} 行");

    // DELETE  — delete the manually inserted test row
    let n = pool.execute("DELETE FROM postgresx_kline WHERE symbol='MANUAL'", &[]).await?;
    println!("  DELETE: 删除 {n} 行");

    Ok(())
}

async fn aggregate_queries(pool: &PostgresPool) -> XResult<()> {
    // BTC 7 日平均
    let row = pool
        .query_one(
            "SELECT AVG(close)::FLOAT8 AS avg_close FROM postgresx_kline \
         WHERE symbol='BTCUSDT'",
            &[],
        )
        .await?;
    let avg: f64 = row.get("avg_close");
    println!("  BTC 平均收盘价: {avg:.2}");

    // ETH 最高最低
    let row = pool.query_one(
        "SELECT MAX(high)::FLOAT8, MIN(low)::FLOAT8 FROM postgresx_kline WHERE symbol='ETHUSDT'",
        &[],
    ).await?;
    let max_h: f64 = row.get(0usize);
    let min_l: f64 = row.get(1usize);
    println!("  ETH 最高: {max_h:.2}  最低: {min_l:.2}");

    let row = pool
        .query_one("SELECT COUNT(*), COUNT(DISTINCT symbol)::INT8 FROM postgresx_kline", &[])
        .await?;
    let total: i64 = row.get(0usize);
    let symbols: i64 = row.get(1usize);
    println!("  总计: {total} 条  品种: {symbols} 个");

    Ok(())
}
