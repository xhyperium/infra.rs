# redisx

Redis storage adapter：

- scaffold：`RedisAdapter`（忽略 TTL）
- mock 验证入口：`MockRedisAdapter`（TTL 模拟 + 单调 PubSub id；**非**真实 Redis）

```rust
use contracts::KeyValueStore;
use redisx::MockRedisAdapter;
use std::time::Duration;

# async fn demo() {
let a = MockRedisAdapter::local();
a.set("k", b"v".to_vec(), Some(Duration::from_secs(60))).await.unwrap();
# }
```
