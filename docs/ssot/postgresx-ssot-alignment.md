# postgresx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `postgresx` |
| SSOT | `.agents/ssot/adapters/storage/postgres/` |
| 实现 | `crates/adapters/storage/postgres` |
| 审计日期 | 2026-07-22 |
| version | `0.3.1` |
| 结论 | **生产默认池/Tx/Repository/TLS/resiliencx 已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `PostgresPool / PgConnection / PgTransaction / PgTxRunner` |
| prod Repository | `PgRepository` + `PgRecord`（`infra_pg_records`） |
| SSL require | `MakeRustlsConnect`（rustls + webpki-roots）；Prefer/Require 走 TLS |
| resiliencx | `with_retry_sync` / `with_retry_async` |
| contracts | `TxRunner` + 生产 `Repository` |
| 环境变量 | `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE}` 或 `DATABASE_URL` |
| live | `tests/live_postgres.rs`（`#[ignore]`） |
| bench | `benches/query_hot_path.rs` |
| 原 OBJECTIVE DEFER | **PASS**（prod Repository / SSL require / resiliencx） |
| 仍 OPEN（非 OBJECTIVE） | COPY / migrations / read-replica |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| POSTGRESX-1 | workspace member | PASS | `cargo metadata -p postgresx` |
| POSTGRESX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| POSTGRESX-3 | from_env | PASS | config |
| POSTGRESX-4 | 离线测试 | PASS | 32 unit |
| POSTGRESX-5 | live 入口 | PASS | `tests/live_postgres.rs` |
| POSTGRESX-6 | bench 有界 | PASS | `benches/query_hot_path.rs` |
| POSTGRESX-7 | crate docs | PASS | docs/* |
| POSTGRESX-8 | SSOT 11 层 | PASS | `.agents/ssot/adapters/storage/postgres/` |
| POSTGRESX-9 | package stable | OPEN | 禁止宣称 |
| POSTGRESX-10 | 生产 Repository | PASS | `src/repository.rs` |
| POSTGRESX-11 | SSL require 路径 | PASS | `src/tls.rs` + pool Prefer/Require |
| POSTGRESX-12 | resiliencx 接入 | PASS | `src/resilience.rs` |

## 验证

```bash
cargo test -p postgresx --all-targets
cargo clippy -p postgresx --all-targets -- -D warnings
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
