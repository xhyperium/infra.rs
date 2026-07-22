//! 重试预算（Retry Budget）。
//!
//! 限制一段逻辑窗口内可消耗的重试次数，防止雪崩。与 [`crate::retry`] 组合使用。

use std::sync::{Arc, Mutex};

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
///
/// 内部使用 [`Mutex`] 保证线程安全，避免 CAS 竞态分支无法单测覆盖。
#[derive(Debug, Clone)]
pub struct RetryBudget {
    remaining: Arc<Mutex<u32>>,
    capacity: u32,
}

impl RetryBudget {
    /// 构造满额预算。
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { remaining: Arc::new(Mutex::new(capacity)), capacity }
    }

    /// 容量。
    #[must_use]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// 剩余令牌。
    #[must_use]
    pub fn remaining(&self) -> u32 {
        *self.remaining.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// 是否已耗尽。
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }

    /// 尝试消耗 1 个令牌；成功 `true`，耗尽 `false`。
    pub fn try_consume(&self) -> bool {
        let mut g = self.remaining.lock().unwrap_or_else(|p| p.into_inner());
        if *g == 0 {
            return false;
        }
        *g -= 1;
        true
    }

    /// 退还 1 个令牌（不超过 capacity）。
    pub fn refund(&self) {
        let mut g = self.remaining.lock().unwrap_or_else(|p| p.into_inner());
        if *g >= self.capacity {
            return;
        }
        *g += 1;
    }

    /// 重置为满额。
    pub fn reset(&self) {
        let mut g = self.remaining.lock().unwrap_or_else(|p| p.into_inner());
        *g = self.capacity;
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
    if max_attempts == 0 {
        return Err(XError::invalid("max_attempts must be >= 1"));
    }
    // 占位；仅在「可重试错误耗尽 / 预算耗尽」路径返回
    let mut last_err = budget_exhausted_error();
    for attempt in 1..=max_attempts {
        if attempt > 1 {
            if !budget.try_consume() {
                return Err(last_err);
            }
            instr.record_retry(op, attempt);
        }
        match f() {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retryable() => last_err = e,
            Err(e) => return Err(e),
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoopInstrumentation;
    use kernel::ErrorKind;

    #[test]
    fn budget_consume_and_exhaust() {
        let b = RetryBudget::new(2);
        assert_eq!(b.capacity(), 2);
        assert!(b.try_consume());
        assert!(b.try_consume());
        assert!(!b.try_consume());
        assert!(b.is_exhausted());
        ensure_budget(&b).unwrap_err();
        b.refund();
        assert_eq!(b.remaining(), 1);
        b.reset();
        assert_eq!(b.remaining(), 2);
        // 满额时 refund 不超容
        b.refund();
        assert_eq!(b.remaining(), 2);
        assert_eq!(b.capacity(), 2);
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

        // 预算充足、尝试次数用尽 → 走循环结束后的 last_err 返回
        let b2 = RetryBudget::new(10);
        let mut n2 = 0u32;
        let err2 = call_with_retry_budget(&b2, 3, "ex", &NoopInstrumentation, || {
            n2 += 1;
            Err::<(), _>(XError::transient("t"))
        })
        .unwrap_err();
        assert_eq!(n2, 3);
        assert_eq!(err2.kind(), ErrorKind::Transient);
    }

    #[test]
    fn call_with_retry_budget_ok_and_non_retryable() {
        let b = RetryBudget::new(3);
        let v = call_with_retry_budget(&b, 3, "ok", &NoopInstrumentation, || Ok(42)).unwrap();
        assert_eq!(v, 42);
        let err = call_with_retry_budget(&b, 3, "nr", &NoopInstrumentation, || {
            Err::<(), _>(XError::invalid("nope"))
        })
        .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Invalid);
        let zero = call_with_retry_budget(&b, 0, "z", &NoopInstrumentation, || Ok(())).unwrap_err();
        assert_eq!(zero.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn budget_recovers_from_poison() {
        let b = RetryBudget::new(1);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = b.remaining.lock().expect("lock");
            panic!("poison budget");
        }));
        assert!(b.try_consume());
        assert!(b.is_exhausted());
        b.refund();
        assert_eq!(b.remaining(), 1);
        b.reset();
        assert_eq!(b.remaining(), 1);
    }
}
