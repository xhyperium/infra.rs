# adapters/storage/postgres — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `postgresx` | PASS | Cargo.toml `0.3.6` |
| S-2 | 生产默认导出 | PASS | `PostgresPool / PgConnection / PgTransaction / PgTxRunner / PgRepository` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_POSTGRESX_*` 或 `DATABASE_URL` |
| S-4 | 离线测试 | PASS | `cargo test -p postgresx --all-targets` |
| S-5 | live ignore 入口 + 本轮 dev live | PASS | `tests/live_postgres.rs` 9/9 |
| S-6 | bench 有界 | PASS | `benches/query_hot_path.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称；`publish = false` |
| S-10 | DEFER 能力 | OPEN | COPY / migrations / read-replica / 远程 TLS live / 自定义 CA·mTLS |
| S-11 | deadline conformance 固定镜像 | PASS | `scripts/postgres-deadline-conformance.mjs`（2026-07-23） |
