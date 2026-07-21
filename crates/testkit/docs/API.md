# testkit 公开 API

**Package**：`testkit` · **角色**：T0 ManualClock  
**生产层级**：L1 ManualClock test-support（**非**生产 runtime）

## 公开消费面

| 类型 | 方法 / 变体 |
|------|-------------|
| `ManualClock` | `new` · `with_monotonic_elapsed` · `domain` · `set_wall` · `advance_wall` · `rewind_wall` · `set_monotonic_elapsed` · `advance_monotonic` · `set_wall_fault` · `clear_wall_fault` · `wall_fault` · `snapshot`；实现 `kernel::Clock`（`now` / `monotonic`） |
| `ManualClockSnapshot` | `wall` · `monotonic_elapsed` · `wall_fault` |
| `ManualClockFault` | `BeforeUnixEpoch` · `Overflow` · `Unavailable`（映射 `ClockError`） |
| `ManualClockError` | `WallOverflow` · `MonotonicOverflow` · `MonotonicRegression` · `Synchronization`（中文 `Display`） |

## 不变量

- 控制路径 checked：失败不修改状态
- 无 `Default` / `Clone`；共享用 `Arc`
- 每个实例独立 `ClockDomain`
- poison：`Clock::monotonic` 恢复；控制路径返回 `Synchronization`

## 最小用法

```rust
use kernel::{Clock, Timestamp};
use testkit::ManualClock;
use std::time::Duration;

let c = ManualClock::new(Timestamp::from_unix_nanos(0));
c.advance_wall(Duration::from_secs(1)).unwrap();
assert_eq!(c.now().unwrap().as_unix_nanos(), 1_000_000_000);
```

```bash
cargo run -p testkit --example basic
```

## 覆盖

`tests/public_api_surface.rs` 驱动全部公开方法与 fault/error 变体。  
API 棘轮：`docs/api-baselines/testkit.txt`。
