# Changelog — postgresx

## [0.3.1] — 2026-07-22

### Added

- **TLS Require/Prefer**：`MakeRustlsConnect`（rustls + webpki-roots）实现
  `tokio_postgres::tls::MakeTlsConnect`
- **生产 Repository**：`PgRepository` / `PgRecord`，表 `infra_pg_records`，
  `ensure_table()` + 参数化 `find`/`save`
- **resiliencx**：`with_retry_sync` / `with_retry_async` / `with_retry_async_no_wait`

### Changed

- 版本 `0.3.0` → `0.3.1`
- `SslMode::Require` 不再在建池前以 Invalid 拒绝；走真实 rustls 连接器

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
