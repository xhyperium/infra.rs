# Changelog — postgresx

## Unreleased

### Added

- 生产默认面：`PostgresConfig` / `PostgresConfigBuilder` / `SslMode`
  - 环境：`FOUNDATIONX_POSTGRESX_*`；`DATABASE_URL` 优先覆盖
- `PostgresPool`：`connect` / `acquire` / `execute` / `query` / `query_one` /
  `query_opt` / `with_transaction` / `begin` / `health` / `stats` / `close`
- `PgConnection` / `PgTransaction`（`TxState::{Active,Committed,RolledBack}`）
- SQLSTATE → `kernel::ErrorKind` 映射与单元测试
- `PgTxRunner`：`contracts::TxRunner` 真实事务边界（诚实限制：无 SQL 句柄）
- feature `scaffold`：原 `PostgresAdapter` / `ObservingPostgresAdapter`
- live `#[ignore]` 测试与 `benches/query_hot_path.rs`（harness = false）
- 文档：`docs/usage.md` / `config.md` / `operations.md`
