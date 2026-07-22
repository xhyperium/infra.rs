# redisx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| SSOT | `.agents/ssot/adapters/storage/redis/` |
| 实现 | `crates/adapters/storage/redis` |
| 审计日期 | 2026-07-22 |
| version | `0.3.2` |
| 结论 | **生产默认客户端已落地**（Standalone / Cluster / Sentinel + TLS + resiliencx）；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `RedisPool / RedisClient / RedisConfig` |
| 模式 | `RedisMode::{Standalone,Cluster,Sentinel}` 均可 `connect` |
| TLS | `TcpTls { insecure: false }`（`tokio-rustls-comp`） |
| resiliencx | `with_retry_sync` / `with_retry_async` |
| contracts | `contracts::KeyValueStore`（+ 可选 pubsub） |
| 环境变量 | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER}` |
| live | `tests/live_kv.rs · tests/live_kv_conformance.rs`（`#[ignore]`） |
| bench | `benches/kv_hot_path.rs` |
| 原 OBJECTIVE DEFER | **PASS**（Cluster / Sentinel / TLS 强制 / resiliencx） |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| REDISX-1 | workspace member | PASS | `cargo metadata -p redisx` |
| REDISX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| REDISX-3 | from_env | PASS | config · `FOUNDATIONX_REDISX_*` |
| REDISX-4 | 离线测试 | PASS | `cargo test -p redisx --all-targets`（29 unit） |
| REDISX-5 | live 入口 | PASS | `tests/live_kv.rs · live_kv_conformance.rs` |
| REDISX-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| REDISX-7 | crate docs | PASS | docs/usage · config · operations |
| REDISX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/redis/` |
| REDISX-9 | package stable | OPEN | 禁止宣称 |
| REDISX-10 | Cluster 模式 | PASS | `pool` Cluster 后端 + `RedisMode::Cluster` |
| REDISX-11 | Sentinel 模式 | PASS | `async_master_for` → ConnectionManager |
| REDISX-12 | TLS 强制路径 | PASS | `to_connection_info` → TcpTls secure |
| REDISX-13 | resiliencx 接入 | PASS | `src/resilience.rs` |

## 验证

```bash
cargo test -p redisx --all-targets
cargo clippy -p redisx --all-targets -- -D warnings
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[gap-matrix.md](./gap-matrix.md)
