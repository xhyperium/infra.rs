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

## Pub/Sub（可选）

```bash
cargo test -p redisx --features pubsub
```

见 `RedisPubSub` / `RedisPubSubFacade`。
