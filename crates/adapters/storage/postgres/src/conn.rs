//! 池化连接句柄 [`PgConnection`]。

use deadpool_postgres::Object;
use kernel::{XError, XResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::error::map_tokio_error;
use crate::tx::PgTransaction;

/// 从连接池借出的连接（归还由 drop 完成）。
pub struct PgConnection {
    pub(crate) client: Option<Object>,
    pub(crate) operation_timeout: Duration,
    raw_exposed: AtomicBool,
}

/// 池对象取消守卫。
///
/// 只要异步操作未明确调用 release，Drop 就把连接从 deadpool 分离。
/// 因此外层 timeout、任务 abort 或 future drop 都不会把未知状态连接归池。
pub(crate) struct PooledObjectGuard {
    object: Option<Object>,
}

impl PooledObjectGuard {
    pub(crate) fn new(object: Object) -> Self {
        Self { object: Some(object) }
    }

    pub(crate) fn object(&self) -> XResult<&Object> {
        self.object.as_ref().ok_or_else(|| XError::invariant("Postgres 连接守卫为空"))
    }

    pub(crate) fn release(mut self) -> XResult<Object> {
        self.object.take().ok_or_else(|| XError::invariant("Postgres 连接守卫重复释放"))
    }
}

impl Drop for PooledObjectGuard {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            drop(Object::take(object));
        }
    }
}

impl PgConnection {
    /// 包装 deadpool 对象。
    pub(crate) fn new(client: Object, operation_timeout: Duration) -> Self {
        Self { client: Some(client), operation_timeout, raw_exposed: AtomicBool::new(false) }
    }

    fn take_guard(&mut self) -> XResult<PooledObjectGuard> {
        self.client
            .take()
            .map(PooledObjectGuard::new)
            .ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }

    /// 参数化 `EXECUTE`，返回影响行数。
    ///
    /// # 安全
    /// 调用方必须使用 `$1..$N` 占位符；禁止字符串拼接用户输入。
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.execute(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres execute 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，期望恰好一行。
    pub async fn query_one(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_one(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query_one 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，返回 0..N 行。
    pub async fn query(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query(sql, params)).await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query 超时；连接已丢弃").with_source(error))
            }
        }
    }

    /// 可选单行：0 行 → `Ok(None)`，>1 行 → 错误。
    pub async fn query_opt(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_opt(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query_opt 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 开启事务，消费本连接。
    pub async fn begin(mut self) -> XResult<PgTransaction> {
        if self.raw_exposed.load(Ordering::Acquire) {
            return Err(XError::unavailable(
                "连接曾通过 deprecated 原始 client API 暴露；已隔离，禁止开启事务",
            ));
        }
        let client =
            self.client.take().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))?;
        PgTransaction::begin(client, self.operation_timeout).await
    }

    /// 访问底层 client 的迁移兼容面。
    ///
    /// 调用后连接会被标记为污染并在 Drop 时永久移出池；调用方仍须自行约束原始操作
    /// deadline。请迁移到本类型的参数化 SQL API。
    #[deprecated(note = "请使用 PgConnection 的参数化 SQL API；原始 client 会强制脱池")]
    pub fn client(&self) -> XResult<&Object> {
        self.raw_exposed.store(true, Ordering::Release);
        self.client.as_ref().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }

    /// 可变访问底层 client 的迁移兼容面。
    ///
    /// 调用后连接会被标记为污染并在 Drop 时永久移出池。
    #[deprecated(note = "请使用 PgConnection 的参数化 SQL API；原始 client 会强制脱池")]
    pub fn client_mut(&mut self) -> XResult<&mut Object> {
        self.raw_exposed.store(true, Ordering::Release);
        self.client.as_mut().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }
}

impl Drop for PgConnection {
    fn drop(&mut self) {
        if self.raw_exposed.load(Ordering::Acquire)
            && let Some(client) = self.client.take()
        {
            drop(Object::take(client));
        }
    }
}
