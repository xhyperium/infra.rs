//! resiliencx 接入：`RetryBudget` 生产路径 + `RetryConfig` 辅助包装。

use std::future::Future;

use kernel::XResult;
use resiliencx::{
    Instrumentation, NoWait, NoopInstrumentation, RetryBudget, RetryConfig, RetrySafety,
    TokioSleepWait, call_with_retry_budget, call_with_retry_budget_async,
    call_with_retry_budget_async_safe, call_with_retry_budget_safe, retry_async, retry_downcast,
    retry_fn, retry_ok,
};

/// Redis 命令的重试副作用分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RedisRetrySafety {
    /// 只读命令；可在 Transient 失败后自动重试。
    ReadOnly,
    /// 固定输入的幂等写入；可在 Transient 失败后按预算重试。
    Idempotent,
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
            Self::Mset => RedisRetrySafety::Idempotent,
            Self::Set | Self::Delete | Self::Expire => RedisRetrySafety::AmbiguousWrite,
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
        matches!(self.retry_safety(), RedisRetrySafety::ReadOnly | RedisRetrySafety::Idempotent)
    }
}

/// 按 [`RedisOperation`] 合同执行一次或自动预算重试。
///
/// 只有只读或幂等操作能进入显式安全 budget 重试环；歧义写入和 publish 即使配置了 budget
/// 也只调用一次。需要依赖参数细分语义的生产路径直接使用 [`with_budget_async_safe_noop`]。
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
    let safety = match operation.retry_safety() {
        RedisRetrySafety::ReadOnly => RetrySafety::ReadOnly,
        RedisRetrySafety::Idempotent => RetrySafety::Idempotent,
        RedisRetrySafety::AmbiguousWrite | RedisRetrySafety::NeverAutomatic => return f().await,
    };
    if let Some(budget) = budget {
        return with_budget_async_safe_noop(budget, max_attempts, safety, op, f).await;
    }
    f().await
}

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

/// 默认 Noop instrumentation 的便捷入口（unchecked compatibility）。
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

/// 带预算的 **async** 重试（unchecked compatibility）：驱动真实 async I/O。
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

/// 同步重试包装（unchecked compatibility）：仅 Transient 可重试。
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

/// 异步重试（TokioSleepWait；unchecked compatibility）。
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

/// 异步重试（NoWait，单测；unchecked compatibility）。
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

    #[test]
    fn redis_safe_sync_wrappers_cover_instrumented_and_noop_paths() {
        let budget = RetryBudget::new(1);
        let instrumented = with_budget_safe(
            &budget,
            1,
            RetrySafety::UnsafeSideEffect,
            "redis.expire.once",
            &NoopInstrumentation,
            || Ok(7u8),
        )
        .expect("单次相对过期操作可执行");
        assert_eq!(instrumented, 7);
        assert_eq!(
            with_budget_safe_noop(&budget, 2, RetrySafety::ReadOnly, "redis.get", || Ok(9u8))
                .expect("只读 GET 可执行"),
            9
        );
    }

    #[tokio::test]
    async fn redis_async_budget_retries_real_async_path() {
        let budget = RetryBudget::new(2);
        let records = RetryRecords::default();
        let n = AtomicU32::new(0);
        let v = with_budget_async(&budget, 4, "redis.get", &records, || {
            let c = n.fetch_add(1, Ordering::SeqCst) + 1;
            async move { if c < 3 { Err(XError::transient("timeout")) } else { Ok(c) } }
        })
        .await
        .unwrap();
        assert_eq!(v, 3);
        assert_eq!(*records.0.lock().expect("读取重试序号"), vec![1, 2]);
    }

    #[tokio::test]
    async fn redis_async_budget_exhausts() {
        let budget = RetryBudget::new(0);
        let records = RetryRecords::default();
        let calls = AtomicU32::new(0);
        let err = with_budget_async(&budget, 5, "redis.set", &records, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(XError::transient("超时"))
        })
        .await
        .unwrap_err();
        assert_eq!(err.to_string(), resiliencx::budget_exhausted_error().to_string());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(records.0.lock().expect("读取重试序号").is_empty());
    }

    #[tokio::test]
    async fn redis_safe_budget_rejects_unsafe_before_future_and_allows_idempotent() {
        let budget = RetryBudget::new(2);
        let constructed = AtomicU32::new(0);
        let err = with_budget_async_safe_noop(
            &budget,
            2,
            RetrySafety::UnsafeSideEffect,
            "redis.expire",
            || {
                constructed.fetch_add(1, Ordering::SeqCst);
                async { Ok::<_, XError>(()) }
            },
        )
        .await
        .expect_err("相对过期时间不得自动多试");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(constructed.load(Ordering::SeqCst), 0);

        let attempts = AtomicU32::new(0);
        let value =
            with_budget_async_safe_noop(&budget, 2, RetrySafety::Idempotent, "redis.mset", || {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if attempt == 1 { Err(XError::transient("暂时失败")) } else { Ok(attempt) }
                }
            })
            .await
            .expect("固定 MSET 可按幂等语义重试");
        assert_eq!(value, 2);
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
        assert!(RedisOperation::Mset.allows_automatic_retry());
        assert_eq!(RedisOperation::Mset.retry_safety(), RedisRetrySafety::Idempotent);
    }

    #[test]
    fn retry_sync_success() {
        let cfg = RetryConfig::fixed(2, 0);
        assert_eq!(with_retry_sync(&cfg, "op", || Ok(1_u8)).unwrap(), 1);
    }
}
