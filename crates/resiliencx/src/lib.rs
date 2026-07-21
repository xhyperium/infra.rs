//! resiliencx —— L1 重试（active SSOT §2；ADR-005 可观测注入）。
//!
//! **当前交付面**：[`RetryConfig`] 与 [`retry_fn`]。
//! **未交付**：熔断、限流、bulkhead、async wait、backoff/jitter、retry budget
//! （见 SSOT §3 / residual OPEN）。
//!
//! 可观测性通过本 crate 的 [`Instrumentation`] trait 注入；**禁止**直接依赖 observex
//! （ADR-005）。本仓无 `xhyper-contracts`，trait 定义在本 crate（与上游 contracts 语义对齐）。
//!
//! # 实现注记
//!
//! 上游 xhyper 使用 `async fn` + 泛型 `Future`。本仓为消除 llvm-cov 对泛型 monomorph
//! 「Unexecuted instantiation」空洞，将 `retry_fn` 定为 **无类型参数** 的同步函数：
//! 闭包返回 `XResult<Box<dyn Any + Send>>`，调用方再 `downcast`。语义与 SSOT §2 一致。

use kernel::{XError, XResult};
use std::any::Any;
use std::thread;
use std::time::Duration;

/// 可观测性注入点（ADR-005）。
///
/// 上游权威面在 `contracts::Instrumentation`；本仓将 trait 放在本 crate，
/// 供 `retry_fn` 与测试 mock 使用，避免引入 contracts / observex 依赖。
pub trait Instrumentation: Send + Sync {
    /// 在真正发起一次重试**之前**记录（`attempt` 为刚失败的那次尝试序号，从 1 起）。
    fn record_retry(&self, op: &str, attempt: u32);
    /// 熔断打开（当前 `retry_fn` 不调用；预留给未来 circuit 能力）。
    fn record_circuit_open(&self, op: &str);
    /// 熔断关闭（当前 `retry_fn` 不调用）。
    fn record_circuit_close(&self, op: &str);
}

/// 空操作 instrumentation（生产可注入真实实现）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopInstrumentation;

impl Instrumentation for NoopInstrumentation {
    fn record_retry(&self, _op: &str, _attempt: u32) {}
    fn record_circuit_open(&self, _op: &str) {}
    fn record_circuit_close(&self, _op: &str) {}
}

/// 重试配置（active SSOT §2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// 最大尝试次数（**含**首次调用）。
    pub max_attempts: u32,
    /// 退避基准延迟（毫秒）。`0` 表示不 sleep。
    ///
    /// `>0` 时当前实现调用 [`thread::sleep`]——会阻塞调用线程，
    /// 且不能由 ManualClock 控制；**不是**已批准的生产 async wait 合同（SSOT §2 第 5 项）。
    pub base_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3, base_delay_ms: 0 }
    }
}

/// 装箱成功值（无泛型 monomorph，保证行覆盖可测）。
pub type RetryValue = Box<dyn Any + Send>;

/// 重试执行操作 `f`。
///
/// - 最多尝试 `max_attempts` 次（含首次）。
/// - **仅**当错误 [`XError::is_retryable`]（`Transient`）且未达上限时退避重试。
/// - 非可重试错误立即返回；耗尽后返回最后一次原始错误。
/// - 每次真正发起 retry 前调用 [`Instrumentation::record_retry`]。
/// - `max_attempts == 0` → [`XError::invalid`]。
///
/// 成功值以 [`RetryValue`] 返回；调用方 `downcast` 到具体类型。
pub fn retry_fn(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
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
                    instrumentation.record_retry(op, attempt);
                    if config.base_delay_ms > 0 {
                        // 已知差距：阻塞 sleep；生产宜注入 async 定时器（OPEN）。
                        thread::sleep(Duration::from_millis(config.base_delay_ms));
                    }
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
