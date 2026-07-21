# kernel 公开 API

**角色**：L0 语义信任根

## 公开消费面

| 类别 | 类型/函数 | 说明 |
|------|-----------|------|
| 错误 | `XError` / `ErrorKind` / `XResult` / `BoxError` | 不透明错误 + 分类 |
| 时钟 | `Clock` / `SystemClock` / `Timestamp` / `MonotonicInstant` / `ClockDomain` / `ClockError` | 墙钟与单调钟分离 |
| 生命周期 | `ShutdownSignal` / `ShutdownGuard` / `ComponentState` / `LifecycleError` | 关停与状态机 |

## 最小用法

```rust
use kernel::{Clock, SystemClock, XError, ShutdownSignal};

let clock = SystemClock::new();
let _ts = clock.now()?;
let (guard, signal) = ShutdownSignal::new();
guard.trigger();
assert!(signal.is_triggered());
let _ = XError::invalid("bad input");
# Ok::<(), kernel::XError>(())
```
