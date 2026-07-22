# redisx

生产默认的异步 Redis 客户端（`contracts::KeyValueStore` + 扩展 API）。

| 模式 | 类型 | 生产？ |
|------|------|--------|
| **默认** | `RedisPool` / `RedisClient`（`redis` crate + 背压） | **是（P0 KV）** |
| `pubsub` | `RedisPubSub` / `RedisPubSubFacade` | 可选 |
| `scaffold` | `RedisAdapter` / `InMemoryRedis` / `MockRedisAdapter` | **否** |

`RedisLiveKv` 为 `RedisClient` 类型别名（兼容旧 live 入口）。

## 快速开始

```bash
export FOUNDATIONX_REDISX_ADDR=127.0.0.1:6379
export FOUNDATIONX_REDISX_USERNAME=default
export FOUNDATIONX_REDISX_PASSWORD=...  # 勿提交
export FOUNDATIONX_REDISX_DB=0

cargo test -p redisx
cargo test -p redisx -- --ignored
cargo test -p redisx --features scaffold
cargo bench -p redisx --bench kv_hot_path
```

```rust
use redisx::{RedisConfig, RedisPool};
use std::time::Duration;

# async fn demo() -> kernel::XResult<()> {
let pool = RedisPool::connect(RedisConfig::from_env()?).await?;
let client = pool.client();
client.set("k", b"v".to_vec(), None).await?;
assert_eq!(client.get("k").await?, Some(b"v".to_vec()));
pool.close(Duration::from_secs(2)).await?;
# Ok(())
# }
```

## 文档

- [docs/usage.md](./docs/usage.md)
- [docs/config.md](./docs/config.md)
- [docs/operations.md](./docs/operations.md)

## 语义要点

- `set(..., Some(0))` → `Invalid`
- `Debug` / `endpoint()` 脱敏密码
- `close` 后拒绝新命令；排空 in-flight
- 全局 Semaphore 限制 in-flight
- `RedisPool` 有 Standalone / Cluster / Sentinel 与安全 TLS 代码路径；真实 Cluster、Sentinel、TLS live 证据仍 **OPEN**
- Pub/Sub 仅支持 Standalone，并复用建池时的 ACL / TLS / deadline；Cluster / Sentinel 明确失败关闭
- `with_retry_budget` 只自动重试 GET / EXISTS / PTTL / MGET；写操作默认单次
- Redis 单命令的服务端原子性不消除“响应丢失后写入结果未知”；`MSET` 跨 Cluster slot 不承诺原子性

## 禁止误用

- **不要**在生产启用 `scaffold` 当 Redis
- **不要**把密码打进日志或提交 git
