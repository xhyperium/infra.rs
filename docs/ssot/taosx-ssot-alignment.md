# taosx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| SSOT | `.agents/ssot/adapters/storage/taos/` |
| 实现 | `crates/adapters/storage/taos` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `TaosPool / TaosClient REST` |
| contracts | contracts::TimeSeriesStore（ts 纳秒 epoch） |
| 环境变量 | `FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION}` |
| live | `tests/live_smoke.rs`（`#[ignore]`） |
| bench | `benches/hot_path.rs（3s 有界）` |
| DEFER | native WS / 全超表治理 / 集群 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| TAOSX-1 | workspace member | PASS | `cargo metadata -p taosx` |
| TAOSX-2 | 生产默认导出 | PASS | `crates/adapters/storage/taos/src/lib.rs` |
| TAOSX-3 | from_env | PASS | config · `FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION}` |
| TAOSX-4 | 离线测试 | PASS | `cargo test -p taosx --all-targets` |
| TAOSX-5 | live 入口 | PASS | `tests/live_smoke.rs` |
| TAOSX-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| TAOSX-7 | crate docs | PASS | docs/usage · config · operations |
| TAOSX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/taos/` |
| TAOSX-9 | package stable | OPEN | 禁止宣称 |
| TAOSX-10 | DEFER 能力 | OPEN | native WS / 全超表治理 / 集群 |

## 验证

```bash
cargo test -p taosx --all-targets
cargo clippy -p taosx --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/taos/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/taos/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/taos/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p taosx -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/taos/`
