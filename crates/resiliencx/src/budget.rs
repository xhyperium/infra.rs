//! 重试预算（Retry Budget）。
//!
//! 限制一段逻辑窗口内可消耗的重试次数，防止雪崩。与 [`crate::retry`] 组合使用。

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use kernel::{XError, XResult};

/// 预算耗尽时的标准错误。
#[must_use]
pub fn budget_exhausted_error() -> XError {
    XError::unavailable("retry budget 已耗尽")
}

/// 若预算已耗尽则返回错误。
pub fn ensure_budget(budget: &RetryBudget) -> XResult<()> {
    if budget.is_exhausted() { Err(budget_exhausted_error()) } else { Ok(()) }
}

/// 重试预算：共享令牌计数。
///
/// - `try_consume`：成功消耗返回 `true`；耗尽返回 `false`；
/// - `refund`：可选退还；
/// - `reset`：恢复满额。
#[derive(Debug, Clone)]
pub struct RetryBudget {
    remaining: Arc<AtomicU32>,
    capacity: u32,
}

impl RetryBudget {
    /// 构造满额预算。
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { remaining: Arc::new(AtomicU32::new(capacity)), capacity }
    }

    /// 容量。
    #[must_use]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// 剩余令牌。
    #[must_use]
    pub fn remaining(&self) -> u32 {
        self.remaining.load(Ordering::SeqCst)
    }

    /// 是否已耗尽。
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }

    /// 尝试消耗 1 个令牌；成功 `true`，耗尽 `false`。
    pub fn try_consume(&self) -> bool {
        loop {
            let cur = self.remaining.load(Ordering::SeqCst);
            if cur == 0 {
                return false;
            }
            if self
                .remaining
                .compare_exchange(cur, cur - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// 退还 1 个令牌（不超过 capacity）。
    pub fn refund(&self) {
        loop {
            let cur = self.remaining.load(Ordering::SeqCst);
            if cur >= self.capacity {
                return;
            }
            if self
                .remaining
                .compare_exchange(cur, cur + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return;
            }
        }
    }

    /// 重置为满额。
    pub fn reset(&self) {
        self.remaining.store(self.capacity, Ordering::SeqCst);
    }
}

/// Adapter 调用路径：带预算的可重试同步闭包。
pub fn call_with_retry_budget<F, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    op: &str,
    instr: &dyn crate::Instrumentation,
    mut f: F,
) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    let max_attempts = max_attempts.max(1);
    let mut last_err: Option<XError> = None;
    for attempt in 1..=max_attempts {
        if attempt > 1 {
            if !budget.try_consume() {
                return Err(last_err.unwrap_or_else(budget_exhausted_error));
            }
            instr.record_retry(op, attempt);
        }
        match f() {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retryable() => last_err = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| XError::unavailable("retry 用尽")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoopInstrumentation;
    use kernel::ErrorKind;

    #[test]
    fn budget_consume_and_exhaust() {
        let b = RetryBudget::new(2);
        assert!(b.try_consume());
        assert!(b.try_consume());
        assert!(!b.try_consume());
        assert!(b.is_exhausted());
        ensure_budget(&b).unwrap_err();
        b.refund();
        assert_eq!(b.remaining(), 1);
        b.reset();
        assert_eq!(b.remaining(), 2);
        assert_eq!(budget_exhausted_error().kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn call_with_retry_budget_path() {
        let b = RetryBudget::new(1);
        let mut n = 0u32;
        let err = call_with_retry_budget(&b, 5, "op", &NoopInstrumentation, || {
            n += 1;
            Err::<(), _>(XError::transient("t"))
        })
        .unwrap_err();
        assert!(n >= 2);
        assert!(matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Transient));
    }
}
