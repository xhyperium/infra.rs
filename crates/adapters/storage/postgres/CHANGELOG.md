# Changelog — postgresx

## [0.3.3] — 2026-07-22

### Added

- 独立 `acquire_timeout` 与 `operation_timeout`；deadpool wait/create/recycle 以及 SQL/事务终结均有界
- 服务端 `statement_timeout` 与调用侧 deadline；超时连接从池中丢弃
- 事务失败态 `TxState::Failed`；服务端语句错误后仅允许回滚，取消后禁止继续执行 SQL
- 固定摘要 PostgreSQL 17 的池饱和、慢查询超时与恢复实验

### Changed

- 移除可绕过 deadline 与取消守卫的底层 deadpool 池公开访问器
- Failed 事务禁止 COMMIT，避免把 PostgreSQL 的隐式 ROLLBACK 响应误报为提交成功
- `DATABASE_URL` 入口仅解析一次并从已校验字段重建连接配置，禁止 TLS 策略漂移

### Security

- 远程 `sslmode=disable/prefer` fail-closed；仅 `require` 可连接远程主机
- SQL / 事务错误保留 source；COMMIT 超时明确为结果未知

## [0.3.2] — 2026-07-22

### Added

- 生产入口接入 resiliencx budget，并提供显式 `*_with_budget` API
- 增加 `with_budget_async` / `with_budget_async_noop` helper

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
- `PgConnection` / `PgTransaction`（`TxState::{Active,Committed,RolledBack,Failed}`）
- SQLSTATE → `kernel::ErrorKind` 映射与单元测试
- `PgTxRunner`：`contracts::TxRunner` 真实事务边界（诚实限制：无 SQL 句柄）
- feature `scaffold`：原 `PostgresAdapter` / `ObservingPostgresAdapter`
- live `#[ignore]` 测试与 `benches/query_hot_path.rs`（harness = false）
- 文档：`docs/usage.md` / `config.md` / `operations.md`
