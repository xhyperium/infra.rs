# redisx 使用说明

当前文档对应 `redisx 0.3.11` 未发布候选。

## 生产入口

```rust
use std::time::Duration;
use contracts::KeyValueStore;
use redisx::{RedisConfig, RedisPool};

# async fn demo() -> kernel::XResult<()> {
let cfg = RedisConfig::from_env()?; // 或 builder / from_url
let pool = RedisPool::connect(cfg).await?;
let client = pool.client();

client.set("k", b"v".to_vec(), Some(Duration::from_secs(60))).await?;
assert_eq!(client.get("k").await?, Some(b"v".to_vec()));

// 池累计指标（进程内；非 OTel 导出器）
let snap = pool.metrics_snapshot();
assert!(snap.commands_ok >= 2);

// 合同层
let kv: &dyn KeyValueStore = &client;
let _ = kv.get("k").await?;

pool.close(Duration::from_secs(3)).await?;
# Ok(())
# }
```

## Builder

```rust
use std::time::Duration;
use redisx::RedisConfig;

# fn demo() -> kernel::XResult<()> {
let cfg = RedisConfig::builder()
    .addr("127.0.0.1:6379")
    .username("default")
    .password(std::env::var("FOUNDATIONX_REDISX_PASSWORD").unwrap_or_default())
    .db(0)
    .max_in_flight(128)
    .command_timeout(Duration::from_secs(2))
    .build()?;
# let _ = cfg;
# Ok(())
# }
```

## 兼容别名

`RedisLiveKv` = `RedisClient`。旧测试可继续 `RedisLiveKv::connect(url)`。

## 重试预算安全分类

`RedisClient::with_retry_budget` 的生产路径显式分类：GET/EXISTS/PTTL/MGET 为只读，无 TTL SET 与
MSET 为幂等，DEL/PEXPIRE/相对 TTL SET 保守视为不安全副作用。不安全操作配置多次尝试时会在
首次 I/O future 前返回 `Invalid`，不会静默执行一次后再失败。

`max_attempts` 不做 `max(1)` 归一化：传 0 时 GET 与 SET/DEL 等路由均在 operation future 构造和
driver I/O 前返回 `Invalid`。

模块级 `with_budget_safe` / `with_budget_async_safe` 可供自定义组合显式传入 `RetrySafety`；旧未带
`safe` 的 budget/retry wrapper 仅为 unchecked compatibility。
旧 `with_budget_async` 虽共享统一 budget 错误与失败 attempt 观测 core，仍不执行 safety 校验。

## Scaffold（非生产）

```bash
cargo test -p redisx --features scaffold
```

`RedisAdapter` / `InMemoryRedis` / `MockRedisAdapter` 仅在 `scaffold` feature 下可用。

## Pub/Sub（可选，仅 Standalone）

```bash
cargo test -p redisx --features pubsub
```

见 `RedisPubSub` / `RedisPubSubFacade`。

从 `RedisPool::subscribe` 建立会话时会复用池的 ACL / TLS / deadline 配置，不读取新的 env。
Cluster / Sentinel 会明确返回错误；当前没有拓扑 Pub/Sub 或断线重订阅保证。

- `into_message_stream()`：`BusMessage` 流；断线**静默结束**
- `into_result_message_stream()`：`Result<BusMessage, XError>`；断线末尾**一次** `Err(Unavailable)`

直接建立会话时使用 `RedisPubSub::connect_config(config, channels)`。旧 `connect(endpoint, ...)`
因无法携带安全配置已 deprecated 并失败关闭；旧 `connect_with_config` 保留编译兼容但忽略外部
display endpoint，端点只从配置脱敏派生。

## 重试

配置 `client.with_retry_budget(...)` 后：

- GET / EXISTS / PTTL / MGET 按 `ReadOnly` 对 Transient 失败使用预算重试；
- 无 TTL SET 与 MSET 按固定输入 `Idempotent` 使用预算重试；
- 相对 TTL SET、DEL、PEXPIRE 按 `UnsafeSideEffect`，`max_attempts > 1` 时在 operation future / driver
  I/O 前返回 `Invalid`，单次尝试仍允许；
- PUBLISH 不自动重试。

`RedisOperation::Set` 是不携带 TTL 参数的粗粒度查询面，保守保持 `AmbiguousWrite`；client 的 SET
路径按实际 `ttl` 参数细分。超时或断连仍可能发生在服务端已执行命令之后，安全分类不等于结果确认。
