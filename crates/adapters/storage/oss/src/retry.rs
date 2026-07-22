//! OSS 操作重试：基于 `resiliencx::RetryConfig` / `retry_async`。
//!
//! 可重试语义：[`ErrorKind::Transient`] 与网络侧 [`ErrorKind::Unavailable`]。
//! 永久错误（如 `Invalid`）立即返回，不重试。

use std::any::Any;
use std::future::Future;

use kernel::{ErrorKind, XError, XResult};
use resiliencx::{
    AsyncWait, Instrumentation, NoWait, NoopInstrumentation, RetryConfig, TokioSleepWait,
    retry_async, retry_downcast, retry_ok,
};

/// 默认重试：3 次尝试、无退避（离线单测友好；生产可覆盖）。
#[must_use]
pub fn default_retry_config() -> RetryConfig {
    RetryConfig::fixed(3, 0)
}

/// 判断 OSS 错误是否值得重试。
#[must_use]
pub fn is_oss_retryable(err: &XError) -> bool {
    matches!(err.kind(), ErrorKind::Transient | ErrorKind::Unavailable)
}

/// 将可重试错误规范为 `Transient`，以便 `resiliencx`（仅认 `is_retryable`）接手。
fn normalize_for_retry(err: XError) -> XError {
    if err.kind() == ErrorKind::Unavailable {
        XError::transient(err.context().to_string())
    } else {
        err
    }
}

/// 异步重试包装：驱动真实 `retry_async`。
///
/// - Transient / Unavailable → 按配置重试
/// - 其他 kind（含 Invalid）→ 立即返回
pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    instrumentation: &dyn Instrumentation,
    op: &str,
    mut f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>> + Send,
    T: Any + Send + 'static,
{
    let no_wait = NoWait;
    let tokio_wait = TokioSleepWait;
    let wait: &dyn AsyncWait =
        if config.base_delay_ms == 0 { &no_wait as &dyn AsyncWait } else { &tokio_wait };

    let boxed = retry_async(config, instrumentation, op, wait, || {
        let fut = f();
        async move {
            match fut.await {
                Ok(v) => Ok(retry_ok(v)),
                Err(e) if is_oss_retryable(&e) => Err(normalize_for_retry(e)),
                Err(e) => Err(e),
            }
        }
    })
    .await?;
    retry_downcast(boxed)
}

/// 便捷：使用 [`NoopInstrumentation`]。
pub async fn with_retry_default<F, Fut, T>(config: &RetryConfig, op: &str, f: F) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>> + Send,
    T: Any + Send + 'static,
{
    with_retry(config, &NoopInstrumentation, op, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn retryable_kinds() {
        assert!(is_oss_retryable(&XError::transient("t")));
        assert!(is_oss_retryable(&XError::unavailable("u")));
        assert!(!is_oss_retryable(&XError::invalid("bad")));
        assert!(!is_oss_retryable(&XError::missing("m")));
    }

    #[tokio::test]
    async fn retries_on_transient_then_succeeds() {
        let attempts = AtomicU32::new(0);
        let cfg = RetryConfig::fixed(5, 0);
        let out =
            with_retry_default(&cfg, "test_op", || {
                let n = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if n < 3 { Err(XError::transient(format!("blip-{n}"))) } else { Ok(42u32) }
                }
            })
            .await
            .expect("should succeed after retries");
        assert_eq!(out, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retries_on_unavailable() {
        let attempts = AtomicU32::new(0);
        let cfg = RetryConfig::fixed(3, 0);
        let out = with_retry_default(&cfg, "net_op", || {
            let n = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if n < 2 { Err(XError::unavailable("conn reset")) } else { Ok("ok".to_string()) }
            }
        })
        .await
        .expect("unavailable should retry");
        assert_eq!(out, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn permanent_invalid_does_not_retry() {
        let attempts = AtomicU32::new(0);
        let cfg = RetryConfig::fixed(5, 0);
        let err = with_retry_default(&cfg, "bad_op", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async move { Err::<(), _>(XError::invalid("bad key")) }
        })
        .await
        .expect_err("invalid must fail");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(attempts.load(Ordering::SeqCst), 1, "must not retry Invalid");
    }

    #[tokio::test]
    async fn exhausts_transient_budget() {
        let attempts = AtomicU32::new(0);
        let cfg = RetryConfig::fixed(3, 0);
        let err = with_retry_default(&cfg, "always_fail", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async move { Err::<(), _>(XError::transient("still bad")) }
        })
        .await
        .expect_err("must exhaust");
        assert!(err.is_retryable() || err.kind() == ErrorKind::Transient);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
