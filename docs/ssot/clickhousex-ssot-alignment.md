# clickhousex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 实现 | `crates/adapters/storage/clickhouse` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `ClickHousePool / ClickHouseClient HTTP` |
| contracts | contracts::AnalyticsSink |
| 环境变量 | `FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT/PORT,USER,PASSWORD,DATABASE}` |
| live | `tests/live_smoke.rs`（`#[ignore]`） |
| bench | `benches/hot_path.rs（3s 有界）` |
| DEFER | native 9000 protocol / cluster / ReplicatedMergeTree 运维面 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| CLICKHOUSEX-1 | workspace member | PASS | `cargo metadata -p clickhousex` |
| CLICKHOUSEX-2 | 生产默认导出 | PASS | `crates/adapters/storage/clickhouse/src/lib.rs` |
| CLICKHOUSEX-3 | from_env | PASS | config · `FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT/PORT,USER,PASSWORD,DATABASE}` |
| CLICKHOUSEX-4 | 离线测试 | PASS | `cargo test -p clickhousex --all-targets` |
| CLICKHOUSEX-5 | live 入口 | PASS | `tests/live_smoke.rs` |
| CLICKHOUSEX-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| CLICKHOUSEX-7 | crate docs | PASS | docs/usage · config · operations |
| CLICKHOUSEX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/clickhouse/` |
| CLICKHOUSEX-9 | package stable | OPEN | 禁止宣称 |
| CLICKHOUSEX-10 | DEFER 能力 | OPEN | native 9000 protocol / cluster / ReplicatedMergeTree 运维面 |

## 验证

```bash
cargo test -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/clickhouse/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/clickhouse/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/clickhouse/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p clickhousex -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/clickhouse/`
