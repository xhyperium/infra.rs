//! [`PostgresPool`]：生产默认入口（connect / SQL / 事务 / health / stats / close）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kernel::{ErrorKind, XError, XResult};
use tokio_postgres::NoTls;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::config::{PostgresConfig, SslMode};
use crate::conn::PgConnection;
use crate::error::{
    TransactionRollbackFailure, map_create_pool_error, map_pool_error, map_tokio_error,
};
use crate::tls::MakeRustlsConnect;
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
///
/// TLS：`SslMode::Disable` → `NoTls`；`Prefer` / `Require` → rustls（webpki-roots）。
#[derive(Clone)]
pub struct PostgresPool {
    inner: deadpool_postgres::Pool,
    legacy_inner: deadpool_postgres::Pool,
    closed: Arc<AtomicBool>,
    config_summary: Arc<String>,
    acquire_timeout: std::time::Duration,
    operation_timeout: std::time::Duration,
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

        let dp_cfg = config.to_deadpool_config();
        let (pool, legacy_inner) = match config.sslmode {
            SslMode::Disable => (
                dp_cfg
                    .clone()
                    .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
                    .map_err(map_create_pool_error)?,
                dp_cfg
                    .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
                    .map_err(map_create_pool_error)?,
            ),
            SslMode::Prefer | SslMode::Require => {
                let tls = MakeRustlsConnect::with_webpki_roots()?;
                (
                    dp_cfg
                        .clone()
                        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tls.clone())
                        .map_err(map_create_pool_error)?,
                    dp_cfg
                        .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tls)
                        .map_err(map_create_pool_error)?,
                )
            }
        };
        // 旧 inner() 只保留源码兼容，永不暴露正式池。关闭的隔离池保证所有旧 I/O
        // 明确 fail-closed，不允许绕过 deadline、回收策略或连接污染防护。
        legacy_inner.close();

        let this = Self {
            inner: pool,
            legacy_inner,
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
            acquire_timeout: config.acquire_timeout,
            operation_timeout: config.operation_timeout,
        };

        // 冒烟：借一条连接跑 SELECT 1
        this.health().await?;
        Ok(this)
    }

    /// 从环境变量建池（`DATABASE_URL` 或 `FOUNDATIONX_POSTGRESX_*`）。
    pub async fn connect_from_env() -> XResult<Self> {
        let cfg = PostgresConfig::from_env()?;
        Self::connect(&cfg).await
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.closed.load(Ordering::Acquire) {
            Err(XError::unavailable("Postgres 连接池已关闭".to_string()))
        } else {
            Ok(())
        }
    }

    /// 借出连接。
    pub async fn acquire(&self) -> XResult<PgConnection> {
        self.ensure_open()?;
        let client = tokio::time::timeout(self.acquire_timeout, self.inner.get())
            .await
            .map_err(|error| XError::deadline_exceeded("Postgres acquire 超时").with_source(error))?
            .map_err(map_pool_error)?;
        Ok(PgConnection::new(client, self.operation_timeout))
    }

    /// 参数化 `EXECUTE`（短借连接）。
    pub async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        let mut conn = self.acquire().await?;
        conn.execute(sql, params).await
    }

    /// 参数化查询，恰好一行。
    pub async fn query_one(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        let mut conn = self.acquire().await?;
        conn.query_one(sql, params).await
    }

    /// 参数化查询，0..N 行。
    pub async fn query(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        let mut conn = self.acquire().await?;
        conn.query(sql, params).await
    }

    /// 可选单行。
    pub async fn query_opt(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        let mut conn = self.acquire().await?;
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
        F: for<'a> AsyncFnOnce(&'a mut PgTransaction) -> XResult<T>,
    {
        let conn = self.acquire().await?;
        let mut tx = conn.begin().await?;
        match f(&mut tx).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                // 业务失败：尽力 rollback；双错误仍保留原分类与完整 source chain。
                match tx.rollback().await {
                    Ok(()) => Err(err),
                    Err(rollback) => Err(with_rollback_failure(err, rollback)),
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
        let mut conn = self.acquire().await?;
        let row = conn.query_one("SELECT 1", &[]).await?;
        let v: i32 = row.try_get(0).map_err(map_tokio_error)?;
        if v != 1 {
            return Err(XError::unavailable(format!("健康检查异常结果: {v}")));
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

    /// 底层 deadpool 池的迁移兼容面。
    ///
    /// 返回独立且已关闭的隔离池；任何 `get` 都明确返回 `PoolError::Closed`，绝不暴露
    /// 正式池。该方法仅保证一个迁移周期内旧调用点仍可编译，请迁移到 [`Self::acquire`]
    /// 或参数化短借方法。
    #[deprecated(note = "请使用 PostgresPool::acquire 与受 deadline 保护的 SQL API")]
    // 兼容方法故意不返回同名正式池；关闭的隔离池是该迁移面的 fail-closed 合同。
    #[allow(clippy::misnamed_getters)]
    #[must_use]
    pub fn inner(&self) -> &deadpool_postgres::Pool {
        &self.legacy_inner
    }
}

fn with_rollback_failure(original: XError, rollback: XError) -> XError {
    let context = format!("{}；此外 rollback 失败", original.context());
    let wrapped = match original.kind() {
        ErrorKind::Invalid => XError::invalid(context),
        ErrorKind::Missing => XError::missing(context),
        ErrorKind::Conflict => XError::conflict(context),
        ErrorKind::Transient => original.retry_after().map_or_else(
            || XError::transient(context.clone()),
            |delay| XError::transient_after(context.clone(), delay),
        ),
        ErrorKind::Unavailable => XError::unavailable(context),
        ErrorKind::Cancelled => XError::cancelled(context),
        ErrorKind::DeadlineExceeded => XError::deadline_exceeded(context),
        ErrorKind::Invariant => XError::invariant(context),
        ErrorKind::Internal => XError::internal(context),
        _ => XError::internal(context),
    };
    wrapped.with_source(TransactionRollbackFailure::new(original, rollback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PostgresConfig;
    use crate::error::TransactionRollbackFailure;
    use crate::tls::MakeRustlsConnect;
    use kernel::ErrorKind;
    use std::error::Error;
    use std::io;
    use std::time::Duration;

    #[test]
    fn require_ssl_builds_rustls_connector() {
        // 不再拒绝 Require；TLS 连接器可构造
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .database("db")
            .user("u")
            .sslmode(SslMode::Require)
            .build()
            .unwrap();
        assert_eq!(cfg.sslmode, SslMode::Require);
        let tls = MakeRustlsConnect::with_webpki_roots().expect("tls");
        let _ = tls;
    }

    #[test]
    fn rollback_wrapper_preserves_kind_and_downcastable_branches() {
        let original = XError::deadline_exceeded("业务超时")
            .with_source(io::Error::new(io::ErrorKind::TimedOut, "业务 source"));
        let rollback = XError::unavailable("回滚断连")
            .with_source(io::Error::new(io::ErrorKind::ConnectionReset, "回滚 source"));
        let wrapped = with_rollback_failure(original, rollback);

        assert_eq!(wrapped.kind(), ErrorKind::DeadlineExceeded);
        let composite = Error::source(&wrapped)
            .and_then(|source| source.downcast_ref::<TransactionRollbackFailure>())
            .expect("外层 source 必须可 downcast 为结构化双失败");
        assert_eq!(composite.original().kind(), ErrorKind::DeadlineExceeded);
        assert_eq!(composite.rollback().kind(), ErrorKind::Unavailable);
        assert!(Error::source(composite.original()).is_some());
        assert!(Error::source(composite.rollback()).is_some());
    }

    #[tokio::test]
    async fn connect_refused_returns_error() {
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .port(1)
            .database("x")
            .user("x")
            .password(String::new())
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
            Err(_) => panic!("PostgresPool::connect 必须受内部截止时间约束"),
        }
    }

    #[tokio::test]
    async fn require_ssl_connect_refused_still_errors() {
        // Require 不再在建池前 Invalid 拒绝；连不上时返回 Unavailable/超时
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .port(1)
            .database("x")
            .user("x")
            .password(String::new())
            .sslmode(SslMode::Require)
            .connect_timeout(Duration::from_millis(300))
            .build()
            .expect("cfg");
        let res = tokio::time::timeout(Duration::from_secs(3), PostgresPool::connect(&cfg)).await;
        match res {
            Ok(Err(err)) => {
                assert_ne!(
                    err.kind(),
                    ErrorKind::Invalid,
                    "Require 不应再因缺 TLS 驱动返回 Invalid: {err}"
                );
            }
            Ok(Ok(_)) => panic!("unexpected success"),
            Err(_) => panic!("PostgresPool::connect TLS 路径必须受内部截止时间约束"),
        }
    }
}
