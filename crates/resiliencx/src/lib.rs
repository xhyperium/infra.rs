//! resiliencx —— L1 弹性：重试 + 熔断 + 限流 + 舱壁 + 重试预算（ADR-005 可观测注入）。
//!
//! | 能力 | 类型 | 说明 |
//! |------|------|------|
//! | 重试 | [`RetryConfig`] / [`retry_fn`] / [`retry_fn_with_wait`] / [`retry_async`] | 同步 `FnMut` + 异步 `AsyncWait` |
//! | 预算 | [`RetryBudget`] / [`retry_fn_with_budget`] | 令牌式重试上限；耗尽停止 |
//! | 熔断 | [`CircuitBreaker`] | 三态；**无墙钟**（拒绝计数推进 HalfOpen） |
//! | 限流 | [`RateLimiter`] | 令牌桶；**无墙钟**（显式 [`RateLimiter::refill`]） |
//! | 舱壁 | [`Bulkhead`] | 并发上限；满载立即拒绝；RAII 许可 |
//!
//! **仍未交付**：package stable。async wait 见 feature `tokio` + [`retry_async`]。
//!
//! 可观测性通过 [`contracts::Instrumentation`] 注入；**禁止**直接依赖 observex（ADR-005）。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod budget;
pub mod bulkhead;
pub mod circuit;
pub mod rate_limit;
pub mod retry;

pub use budget::{RetryBudget, budget_exhausted_error, call_with_retry_budget, ensure_budget};
pub use bulkhead::{Bulkhead, BulkheadConfig, BulkheadPermit};
pub use circuit::{CircuitBreaker, CircuitConfig, CircuitState};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use retry::{
    AsyncWait, Backoff, NoWait, RecordingWait, RetryConfig, RetryValue, ThreadSleepWait, Wait,
    apply_deterministic_jitter, retry_async, retry_delay_ms, retry_downcast, retry_fn,
    retry_fn_with_budget, retry_fn_with_wait, retry_fn_with_wait_budget, retry_ok,
};

#[cfg(feature = "tokio")]
pub use retry::TokioSleepWait;

/// 可观测性注入点（ADR-005）——权威定义在 `xhyper-contracts`。
pub use contracts::Instrumentation;

/// 空操作 instrumentation（生产可注入真实实现）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopInstrumentation;

impl Instrumentation for NoopInstrumentation {
    fn record_retry(&self, _op: &str, _attempt: u32) {}
    fn record_circuit_open(&self, _op: &str) {}
    fn record_circuit_close(&self, _op: &str) {}
}
