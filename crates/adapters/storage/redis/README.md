# redisx

Redis storage adapter scaffold。

实现 `contracts::KeyValueStore` + `contracts::PubSub`（进程内内存；非真实 Redis）。

```rust
use contracts::KeyValueStore;
use redisx::RedisAdapter;

# async fn demo() {
let a = RedisAdapter::local();
a.set("k", b"v".to_vec(), None).await.unwrap();
# }
```
