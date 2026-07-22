# postgresx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `postgresx` |
| SSOT | `.agents/ssot/adapters/storage/postgres/` |
| 实现 | `crates/adapters/storage/postgres` |
| 审计日期 | 2026-07-22 |
| 结论 | **P0 生产默认客户端已落地** + live/bench；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `PostgresPool / PgConnection / PgTransaction / PgTxRunner` |
| contracts | contracts::TxRunner 边界 + SQL 参数化 API |
| resiliencx | `with_retry_budget` / `execute_with_budget` / `query*_with_budget` 经 `with_budget_async` |
| 环境变量 | `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE} 或 DATABASE_URL` |
| live | `tests/live_postgres.rs`（`#[ignore]`） |
| bench | `benches/query_hot_path.rs` |
| DEFER | COPY / migrations / read-replica / SSL require-only 默认 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| POSTGRESX-1 | workspace member | PASS | `cargo metadata -p postgresx` |
| POSTGRESX-2 | 生产默认导出 | PASS | `crates/adapters/storage/postgres/src/lib.rs` |
| POSTGRESX-3 | from_env | PASS | config · `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE} 或 DATABASE_URL` |
| POSTGRESX-4 | 离线测试 | PASS | `cargo test -p postgresx --all-targets` |
| POSTGRESX-5 | live 入口 | PASS | `tests/live_postgres.rs` |
| POSTGRESX-6 | bench 有界 | PASS | `benches/query_hot_path.rs` |
| POSTGRESX-7 | crate docs | PASS | docs/usage · config · operations |
| POSTGRESX-8 | SSOT 11 层 + landing/draft | PASS | `.agents/ssot/adapters/storage/postgres/` |
| POSTGRESX-9 | package stable | OPEN | 禁止宣称 |
| POSTGRESX-10 | DEFER 能力 | OPEN | COPY / migrations / read-replica / SSL require-only 默认 |
| POSTGRESX-11 | resiliencx 生产入口 | PASS | `pool.rs` · `execute`/`query*` + `*_with_budget`；`resilience.rs` async |

## 验证

```bash
cargo test -p postgresx --all-targets
cargo clippy -p postgresx --all-targets -- -D warnings
test -f .agents/ssot/adapters/storage/postgres/plan/infra-rs-landing.md
test -f .agents/ssot/adapters/storage/postgres/plan/infra-rs-draft-spec-goal.md
test -f .agents/ssot/adapters/storage/postgres/goal/goal.md
# optional live
# node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
# set -a; source /tmp/foundationx-live.env; set +a
# cargo test -p postgresx -- --ignored
```

## 相关

- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
- adapters 汇总：[adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- gap：[draft-gap-matrix.md](./draft-gap-matrix.md)
- SSOT 树：`.agents/ssot/adapters/storage/postgres/`
