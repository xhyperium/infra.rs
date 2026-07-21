# resiliencx

L1 **弹性**（重试 + 退避 + 熔断 + 限流 + 舱壁；ADR-005）。

| 能力 | 类型 | 墙钟 |
|------|------|------|
| 重试 | `RetryConfig` / `retry_fn` / `retry_fn_with_wait` | 默认可 `ThreadSleepWait`；可注入 |
| 退避 | `Backoff::{Constant, Exponential}` + 确定性 `jitter_bps` | 纯计算 |
| 熔断 | `CircuitBreaker` 三态 | **无** |
| 限流 | `RateLimiter` 令牌桶 | **无**；显式 `refill` |
| 舱壁 | `Bulkhead` / `BulkheadPermit` | **无**；满载立即拒绝 |

**仍未交付**：async runtime wait / retry budget / package stable。

## 重试示例

```rust
use resiliencx::{Backoff, RecordingWait, RetryConfig, retry_fn_with_wait, retry_ok, NoopInstrumentation};

let cfg = RetryConfig {
    max_attempts: 4,
    base_delay_ms: 10,
    backoff: Backoff::Exponential { factor: 2, max_delay_ms: 100 },
    jitter_bps: 0,
};
let wait = RecordingWait::new(); // 测试：不睡眠，只记延迟
// let wait = resiliencx::ThreadSleepWait; // 生产默认路径
```

## 验证

```bash
cargo test -p resiliencx --all-targets
cargo clippy -p resiliencx --all-targets -- -D warnings
node scripts/cov-gate-100.mjs -p resiliencx --filter crates/resiliencx/src
```

## SSOT

`.agents/ssot/resiliencx/spec/spec.md`  
对齐：[docs/ssot/resiliencx-ssot-alignment.md](../../docs/ssot/resiliencx-ssot-alignment.md)
