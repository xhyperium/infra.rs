//! 重试（active SSOT §2 + 本仓退避/可注入 wait）。

use crate::Instrumentation;
use async_trait::async_trait;
use kernel::{XError, XResult};
use std::any::Any;
use std::future::Future;
use std::thread;
use std::time::Duration;

// ── Backoff / Wait ─────────────────────────────────────────────────────────

/// 退避策略（相对 [`RetryConfig::base_delay_ms`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backoff {
    /// 每次重试使用固定 `base_delay_ms`（历史默认）。
    #[default]
    Constant,
    /// 指数：`min(base * factor^(attempt-1), max_delay_ms)`；`attempt` 为已完成失败次数（≥1）。
    Exponential {
        /// 倍数，须 ≥ 1；`1` 等价常数。
        factor: u32,
        /// 上限毫秒。
        max_delay_ms: u64,
    },
}

/// 等待策略（对象安全）。生产默认 [`ThreadSleepWait`]；测试可用 [`NoWait`] / [`RecordingWait`]。
pub trait Wait: Send + Sync {
    /// 等待 `ms` 毫秒（`0` 应为空操作）。
    fn wait_ms(&self, ms: u64);
}

/// 使用 [`thread::sleep`] 的默认 wait（会阻塞调用线程）。
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadSleepWait;

impl Wait for ThreadSleepWait {
    fn wait_ms(&self, ms: u64) {
        if ms > 0 {
            thread::sleep(Duration::from_millis(ms));
        }
    }
}

/// 空 wait（不睡眠；配合 delay 计算测试）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoWait;

impl Wait for NoWait {
    fn wait_ms(&self, _ms: u64) {}
}

/// 记录每次请求延迟的 wait（测试用；不睡眠）。
#[derive(Debug, Default)]
pub struct RecordingWait {
    delays: std::sync::Mutex<Vec<u64>>,
}

impl RecordingWait {
    /// 构造空记录器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 已记录的延迟序列（毫秒）。
    #[must_use]
    pub fn delays(&self) -> Vec<u64> {
        self.delays.lock().expect("recording wait lock").clone()
    }
}

impl Wait for RecordingWait {
    fn wait_ms(&self, ms: u64) {
        self.delays.lock().expect("recording wait lock").push(ms);
    }
}

// ── AsyncWait / 非阻塞重试 ─────────────────────────────────────────────────

/// 异步等待策略（对象安全）。
///
/// async 服务路径应使用本 trait + [`retry_async`]，**禁止**在 async 任务中直接调用
/// 默认阻塞的 [`retry_fn`] / [`ThreadSleepWait`]。
#[async_trait]
pub trait AsyncWait: Send + Sync {
    /// 异步等待 `ms` 毫秒（`0` 应为空操作）。
    async fn wait_ms(&self, ms: u64);
}

#[async_trait]
impl AsyncWait for NoWait {
    async fn wait_ms(&self, _ms: u64) {}
}

#[async_trait]
impl AsyncWait for RecordingWait {
    async fn wait_ms(&self, ms: u64) {
        self.delays.lock().expect("recording wait lock").push(ms);
    }
}

/// 基于 `tokio::time::sleep` 的非阻塞 wait（feature `tokio`）。
///
/// 在 async runtime 内 `await` 等待，**不**占用阻塞线程。
#[cfg(feature = "tokio")]
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioSleepWait;

#[cfg(feature = "tokio")]
#[async_trait]
impl AsyncWait for TokioSleepWait {
    async fn wait_ms(&self, ms: u64) {
        if ms > 0 {
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
    }
}

// ── Config ─────────────────────────────────────────────────────────────────

/// 重试配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// 最大尝试次数（**含**首次调用）。
    pub max_attempts: u32,
    /// 退避基准延迟（毫秒）。`0` 表示不 wait。
    pub base_delay_ms: u64,
    /// 退避策略。
    pub backoff: Backoff,
    /// 全抖动幅度（basis points，0..=10000）。
    ///
    /// `0`：无抖动。`>0`：在计算出的延迟上施加**确定性**伪抖动
    /// （由 `attempt` 驱动，非加密 RNG），便于单测。
    pub jitter_bps: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3, base_delay_ms: 0, backoff: Backoff::Constant, jitter_bps: 0 }
    }
}

