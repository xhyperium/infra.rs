# clickhousex

clickhouse storage adapter scaffold（进程内 KV）。

Package：`clickhousex` · path：`crates/adapters/storage/clickhouse`

## 状态

**scaffold** — 本地 `StorageAdapter` + HashMap；**非**真实 clickhouse 客户端；不宣称 package stable。

## 最小用法

```rust
use clickhousex::{ClickHouseAdapter, StorageAdapter};

let mut a = ClickHouseAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
