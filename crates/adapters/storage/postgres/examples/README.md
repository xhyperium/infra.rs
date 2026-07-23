# postgresx 示例

完整的 postgresx 使用示例，覆盖连接池、SQL、COPY、CRUD、迁移等核心功能。

## 运行前提

设置 Postgres 环境变量（或 `DATABASE_URL`）：

```bash
export FOUNDATIONX_POSTGRESX_HOST=127.0.0.1
export FOUNDATIONX_POSTGRESX_PORT=5432
export FOUNDATIONX_POSTGRESX_DATABASE=market_binance
export FOUNDATIONX_POSTGRESX_USER=market_binance
export FOUNDATIONX_POSTGRESX_PASSWORD=<your_password>
export FOUNDATIONX_POSTGRESX_SSLMODE=disable
```

## 示例列表

| 示例 | 说明 | 运行命令 |
|------|------|----------|
| `basic.rs` | 最小连接示例 — connect → SELECT → close | `cargo run --example basic` |
| `crud_kline.rs` | 完整 CRUD — 建表 + BTC/ETH K 线导入 + 增删改查 + 聚合查询 | `cargo run --example crud_kline` |
| `migrate_schema.rs` | 迁移工作流 — apply / status / down 完整生命周期 | `cargo run --example migrate_schema` |
| `selfcheck_report.rs` | 自检报告 — LIB-SELFCHECK §6.1 自验证 JSON 输出 | `cargo run --example selfcheck_report` |

## basic.rs

```rust
// 最小连接示例
let pool = PostgresPool::connect(&PostgresConfig::from_env()?).await?;
let row = pool.query_one("SELECT 1 AS n", &[]).await?;
assert_eq!(row.get::<&str, i32>("n"), 1);
pool.close();
```

## crud_kline.rs

��整 CRUD 示例，演示：

1. **建表** — `CREATE TABLE IF NOT EXISTS` 幂等建表
2. **COPY 导入** — 从 CSV 文件批量导入 BTCUSDT / ETHUSDT 日 K 线数据
3. **CREATE** — 单条 INSERT（含冲突处理 `ON CONFLICT DO NOTHING`）
4. **READ** — `query_one` 单行查询、`COUNT` 聚合、列索引取值
5. **UPDATE** — 条件更新
6. **DELETE** — 条件删除
7. **聚合查询** — AVG / MAX / MIN / COUNT DISTINCT
8. **池统计** — `pool.stats()` 查看连接池状态

数据来源：`/home/workspace/data/binance_futures/merged/`

预期输出：

```
=== postgresx CRUD 示例 ===

✓ 连接: 127.0.0.1:5432/market_binance user=market_binance sslmode=disable pool=16

--- 1. 建表 ---
✓ 表 postgresx_kline 就绪

--- 2. 数据导入 ---
✓ BTCUSDT: 1086 条  |  ETHUSDT: 1086 条

--- 3. CRUD ---
  CREATE: 硬编码 INSERT 测试行
  READ:   最新 BTCUSDT close=66522.40
  READ:   BTCUSDT 共 1086 条
  UPDATE: 更新 0 行
  DELETE: 删除 1 行

--- 4. 聚合查询 ---
  BTC 平均收盘价: 74545.58
  ETH 最高: 4957.67  最低: 1384.00
  总计: 2172 条  品种: 2 个

--- 5. 池统计 ---
最大=16  当前=1  可用=1  关闭=false

✓ 完成
```

## migrate_schema.rs

迁移工作流示例，演示 `Migrator` 完整生命周期：

1. 创建 migration（含 `with_down` 回滚 SQL）
2. `Migrator::apply()` 执行所有待迁移 migration
3. `Migrator::status()` 查看已应用 / 待应用状态
4. SQL 验证表已创建
5. `Migrator::down()` 回滚最近已应用 migration
6. 重新 apply 验证幂等性

关键 API：

```rust
// 创建带回滚的 migration
let m = Migration::with_down(1, "create_table", up_sql, down_sql)?;

// 不带回滚的 migration
let m = Migration::new(2, "add_index", sql)?;

// 应用与回滚
migrator.apply().await?;
migrator.down().await?;
```

## selfcheck_report.rs

LIB-SELFCHECK §6.1 模块自验证示例：

- 运行 Full 级自检（11 项检查）
- 输出 `ValidationReport` JSON

## 表结构

### postgresx_kline

| 列 | 类型 | 说明 |
|----|------|------|
| id | BIGSERIAL PK | 自增主键 |
| symbol | TEXT NOT NULL | 交易对（BTCUSDT / ETHUSDT） |
| interval | TEXT NOT NULL | K 线周期（1d / 1h / 4h 等） |
| open_time | BIGINT NOT NULL | 开盘时间戳（ms） |
| open / high / low / close | DECIMAL | OHLC 价格 |
| volume | DECIMAL | 成交量 |
| close_time | BIGINT | 收盘时间戳（ms） |
| quote_volume | DECIMAL | 成交额 |
| count | BIGINT | 成交笔数 |
| taker_buy_volume / taker_buy_quote_volume | DECIMAL | 主动买入量/额 |
| UNIQUE | (symbol, interval, open_time) | 唯一约束 |
