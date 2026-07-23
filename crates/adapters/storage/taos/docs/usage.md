# taosx 用法

## 最小示例

生产默认入口：`connect` → 写/查 → `close`。

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

配置通过 `FOUNDATIONX_TAOSX_*` 环境变量注入；**禁止**把密钥写入仓库。

详见同目录 `config.md` 与 `operations.md`。

## 公开 API（默认 feature）

| 导出 | 用途 |
|------|------|
| `TaosConfig` / `TransportMode` / `TsPrecision` | 配置与精度 |
| `HARD_MAX_*` | 资源硬上限常量 |
| `TaosPool` / `TaosClient` | REST 池（别名） |
| `TaosExecResult` / `TaosPoolStats` | 执行结果与池统计 |
| `build_insert_sql_chunks` | 纯函数批 SQL 切分 |
| `build_native_ws_url` / `connect_native_ws` / `validate_mode` | WS 探测（非 SQL 会话） |

`feature = "scaffold"` 另导出内存 `TaosAdapter`（**非**生产）。

## 测试

```bash
# 单元（离线）
cargo test -p taosx --all-targets

# 真实 dev live（密钥仅进子进程；需本机 TDengine REST 6041）
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p taosx --test live_smoke -- --ignored --nocapture

# 隔离 live（固定 digest、动态 loopback；不使用 prod）
node scripts/taos-live-conformance.mjs

# 有界 bench
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo bench -p taosx --bench hot_path -- --quick
```
