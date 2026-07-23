# Changelog

## [0.3.12] — 2026-07-23

### Fixed

- **E2E 无数据 soft-skip**：CSV/`REDISX_DATA_ROOT` 缺失时不再 panic（CI `live` 不因无本地 data 红）

### Added

- live：`with_retry_budget` / `get_with_budget` / `set_with_budget`；`into_result_message_stream`；`RedisPubSubFacade`
- selfcheck：`run_with_context` / `run_json` / `ValidationReport::to_json_string`；cancel→全 Skipped 单测；§6.5 ID 对齐常量

### Boundaries

- Cluster / Sentinel / TLS live、package stable、跨模块 SelfValidator/HTTP/Prometheus 仍 OPEN/NO-GO
- 未宣称 Draft 全文 DoD 100%

## [0.3.11] — 2026-07-23

### Added

- **集成测试** `tests/integration_all_api.rs`：公开 API 全量 live（pool/KV/pipeline/lua/lock/selfcheck/pubsub）
- **E2E CRUD** `tests/e2e_klines_crud.rs`：`/home/workspace/data/binance_futures` K 线 CSV → set/get/mset/mget/pipeline/delete
- **基准** `benches/api_matrix.rs`：ping / set+get / mset+mget / pipeline_set

### Docs

- live 凭据：`scripts/live/export-foundationx-env.sh --env dev`（ZoneCNH `secrets/env/dev.md`，不回显密码）

### Boundaries

- 默认 `#[ignore]`，需真实 Redis + secrets 注入
- pipeline 大批量建议分块（E2E 默认 32）
- 未宣称 package stable

## [0.3.10] — 2026-07-23

### Added

- **自验证 `redisx::selfcheck`**（LIB-SELFCHECK-SPEC / `.cargo/draft/verifyctl.md` §6.5）
  - 模型：`CheckLevel` / `CheckStatus` / `CheckItem` / `ValidationReport` / `CheckDescriptor`
  - `RedisValidator` + `Validatable`：catalog 11 项；Basic / ReadWrite / Full 级别与短路
  - 资源命名 `_sc:{token}:*` + TTL 兜底；配置 skip / baseline / memory 阈值
  - 检查：`ping`、`set_get_del`、`ttl_semantics`、`data_structures`、`pipeline`、`multi_exec`、`lua_cas`、`pubsub`（feature）、`dist_lock`、`memory_pressure`、`cluster_slots`（非 Cluster → Skipped）

### Boundaries

- **不是** `tools/verifyctl`（Goal Contract 变更验证）
- **未**实现跨模块 `SelfValidator`、HTTP 探针、Prometheus 导出
- Cluster / Sentinel / TLS live 仍 OPEN；未宣称 package stable

## [0.3.9] — 2026-07-23

### Added

- **池累计指标**：`RedisMetricsSnapshot` + `RedisPool::metrics_snapshot`（commands_ok/err/timeout、acquire_timeout、rejected_closed；低基数、进程内）
- **Pub/Sub 结果流**（feature `pubsub`）：`RedisPubSub::into_result_message_stream`，断线时末尾**恰好一次** `Err(Unavailable)`

### Boundaries

- 指标不是 OpenTelemetry / Prometheus 导出器
- Pub/Sub 仍无可靠投递、无自动重连；Cluster / Sentinel / TLS live 仍 OPEN
- 未宣称 package stable

## [0.3.8] — 2026-07-23

### Added

- **调用级总 deadline**：`RedisClient::with_call_deadline`；排队（acquire）计入总预算
- **字节别名**：`get_bytes` / `set_bytes`
- **扩展**：`pipeline_set`、`eval_script`、带 fencing 的 `lock_acquire` / `lock_release` / `lock_extend`（`RedisLock`）
- workspace `redis` 启用 `script` feature（Lua）
- live：deadline/bytes、pipeline+锁 fencing（`#[ignore]`）

### Boundaries

- Cluster / Sentinel / TLS live 仍 OPEN
- 锁不提供「锁即正确」业务保证；关键写须使用 `fence`
- 未宣称 package stable

## [0.3.7] — 2026-07-23

### Added

- `error_map` 补齐 Cluster/Sentinel/Response/Extension 分支离线单测（行覆盖约 99%）
- `client` probe 路径：`duration_to_millis`、unsafe 副作用多试拒绝、budget/endpoint 形状测试
- 交付记录：PR #281 merge SHA 写入 SSOT evidence

### Boundaries

- 行覆盖率仍非 100%（Cluster/TLS live 等 OPEN 路径）；见 coverage-residual
- 未宣称 package stable

## [0.3.6] — 2026-07-23

### Changed

