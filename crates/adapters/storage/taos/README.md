# taosx

TDengine 时序适配器 — `contracts::TimeSeriesStore`（`Tick.ts` = 纳秒 epoch）。

| 路径 | 类型 | 生产？ |
|------|------|--------|
| **默认** | `TaosPool` / `TaosClient`（REST `:6041`） | **是** |
| `feature = "scaffold"` | `TaosAdapter` 内存 HashMap | 否 |

## 配置

环境变量前缀 `FOUNDATIONX_TAOSX_`：

| 变量 | 默认 |
|------|------|
| `HOST` | `127.0.0.1` |
| `PORT` | `6041` |
| `DATABASE` | `infra_draft` |
| `USER` | `root` |
| `PASSWORD` | 空（**勿**提交密钥） |
| `TLS` | 关闭（`1`/`true` 启用） |
| `TIMEOUT_MS` | `10000` |
| `PRECISION` | 连接后探测；可强制 `ms`/`us`/`ns` |

## 最小用法

```rust
use canonical::Tick;
use contracts::TimeSeriesStore;
use taosx::{TaosConfig, TaosPool};

# async fn demo(ticks: Vec<Tick>) -> kernel::XResult<()> {
let cfg = TaosConfig::from_env();
let pool = TaosPool::connect(cfg).await?;
pool.write_series("market_ticks", ticks).await?;
let rows = pool.query_series("market_ticks", 0, i64::MAX).await?;
pool.close().await?;
# let _ = rows;
# Ok(())
# }
```

写入路径：`table` 作为 **STABLE**；按 `Tick.symbol` 派生子表，tags 存原始 symbol。  
时间：合同侧始终纳秒；按库 `precision` 换算后写入。

## 测试

```bash
cargo test -p taosx
cargo test -p taosx --features scaffold
export FOUNDATIONX_TAOSX_PASSWORD='***'
cargo test -p taosx --test live_smoke -- --ignored --nocapture
```

## Bench

```bash
cargo run -p taosx --bench hot_path -- --quick
FOUNDATIONX_TAOSX_PASSWORD='***' cargo run -p taosx --bench hot_path -- --live --quick
```

文档：[docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)
