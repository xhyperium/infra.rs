//! resiliencx 接入：对可重试 KV 操作施加 [`RetryBudget`]。
//!
//! 关闭「resiliencx 未接入 adapters」DEFER 的 redis 侧证据。

use kernel::XResult;
use resiliencx::{Instrumentation, NoopInstrumentation, RetryBudget, call_with_retry_budget};

/// 带预算的同步重试包装（供非 async 编排或测试 double 使用）。
pub fn with_budget<F, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    op: &str,
    instr: &dyn Instrumentation,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    call_with_retry_budget(budget, max_attempts, op, instr, f)
}

/// 默认 Noop instrumentation 的便捷入口。
pub fn with_budget_noop<F, T>(budget: &RetryBudget, max_attempts: u32, op: &str, f: F) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    with_budget(budget, max_attempts, op, &NoopInstrumentation, f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{ErrorKind, XError};

    #[test]
    fn redis_resilience_budget_stops_retries() {
        let budget = RetryBudget::new(1);
        let mut n = 0u32;
        let err = with_budget_noop(&budget, 5, "redis.get", || {
            n += 1;
            Err::<(), _>(XError::transient("timeout"))
        })
        .unwrap_err();
        assert!(n >= 2);
        assert!(matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Transient));
        assert!(budget.is_exhausted());
    }

    #[test]
    fn redis_resilience_success_path() {
        let budget = RetryBudget::new(3);
        let v = with_budget_noop(&budget, 3, "redis.set", || Ok(7u8)).unwrap();
        assert_eq!(v, 7);
        assert_eq!(budget.remaining(), 3);
    }
}
