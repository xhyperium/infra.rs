# natsx

nats storage adapter scaffold（进程内 KV）。

Package：`natsx` · path：`crates/adapters/storage/nats`

## 状态

**scaffold** — 本地 `StorageAdapter` + HashMap；**非**真实 nats 客户端；不宣称 package stable。

## 最小用法

```rust
use natsx::{NatsAdapter, StorageAdapter};

let mut a = NatsAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
