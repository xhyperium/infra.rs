//! resiliencx 接入：`RetryBudget` 生产路径 + `RetryConfig` 辅助包装。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoWait, NoopInstrumentation, RetryBudget, RetryConfig, RetrySafety,
    TokioSleepWait, call_with_retry_budget, call_with_retry_budget_async,
    call_with_retry_budget_async_safe, call_with_retry_budget_safe, retry_async, retry_downcast,
    retry_fn, retry_ok,
};

/// 带预算的同步重试包装（unchecked compatibility）。
///
/// 本兼容入口不校验 [`RetrySafety`]；新生产接线使用 [`with_budget_safe`]。
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

/// 显式 [`RetrySafety`] 的生产安全同步预算包装。
pub fn with_budget_safe<F, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    safety: RetrySafety,
    op: &str,
    instr: &dyn Instrumentation,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    call_with_retry_budget_safe(budget, max_attempts, safety, op, instr, f)
}

/// 默认 Noop instrumentation（unchecked compatibility）。
pub fn with_budget_noop<F, T>(budget: &RetryBudget, max_attempts: u32, op: &str, f: F) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    with_budget(budget, max_attempts, op, &NoopInstrumentation, f)
}

/// 默认 Noop instrumentation 的生产安全同步预算包装。
pub fn with_budget_safe_noop<F, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    safety: RetrySafety,
    op: &str,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> XResult<T>,
{
    with_budget_safe(budget, max_attempts, safety, op, &NoopInstrumentation, f)
}

/// 带预算的 **async** 重试（unchecked compatibility）。
///
/// 本兼容入口不校验 [`RetrySafety`]；新生产接线使用 [`with_budget_async_safe`]。
pub async fn with_budget_async<F, Fut, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    op: &str,
    instr: &dyn Instrumentation,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    call_with_retry_budget_async(budget, max_attempts, op, instr, f).await
}

/// 显式 [`RetrySafety`] 的生产安全异步预算包装。
pub async fn with_budget_async_safe<F, Fut, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    safety: RetrySafety,
    op: &str,
    instr: &dyn Instrumentation,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    call_with_retry_budget_async_safe(budget, max_attempts, safety, op, instr, f).await
}

/// Noop instrumentation 的 async 便捷入口（unchecked compatibility）。
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

/// Noop instrumentation 的生产安全异步预算包装。
pub async fn with_budget_async_safe_noop<F, Fut, T>(
    budget: &RetryBudget,
    max_attempts: u32,
    safety: RetrySafety,
    op: &str,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    with_budget_async_safe(budget, max_attempts, safety, op, &NoopInstrumentation, f).await
}

/// 同步重试包装（unchecked compatibility）。
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

/// 异步重试（unchecked compatibility）。
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

/// 异步重试（NoWait；unchecked compatibility）。
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
pub use resiliencx::RetryConfig as PgRetryConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{ErrorKind, XError};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Default)]
    struct RetryRecords(Mutex<Vec<u32>>);

    impl Instrumentation for RetryRecords {
        fn record_retry(&self, _op: &str, attempt: u32) {
            self.0.lock().expect("记录重试序号").push(attempt);
        }

        fn record_circuit_open(&self, _op: &str) {}

        fn record_circuit_close(&self, _op: &str) {}
    }

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

    #[test]
    fn postgres_safe_sync_wrappers_cover_instrumented_and_noop_paths() {
        let budget = RetryBudget::new(1);
        let instrumented = with_budget_safe(
            &budget,
            1,
            RetrySafety::UnsafeSideEffect,
            "pg.execute.once",
            &NoopInstrumentation,
            || Ok(7u8),
        )
        .expect("单次任意 SQL 可执行");
        assert_eq!(instrumented, 7);
        assert_eq!(
            with_budget_safe_noop(&budget, 2, RetrySafety::ReadOnly, "pg.query", || Ok(9u8))
                .expect("只读查询可执行"),
            9
        );
    }

    #[tokio::test]
    async fn postgres_async_budget_retries_real_async_path() {
        let budget = RetryBudget::new(2);
        let records = RetryRecords::default();
        let n = AtomicU32::new(0);
        let v = with_budget_async(&budget, 4, "pg.execute", &records, || {
            let c = n.fetch_add(1, Ordering::SeqCst) + 1;
            async move { if c < 2 { Err(XError::transient("reset")) } else { Ok(u64::from(c)) } }
        })
        .await
        .unwrap();
        assert_eq!(v, 2);
        assert_eq!(budget.remaining(), 1);
        assert_eq!(*records.0.lock().expect("读取重试序号"), vec![1]);
    }

    #[tokio::test]
    async fn postgres_legacy_async_budget_exhaustion_is_standard() {
        let budget = RetryBudget::new(0);
        let records = RetryRecords::default();
        let calls = AtomicU32::new(0);
        let err = with_budget_async(&budget, 4, "pg.execute", &records, || {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(XError::transient("连接重置")) }
        })
        .await
        .expect_err("预算耗尽必须返回标准错误");
        assert_eq!(err.to_string(), resiliencx::budget_exhausted_error().to_string());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(records.0.lock().expect("读取重试序号").is_empty());
    }

    #[tokio::test]
    async fn postgres_safe_budget_rejects_unsafe_before_future_and_allows_read_only() {
        let budget = RetryBudget::new(2);
        let constructed = AtomicU32::new(0);
        let err = with_budget_async_safe_noop(
            &budget,
            2,
            RetrySafety::UnsafeSideEffect,
            "pg.execute",
            || {
                constructed.fetch_add(1, Ordering::SeqCst);
                async { Ok::<_, XError>(()) }
            },
        )
        .await
        .expect_err("任意 SQL 的多次尝试必须在 future 前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(constructed.load(Ordering::SeqCst), 0);

        let attempts = AtomicU32::new(0);
        let value =
            with_budget_async_safe_noop(&budget, 2, RetrySafety::ReadOnly, "pg.query", || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if attempt == 1 { Err(XError::transient("暂时失败")) } else { Ok(attempt) }
                }
            })
            .await
            .expect("显式只读 SQL 可重试");
        assert_eq!(value, 2);
    }

    #[test]
    fn retry_sync_ok() {
        let cfg = RetryConfig::fixed(2, 0);
        assert_eq!(with_retry_sync(&cfg, "pg", || Ok(9_u8)).unwrap(), 9);
    }
}
