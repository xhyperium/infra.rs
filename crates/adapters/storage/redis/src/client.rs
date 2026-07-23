//! 可克隆的 Redis 命令客户端（共享 [`RedisPool`]）。

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use contracts::KeyValueStore;
use kernel::{XError, XResult};
use redis::AsyncCommands;
use resiliencx::{RetryBudget, RetrySafety};

use crate::error_map::map_redis_result;
use crate::pool::RedisPool;
use crate::resilience::{RedisOperation, with_automatic_budget, with_budget_async_safe_noop};

/// 生产 Redis KV 客户端。
///
/// `Clone` 只共享底层池引用；所有命令受池级 Semaphore 与超时约束。
/// 可选 [`RetryBudget`]：配置后按显式 [`RetrySafety`] 分类路由。只读与幂等操作可以重试；
/// 结果不明的副作用操作在多次尝试进入 driver 前 fail-closed。
#[derive(Clone, Debug)]
pub struct RedisClient {
    pool: RedisPool,
    /// 可选重试预算（与 `budget_max_attempts` 一起启用）。
    budget: Option<Arc<RetryBudget>>,
    budget_max_attempts: u32,
}

impl RedisClient {
    pub(crate) fn from_pool(pool: RedisPool) -> Self {
        Self { pool, budget: None, budget_max_attempts: 3 }
    }

    /// 注入 [`RetryBudget`]：后续操作按各自 [`RetrySafety`] 走 resiliencx 异步安全路由。
    #[must_use]
    pub fn with_retry_budget(mut self, budget: RetryBudget, max_attempts: u32) -> Self {
        self.budget = Some(Arc::new(budget));
        self.budget_max_attempts = max_attempts;
        self
    }

    /// 是否已配置重试预算。
    #[must_use]
    pub fn has_retry_budget(&self) -> bool {
        self.budget.is_some()
    }

    /// 兼容旧 `RedisLiveKv::connect(url)`：从 URL 建池并返回客户端。
    pub async fn connect(url: &str) -> XResult<Self> {
        let cfg = crate::config::RedisConfig::from_url(url)?;
        let pool = RedisPool::connect(cfg).await?;
        Ok(pool.client())
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Ok(RedisPool::connect_from_env().await?.client())
    }

    /// 所属池。
    #[must_use]
    pub fn pool(&self) -> &RedisPool {
        &self.pool
    }

