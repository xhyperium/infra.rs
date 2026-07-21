# taosx

taos storage adapter scaffold（进程内 KV）。

Package：`taosx` · path：`crates/adapters/storage/taos`

## 状态

**scaffold** — 本地 `StorageAdapter` + HashMap；**非**真实 taos 客户端；不宣称 package stable。

## 最小用法

```rust
use taosx::{TaosAdapter, StorageAdapter};

let mut a = TaosAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
