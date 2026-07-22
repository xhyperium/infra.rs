//! resiliencx 接入：对可重试 KV 操作施加 [`RetryBudget`]。
//!
//! 生产路径：[`crate::RedisClient::get`] / [`crate::RedisClient::set`] 在配置了 budget 时经
//! [`with_budget_async`] 驱动真实 I/O。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoopInstrumentation, RetryBudget, budget_exhausted_error,
    call_with_retry_budget,
};

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

/// 带预算的 **async** 重试：驱动真实 async I/O（redis get/set 生产入口）。
///
/// 第 1 次不扣 budget；第 2 次起 `try_consume`；预算耗尽返回最后一次可重试错误。
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

    #[tokio::test]
    async fn redis_async_budget_retries_real_async_path() {
        let budget = RetryBudget::new(2);
        let n = AtomicU32::new(0);
        let v = with_budget_async_noop(&budget, 4, "redis.get", || {
            let c = n.fetch_add(1, Ordering::SeqCst) + 1;
            async move { if c < 3 { Err(XError::transient("timeout")) } else { Ok(c) } }
        })
        .await
        .unwrap();
        assert_eq!(v, 3);
        // attempts 2 and 3 consumed budget (max 2)
        assert_eq!(budget.remaining(), 0);
    }

    #[tokio::test]
    async fn redis_async_budget_exhausts() {
        let budget = RetryBudget::new(1);
        let n = AtomicU32::new(0);
        let err = with_budget_async_noop(&budget, 5, "redis.set", || {
            n.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(XError::transient("again")) }
        })
        .await
        .unwrap_err();
        assert!(n.load(Ordering::SeqCst) >= 2);
        assert!(matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Transient));
    }
}
