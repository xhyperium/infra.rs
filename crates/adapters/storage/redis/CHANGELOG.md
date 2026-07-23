# Changelog

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
