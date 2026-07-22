# redisx 使用说明

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
直接建立会话时使用 `RedisPubSub::connect_config(config, channels)`。旧 `connect(endpoint, ...)`
因无法携带安全配置已 deprecated 并失败关闭；旧 `connect_with_config` 保留编译兼容但忽略外部
display endpoint，端点只从配置脱敏派生。

## 重试

`client.with_retry_budget(...)` 仅让 GET / EXISTS / PTTL / MGET 自动重试。SET / DEL /
PEXPIRE / MSET 默认单次执行，因为超时或断连时写入可能已经生效。只有
`set_with_budget(...)` 是明确的写重试 opt-in，调用方必须接受 TTL 重置与结果不确定风险。
