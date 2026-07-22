//! resiliencx 接入：`RetryBudget` 生产路径 + `RetryConfig` 辅助包装。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoWait, NoopInstrumentation, RetryBudget, RetryConfig, TokioSleepWait,
    budget_exhausted_error, call_with_retry_budget, retry_async, retry_downcast, retry_fn,
    retry_ok,
};

/// Redis 命令的重试副作用分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RedisRetrySafety {
    /// 只读命令；可在 Transient 失败后自动重试。
    ReadOnly,
    /// 写命令的响应可能丢失；重试可能重复副作用，只能由调用方显式选择。
    AmbiguousWrite,
    /// 自动重试会破坏合同（例如 Pub/Sub 可能重复投递）。
    NeverAutomatic,
}

/// Redis 命令的原子性边界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RedisAtomicity {
    /// 单条 Redis 命令；服务端执行原子，但客户端超时不代表命令未生效。
    SingleCommand,
    /// 多 key 单条命令；仅在 Standalone 或 Cluster 同一 hash slot 内成立。
    MultiKeySingleSlot,
    /// 无可靠投递或事务原子性保证。
    None,
}

/// 当前公开操作，用于查询可测试的重试与原子性合同。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RedisOperation {
    /// GET。
    Get,
    /// SET / PSETEX。
    Set,
    /// DEL。
    Delete,
    /// EXISTS。
    Exists,
    /// PEXPIRE。
    Expire,
    /// PTTL。
    Ttl,
    /// MGET。
    Mget,
    /// MSET。
    Mset,
    /// PUBLISH。
    Publish,
}

impl RedisOperation {
    /// 该操作的自动重试安全分类。
    #[must_use]
    pub const fn retry_safety(self) -> RedisRetrySafety {
        match self {
            Self::Get | Self::Exists | Self::Ttl | Self::Mget => RedisRetrySafety::ReadOnly,
            Self::Set | Self::Delete | Self::Expire | Self::Mset => {
                RedisRetrySafety::AmbiguousWrite
            }
            Self::Publish => RedisRetrySafety::NeverAutomatic,
        }
    }

    /// 该操作的 Redis 服务端原子性边界。
    #[must_use]
    pub const fn atomicity(self) -> RedisAtomicity {
        match self {
            Self::Mget | Self::Mset => RedisAtomicity::MultiKeySingleSlot,
            Self::Publish => RedisAtomicity::None,
            Self::Get | Self::Set | Self::Delete | Self::Exists | Self::Expire | Self::Ttl => {
                RedisAtomicity::SingleCommand
            }
        }
    }

    /// 是否允许客户端在配置预算后自动重试。
    #[must_use]
    pub const fn allows_automatic_retry(self) -> bool {
        matches!(self.retry_safety(), RedisRetrySafety::ReadOnly)
    }
}

/// 按 [`RedisOperation`] 合同执行一次或自动预算重试。
///
/// 生产客户端的所有默认操作都经过此分派；只有 [`RedisRetrySafety::ReadOnly`] 能进入
/// budget 重试环，写入和 publish 即使配置了 budget 也只调用一次。
pub(crate) async fn with_automatic_budget<F, Fut, T>(
    operation: RedisOperation,
    budget: Option<&RetryBudget>,
    max_attempts: u32,
    op: &str,
    mut f: F,
) -> XResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = XResult<T>>,
{
    if operation.allows_automatic_retry() {
        if let Some(budget) = budget {
            return with_budget_async_noop(budget, max_attempts, op, f).await;
        }
    }
    f().await
}

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

    #[tokio::test]
    async fn non_retryable_failure_is_attempted_once() {
        let budget = RetryBudget::new(4);
        let attempts = AtomicU32::new(0);
        let err = with_budget_async_noop(&budget, 5, "redis.get", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(XError::invalid("bad request")) }
        })
        .await
        .expect_err("invalid must not retry");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(budget.remaining(), 4);
    }

    #[tokio::test]
    async fn production_dispatch_attempts_ambiguous_write_once() {
        let budget = RetryBudget::new(4);
        let attempts = AtomicU32::new(0);
        let err = with_automatic_budget(RedisOperation::Set, Some(&budget), 5, "redis.set", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(XError::transient("response lost")) }
        })
        .await
        .expect_err("ambiguous write must not retry automatically");
        assert_eq!(err.kind(), ErrorKind::Transient);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(budget.remaining(), 4);
    }

    #[tokio::test]
    async fn production_dispatch_retries_read_with_budget() {
        let budget = RetryBudget::new(2);
        let attempts = AtomicU32::new(0);
        let value = with_automatic_budget(
            RedisOperation::Get,
            Some(&budget),
            3,
            "redis.get",
            || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if attempt == 1 { Err(XError::transient("retry")) } else { Ok(attempt) }
                }
            },
        )
        .await
        .expect("read retries");
        assert_eq!(value, 2);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(budget.remaining(), 1);
    }

    #[test]
    fn operation_contract_forbids_automatic_write_retry() {
        for operation in [
            RedisOperation::Set,
            RedisOperation::Delete,
            RedisOperation::Expire,
            RedisOperation::Mset,
            RedisOperation::Publish,
        ] {
            assert!(!operation.allows_automatic_retry(), "operation={operation:?}");
        }
        assert_eq!(RedisOperation::Set.atomicity(), RedisAtomicity::SingleCommand);
        assert_eq!(RedisOperation::Mset.atomicity(), RedisAtomicity::MultiKeySingleSlot);
        assert_eq!(RedisOperation::Publish.atomicity(), RedisAtomicity::None);
    }

    #[test]
    fn operation_contract_allows_read_retry() {
        for operation in
            [RedisOperation::Get, RedisOperation::Exists, RedisOperation::Ttl, RedisOperation::Mget]
        {
            assert!(operation.allows_automatic_retry(), "operation={operation:?}");
            assert_eq!(operation.retry_safety(), RedisRetrySafety::ReadOnly);
        }
    }

    #[test]
    fn retry_sync_success() {
        let cfg = RetryConfig::fixed(2, 0);
        assert_eq!(with_retry_sync(&cfg, "op", || Ok(1_u8)).unwrap(), 1);
    }
}
