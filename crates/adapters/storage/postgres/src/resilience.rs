//! resiliencx 接入：对可重试 SQL 操作施加 [`RetryBudget`]。
//!
//! 生产路径：[`crate::PostgresPool::execute`] / [`crate::PostgresPool::query`] 在配置了 budget 时经
//! [`with_budget_async`] 驱动真实 I/O。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoopInstrumentation, RetryBudget, budget_exhausted_error,
    call_with_retry_budget,
};

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

/// 带预算的 **async** 重试：驱动真实 async I/O（postgres execute/query 生产入口）。
pub async fn with_budget_async<F, Fut, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    op: &str,
    instr: &dyn Instrumentation,
    mut f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    if max_attempts == 0 {
        return Err(kernel::XError::invalid("max_attempts must be >= 1"));
    }
    let mut last_err = budget_exhausted_error();
    for attempt in 1..=max_attempts {
        if attempt > 1 {
            if !budget.try_consume() {
                return Err(last_err);
            }
            instr.record_retry(op, attempt);
        }
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retryable() => last_err = e,
            Err(e) => return Err(e),
        }
    }
    Err(last_err)
}

/// Noop instrumentation 的 async 便捷入口。
pub async fn with_budget_async_noop<F, Fut, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    op: &str,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    with_budget_async(budget, max_attempts, op, &NoopInstrumentation, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{ErrorKind, XError};
    use std::sync::atomic::{AtomicU32, Ordering};

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

    #[tokio::test]
    async fn postgres_async_budget_retries_real_async_path() {
        let budget = RetryBudget::new(2);
        let n = AtomicU32::new(0);
        let v = with_budget_async_noop(&budget, 4, "pg.execute", || {
            let c = n.fetch_add(1, Ordering::SeqCst) + 1;
            async move { if c < 2 { Err(XError::transient("reset")) } else { Ok(u64::from(c)) } }
        })
        .await
        .unwrap();
        assert_eq!(v, 2);
        assert_eq!(budget.remaining(), 1);
    }
}
