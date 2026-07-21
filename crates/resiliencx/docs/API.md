# resiliencx 公开 API

**角色**：重试/熔断/限流/舱壁

## 公开消费面

| 能力 | API |
|------|-----|
| 重试 | `RetryConfig` / `retry_fn` / `retry_fn_with_wait` / `Wait` / `NoWait` / `RecordingWait` / `ThreadSleepWait` |
| 熔断 | `CircuitBreaker` / `CircuitConfig` / `CircuitState` |
| 限流 | `RateLimiter` / `RateLimitConfig` |
| 舱壁 | `Bulkhead` / `BulkheadConfig` / `BulkheadPermit` |
| 观测 | `NoopInstrumentation` / re-export `Instrumentation` |

## 最小用法

```rust
use resiliencx::{NoopInstrumentation, RetryConfig, retry_fn, retry_ok, retry_downcast};

let cfg = RetryConfig::fixed(2, 0);
let mut op = || Ok(retry_ok(1u8));
let v = retry_downcast::<u8>(retry_fn(&cfg, &NoopInstrumentation, "op", &mut op).unwrap()).unwrap();
assert_eq!(v, 1);
```
