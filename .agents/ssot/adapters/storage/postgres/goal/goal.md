# adapters/storage/postgres — Goal（infra.rs P0）

| 字段 | 值 |
|------|-----|
| package | `postgresx` |
| 标题 | PostgreSQL |
| 实现 | `crates/adapters/storage/postgres` |
| 战役 | draft SPEC_GOAL → 本仓生产默认路径 |
| 状态 | **P0 生产入口已落地**（#188–#191；`0.3.6` foundation 闭合）；package stable **未宣称** |

## Outcome

在 infra.rs workspace 中提供 **可配置、可关闭、可 live 验证** 的 PostgreSQL 生产默认客户端，满足 draft P0 DoD，且默认 `cargo test` 离线绿灯。

## Acceptance（本仓可验证）

1. workspace member `postgresx` 可 `cargo test -p postgresx --all-targets`
2. 生产默认面：`PostgresPool / PgConnection / PgTransaction / PgTxRunner`
3. 环境注入：`FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE} 或 DATABASE_URL`（密钥不入库）
4. live：`tests/live_postgres.rs` 默认 `#[ignore]`，真凭据可绿（含 Repository / query_opt / begin / resiliencx）
5. bench：`benches/query_hot_path.rs`（不得挂死 `--all-targets`）
6. scaffold 仅 `feature = "scaffold"`，禁止当作生产默认
7. deadline 固定镜像：`node scripts/postgres-deadline-conformance.mjs` 可绿

## Not in scope

COPY / migrations / read-replica / 远程 TLS live 强制 / package stable crates.io

## 证据指针

- 落地说明：[../plan/infra-rs-landing.md](../plan/infra-rs-landing.md)
- draft 快照：[../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/postgresx-ssot-alignment.md](../../../../../docs/ssot/postgresx-ssot-alignment.md)
