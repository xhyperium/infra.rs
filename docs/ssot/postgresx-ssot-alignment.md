# postgresx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `postgresx` |
| SSOT | `.agents/ssot/adapters/storage/postgres/`（**非** `postgresx` 目录） |
| 实现 | `crates/adapters/storage/postgres` |
| 审计日期 | 2026-07-23 |
| version | `0.3.12` |
| 结论 | **生产默认池/Tx/Repository/TLS 实现路径已落地**；dev live + 远程 Require TLS + deadline + **raw fail-closed live** 已跑通；mTLS 客户端身份已落地；**§6.1 selfcheck 自验证** Basic+RW+Full live 通过；**未**宣称 package stable / 服务端强制 mTLS 全网 live |

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
| 环境变量 | `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE,TLS_CA_FILE,TLS_SERVER_NAME,TLS_CLIENT_CERT,TLS_CLIENT_KEY}` 或 `DATABASE_URL` |
| live | `tests/live_postgres.rs`（12 cases）+ `tests/live_selfcheck.rs`（Basic/RW/Full） |
| selfcheck | `src/selfcheck/*`（ID=`postgres.*`；crate=`postgresx`） |
| deadline 实验 | `tests/deadline_conformance.rs` + `scripts/postgres-deadline-conformance.mjs` |
| bench | `benches/query_hot_path.rs` |
| 原 OBJECTIVE DEFER | **PASS**（prod Repository / SSL require 路径 / resiliencx） |
| 仍 OPEN / DEFER | 无限流式 COPY / read-replica / package stable；mTLS 服务端强制 live（部署依赖）；down migration |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| POSTGRESX-1 | workspace member | PASS | `cargo metadata -p postgresx` |
| POSTGRESX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| POSTGRESX-3 | from_env | PASS | config |
| POSTGRESX-4 | 离线测试 | PASS | unit tests 全绿 |
| POSTGRESX-5 | live 入口 + 本轮 dev 结果 | PASS | `tests/live_postgres.rs` 12/12 with secrets inject |
| POSTGRESX-6 | bench 有界 | PASS | `benches/query_hot_path.rs`（200 iters 完成） |
| POSTGRESX-7 | crate docs | PASS | docs/* |
| POSTGRESX-8 | SSOT 11 层 | PASS | `.agents/ssot/adapters/storage/postgres/` |
| POSTGRESX-9 | package stable | OPEN | 禁止宣称；`publish = false` |
| POSTGRESX-10 | 生产 Repository | PASS | `src/repository.rs` + live roundtrip |
| POSTGRESX-11 | SSL require 路径 + 远程 live | PASS | rustls + 可选 CA/SNI；远程自签 Require live 已通过（`TLS_CA_FILE` + `TLS_SERVER_NAME`） |
| POSTGRESX-12 | resiliencx 接入 | PASS | `src/resilience.rs` + live wrapper |
| POSTGRESX-13 | pool/SQL deadline 与连接隔离 | PASS | 固定摘要 Postgres 17 实验通过 |
| POSTGRESX-14 | 取消/abort 与事务 Failed 状态 | PASS | `src/{conn,tx}.rs` + deadline_conformance live |
| POSTGRESX-15 | 业务+rollback 双错误保真 | PASS | `src/error.rs` + `src/pool.rs` 单元测试 |
| POSTGRESX-16 | deprecated raw 访问 fail-closed | PASS | `deadline_conformance` + `live_raw_client_and_pool_fail_closed` |
| POSTGRESX-17 | Release 候选身份 | OPEN | package stable 未宣称；本轮交付为内部 foundation 闭合 |
| POSTGRESX-18 | acquire_with | PASS | `pool.acquire_with` + live |
| POSTGRESX-19 | 有界 COPY IN/OUT | PASS | `copy_in_bytes`/`copy_out_bytes` + live TEMP 往返 |
| POSTGRESX-20 | mTLS 客户端证书配置 | PASS | cert+key 成对；离线 openssl 构建 + fail-closed；服务端强制 mTLS live 依赖部署 |
| POSTGRESX-21 | Migrator verify/apply | PASS | advisory lock + checksum；live verify/apply/漂移 |
| POSTGRESX-22 | LIB-SELFCHECK §6.1 自验证 | PASS | `selfcheck` catalog 11 项；live Basic+RW+Full；replication_lag 默认可 Skip |

## 本轮验证（2026-07-23）

```bash
# 凭据注入（不入库）
node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/foundationx-live.env
set -a; source /tmp/foundationx-live.env; set +a

RUSTC_WRAPPER= cargo test -p postgresx --lib
RUSTC_WRAPPER= cargo test -p postgresx --test live_selfcheck -- --ignored --test-threads=1
RUSTC_WRAPPER= cargo clippy -p postgresx --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/check-workspace-deps.mjs
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
- 10 轮审查：`.agents/ssot/adapters/storage/postgres/evidence/2026-07-23/postgresx-10x-review.md`
