# redisx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `redisx` |
| SSOT | `.agents/ssot/adapters/storage/redis/` |
| 实现 | `crates/adapters/storage/redis` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `RedisPool / RedisClient / RedisConfig` |
| contracts | contracts::KeyValueStore（+ 可选 pubsub） |
| resiliencx | `with_retry_budget` / `get_with_budget` / `set_with_budget` 经 `with_budget_async` |
| 环境变量 | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS}` |
| live | `tests/live_kv.rs · tests/live_kv_conformance.rs`（`#[ignore]`） |
| bench | `benches/kv_hot_path.rs` |
| DEFER | Cluster / Sentinel / Streams full / pubsub 默认关闭 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| REDISX-1 | workspace member | PASS | `cargo metadata -p redisx` |
| REDISX-2 | 生产默认导出 | PASS | `crates/adapters/storage/redis/src/lib.rs` |
| REDISX-3 | from_env | PASS | config · `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS}` |
| REDISX-4 | 离线测试 | PASS | `cargo test -p redisx --all-targets` |
| REDISX-5 | live 入口 | PASS | `tests/live_kv.rs · tests/live_kv_conformance.rs` |
| REDISX-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| REDISX-7 | crate docs | PASS | docs/usage · config · operations |
| REDISX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/redis/` |
| REDISX-9 | package stable | OPEN | 禁止宣称 |
| REDISX-10 | DEFER 能力 | OPEN | Cluster / Sentinel / Streams full / pubsub 默认关闭 |
| REDISX-11 | resiliencx 生产入口 | PASS | `client.rs` · `get`/`set` + `*_with_budget`；`resilience.rs` async |

## 验证

```bash
cargo test -p redisx --all-targets
cargo clippy -p redisx --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/redis/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/redis/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/redis/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p redisx -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/redis/`
