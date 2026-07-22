//! 池化连接句柄 [`PgConnection`]。

use deadpool_postgres::Object;
use kernel::{XError, XResult};
use std::time::Duration;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::error::map_tokio_error;
use crate::tx::PgTransaction;

/// 从连接池借出的连接（归还由 drop 完成）。
pub struct PgConnection {
    pub(crate) client: Option<Object>,
    pub(crate) operation_timeout: Duration,
}

impl PgConnection {
    /// 包装 deadpool 对象。
    pub(crate) fn new(client: Object, operation_timeout: Duration) -> Self {
        Self { client: Some(client), operation_timeout }
    }

    fn object(&self) -> XResult<&Object> {
        self.client.as_ref().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }

    fn discard(&mut self) {
        if let Some(client) = self.client.take() {
            drop(Object::take(client));
        }
    }

    /// 参数化 `EXECUTE`，返回影响行数。
    ///
    /// # 安全
    /// 调用方必须使用 `$1..$N` 占位符；禁止字符串拼接用户输入。
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        match tokio::time::timeout(self.operation_timeout, self.object()?.execute(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres execute 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，期望恰好一行。
    pub async fn query_one(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        match tokio::time::timeout(self.operation_timeout, self.object()?.query_one(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres query_one 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，返回 0..N 行。
    pub async fn query(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        match tokio::time::timeout(self.operation_timeout, self.object()?.query(sql, params)).await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
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
        match tokio::time::timeout(self.operation_timeout, self.object()?.query_opt(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres query_opt 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 开启事务，消费本连接。
    pub async fn begin(mut self) -> XResult<PgTransaction> {
        let client =
            self.client.take().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))?;
        PgTransaction::begin(client, self.operation_timeout).await
    }

    /// 访问底层 client（高级用例）。
    pub fn client(&self) -> XResult<&Object> {
        self.object()
    }

    /// 可变访问底层 client。
    pub fn client_mut(&mut self) -> XResult<&mut Object> {
        self.client.as_mut().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }
}
