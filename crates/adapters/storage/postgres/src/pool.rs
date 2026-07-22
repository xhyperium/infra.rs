//! [`PostgresPool`]：生产默认入口（connect / SQL / 事务 / health / stats / close）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kernel::{XError, XResult};
use resiliencx::RetryBudget;
use tokio_postgres::NoTls;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::config::{PostgresConfig, SslMode};
use crate::conn::PgConnection;
use crate::error::{map_create_pool_error, map_pool_error, map_tokio_error};
use crate::resilience::with_budget_async_noop;
use crate::tx::PgTransaction;

/// 连接池快照统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolStats {
    /// 配置的最大连接数。
    pub max_size: usize,
    /// 当前池内连接数。
    pub size: usize,
    /// 当前空闲可借连接数。
    pub available: usize,
    /// 等待获取连接的任务数。
    pub waiting: usize,
    /// 是否已 `close`。
    pub closed: bool,
}

/// 生产 Postgres 连接池。
///
/// 默认导出；所有 SQL 路径均为参数化（`ToSql` + `$N`）。
/// 可选 [`RetryBudget`]：配置后 `execute`/`query` 经 resiliencx 重试生产路径。
#[derive(Clone)]
pub struct PostgresPool {
    inner: deadpool_postgres::Pool,
    closed: Arc<AtomicBool>,
    config_summary: Arc<String>,
    budget: Option<Arc<RetryBudget>>,
    budget_max_attempts: u32,
}

impl std::fmt::Debug for PostgresPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresPool")
            .field("closed", &self.closed.load(Ordering::Relaxed))
            .field("summary", &self.config_summary)
            .field("stats", &self.stats())
            .finish()
    }
}

impl PostgresPool {
    /// 按配置建池并验证至少能借出连接。
    pub async fn connect(config: &PostgresConfig) -> XResult<Self> {
        config.validate()?;
        Self::ensure_tls_supported(config.sslmode)?;

        let dp_cfg = config.to_deadpool_config();
        let pool = dp_cfg
            .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
            .map_err(map_create_pool_error)?;

        let this = Self {
            inner: pool,
            closed: Arc::new(AtomicBool::new(false)),
            config_summary: Arc::new(format!(
                "{}:{}/{} user={} sslmode={} pool={}",
                config.host,
                config.port,
                config.database,
                config.user,
                config.sslmode.as_str(),
                config.max_pool_size
            )),
            budget: None,
            budget_max_attempts: 3,
        };

        // 冒烟：借一条连接跑 SELECT 1
        this.health().await?;
        Ok(this)
    }

    /// 注入 [`RetryBudget`]：后续 `execute`/`query` 走 resiliencx 异步重试。
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

    /// 从环境变量建池（`DATABASE_URL` 或 `FOUNDATIONX_POSTGRESX_*`）。
    pub async fn connect_from_env() -> XResult<Self> {
        let cfg = PostgresConfig::from_env()?;
        Self::connect(&cfg).await
    }

    fn ensure_tls_supported(mode: SslMode) -> XResult<()> {
        match mode {
            SslMode::Disable => Ok(()),
            SslMode::Prefer => {
                // 当前构建仅 NoTls：prefer 降级为 disable
                Ok(())
            }
            SslMode::Require => Err(XError::invalid(
                "sslmode=require 需要 TLS 驱动；当前 postgresx 构建仅支持 NoTls（disable）"
                    .to_string(),
            )),
        }
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.closed.load(Ordering::Acquire) {
            Err(XError::unavailable("PostgresPool 已 close".to_string()))
        } else {
            Ok(())
        }
    }

    /// 借出连接。
    pub async fn acquire(&self) -> XResult<PgConnection> {
        self.ensure_open()?;
        let client = self.inner.get().await.map_err(map_pool_error)?;
        Ok(PgConnection::new(client))
    }

    /// 单次 `EXECUTE` I/O（无 budget 环）。
    async fn execute_once(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        let conn = self.acquire().await?;
        conn.execute(sql, params).await
    }

