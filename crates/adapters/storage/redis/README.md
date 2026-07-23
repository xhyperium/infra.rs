# redisx

生产默认的异步 Redis 客户端（`contracts::KeyValueStore` + 扩展 API）。

- 调用级 deadline：`client.with_call_deadline(Duration::from_secs(2))`
- 扩展：`pipeline_set` / `eval_script` / `lock_acquire`（fencing）
- 池指标：`pool.metrics_snapshot()` → `RedisMetricsSnapshot`
- Pub/Sub 结果流：`session.into_result_message_stream()`（断线一次 `Err`）
- 自验证：`RedisValidator::new(client).run(CheckLevel::ReadWrite)`（§6.5）

当前 workspace 版本为 `0.3.10` 未发布候选；`publish = false`，不代表已经发布。

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
- 配置 `with_retry_budget` 后，GET / EXISTS / PTTL / MGET 与无 TTL SET / MSET 进入安全预算重试；
  相对 TTL SET、DEL、PEXPIRE 的多次尝试在 I/O 前拒绝；PUBLISH 不自动重试
- Redis 单命令的服务端原子性不消除“响应丢失后写入结果未知”；`MSET` 跨 Cluster slot 不承诺原子性

## 重试安全

配置 `RedisClient::with_retry_budget` 后，真实 client 只使用 safety-aware wrapper：

- `max_attempts` 原样保存；0 会在 future 构造和 driver I/O 前返回 `Invalid`；
- `GET` / `EXISTS` / `PTTL` / `MGET`：`ReadOnly`；
- 无 TTL `SET` / `MSET`：`Idempotent`；
- `DEL` / `PEXPIRE` / 相对 TTL `SET`：`UnsafeSideEffect`，多次尝试在首次 I/O future 前拒绝。
- `PUBLISH`：`NeverAutomatic`，不进入自动预算重试。

公开 `RedisOperation::Set` 同时覆盖无 TTL SET 与相对 TTL PSETEX，枚举本身无法携带 TTL 参数，
因此保守保持 `AmbiguousWrite`；真实 client 路由按 `ttl` 参数细分，不以该粗粒度值覆盖参数语义。

最终本地测试以 `cargo test -p redisx --features pubsub --lib` 为准；ignored live 项需要外部 Redis。

模块级旧 `with_budget*` / `with_retry*` 为 unchecked compatibility，不得作为新生产默认。

旧 `with_budget_async` 委托统一 unchecked core，预算耗尽返回标准 budget 错误并记录刚失败的
attempt（从 1 起），但仍不会校验 `RetrySafety`。

## 禁止误用

- **不要**在生产启用 `scaffold` 当 Redis
- **不要**把密码打进日志或提交 git
