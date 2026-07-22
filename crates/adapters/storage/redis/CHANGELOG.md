# Changelog

## [0.3.1] — 2026-07-22

### Added

- **Cluster**：`RedisMode::Cluster` + `ClusterClient` / `cluster_async::ClusterConnection`
- **Sentinel**：`RedisMode::Sentinel` + `sentinel_master`；发现 master 后走 `ConnectionManager`
- **TLS**：`tls=true` → `ConnectionAddr::TcpTls { insecure: false }`（拒绝 insecure）
- **resiliencx**：`with_retry_sync` / `with_retry_async` / `with_retry_async_no_wait`
- 配置扩展：`nodes`、`sentinel_master`；环境变量
  `FOUNDATIONX_REDISX_MODE` / `NODES` / `SENTINEL_MASTER`
- 双后端池：`Standalone(ConnectionManager) | Cluster(ClusterConnection)`

### Changed

- 版本 `0.3.0` → `0.3.1`
- Cluster / Sentinel 校验改为接受合法配置（不再 P0 拒绝）

## [Unreleased]

### Changed

- **生产默认**：`RedisPool` + `RedisClient`（`ConnectionManager` + Semaphore 背压）
- `redis` 依赖改为非 optional；scaffold 迁至 feature `scaffold`
- `RedisLiveKv` 现为 `RedisClient` 类型别名

### Added

- `RedisConfig` / `RedisConfigBuilder` / `from_env` / `from_url`
- KV 扩展：`delete` / `exists` / `expire` / `ttl` / `mget` / `mset`
- feature `pubsub`：`RedisPubSub` / `RedisPubSubFacade`
- live tests：`tests/live_kv.rs`；bench：`benches/kv_hot_path.rs`
- 文档：`docs/usage.md` / `config.md` / `operations.md`

### Fixed

- `set` TTL `Some(0)` 固定为 `Invalid`（不再 `max(1)` 隐式改写）
