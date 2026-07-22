# configx 公开 API

**角色**：内存 KV 配置

## 公开消费面

`ConfigStore::{new, get, set}` + `Default`。

## 最小用法

```rust
use configx::ConfigStore;

let s = ConfigStore::new();
s.set("k", "v").unwrap();
assert_eq!(s.get("k").as_deref(), Some("v"));
```

## 2026-07-22 dual-bar surface

Public helpers added for STATUS 100% structure + declared-surface hardening; see crate root docs and ssot alignment. **Not** Production Ready / L5.
