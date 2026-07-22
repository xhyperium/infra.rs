# postgresx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `postgresx` |
| SSOT | `.agents/ssot/adapters/storage/postgres/` |
| 实现 | `crates/adapters/storage/postgres` |
| 审计日期 | 2026-07-23 |
| version | `0.3.3` |
| 结论 | **生产默认池/Tx/Repository/TLS 实现路径已落地；deadline/连接隔离有固定镜像证据**；**未**宣称真实 PostgreSQL TLS 已验证或 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `PostgresPool / PgConnection / PgTransaction / PgTxRunner` |
| prod Repository | `PgRepository` + `PgRecord`（`infra_pg_records`） |
| SSL require | `MakeRustlsConnect`（rustls + webpki-roots）；远程仅 Require，Disable/Prefer fail-closed |
| deadline | acquire 与 SQL/事务终结独立有界；服务端 `statement_timeout` + 调用侧 deadline |
| 超时连接卫生 | RAII guard 覆盖内/外层取消；未知连接脱池；COMMIT timeout 结果未知 |
| 事务状态 | 非穷尽 `TxStatus` 准确表达 rollback-only `Failed`；旧 `TxState` 仅为兼容视图 |
| 双错误 | `TransactionRollbackFailure` 结构化保留原错误与 rollback 错误的两个 source 分支 |
| 旧逃逸面 | deprecated raw client 使用后强制脱池；raw pool 返回关闭的独立隔离池 |
| resiliencx | `with_retry_sync` / `with_retry_async` |
| contracts | `TxRunner` + 生产 `Repository` |
| 环境变量 | `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE}` 或 `DATABASE_URL` |
| live | `tests/live_postgres.rs`；固定摘要 deadline 实验 `tests/deadline_conformance.rs`（均 `#[ignore]`） |
| bench | `benches/query_hot_path.rs` |
| 原 OBJECTIVE DEFER | **PASS**（prod Repository / SSL require / resiliencx） |
| 仍 OPEN（非 OBJECTIVE） | COPY / migrations / read-replica |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| POSTGRESX-1 | workspace member | PASS | `cargo metadata -p postgresx` |
| POSTGRESX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| POSTGRESX-3 | from_env | PASS | config |
| POSTGRESX-4 | 离线测试 | PASS | 33 unit + ignored conformance 编译 |
| POSTGRESX-5 | live 入口 | PASS | `tests/live_postgres.rs` |
| POSTGRESX-6 | bench 有界 | PASS | `benches/query_hot_path.rs` |
| POSTGRESX-7 | crate docs | PASS | docs/* |
| POSTGRESX-8 | SSOT 11 层 | PASS | `.agents/ssot/adapters/storage/postgres/` |
| POSTGRESX-9 | package stable | OPEN | 禁止宣称 |
| POSTGRESX-10 | 生产 Repository | PASS | `src/repository.rs` |
| POSTGRESX-11 | SSL require 路径 | PASS | `src/tls.rs` + pool Prefer/Require |
| POSTGRESX-12 | resiliencx 接入 | PASS | `src/resilience.rs` |
| POSTGRESX-13 | pool/SQL deadline 与连接隔离 | PASS | 固定摘要 Postgres 17 实验 |
| POSTGRESX-14 | 取消/abort 与事务 Failed 状态 | CODE PASS / LIVE OPEN | `src/{conn,tx}.rs` + ignored `tests/deadline_conformance.rs`；本轮未运行真实 PostgreSQL |
| POSTGRESX-15 | 业务+rollback 双错误保真 | PASS | `src/error.rs` + `src/pool.rs` 单元测试 |
| POSTGRESX-16 | deprecated raw 访问 fail-closed | CODE PASS / LIVE OPEN | `src/{conn,pool}.rs` + ignored conformance；本轮未运行真实 PostgreSQL |
| POSTGRESX-17 | Release 候选身份 | OPEN | Code 阶段不记录未提交 worktree/旧候选 SHA；由 Release 对实际候选提交重算 |

## 验证

```bash
RUSTC_WRAPPER= cargo test -p postgresx --all-targets
RUSTC_WRAPPER= cargo clippy -p postgresx --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/check-workspace-deps.mjs
cmp .agents/ssot/adapters/storage/postgres/spec/spec.md \
  .agents/ssot/adapters/storage/postgres/spec/xhyper-postgresx-complete-spec.md

# 需要本地 PostgreSQL 容器；Code 阶段未运行，不得据此关闭 issue
node scripts/postgres-deadline-conformance.mjs
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
