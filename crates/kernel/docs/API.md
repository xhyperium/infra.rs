# kernel 公开 API

**Package**：`kernel` · **角色**：L0 语义信任根  
**生产层级**：L1 Internal Ready + L4 Platform Ready

## 公开消费面（crate 根 re-export）

### error

| 符号 | 说明 |
|------|------|
| `XError` | 不透明错误；构造器：`invalid` / `missing` / `conflict` / `transient` / `transient_after` / `unavailable` / `cancelled` / `deadline_exceeded` / `invariant` / `internal` |
| `XError::kind` / `context` / `retry_after` / `is_retryable` / `is_bug` / `with_source` | 查询与 source 链 |
| `ErrorKind` | 9 种分类：`Invalid` · `Missing` · `Conflict` · `Transient` · `Unavailable` · `Cancelled` · `DeadlineExceeded` · `Invariant` · `Internal` |
| `XResult<T>` | `Result<T, XError>` |
| `BoxError` | `Box<dyn Error + Send + Sync + 'static>` |
| `From<ClockError> for XError` | 全部映射为 `ErrorKind::Unavailable`，保留 source |

### clock

| 符号 | 说明 |
|------|------|
| `Clock` | trait：`now()` / `monotonic()` |
| `SystemClock` | 系统时钟实现：`new` / `Default` / `Clone` |
| `Timestamp` | Unix 纳秒：`from_unix_nanos` / `as_unix_nanos` / `checked_add` / `checked_sub` / `checked_duration_since` |
| `MonotonicInstant` | 单调点：`from_clock_elapsed` / `from_clock_elapsed_in` / `domain` / `checked_duration_since` |
| `ClockDomain` | 单调域：`PROCESS` / `from_raw` / `as_raw` |
| `ClockError` | `BeforeUnixEpoch` · `Overflow` · `Unavailable` |

### lifecycle

| 符号 | 说明 |
|------|------|
| `ComponentState` | `Created` · `Starting` · `Running` · `Draining` · `Stopped` · `Failed`；`can_transition_to` / `try_transition` |
| `LifecycleError` | 非法转换：`from` / `to` 字段 |
| `ShutdownSignal` | `new` → `(ShutdownGuard, ShutdownSignal)`；`is_triggered` / `wait` / `wait_timeout` / `Clone` |
| `ShutdownGuard` | 唯一触发入口：`trigger(self)` |

### 模块路径

`kernel::clock` · `kernel::error` · `kernel::lifecycle` 亦可直接使用（与根 re-export 同义）。

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

```bash
cargo run -p kernel --example basic
```

## 覆盖

集成测试 `tests/public_api_surface.rs` 驱动上述全部根 re-export 构造器/方法并断言返回值。  
API 棘轮：`docs/api-baselines/kernel.txt`（`node scripts/quality-gates/check-public-api.mjs`）。
