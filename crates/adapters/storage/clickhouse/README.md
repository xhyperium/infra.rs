# clickhousex

ClickHouse 分析汇聚适配器 — `contracts::AnalyticsSink`。

| 路径 | 类型 | 生产？ |
|------|------|--------|
| **默认** | `ClickHousePool` / `ClickHouseClient`（HTTP `:8123`） | **是** |
| `feature = "scaffold"` | `ClickHouseAdapter` 内存 Vec | 否 |

## 配置

环境变量前缀 `FOUNDATIONX_CLICKHOUSEX_`：

| 变量 | 默认 |
|------|------|
| `HOST` | `127.0.0.1` |
| `HTTP_PORT` / `PORT` | `8123`；前者优先，两者冲突时 fail-closed |
| `TLS` | `false`；远程主机必须为 `true` |
| `TLS_CA_FILE` | 可选 PEM CA |
| `USER` | `default` |
| `PASSWORD` | 空（**勿**提交密钥） |
| `DATABASE` | `default` |
| `TIMEOUT_MS` | `10000` |
| `MAX_IDLE_PER_HOST` / `MAX_IN_FLIGHT` | `8` / `64` |
| `ACQUIRE_TIMEOUT_MS` | `5000` |

远程主机使用明文 HTTP 会在连接前拒绝。HTTP 错误只暴露状态与可选 ClickHouse
数字错误码，不回显服务端响应中的 SQL、payload 或认证细节。

## 最小用法

```rust
use bytes::Bytes;
use clickhousex::{ClickHouseConfig, ClickHousePool};
use contracts::AnalyticsSink;

# async fn demo() -> kernel::XResult<()> {
let mut cfg = ClickHouseConfig::from_env()?;
// cfg.password = std::env::var("FOUNDATIONX_CLICKHOUSEX_PASSWORD").unwrap_or_default();
let pool = ClickHousePool::connect(cfg).await?;
pool.ping().await?;
pool.sink("order_filled", Bytes::from_static(b"{\"id\":1}")).await?;
let rows = pool.query_rows("SELECT event FROM analytics_events LIMIT 1").await?;
pool.close().await?;
# let _ = rows;
# Ok(())
# }
```

## 测试

```bash
cargo test -p clickhousex
cargo test -p clickhousex --features scaffold
# live（需本机 ClickHouse + 密码）
export FOUNDATIONX_CLICKHOUSEX_PASSWORD='***'
cargo test -p clickhousex --test live_smoke -- --ignored --nocapture
node scripts/clickhouse-https-conformance.mjs
```

## Bench

```bash
cargo run -p clickhousex --bench hot_path -- --quick
FOUNDATIONX_CLICKHOUSEX_PASSWORD='***' cargo run -p clickhousex --bench hot_path -- --live --quick
```

文档：[docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)
