//! 重试（active SSOT §2）。

use crate::Instrumentation;
use kernel::{XError, XResult};
use std::any::Any;
use std::thread;
use std::time::Duration;

/// 重试配置（active SSOT §2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConfig {
    /// 最大尝试次数（**含**首次调用）。
    pub max_attempts: u32,
    /// 退避基准延迟（毫秒）。`0` 表示不 sleep。
    ///
    /// `>0` 时当前实现调用 [`thread::sleep`]——会阻塞调用线程，
    /// 且不能由 ManualClock 控制；**不是**已批准的生产 async wait 合同。
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
