# Changelog — resiliencx

## [Unreleased]

### 新增

- **退避**：`Backoff::{Constant, Exponential}`、`retry_delay_ms`
- **确定性 jitter**：`jitter_bps` + `apply_deterministic_jitter`
- **可注入 wait**：`Wait` / `ThreadSleepWait` / `NoWait` / `RecordingWait` / `retry_fn_with_wait`
- `RetryConfig::fixed` 便捷构造

### 诚实边界

- 默认 `retry_fn` 仍可能 `thread::sleep`（非 async runtime wait）
- jitter 为确定性伪随机，非加密 RNG
- retry budget / package stable：**仍未实现**

## [0.1.0] - 2026-07-21

- 重试 / 熔断 / 限流 / 舱壁初始落地；LCOV 100%