impl RetryConfig {
    /// 仅设置 attempts + 固定 base delay（兼容旧构造习惯）。
    #[must_use]
    pub const fn fixed(max_attempts: u32, base_delay_ms: u64) -> Self {
        Self { max_attempts, base_delay_ms, backoff: Backoff::Constant, jitter_bps: 0 }
    }
}

/// 计算第 `attempt` 次失败后、发起下一次尝试前应等待的毫秒数。
///
/// `attempt` 为刚失败的尝试序号（从 1 起）。
#[must_use]
pub fn retry_delay_ms(config: &RetryConfig, attempt: u32) -> u64 {
    if config.base_delay_ms == 0 || attempt == 0 {
        return 0;
    }
    let raw = match config.backoff {
        Backoff::Constant => config.base_delay_ms,
        Backoff::Exponential { factor, max_delay_ms } => {
            let f = factor.max(1);
            // base * f^(attempt-1)，饱和到 max
            let mut d = config.base_delay_ms;
            let mut i = 1u32;
            while i < attempt {
                d = d.saturating_mul(u64::from(f));
                if d >= max_delay_ms {
                    d = max_delay_ms;
                    break;
                }
                i = i.saturating_add(1);
            }
            d.min(max_delay_ms)
        }
    };
    apply_deterministic_jitter(raw, config.jitter_bps, attempt)
}

/// 确定性伪抖动：`delay * (10000 - offset) / 10000`，`offset ∈ [0, jitter_bps]`。
#[must_use]
pub fn apply_deterministic_jitter(delay_ms: u64, jitter_bps: u32, attempt: u32) -> u64 {
    if delay_ms == 0 || jitter_bps == 0 {
        return delay_ms;
    }
    let span = u64::from(jitter_bps.min(10_000));
    // 简单确定性序列：与 attempt 绑定，避免 flaky
    let offset = (u64::from(attempt).wrapping_mul(7919)) % (span.saturating_add(1));
    let keep = 10_000u64.saturating_sub(offset);
    delay_ms.saturating_mul(keep) / 10_000
}

// ── retry_fn ───────────────────────────────────────────────────────────────

/// 装箱成功值（无泛型 monomorph，保证行覆盖可测）。
pub type RetryValue = Box<dyn Any + Send>;

/// 重试执行操作 `f`（默认 [`ThreadSleepWait`]）。
///
/// 见 [`retry_fn_with_wait`]。
pub fn retry_fn(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    f: &mut dyn FnMut() -> XResult<RetryValue>,
) -> XResult<RetryValue> {
    retry_fn_with_wait(config, instrumentation, op, &ThreadSleepWait, f)
}

/// 带可注入 [`Wait`] 的重试。
///
/// - 最多尝试 `max_attempts` 次（含首次）。
/// - **仅**当错误 [`XError::is_retryable`]（`Transient`）且未达上限时退避重试。
/// - 非可重试错误立即返回；耗尽后返回最后一次原始错误。
/// - 每次真正发起 retry 前：`record_retry` 然后 `wait.wait_ms(retry_delay_ms(...))`。
/// - `max_attempts == 0` → [`XError::invalid`]。
pub fn retry_fn_with_wait(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    wait: &dyn Wait,
    f: &mut dyn FnMut() -> XResult<RetryValue>,
) -> XResult<RetryValue> {
    retry_fn_with_wait_budget(config, instrumentation, op, wait, None, f)
}

