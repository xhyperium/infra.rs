# Changelog — postgresx

## [0.3.4] — 未发布（2026-07-23 候选）

- 在 `0.3.3` main 历史基础上收敛最新 Adapter safety / unchecked compatibility 合同与回归证据。
- 最终本地测试为 52 passed + 6 ignored；ignored live 测试不冒充默认 CI 证据。
- 候选曾冻结；治理修正后最终 SHA、reviewer/verifier、CI 与发布均 pending。

## [0.3.3] — 未发布（2026-07-23 候选）

### Added

- 独立 `acquire_timeout` 与 `operation_timeout`；deadpool wait/create/recycle 以及 SQL/事务终结均有界
- 服务端 `statement_timeout` 与调用侧 deadline；超时连接从池中丢弃
- 准确状态 `TxStatus::Failed`；服务端语句错误后仅允许回滚，取消后禁止继续执行 SQL
- 结构化 `TransactionRollbackFailure` 同时保留原错误与 rollback 错误 source
- 固定摘要 PostgreSQL 17 的池饱和、慢查询超时与恢复实验

### Changed

- 旧 raw client/pool 与 `database_url` 字段保留一个 deprecation 周期；raw client 强制脱池，
  raw pool 返回关闭的隔离池，URL/结构化字段漂移 fail-closed
- 新增非穷尽 `TxStatus`；deprecated 三态 `TxState` 不增加 variant
- Failed 事务禁止 COMMIT，避免把 PostgreSQL 的隐式 ROLLBACK 响应误报为提交成功
- `DATABASE_URL` 入口仅解析一次并从已校验字段重建连接配置，禁止 TLS 策略漂移

### Security

- 远程 `sslmode=disable/prefer` fail-closed；仅 `require` 可连接远程主机
- SQL / 事务错误保留 source；COMMIT 超时明确为结果未知
- 新增显式 `RetrySafety` 的 sync/async budget wrapper；任意 SQL 默认不被假定为只读或幂等
- 明确 `PostgresPool` 当前没有 budget 自动接线，旧无 safety helper 为 unchecked compatibility
- legacy `with_budget_async` 委托 resiliencx unchecked generic async core，统一标准 budget 错误与
  从 1 起的失败 attempt 观测；兼容入口仍不校验 `RetrySafety`

### Version

- root 已按 R-C2 将 `postgresx 0.3.2` bump 至 `0.3.3`；当前仍是未发布候选，未创建 tag 或发布制品

## [0.3.2] — 2026-07-22

### Added

- 生产入口接入 resiliencx budget，并提供显式 `*_with_budget` API
- 增加 `with_budget_async` / `with_budget_async_noop` helper

## [0.3.1] — 2026-07-22

### 新增

- **TLS Require/Prefer**：`MakeRustlsConnect`（rustls + webpki-roots）实现
  `tokio_postgres::tls::MakeTlsConnect`
- **生产 Repository**：`PgRepository` / `PgRecord`，表 `infra_pg_records`，
  `ensure_table()` + 参数化 `find`/`save`
- **resiliencx**：`with_retry_sync` / `with_retry_async` / `with_retry_async_no_wait`

### 变更

- 版本 `0.3.0` → `0.3.1`
- `SslMode::Require` 不再在建池前以 Invalid 拒绝；走真实 rustls 连接器
