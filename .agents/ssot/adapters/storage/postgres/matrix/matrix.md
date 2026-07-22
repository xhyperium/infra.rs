# adapters/storage/postgres — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `postgresx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `PostgresPool / PgConnection / PgTransaction / PgTxRunner` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE} 或 DATABASE_URL` |
| S-4 | 离线测试 | PASS | cargo test -p postgresx |
| S-5 | live ignore 入口 | PASS | `tests/live_postgres.rs` |
| S-6 | bench 有界 | PASS | `benches/query_hot_path.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | DEFER 能力 | OPEN | COPY / migrations / read-replica / SSL require-only 默认 |