/// 带可选 [`crate::RetryBudget`] 的重试。
///
/// 在即将发起第 N 次重试（非首次）前 `try_consume`；预算耗尽返回 budget 错误。
pub fn retry_fn_with_budget(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    budget: &crate::RetryBudget,
    f: &mut dyn FnMut() -> XResult<RetryValue>,
) -> XResult<RetryValue> {
    retry_fn_with_wait_budget(config, instrumentation, op, &ThreadSleepWait, Some(budget), f)
}

/// 完整：Wait + 可选预算。
pub fn retry_fn_with_wait_budget(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    wait: &dyn Wait,
    budget: Option<&crate::RetryBudget>,
    f: &mut dyn FnMut() -> XResult<RetryValue>,
) -> XResult<RetryValue> {
    let mut last_err = None;
    for attempt in 1..=config.max_attempts {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                let retryable = e.is_retryable();
                last_err = Some(e);
                if retryable && attempt < config.max_attempts {
                    if let Some(b) = budget {
                        if !b.try_consume() {
                            return Err(crate::budget_exhausted_error());
                        }
                    }
                    instrumentation.record_retry(op, attempt);
                    let delay = retry_delay_ms(config, attempt);
                    wait.wait_ms(delay);
                } else {
                    break;
                }
            }
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Err(XError::invalid("max_attempts must be >= 1")),
    }
}

/// 异步重试：退避时 `await` [`AsyncWait`]，不阻塞 worker 线程。
///
/// 语义与 [`retry_fn_with_wait`] 一致，但操作 `f` 为返回 `Future` 的闭包。
///
/// # 生产路径
///
/// - async 服务：本函数 + [`NoWait`]（测试）或 feature `tokio` 的 [`TokioSleepWait`]
/// - 同步批处理：继续用 [`retry_fn`] / [`retry_fn_with_wait`]
///
/// # 禁止
///
/// 在 async 任务内调用 [`retry_fn`]（默认 [`ThreadSleepWait`] 会阻塞 runtime 线程）。
pub async fn retry_async<F, Fut>(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    wait: &dyn AsyncWait,
    mut f: F,
) -> XResult<RetryValue>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<RetryValue>> + Send,
{
    let mut last_err = None;
    for attempt in 1..=config.max_attempts {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let retryable = e.is_retryable();
                last_err = Some(e);
                if retryable && attempt < config.max_attempts {
                    instrumentation.record_retry(op, attempt);
                    let delay = retry_delay_ms(config, attempt);
                    wait.wait_ms(delay).await;
                } else {
                    break;
                }
            }
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Err(XError::invalid("max_attempts must be >= 1")),
    }
}

/// 将具体成功值装箱为 [`RetryValue`]。
#[must_use]
pub fn retry_ok<T: Any + Send>(value: T) -> RetryValue {
    Box::new(value)
}

