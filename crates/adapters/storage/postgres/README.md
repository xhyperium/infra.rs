# postgresx

postgres storage adapter scaffold（进程内 KV）。

Package：`postgresx` · path：`crates/adapters/storage/postgres`

## 状态

**scaffold** — 本地 `StorageAdapter` + HashMap；**非**真实 postgres 客户端；不宣称 package stable。

## 最小用法

```rust
use postgresx::{PostgresAdapter, StorageAdapter};

let mut a = PostgresAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
