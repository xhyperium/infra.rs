# schedulex 公开 API

**角色**：任务 ID 登记表

## 公开消费面

`Scheduler::{new, schedule, cancel, list}` + `Default`。

## 最小用法

```rust
use schedulex::Scheduler;

let mut s = Scheduler::new();
s.schedule("job-1");
assert!(s.cancel("job-1"));
```

## 2026-07-22 dual-bar surface

Public helpers added for STATUS 100% structure + declared-surface hardening; see crate root docs and ssot alignment. **Not** Production Ready / L5.
