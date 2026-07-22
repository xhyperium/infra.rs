//! OSS 操作重试：基于 `resiliencx::RetryConfig` / `retry_async`。
//!
//! 可重试语义：[`ErrorKind::Transient`] 与网络侧 [`ErrorKind::Unavailable`]。
//! 永久错误（如 `Invalid`）立即返回，不重试。

use std::any::Any;
use std::future::Future;
use std::time::Duration;

use kernel::{ErrorKind, XError, XResult};
use resiliencx::{
    AsyncWait, Instrumentation, NoWait, NoopInstrumentation, RetryConfig, TokioSleepWait,
    retry_async, retry_downcast, retry_ok,
};

/// 重试次数硬上界，防止错误配置制造无界放大。
pub const MAX_RETRY_ATTEMPTS: u32 = 10;

/// 默认重试：3 次尝试、固定 100ms 退避；生产可覆盖。
#[must_use]
pub fn default_retry_config() -> RetryConfig {
    RetryConfig::fixed(3, 100)
}

/// 判断 OSS 错误是否值得重试。
#[must_use]
pub fn is_oss_retryable(err: &XError) -> bool {
    // 鉴权/权限失败不可重试（status_error 将 401/403 映射为 Unavailable）
    let ctx = err.context().to_ascii_lowercase();
    if ctx.contains("auth/forbidden") || ctx.contains("unauthorized") || ctx.contains("forbidden") {
        return false;
    }
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
    validate_retry_config(config)?;
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

/// 在单一 deadline 内执行完整重试过程。
///
/// deadline 到期会丢弃当前尝试 future，并返回不可自动重试的
/// [`ErrorKind::DeadlineExceeded`]，从而避免每次尝试各自耗尽请求超时后继续放大。
pub async fn with_retry_deadline<F, Fut, T>(
    config: &RetryConfig,
    op: &str,
    deadline: Duration,
    f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>> + Send,
    T: Any + Send + 'static,
{
    if deadline.is_zero() {
        return Err(XError::invalid("oss retry deadline 必须大于零"));
    }
    match tokio::time::timeout(deadline, with_retry_default(config, op, f)).await {
        Ok(result) => result,
        Err(_) => Err(XError::deadline_exceeded(format!(
            "oss {op} 超过总 deadline {}ms",
            deadline.as_millis()
        ))),
    }
}

fn validate_retry_config(config: &RetryConfig) -> XResult<()> {
    if config.max_attempts == 0 || config.max_attempts > MAX_RETRY_ATTEMPTS {
        return Err(XError::invalid(format!(
            "oss retry max_attempts 必须在 1..={MAX_RETRY_ATTEMPTS} 范围内"
        )));
    }
    Ok(())
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
    use std::time::Duration;

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

    #[tokio::test]
    async fn retry_deadline_bounds_the_whole_operation() {
        let attempts = AtomicU32::new(0);
        let cfg = RetryConfig::fixed(5, 0);
        let error = with_retry_deadline(&cfg, "slow_op", Duration::from_millis(10), || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Err::<(), _>(XError::transient("slow"))
            }
        })
        .await
        .expect_err("deadline 必须终止整个重试过程");
        assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn excessive_retry_attempts_fail_closed() {
        let cfg = RetryConfig::fixed(MAX_RETRY_ATTEMPTS + 1, 0);
        let error = with_retry_default(&cfg, "too_many", || async { Ok::<_, XError>(()) })
            .await
            .expect_err("重试次数必须有硬上界");
        assert_eq!(error.kind(), ErrorKind::Invalid);
    }
}