    /// 参数化 `EXECUTE`（短借连接）。
    ///
    /// 若已 [`Self::with_retry_budget`]，经 resiliencx 异步预算重试。
    pub async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        if let Some(budget) = self.budget.as_ref() {
            return self
                .execute_with_budget(sql, params, budget.as_ref(), self.budget_max_attempts)
                .await;
        }
        self.execute_once(sql, params).await
    }

    /// 显式 budget 的 `EXECUTE`：始终经 resiliencx 驱动真实 I/O。
    ///
    /// `params` 在整个重试环中被借用，调用方须保证其存活。
    pub async fn execute_with_budget(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<u64> {
        with_budget_async_noop(budget, max_attempts, "pg.execute", || async {
            self.execute_once(sql, params).await
        })
        .await
    }

    /// 参数化查询，恰好一行。
    pub async fn query_one(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        if let Some(budget) = self.budget.as_ref() {
            return self
                .query_one_with_budget(sql, params, budget.as_ref(), self.budget_max_attempts)
                .await;
        }
        let conn = self.acquire().await?;
        conn.query_one(sql, params).await
    }

    /// 显式 budget 的 `query_one`。
    pub async fn query_one_with_budget(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<Row> {
        with_budget_async_noop(budget, max_attempts, "pg.query_one", || async {
            let conn = self.acquire().await?;
            conn.query_one(sql, params).await
        })
        .await
    }

    /// 参数化查询，0..N 行。
    pub async fn query(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        if let Some(budget) = self.budget.as_ref() {
            return self
                .query_with_budget(sql, params, budget.as_ref(), self.budget_max_attempts)
                .await;
        }
        let conn = self.acquire().await?;
        conn.query(sql, params).await
    }

    /// 显式 budget 的 `query`。
    pub async fn query_with_budget(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        budget: &RetryBudget,
        max_attempts: u32,
    ) -> XResult<Vec<Row>> {
        with_budget_async_noop(budget, max_attempts, "pg.query", || async {
            let conn = self.acquire().await?;
            conn.query(sql, params).await
        })
        .await
    }

    /// 可选单行。
    pub async fn query_opt(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        let conn = self.acquire().await?;
        conn.query_opt(sql, params).await
    }

    /// 在事务中执行异步闭包：`Ok` → commit，`Err` → rollback。
    ///
    /// 闭包获得 [`PgTransaction`]，可在同一事务内执行多条参数化 SQL。
    ///
    /// 需要 **async 闭包**（Rust 1.85+ `AsyncFnOnce`）：
    ///
    /// ```ignore
    /// pool.with_transaction(async |tx| {
    ///     tx.execute("INSERT INTO t (id) VALUES ($1)", &[&1i32]).await?;
    ///     Ok::<_, kernel::XError>(())
    /// }).await?;
    /// ```
    pub async fn with_transaction<F, T>(&self, f: F) -> XResult<T>
    where
        F: for<'a> AsyncFnOnce(&'a PgTransaction) -> XResult<T>,
    {
        let conn = self.acquire().await?;
        let tx = conn.begin().await?;
        match f(&tx).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                // 业务失败：尽力 rollback，保留原错误
                match tx.rollback().await {
                    Ok(()) => Err(err),
                    Err(rb) => Err(XError::internal(format!(
                        "事务业务错误且 rollback 失败: business={err}; rollback={rb}"
                    ))),
                }
            }
        }
    }

    /// 显式开启事务（调用方负责 commit/rollback）。
    pub async fn begin(&self) -> XResult<PgTransaction> {
        let conn = self.acquire().await?;
        conn.begin().await
    }

    /// 健康检查：`SELECT 1`。
    pub async fn health(&self) -> XResult<()> {
        self.ensure_open()?;
        let conn = self.acquire().await?;
        let row = conn
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| XError::unavailable(format!("health 检查失败: {e}")))?;
        let v: i32 = row.try_get(0).map_err(map_tokio_error)?;
        if v != 1 {
            return Err(XError::unavailable(format!("health 检查异常结果: {v}")));
        }
        Ok(())
    }

    /// 池统计快照。
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        let st = self.inner.status();
        PoolStats {
            max_size: st.max_size,
            size: st.size,
            available: st.available,
            waiting: st.waiting,
            closed: self.closed.load(Ordering::Relaxed),
        }
    }

    /// 关闭池；此后 `acquire` / SQL 返回 Unavailable。
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.inner.close();
    }

    /// 配置摘要（无密码）。
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.config_summary
    }

    /// 底层 deadpool 池（高级用例）。
    #[must_use]
    pub fn inner(&self) -> &deadpool_postgres::Pool {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PostgresConfig;
    use kernel::ErrorKind;
    use std::time::Duration;

    #[test]
    fn require_ssl_rejected() {
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .database("db")
            .user("u")
            .sslmode(SslMode::Require)
            .build()
            .unwrap();
        let err = PostgresPool::ensure_tls_supported(cfg.sslmode).unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn connect_refused_returns_error() {
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .port(1)
            .database("x")
            .user("x")
            .password("x")
            .sslmode(SslMode::Disable)
            .connect_timeout(Duration::from_millis(300))
            .build()
            .expect("cfg");
        let res = tokio::time::timeout(Duration::from_secs(3), PostgresPool::connect(&cfg)).await;
        match res {
            Ok(Err(err)) => {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
                    ),
                    "kind={:?}",
                    err.kind()
                );
            }
            Ok(Ok(_)) => panic!("unexpected success"),
            Err(_) => {}
        }
    }

    #[tokio::test]
    async fn execute_with_budget_live_or_offline_budget_api() {
        let budget = RetryBudget::new(2);
        if let Ok(cfg) = PostgresConfig::from_env() {
            if let Ok(pool) = PostgresPool::connect(&cfg).await {
                let pool = pool.with_retry_budget(RetryBudget::new(2), 3);
                assert!(pool.has_retry_budget());
                // 真实 I/O + budget 环
                let n = pool
                    .execute_with_budget("SELECT 1", &[], &budget, 2)
                    .await
                    .expect("execute_with_budget");
                assert!(n == 0 || n == 1);
                let rows = pool.query_with_budget("SELECT 1 AS x", &[], &budget, 2).await.unwrap();
                assert!(!rows.is_empty());
                return;
            }
        }
        assert_eq!(budget.remaining(), 2);
    }
}
