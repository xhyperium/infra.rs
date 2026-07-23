# Changelog

## [0.3.9] — 2026-07-23

### Added

- `PostgresPool::acquire_with(deadline)`：按调用方截止时间借连接
- 有界 `COPY`：`copy_in_bytes` / `copy_out_bytes`（连接级与池级；默认 16 MiB 上限 + operation_timeout）
- live：`live_acquire_with_and_copy_roundtrip`（11/11）

### Boundaries

- 流式无限 COPY、二进制 COPY 协议高级选项、migrations runner、mTLS、package stable **未承诺**

## [0.3.8] — 2026-07-23

### Added

- live：`live_raw_client_and_pool_fail_closed` — deprecated raw client 脱池 / begin 拒绝 / raw pool Closed
- 对齐 POSTGRESX-16 / matrix S-13 升格为 PASS（独立 live 证据）

### Boundaries

- mTLS、COPY/migrations/read-replica、package stable **未承诺**

## [0.3.7] — 2026-07-23

### Added

- TLS 可选 `tls_ca_file` / `FOUNDATIONX_POSTGRESX_TLS_CA_FILE`：webpki 公共根叠加 PEM 企业/自签 CA（仍强制校验）
- TLS 可选 `tls_server_name` / `FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME`：host 为 IP 时 `hostaddr` + SNI 分离
- 远程 Require live 证据：自签证书 + SNI 对真实主机 9/9 通过

### Boundaries

- mTLS 客户端证书、package stable **未承诺**

## [0.3.6] — 2026-07-23

### Added

- live 集成测扩展：`connect_from_env` / `query` / `query_opt` / `execute` / `begin` / `PgRepository` roundtrip / resiliencx 包装
- SQLSTATE 映射表文档锚点（FK `23503` → `Invalid` 本仓选型说明）
- SSOT/对齐文档同步至 `0.3.6`；10 轮 draft 对照审查落盘 `evidence/2026-07-23/`
- 本轮 dev live 9/9 + deadline conformance 固定镜像 + bench 有界证据

### Documentation

- `docs/ssot/postgresx-ssot-alignment.md` / gap-matrix / crate docs 版本与 LIVE 状态诚实更新
- 明确：远程 TLS 握手 live 与 package stable **仍 OPEN**；SSOT 路径为 `postgres` 非 `postgresx`

### Boundaries

- 自定义 CA/mTLS、迁移/COPY、HA 故障切换、远程 TLS live、package stable **未承诺**

## [0.3.5] — 2026-07-23

### Added

- 抽取 `channel_binding_policy`：显式锚定「无 channel binding / 无 SCRAM-PLUS」合同，离线单测可复核
- SSOT 诚实声明：`raw_exposed` 隔离证据仅 live-only；channel binding 未实现

### Boundaries

- 自定义 CA/mTLS、迁移/COPY、HA 故障切换、package stable **未承诺**

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