- 对齐文档与 crate 文档同步为 **0.3.6**；冻结 Draft×SSOT×实现 10 轮审查与 gap matrix
- `closed_pool_is_closed_flag`：env 已设时禁止 connect 失败 silent pass
- Standalone live（KV / conformance / pubsub）在真实 Redis 下复验通过（`#[ignore]` 仍默认）
- SSOT 路径裁决：canonical 保持 `.agents/ssot/adapters/storage/redis/`（不新增 `redisx/` 目录）

### Boundaries

- Cluster / Sentinel / TLS live **仍 OPEN**
- Draft 全文 DoD / package stable **未宣称**
- 覆盖率：见 `docs/ssot/redisx-ssot-alignment.md` 与 evidence 残余说明

## [0.3.5] — 2026-07-23

### Added

- `error_map`：ClusterDown/MOVED/ExecAbort/NoScript/Response LOADING·NOAUTH 与 `map_redis_result` 路径补齐离线单测，锁定可重试/不可用/冲突/缺失分类

### Boundaries

- 真实 Cluster / Sentinel / TLS live **仍 OPEN**
- 未宣称 package stable

## [0.3.4] — 未发布（2026-07-23 候选）

### Changed

- 当前 client budget 路由按参数细分：GET/EXISTS/PTTL/MGET 为 `ReadOnly`；无 TTL SET/MSET 为
  `Idempotent`；相对 TTL SET、DEL、PEXPIRE 为 `UnsafeSideEffect` 并在多试 I/O 前拒绝。
- PUBLISH 保持 `NeverAutomatic`，不自动重试。
- 粗粒度 `RedisOperation::Set` 因无法表达 TTL 参数而保持 `AmbiguousWrite`；client 不受该粗粒度
  枚举限制，按实际参数选择 safety。
- 最终本地测试为 51 passed + 8 ignored；Cluster/Sentinel/TLS live 仍 OPEN。

### Version

- Cargo 当前为 `0.3.4`；`0.3.3` 是 main 既有历史。本候选尚无 tag 或发布制品。

## [0.3.3] — 未发布（2026-07-23 候选）

### Changed

- PubSub 复用 pool 安全配置，非 Standalone 拓扑 fail-closed。
- seed URL 的 Debug、endpoint 与错误上下文全路径脱敏。
- RedisClient 的 budget 路径迁移到显式 `RetrySafety` wrapper：读取为 `ReadOnly`，无 TTL SET/MSET
  为 `Idempotent`，DEL/PEXPIRE/相对 TTL SET 为 `UnsafeSideEffect`。
- 旧无 safety 的 `with_budget*` / `with_retry*` 明确为 unchecked compatibility。
- `with_retry_budget` 原样保存零 attempts，使 GET/SET 等路由在 future/driver 前返回 `Invalid`。
- legacy `with_budget_async` 委托统一 unchecked core，标准化 budget exhaustion 与失败 attempt 观测，
  但仍不校验 `RetrySafety`。
- 版本 PATCH 0.3.2 → 0.3.3。

### Version

- root 已按 R-C2 将 `redisx 0.3.2` bump 至 `0.3.3`；当前仍是未发布候选，未创建 tag 或发布制品。

## [0.3.2] — 2026-07-22

### 变更

- **生产默认**：`RedisPool` + `RedisClient`（`ConnectionManager` + Semaphore 背压）
- `redis` 依赖改为非 optional；scaffold 迁至 feature `scaffold`
- `RedisLiveKv` 现为 `RedisClient` 类型别名
- 历史 budget 路径未接收 `RetrySafety`，现归类为 unchecked compatibility。

### 新增

- `RedisConfig` / `RedisConfigBuilder` / `from_env` / `from_url`
- KV 扩展：`delete` / `exists` / `expire` / `ttl` / `mget` / `mset`
- feature `pubsub`：`RedisPubSub` / `RedisPubSubFacade`
- live tests：`tests/live_kv.rs`；bench：`benches/kv_hot_path.rs`
- 文档：`docs/usage.md` / `config.md` / `operations.md`

### 修复

- `set` TTL `Some(0)` 固定为 `Invalid`（不再 `max(1)` 隐式改写）

## [0.3.1] — 2026-07-22

### 新增

- **Cluster**：`RedisMode::Cluster` + `ClusterClient` / `cluster_async::ClusterConnection`
- **Sentinel**：`RedisMode::Sentinel` + `sentinel_master`；发现 master 后走 `ConnectionManager`
- **TLS**：`tls=true` → `ConnectionAddr::TcpTls { insecure: false }`（拒绝 insecure）
- **resiliencx**：`with_retry_sync` / `with_retry_async` / `with_retry_async_no_wait`
- 配置扩展：`nodes`、`sentinel_master`；环境变量
  `FOUNDATIONX_REDISX_MODE` / `NODES` / `SENTINEL_MASTER`
- 双后端池：`Standalone(ConnectionManager) | Cluster(ClusterConnection)`

### 变更

- 版本 `0.3.0` → `0.3.1`
- Cluster / Sentinel 校验改为接受合法配置（不再 P0 拒绝）