    /// 脱敏端点。
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.pool.endpoint()
    }

    /// 单次 `GET` I/O（无 budget 环）。
    async fn get_once(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let v: Option<Vec<u8>> = map_redis_result(conn.get(key).await)?;
                Ok(v)
            })
            .await
    }

    /// 单次 `SET` I/O（无 budget 环）。
    async fn set_once(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        validate_ttl(ttl)?;
        let key = key.to_owned();
        self.pool
            .with_conn(move |mut conn| async move {
                if let Some(ttl) = ttl {
                    let ms = duration_to_millis(ttl)?;
                    let _: () = map_redis_result(conn.pset_ex(key, val, ms).await)?;
                } else {
                    let _: () = map_redis_result(conn.set(key, val).await)?;
                }
                Ok(())
            })
            .await
    }

    /// `GET`；缺失返回 `Ok(None)`。空字节串视为合法值。
    ///
    /// 若已 [`Self::with_retry_budget`]，经 resiliencx 异步预算重试。
    pub async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let this = self.clone();
        let key = key.to_owned();
        with_automatic_budget(
            RedisOperation::Get,
            self.budget.as_deref(),
            self.budget_max_attempts,
            "redis.get",
            || {
                let this = this.clone();
                let key = key.clone();
                async move { this.get_once(&key).await }
            },
        )
        .await
    }

    /// 显式 budget 的 `GET`：以 [`RetrySafety::ReadOnly`] 驱动真实 I/O。
    pub async fn get_with_budget(
        &self,
        key: &str,
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<Option<Vec<u8>>> {
        let this = self.clone();
        let key = key.to_owned();
        route_with_budget(budget, max_attempts, RetrySafety::ReadOnly, "redis.get", || {
            let this = this.clone();
            let key = key.clone();
            async move { this.get_once(&key).await }
        })
        .await
    }

    /// `SET` / `PSETEX`。
    ///
    /// - `ttl = None`：不过期；
    /// - `ttl = Some(0)` 或 `< 1ms`：[`XError::invalid`]；
    /// - 其余：使用毫秒精度 `PSETEX`。
    ///
    /// 若已 [`Self::with_retry_budget`]，无 TTL 的固定值写入按幂等语义路由；相对 TTL 写入因
    /// 超时或断连后结果未知，在多次尝试进入 driver 前 fail-closed。
    pub async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        if let Some(budget) = self.budget.as_ref() {
            return self
                .set_with_budget(key, val, ttl, budget.as_ref(), self.budget_max_attempts)
                .await;
        }
        self.set_once(key, val, ttl).await
    }

    /// 显式 budget 的 `SET`：无 TTL 时为幂等；相对 TTL 保守视为不安全副作用。
    ///
    /// `PSETEX` 的 value + TTL 在服务端是单命令原子；若响应丢失，重试会重新开始 TTL，
    /// 因此多次尝试会在操作前被拒绝。
    pub async fn set_with_budget(
        &self,
        key: &str,
        val: Vec<u8>,
        ttl: Option<Duration>,
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<()> {
        let this = self.clone();
        let key = key.to_owned();
        let safety = set_retry_safety(ttl);
        route_with_budget(budget, max_attempts, safety, "redis.set", || {
            let this = this.clone();
            let key = key.clone();
            let val = val.clone();
            async move { this.set_once(&key, val, ttl).await }
        })
        .await
    }

    /// 单次 `DEL` I/O（无 budget 环）。
    async fn delete_once(&self, key: &str) -> XResult<bool> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let n: i64 = map_redis_result(conn.del(key).await)?;
                Ok(n > 0)
            })
            .await
    }

    /// `DEL`；返回是否删除了 key。未配置 budget 时执行一次；多次尝试的 budget 会在 driver
    /// 前被拒绝，避免重试把首次成功改写成 `false`。
    pub async fn delete(&self, key: &str) -> XResult<bool> {
        if let Some(budget) = self.budget.as_ref() {
            let this = self.clone();
            let key = key.to_owned();
            return route_with_budget(
                budget,
                self.budget_max_attempts,
                RetrySafety::UnsafeSideEffect,
                "redis.del",
                || {
                    let this = this.clone();
                    let key = key.clone();
                    async move { this.delete_once(&key).await }
                },
            )
            .await;
        }
        self.delete_once(key).await
    }

    /// 单次 `EXISTS` I/O（无 budget 环）。
    async fn exists_once(&self, key: &str) -> XResult<bool> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let n: i64 = map_redis_result(conn.exists(key).await)?;
                Ok(n > 0)
            })
            .await
    }

    /// `EXISTS`。配置 budget 时经 resiliencx 重试。
    pub async fn exists(&self, key: &str) -> XResult<bool> {
        let this = self.clone();
        let key = key.to_owned();
        with_automatic_budget(
            RedisOperation::Exists,
            self.budget.as_deref(),
            self.budget_max_attempts,
            "redis.exists",
            || {
                let this = this.clone();
                let key = key.clone();
                async move { this.exists_once(&key).await }
            },
        )
        .await
    }

    /// 单次 `PEXPIRE` I/O（无 budget 环）。
    async fn expire_once(&self, key: &str, ttl: Duration) -> XResult<bool> {
        validate_ttl(Some(ttl))?;
        let ms =
            i64::try_from(duration_to_millis(ttl)?).map_err(|_| XError::invalid("TTL 过大"))?;
        let key = key.to_owned();
        self.pool
            .with_conn(move |mut conn| async move {
                let n: i64 = map_redis_result(
                    redis::cmd("PEXPIRE").arg(&key).arg(ms).query_async(&mut conn).await,
                )?;
                Ok(n > 0)
            })
            .await
    }

    /// `PEXPIRE`；key 不存在返回 `Ok(false)`。未配置 budget 时执行一次；多次尝试的 budget
    /// 会在 driver 前被拒绝，避免重试重置 TTL 起点。
    pub async fn expire(&self, key: &str, ttl: Duration) -> XResult<bool> {
        if let Some(budget) = self.budget.as_ref() {
            let this = self.clone();
            let key = key.to_owned();
            return route_with_budget(
                budget,
                self.budget_max_attempts,
                RetrySafety::UnsafeSideEffect,
                "redis.expire",
                || {
                    let this = this.clone();
                    let key = key.clone();
                    async move { this.expire_once(&key, ttl).await }
                },
            )
            .await;
        }
        self.expire_once(key, ttl).await
    }

    /// 单次 `PTTL` I/O（无 budget 环）。
    async fn ttl_once(&self, key: &str) -> XResult<Option<Duration>> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let ms: i64 =
                    map_redis_result(redis::cmd("PTTL").arg(&key).query_async(&mut conn).await)?;
                if ms == -2 {
                    Err(XError::missing(format!("redis key 不存在: {key}")))
                } else if ms == -1 {
                    Ok(None)
                } else if ms < 0 {
                    Err(XError::internal(format!("redis PTTL 异常: {ms}")))
                } else {
                    Ok(Some(Duration::from_millis(ms as u64)))
                }
            })
            .await
    }

    /// `PTTL`：
    /// - key 不存在 → [`XError::missing`]
    /// - 无过期 → `Ok(None)`
    /// - 有过期 → `Ok(Some(duration))`
    ///
    /// 配置 budget 时经 resiliencx 重试。
    pub async fn ttl(&self, key: &str) -> XResult<Option<Duration>> {
        let this = self.clone();
        let key = key.to_owned();
        with_automatic_budget(
            RedisOperation::Ttl,
            self.budget.as_deref(),
            self.budget_max_attempts,
            "redis.ttl",
            || {
                let this = this.clone();
                let key = key.clone();
                async move { this.ttl_once(&key).await }
            },
        )
        .await
    }

    /// 单次 `MGET` I/O（无 budget 环）。
    async fn mget_once(&self, keys: &[&str]) -> XResult<Vec<Option<Vec<u8>>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let owned: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
        self.pool
            .with_conn(|mut conn| async move {
                let vals: Vec<Option<Vec<u8>>> = map_redis_result(conn.get(owned).await)?;
                Ok(vals)
            })
            .await
    }

    /// `MGET`。配置 budget 时经 resiliencx 重试。
    pub async fn mget(&self, keys: &[&str]) -> XResult<Vec<Option<Vec<u8>>>> {
        let this = self.clone();
        let owned: Vec<String> = keys.iter().map(|k| (*k).to_owned()).collect();
        with_automatic_budget(
            RedisOperation::Mget,
            self.budget.as_deref(),
            self.budget_max_attempts,
            "redis.mget",
            || {
                let this = this.clone();
                let owned = owned.clone();
                async move {
                    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
                    this.mget_once(&refs).await
                }
            },
        )
        .await
    }

    /// 单次 `MSET` I/O（无 budget 环）。
    async fn mset_once(&self, items: &[(&str, &[u8])]) -> XResult<()> {
        if items.is_empty() {
            return Ok(());
        }
        let owned: Vec<(String, Vec<u8>)> =
            items.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_vec())).collect();
        self.pool
            .with_conn(|mut conn| async move {
                let _: () = map_redis_result(conn.mset(&owned).await)?;
                Ok(())
            })
            .await
    }

    /// `MSET`（无 TTL；需要 TTL 请逐条 `set`）。配置 budget 时按幂等语义重试。
    ///
    /// 原子性只在 Standalone 或 Cluster 同一 hash slot 的单条命令边界成立；本客户端不承诺
    /// 跨 slot 原子性。
    pub async fn mset(&self, items: &[(&str, &[u8])]) -> XResult<()> {
        if let Some(budget) = self.budget.as_ref() {
            let this = self.clone();
            let owned: Vec<(String, Vec<u8>)> =
                items.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_vec())).collect();
            return route_with_budget(
                budget,
                self.budget_max_attempts,
                RetrySafety::Idempotent,
                "redis.mset",
                || {
                    let this = this.clone();
                    let owned = owned.clone();
                    async move {
                        let refs: Vec<(&str, &[u8])> =
                            owned.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
                        this.mset_once(&refs).await
                    }
                },
            )
            .await;
        }
        self.mset_once(items).await
    }
}

