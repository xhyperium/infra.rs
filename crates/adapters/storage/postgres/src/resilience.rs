//! 基于 resiliencx 的重试辅助（生产路径可调用）。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    NoWait, NoopInstrumentation, RetryConfig, TokioSleepWait, retry_async, retry_downcast,
    retry_fn, retry_ok,
};

/// 同步重试包装：仅 [`kernel::ErrorKind::Transient`] 可重试。
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

/// 异步重试包装（[`TokioSleepWait`]）。
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

/// 异步重试包装（[`NoWait`]，便于单测）。
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

/// 重导出，方便调用方配置。
pub use resiliencx::RetryConfig as PgRetryConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{ErrorKind, XError};
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn sync_retries_transient_then_succeeds() {
        let cfg = RetryConfig::fixed(3, 0);
        let n = AtomicU32::new(0);
        let result = with_retry_sync(&cfg, "sync_ok", || {
            let i = n.fetch_add(1, Ordering::SeqCst) + 1;
            if i < 3 { Err(XError::transient(format!("temp-{i}"))) } else { Ok(99_i32) }
        })
        .expect("should succeed");
        assert_eq!(result, 99);
        assert_eq!(n.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn sync_stops_on_non_retryable() {
        let cfg = RetryConfig::fixed(5, 0);
        let n = AtomicU32::new(0);
        let err = with_retry_sync(&cfg, "sync_invalid", || {
            n.fetch_add(1, Ordering::SeqCst);
            Err::<i32, _>(XError::invalid("nope"))
        })
        .expect_err("must fail");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn async_retries_transient_then_succeeds() {
        let cfg = RetryConfig::fixed(3, 0);
        let n = AtomicU32::new(0);
        let result =
            with_retry_async_no_wait(&cfg, "async_ok", || {
                let i = n.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if i < 3 { Err(XError::transient(format!("temp-{i}"))) } else { Ok(7_u64) }
                }
            })
            .await
            .expect("ok");
        assert_eq!(result, 7);
        assert_eq!(n.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn async_stops_on_non_retryable() {
        let cfg = RetryConfig::fixed(4, 0);
        let n = AtomicU32::new(0);
        let err = with_retry_async_no_wait(&cfg, "async_unavailable", || {
            n.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(XError::unavailable("permanent")) }
        })
        .await
        .expect_err("fail");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn async_success_first_try() {
        let cfg = RetryConfig::fixed(3, 0);
        let n = AtomicU32::new(0);
        let v = with_retry_async_no_wait(&cfg, "async_first", || {
            n.fetch_add(1, Ordering::SeqCst);
            async { Ok("ok".to_string()) }
        })
        .await
        .expect("ok");
        assert_eq!(v, "ok");
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }
}
