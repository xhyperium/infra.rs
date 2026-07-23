# clickhousex 示例

在生产环境中使用 `clickhousex` 的完整示例集合。

## 前置条件

- 本机 ClickHouse 运行在 `127.0.0.1:8123`
- 密码通过环境变量提供（例如 `CLICKHOUSE_PASSWORD`，勿写入仓库）
- 数据源：本地期货 K 线合并目录（示例默认路径见各 example 源码）

## 示例列表

| 文件 | 说明 | 命令 |
|------|------|------|
| `ddl_schema.rs` | 创建 `binance_futures` 数据库和 K 线表 | `cargo run -p clickhousex --example ddl_schema` |
| `import_klines.rs` | 批量导入 BTC/ETH K 线 CSV 数据 | `cargo run -p clickhousex --example import_klines -- --all` |
| `crud_operations.rs` | CREATE/READ/UPDATE/DELETE 完整示例 | `cargo run -p clickhousex --example crud_operations` |
| `analytics_queries.rs` | 开高低收成交量聚合 / 移动平均 / 波动率 / 资金费率分析 | `cargo run -p clickhousex --example analytics_queries` |

## 快速开始

```bash
# 1. 建表
cargo run -p clickhousex --example ddl_schema

# 2. 导入全部 BTC/ETH 数据（约 4M 行）
cargo run -p clickhousex --example import_klines -- --all

# 3. CRUD 操作验证
cargo run -p clickhousex --example crud_operations

# 4. 分析查询
cargo run -p clickhousex --example analytics_queries
```

## 数据结构

### K 线表 (`klines_1m` / `5m` / `15m` / `1h` / `4h` / `1d`)

| 列 | 类型 | 说明 |
|----|------|------|
| open_time | UInt64 | K 线开始时间（毫秒 UTC） |
| symbol | String | 交易对 |
| open/high/low/close | Float64 | 开高低收价格 |
| volume | Float64 | 成交量 |
| close_time | UInt64 | K 线结束时间（毫秒） |
| quote_volume | Float64 | 成交额（USDT） |
| count | UInt32 | 成交笔数 |
| taker_buy_volume | Float64 | 主动买入量 |
| taker_buy_quote_volume | Float64 | 主动买入额 |

### 资金费率表 (funding_rate)

| 列 | 类型 | 说明 |
|----|------|------|
| calc_time | UInt64 | 计算时间（毫秒） |
| symbol | String | 交易对 |
| funding_interval_hours | UInt8 | 费率间隔（小时） |
| last_funding_rate | Float64 | 资金费率 |

## 数据统计

| Symbol | 1m | 5m | 15m | 1h | 4h | 1d | funding |
|--------|-----|------|------|------|------|------|---------|
| `BTCUSDT` | 1,563,840 | 312,768 | 104,256 | 26,064 | 6,516 | 1,086 | 3,196 |
| `ETHUSDT` | 1,563,840 | 312,768 | 104,256 | 26,064 | 6,516 | 1,086 | 3,196 |
