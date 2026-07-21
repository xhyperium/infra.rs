# kafkax

kafka storage adapter。

Package：`kafkax` · path：`crates/adapters/storage/kafka`

合约：`infra-contracts::StorageAdapter`

## 状态

**scaffold** — 进程内 HashMap 模拟 KV；**非**真实 kafka 客户端；不宣称 package stable。

## 最小用法

```rust
use crate::StorageAdapter;
use kafkax::KafkaAdapter;

let mut a = KafkaAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
