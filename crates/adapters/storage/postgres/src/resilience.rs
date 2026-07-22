//! resiliencx 接入：对可重试 SQL 操作施加 [`RetryBudget`]。
//!
//! 关闭「resiliencx 未接入 adapters」DEFER 的 postgres 侧证据。

use kernel::XResult;
use resiliencx::{Instrumentation, NoopInstrumentation, RetryBudget, call_with_retry_budget};

/// 带预算的同步重试包装。
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

/// 默认 Noop instrumentation。
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
    fn postgres_resilience_budget_path() {
        let budget = RetryBudget::new(1);
        let mut n = 0u32;
        let err = with_budget_noop(&budget, 4, "pg.query", || {
            n += 1;
            Err::<(), _>(XError::transient("conn reset"))
        })
        .unwrap_err();
        assert!(n >= 2);
        assert!(matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Transient));
    }

    #[test]
    fn postgres_resilience_ok() {
        let budget = RetryBudget::new(2);
        assert_eq!(with_budget_noop(&budget, 2, "pg.exec", || Ok("done")).unwrap(), "done");
    }
}