#[async_trait]
impl KeyValueStore for RedisClient {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        RedisClient::get(self, key).await
    }

    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        RedisClient::set(self, key, val, ttl).await
    }
}

fn validate_ttl(ttl: Option<Duration>) -> XResult<()> {
    match ttl {
        None => Ok(()),
        Some(d) if d.is_zero() => Err(XError::invalid("TTL 不能为 0（Some(0) → Invalid）")),
        Some(d) if d.as_millis() == 0 => Err(XError::invalid("TTL 过短（小于 1ms）")),
        Some(_) => Ok(()),
    }
}

async fn route_with_budget<F, Fut, T>(
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
    with_budget_async_safe_noop(budget, max_attempts, safety, op, f).await
}

fn set_retry_safety(ttl: Option<Duration>) -> RetrySafety {
    if ttl.is_some() { RetrySafety::UnsafeSideEffect } else { RetrySafety::Idempotent }
}

fn duration_to_millis(d: Duration) -> XResult<u64> {
    let ms = d.as_millis();
    if ms == 0 {
        return Err(XError::invalid("TTL 过短（小于 1ms）"));
    }
    u64::try_from(ms).map_err(|_| XError::invalid("TTL 过大"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn ttl_zero_is_invalid() {
        let err = validate_ttl(Some(Duration::ZERO)).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn ttl_sub_millis_is_invalid() {
        let err = validate_ttl(Some(Duration::from_nanos(100))).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn ttl_none_ok() {
        validate_ttl(None).unwrap();
    }

    #[test]
    fn with_retry_budget_api_shape() {
        // 离线：budget API 形状；live I/O 见 tests/live_* #[ignore]
        let budget = RetryBudget::new(2);
        assert_eq!(budget.remaining(), 2);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn set_retry_safety_is_conservative_for_relative_ttl() {
        assert_eq!(set_retry_safety(None), RetrySafety::Idempotent);
        assert_eq!(set_retry_safety(Some(Duration::from_secs(1))), RetrySafety::UnsafeSideEffect);
    }

    #[tokio::test]
    async fn get_route_zero_attempts_rejects_before_driver_future() {
        let budget = RetryBudget::new(1);
        let constructed = std::cell::Cell::new(0u32);
        let err = route_with_budget(&budget, 0, RetrySafety::ReadOnly, "redis.get", || {
            constructed.set(constructed.get() + 1);
            async { Ok::<_, XError>(Some(Vec::<u8>::new())) }
        })
        .await
        .expect_err("零次 GET 必须在 driver future 前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(constructed.get(), 0);
    }

    #[tokio::test]
    async fn set_route_zero_attempts_rejects_before_driver_future() {
        let budget = RetryBudget::new(1);
        let constructed = std::cell::Cell::new(0u32);
        let err = route_with_budget(&budget, 0, RetrySafety::Idempotent, "redis.set", || {
            constructed.set(constructed.get() + 1);
            async { Ok::<_, XError>(()) }
        })
        .await
        .expect_err("零次 SET 必须在 driver future 前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(constructed.get(), 0);
    }

    #[tokio::test]
    async fn configured_zero_attempts_get_does_not_call_driver() {
        let driver_calls = Arc::new(AtomicUsize::new(0));
        let client = RedisPool::test_probe(driver_calls.clone())
            .client()
            .with_retry_budget(RetryBudget::new(1), 0);
        let err = client.get("key").await.expect_err("零次 GET 必须提前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(driver_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn configured_zero_attempts_set_does_not_call_driver() {
        let driver_calls = Arc::new(AtomicUsize::new(0));
        let client = RedisPool::test_probe(driver_calls.clone())
            .client()
            .with_retry_budget(RetryBudget::new(1), 0);
        let err =
            client.set("key", b"value".to_vec(), None).await.expect_err("零次 SET 必须提前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(driver_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn duration_to_millis_rejects_sub_ms_and_accepts_ms() {
        let err = duration_to_millis(Duration::from_nanos(500)).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(duration_to_millis(Duration::from_millis(1)).unwrap(), 1);
        assert_eq!(duration_to_millis(Duration::from_secs(2)).unwrap(), 2000);
    }

    #[test]
    fn ttl_ok_and_probe_client_shape() {
        validate_ttl(Some(Duration::from_millis(10))).unwrap();
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        assert!(!client.has_retry_budget());
        assert!(client.endpoint().contains("测试"));
        let with_budget = client.with_retry_budget(RetryBudget::new(3), 2);
        assert!(with_budget.has_retry_budget());
        assert_eq!(with_budget.pool().stats().open, 1);
    }

    #[tokio::test]
    async fn unsafe_side_effect_ops_reject_multi_attempt_before_driver() {
        let driver_calls = Arc::new(AtomicUsize::new(0));
        let client = RedisPool::test_probe(driver_calls.clone())
            .client()
            .with_retry_budget(RetryBudget::new(5), 3);

        let del_err = client.delete("k").await.expect_err("DEL multi-attempt");
        assert_eq!(del_err.kind(), ErrorKind::Invalid);

        let exp_err =
            client.expire("k", Duration::from_secs(1)).await.expect_err("PEXPIRE multi-attempt");
        assert_eq!(exp_err.kind(), ErrorKind::Invalid);

        let set_ttl_err = client
            .set("k", b"v".to_vec(), Some(Duration::from_secs(1)))
            .await
            .expect_err("relative TTL SET multi-attempt");
        assert_eq!(set_ttl_err.kind(), ErrorKind::Invalid);

        assert_eq!(driver_calls.load(Ordering::SeqCst), 0, "不安全副作用不得进入 probe driver");
    }

    #[tokio::test]
    async fn set_with_budget_ttl_zero_rejects_before_route() {
        let driver_calls = Arc::new(AtomicUsize::new(0));
        let client = RedisPool::test_probe(driver_calls.clone()).client();
        let budget = RetryBudget::new(2);
        let err = client
            .set_with_budget("k", b"v".to_vec(), Some(Duration::ZERO), &budget, 1)
            .await
            .expect_err("ttl 0");
        // validate_ttl 在 set_once 内；budget 路由会调用闭包 → set_once → Invalid
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }
}
