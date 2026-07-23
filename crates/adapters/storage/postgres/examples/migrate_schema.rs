//! postgresx Migrator 示例
//!
//! 演示完整 migration 工作流：正向 apply、状态查询、回滚。
//!
//! 用法: cargo run --example migrate_schema

use kernel::XResult;
use postgresx::*;

#[tokio::main]
async fn main() -> XResult<()> {
    // 1. 连接
    let cfg = PostgresConfig::from_env()?;
    let pool = PostgresPool::connect(&cfg).await?;
    println!("已连接 {}", pool.summary());

    // 2. 创建 migration
    let m1 = Migration::with_down(
        1,
        "create_prices",
        "CREATE TABLE IF NOT EXISTS postgresx_prices (\
           id SERIAL PRIMARY KEY, \
           symbol TEXT NOT NULL, \
           price DECIMAL NOT NULL, \
           ts TIMESTAMPTZ DEFAULT now()\
         )",
        "DROP TABLE IF EXISTS postgresx_prices",
    )?;

    let m2 = Migration::new(
        2,
        "add_index",
        "CREATE INDEX IF NOT EXISTS idx_prices_symbol ON postgresx_prices(symbol)",
    )?;

    let m3 = Migration::new(
        3,
        "add_volume",
        "ALTER TABLE postgresx_prices ADD COLUMN IF NOT EXISTS volume DECIMAL DEFAULT 0",
    )?;

    // 3. 应用全部
    let mig = Migrator::new(pool.clone(), vec![m1, m2, m3])?;
    let report = mig.apply().await?;
    println!("已应用 {} 条 migration: {:?}", report.applied_now.len(), report.applied_now);

    // 4. 查看状态
    let status = mig.status().await?;
    println!(
        "已应用版本: {:?}, 待执行版本: {:?}",
        status.applied.iter().map(|a| a.version).collect::<Vec<_>>(),
        status.pending
    );

    // 5. 测试表
    pool.execute(
        "INSERT INTO postgresx_prices (symbol, price, volume) VALUES ($1, $2, $3)",
        &[&"BTCUSDT", &50000.0f64, &100.0f64],
    )
    .await?;
    let row = pool
        .query_one(
            "SELECT symbol, price, volume FROM postgresx_prices WHERE symbol=$1",
            &[&"BTCUSDT"],
        )
        .await?;
    println!(
        "已插入: {} price={} volume={}",
        row.get::<_, &str>("symbol"),
        row.get::<_, f64>("price"),
        row.get::<_, f64>("volume")
    );

    // 6. 回滚（仅 migration 1 含 down_sql）
    let rolled_back = mig.down().await?;
    println!("已回滚版本: {:?}", rolled_back);

    // 7. 验证回滚后无待执行 migration 被意外移除
    let status_after = mig.status().await?;
    println!(
        "回滚后已应用版本: {:?}, 待执行版本: {:?}",
        status_after.applied.iter().map(|a| a.version).collect::<Vec<_>>(),
        status_after.pending
    );

    // 8. 再次 apply（重放）
    let report2 = mig.apply().await?;
    println!("重放完成: {} 条", report2.applied_now.len());

    pool.close();
    Ok(())
}
