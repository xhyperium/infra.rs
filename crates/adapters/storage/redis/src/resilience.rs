//! resiliencx 接入：`RetryBudget` 生产路径 + `RetryConfig` 辅助包装。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoWait, NoopInstrumentation, RetryBudget, RetryConfig, TokioSleepWait,
    budget_exhausted_error, call_with_retry_budget, retry_async, retry_downcast, retry_fn,
    retry_ok,
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

/// 默认 Noop instrumentation 的便捷入口。
pub fn with_budget_noop<F, T>(budget: &RetryBudget, max_attempts: u32, op: &str, f: F) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    with_budget(budget, max_attempts, op, &NoopInstrumentation, f)
}

/// 带预算的 **async** 重试：驱动真实 async I/O。
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

/// 同步重试包装：仅 Transient 可重试。
pub fn with_retry_sync<T, F>(config: &RetryConfig, op: &str, mut f: F) -> XResult<T>
where
    T: 'static + Send,
    F: FnMut() -> XResult<T>,
{
    let mut wrapped = || match f() {
        Ok(v) => Ok(retry_ok(v)),
        Err(e) => Err(e),
    };
    let value = retry_fn(config, &NoopInstrumentation, op, &mut wrapped)?;
    retry_downcast(value)
}

/// 异步重试（TokioSleepWait）。
pub async fn with_retry_async<T, F, Fut>(config: &RetryConfig, op: &str, mut f: F) -> XResult<T>
where
    T: 'static + Send,
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>> + Send,
{
    let value = retry_async(config, &NoopInstrumentation, op, &TokioSleepWait, || {
        let fut = f();
        async move {
            match fut.await {
                Ok(v) => Ok(retry_ok(v)),
                Err(e) => Err(e),
            }
        }
    })
    .await?;
    retry_downcast(value)
}

/// 异步重试（NoWait，单测）。
pub async fn with_retry_async_no_wait<T, F, Fut>(
    config: &RetryConfig,
    op: &str,
    mut f: F,
) -> XResult<T>
where
    T: 'static + Send,
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>> + Send,
{
    let value = retry_async(config, &NoopInstrumentation, op, &NoWait, || {
        let fut = f();
        async move {
            match fut.await {
                Ok(v) => Ok(retry_ok(v)),
                Err(e) => Err(e),
            }
        }
    })
    .await?;
    retry_downcast(value)
}

/// 重导出。
pub use resiliencx::RetryConfig as RedisRetryConfig;

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
    }

    #[tokio::test]
    async fn redis_async_budget_exhausts() {
        let budget = RetryBudget::new(1);
        let err = with_budget_async_noop(&budget, 5, "redis.set", || async {
            Err::<(), _>(XError::transient("timeout"))
        })
        .await
        .unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Transient));
    }

    #[test]
    fn retry_sync_success() {
        let cfg = RetryConfig::fixed(2, 0);
        assert_eq!(with_retry_sync(&cfg, "op", || Ok(1_u8)).unwrap(), 1);
    }
}
