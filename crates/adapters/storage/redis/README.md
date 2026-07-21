# redisx

redis storage adapter。

Package：`redisx` · path：`crates/adapters/storage/redis`

合约：`infra-contracts::StorageAdapter`

## 状态

**scaffold** — 进程内 HashMap 模拟 KV；**非**真实 redis 客户端；不宣称 package stable。

## 最小用法

```rust
use infra_contracts::storage::StorageAdapter;
use redisx::RedisAdapter;

let mut a = RedisAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