/// 将 [`RetryValue`] downcast 为 `T`；类型不匹配时返回 `Invalid`。
pub fn retry_downcast<T: Any>(value: RetryValue) -> XResult<T> {
    match value.downcast::<T>() {
        Ok(b) => Ok(*b),
        Err(_) => Err(XError::invalid("retry value type mismatch")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_zero_base_or_attempt() {
        let cfg = RetryConfig::fixed(3, 0);
        assert_eq!(retry_delay_ms(&cfg, 1), 0);
        let cfg = RetryConfig::fixed(3, 10);
        assert_eq!(retry_delay_ms(&cfg, 0), 0);
    }

    #[test]
    fn constant_backoff() {
        let cfg = RetryConfig {
            max_attempts: 5,
            base_delay_ms: 10,
            backoff: Backoff::Constant,
            jitter_bps: 0,
        };
        assert_eq!(retry_delay_ms(&cfg, 1), 10);
        assert_eq!(retry_delay_ms(&cfg, 3), 10);
    }

    #[test]
    fn exponential_backoff_caps() {
        let cfg = RetryConfig {
            max_attempts: 10,
            base_delay_ms: 10,
            backoff: Backoff::Exponential { factor: 2, max_delay_ms: 40 },
            jitter_bps: 0,
        };
        assert_eq!(retry_delay_ms(&cfg, 1), 10);
        assert_eq!(retry_delay_ms(&cfg, 2), 20);
        assert_eq!(retry_delay_ms(&cfg, 3), 40);
        assert_eq!(retry_delay_ms(&cfg, 4), 40);
    }

    #[test]
    fn exponential_factor_one_equals_constant_until_cap() {
        let cfg = RetryConfig {
            max_attempts: 5,
            base_delay_ms: 7,
            backoff: Backoff::Exponential { factor: 0, max_delay_ms: 100 }, // 0 → max(1)
            jitter_bps: 0,
        };
        assert_eq!(retry_delay_ms(&cfg, 1), 7);
        assert_eq!(retry_delay_ms(&cfg, 3), 7);
    }

    #[test]
    fn jitter_zero_unchanged() {
        assert_eq!(apply_deterministic_jitter(100, 0, 1), 100);
        assert_eq!(apply_deterministic_jitter(0, 5000, 1), 0);
    }

    #[test]
    fn jitter_reduces_or_keeps() {
        let j = apply_deterministic_jitter(1000, 5000, 1);
        assert!(j <= 1000);
        // 同一 attempt 确定性
        assert_eq!(j, apply_deterministic_jitter(1000, 5000, 1));
        assert_ne!(j, apply_deterministic_jitter(1000, 5000, 2));
    }

    #[test]
    fn recording_wait_captures_exponential_delays() {
        let cfg = RetryConfig {
            max_attempts: 4,
            base_delay_ms: 5,
            backoff: Backoff::Exponential { factor: 2, max_delay_ms: 100 },
            jitter_bps: 0,
        };
        let wait = RecordingWait::new();
        let hits = std::sync::Mutex::new(0u32);
        let mut op = || {
            let mut g = hits.lock().expect("h");
            *g += 1;
            if *g < 4 { Err(XError::transient("t")) } else { Ok(retry_ok(())) }
        };
        let instr = crate::NoopInstrumentation;
        retry_fn_with_wait(&cfg, &instr, "op", &wait, &mut op).expect("ok");
        assert_eq!(wait.delays(), vec![5, 10, 20]);
    }

    #[test]
    fn no_wait_and_thread_sleep_zero() {
        Wait::wait_ms(&NoWait, 0);
        Wait::wait_ms(&NoWait, 1);
        Wait::wait_ms(&ThreadSleepWait, 0);
        let _ = format!("{:?}", ThreadSleepWait);
        let _ = format!("{:?}", NoWait);
        let _ = format!("{:?}", Backoff::default());
        let _ = RetryConfig::fixed(2, 0);
    }

    #[test]
    fn config_default_has_constant_backoff() {
        let d = RetryConfig::default();
        assert_eq!(d.backoff, Backoff::Constant);
        assert_eq!(d.jitter_bps, 0);
    }

    #[test]
    fn budget_stops_further_retries() {
        let cfg = RetryConfig::fixed(10, 0);
        let budget = crate::RetryBudget::new(1); // only one retry token
        let hits = std::sync::Mutex::new(0u32);
        let mut op = || {
            let mut g = hits.lock().unwrap();
            *g += 1;
            Err(XError::transient("t"))
        };
        let instr = crate::NoopInstrumentation;
        let err = retry_fn_with_wait_budget(&cfg, &instr, "op", &NoWait, Some(&budget), &mut op)
            .unwrap_err();
        // 第 1 次失败 → consume → 第 2 次失败 → consume 失败 → Unavailable
        assert_eq!(err.kind(), kernel::ErrorKind::Unavailable);
        assert_eq!(*hits.lock().unwrap(), 2);
        assert!(budget.is_exhausted());
        budget.reset();
        assert_eq!(budget.remaining(), 1);
    }
}
