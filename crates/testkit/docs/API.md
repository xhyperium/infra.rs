# testkit 公开 API

**角色**：T0 ManualClock

## 公开消费面

| 类型 | 方法 |
|------|------|
| `ManualClock` | `new` / `with_monotonic_elapsed` / `set_wall` / `advance_wall` / `rewind_wall` / `set_monotonic_elapsed` / `advance_monotonic` / `set_wall_fault` / `clear_wall_fault` / `wall_fault` / `snapshot` / `domain` |
| `ManualClockSnapshot` | `wall` / `monotonic_elapsed` / `wall_fault` |
| `ManualClockFault` / `ManualClockError` | 故障注入与错误分类 |

## 最小用法

```rust
use kernel::{Clock, Timestamp};
use testkit::ManualClock;
use std::time::Duration;

let c = ManualClock::new(Timestamp::from_unix_nanos(0));
c.advance_wall(Duration::from_secs(1)).unwrap();
assert_eq!(c.now().unwrap().as_unix_nanos(), 1_000_000_000);
```
