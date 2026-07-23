# taosx

TDengine 时序适配器 — `contracts::TimeSeriesStore`（`Tick.ts` = 纳秒 epoch）。

| 路径 | 类型 | 生产？ |
|------|------|--------|
| **默认** | `TaosPool` / `TaosClient`（REST `:6041`） | **是** |
| `TransportMode::NativeWs` | `/rest/ws` 握手/关闭可达性探测；SQL 仍 REST | 部分 |
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
| `MAX_IN_FLIGHT` | `64`（硬上限 `1024`） |
| `BATCH_MAX_ROWS` / `BATCH_MAX_BYTES` | `500` / `1 MiB`（硬上限 `10000` / `8 MiB`） |
| `MAX_RESPONSE_BYTES` / `MAX_QUERY_ROWS` | `8 MiB` / `10000`（硬上限 `64 MiB` / `100000`） |
| `CLOSE_TIMEOUT_MS` | `5000`（硬上限 `30000`） |

远程 host 必须 TLS 且配置非空 user/password；只有严格 loopback 地址允许明文。

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

写入路径：`table` 作为 **STABLE**；按 `Tick.symbol` 无碰撞编码派生子表，tags 存原始 symbol。
时间：合同侧始终纳秒；按库 `precision` 换算后写入。
Decimal：bid/ask 以 `NCHAR(64+)` 文本落库；存量 `DOUBLE` schema 会 fail-closed。

## 测试

```bash
cargo test -p taosx
cargo test -p taosx --features scaffold
# 隔离、固定镜像 digest、动态 loopback 端口（不使用 prod）
node scripts/taos-live-conformance.mjs
```

## Bench

```bash
# 有界 bench（需 FOUNDATIONX_TAOSX_* 或本机 REST）
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo bench -p taosx --bench hot_path -- --quick
```

文档：[docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)  
对齐：[docs/ssot/taosx-ssot-alignment.md](../../../../docs/ssot/taosx-ssot-alignment.md)  
十轮审查：`docs/report/2026-07-23/taosx-ten-round-review.md`

Native SQL / FFI、WS 认证长会话、自动幂等重试、HA/Cluster 与 package stable 均为 **NO-GO**。  
SSOT 路径：`.agents/ssot/adapters/storage/taos/`（**不**另建 `taosx/`）。
