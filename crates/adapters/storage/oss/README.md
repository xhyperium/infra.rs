# ossx

oss storage adapter scaffold（进程内 KV）。

Package：`ossx` · path：`crates/adapters/storage/oss`

## 状态

**scaffold** — 本地 `StorageAdapter` + HashMap；**非**真实 oss 客户端；不宣称 package stable。

## 最小用法

```rust
use ossx::{OssAdapter, StorageAdapter};

let mut a = OssAdapter::local();
a.connect().expect("connect");
a.write("k", b"v").expect("write");
```
