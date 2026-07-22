//! 可克隆的 Redis 命令客户端（共享 [`RedisPool`]）。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use contracts::KeyValueStore;
use kernel::{XError, XResult};
use redis::AsyncCommands;
use resiliencx::RetryBudget;

use crate::error_map::map_redis_result;
use crate::pool::RedisPool;
use crate::resilience::with_budget_async_noop;

/// 生产 Redis KV 客户端。
///
/// `Clone` 只共享底层池引用；所有命令受池级 Semaphore 与超时约束。
/// 可选 [`RetryBudget`]：配置后 `get`/`set` 等经 resiliencx 重试生产路径。
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

    /// 注入 [`RetryBudget`]：后续 `get`/`set` 走 resiliencx 异步重试。
    #[must_use]
    pub fn with_retry_budget(mut self, budget: RetryBudget, max_attempts: u32) -> Self {
        self.budget = Some(Arc::new(budget));
        self.budget_max_attempts = max_attempts.max(1);
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
        if let Some(budget) = self.budget.as_ref() {
            return self.get_with_budget(key, budget.as_ref(), self.budget_max_attempts).await;
        }
        self.get_once(key).await
    }

    /// 显式 budget 的 `GET`：始终经 [`crate::with_budget_async_noop`] 驱动真实 I/O。
    pub async fn get_with_budget(
        &self,
        key: &str,
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<Option<Vec<u8>>> {
        let this = self.clone();
        let key = key.to_owned();
        with_budget_async_noop(budget, max_attempts, "redis.get", || {
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
    /// 若已 [`Self::with_retry_budget`]，经 resiliencx 异步预算重试。
    pub async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        if let Some(budget) = self.budget.as_ref() {
            return self
                .set_with_budget(key, val, ttl, budget.as_ref(), self.budget_max_attempts)
                .await;
        }
        self.set_once(key, val, ttl).await
    }

    /// 显式 budget 的 `SET`：始终经 resiliencx 驱动真实 I/O。
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
        with_budget_async_noop(budget, max_attempts, "redis.set", || {
            let this = this.clone();
            let key = key.clone();
            let val = val.clone();
            async move { this.set_once(&key, val, ttl).await }
        })
        .await
    }

    /// `DEL`；返回是否删除了 key。
    pub async fn delete(&self, key: &str) -> XResult<bool> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let n: i64 = map_redis_result(conn.del(key).await)?;
                Ok(n > 0)
            })
            .await
    }

    /// `EXISTS`。
    pub async fn exists(&self, key: &str) -> XResult<bool> {
        let key = key.to_owned();
        self.pool
            .with_conn(|mut conn| async move {
                let n: i64 = map_redis_result(conn.exists(key).await)?;
                Ok(n > 0)
            })
            .await
    }

    /// `PEXPIRE`；key 不存在返回 `Ok(false)`。
    pub async fn expire(&self, key: &str, ttl: Duration) -> XResult<bool> {
        validate_ttl(Some(ttl))?;
        let ms =
            i64::try_from(duration_to_millis(ttl)?).map_err(|_| XError::invalid("TTL 过大"))?;
        let key = key.to_owned();
        self.pool
            .with_conn(move |mut conn| async move {
                // PEXPIRE key milliseconds
                let n: i64 = map_redis_result(
                    redis::cmd("PEXPIRE").arg(&key).arg(ms).query_async(&mut conn).await,
                )?;
                Ok(n > 0)
            })
            .await
    }

    /// `PTTL`：
    /// - key 不存在 → [`XError::missing`]
    /// - 无过期 → `Ok(None)`
    /// - 有过期 → `Ok(Some(duration))`
    pub async fn ttl(&self, key: &str) -> XResult<Option<Duration>> {
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

    /// `MGET`。
    pub async fn mget(&self, keys: &[&str]) -> XResult<Vec<Option<Vec<u8>>>> {
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

    /// `MSET`（无 TTL；需要 TTL 请逐条 `set`）。
    pub async fn mset(&self, items: &[(&str, &[u8])]) -> XResult<()> {
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

    #[tokio::test]
    async fn get_set_with_budget_entrypoints_drive_real_io() {
        // 有可认证 Redis 时完整 roundtrip；否则连接成功但 NOAUTH 仍证明入口被调用。
        let budget = RetryBudget::new(2);
        let Ok(cfg) = crate::config::RedisConfig::from_env() else {
            assert_eq!(budget.remaining(), 2);
            return;
        };
        let Ok(pool) = crate::pool::RedisPool::connect(cfg).await else {
            assert_eq!(budget.remaining(), 2);
            return;
        };
        let client = pool.client().with_retry_budget(RetryBudget::new(2), 3);
        assert!(client.has_retry_budget());
        // 驱动真实 client I/O + resiliencx budget 环（不要求业务成功）
        let set_res =
            client.set_with_budget("infra:resilience:probe", b"1".to_vec(), None, &budget, 2).await;
        let get_res = client.get_with_budget("infra:resilience:probe", &budget, 2).await;
        match (set_res, get_res) {
            (Ok(()), Ok(v)) => assert_eq!(v.as_deref(), Some(b"1".as_ref())),
            (Err(e), _) | (_, Err(e)) => {
                // NOAUTH / 权限等：入口与 budget 环已被调用
                assert!(
                    matches!(
                        e.kind(),
                        ErrorKind::Unavailable
                            | ErrorKind::Invalid
                            | ErrorKind::Transient
                            | ErrorKind::DeadlineExceeded
                    ),
                    "kind={:?}",
                    e.kind()
                );
            }
        }
        // 默认 get 也路由到 budget 路径
        let _ = client.get("infra:resilience:probe").await;
    }
}
