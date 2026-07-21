//! 池化连接句柄 [`PgConnection`]。

use deadpool_postgres::Object;
use kernel::XResult;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::error::map_tokio_error;
use crate::tx::PgTransaction;

/// 从连接池借出的连接（归还由 drop 完成）。
pub struct PgConnection {
    pub(crate) client: Object,
}

impl PgConnection {
    /// 包装 deadpool 对象。
    pub(crate) fn new(client: Object) -> Self {
        Self { client }
    }

    /// 参数化 `EXECUTE`，返回影响行数。
    ///
    /// # 安全
    /// 调用方必须使用 `$1..$N` 占位符；禁止字符串拼接用户输入。
    pub async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        self.client.execute(sql, params).await.map_err(map_tokio_error)
    }

    /// 参数化查询，期望恰好一行。
    pub async fn query_one(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        self.client.query_one(sql, params).await.map_err(map_tokio_error)
    }

    /// 参数化查询，返回 0..N 行。
    pub async fn query(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        self.client.query(sql, params).await.map_err(map_tokio_error)
    }

    /// 可选单行：0 行 → `Ok(None)`，>1 行 → 错误。
    pub async fn query_opt(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        self.client.query_opt(sql, params).await.map_err(map_tokio_error)
    }

    /// 开启事务，消费本连接。
    pub async fn begin(self) -> XResult<PgTransaction> {
        PgTransaction::begin(self.client).await
    }

    /// 访问底层 client（高级用例）。
    pub fn client(&self) -> &Object {
        &self.client
    }

    /// 可变访问底层 client。
    pub fn client_mut(&mut self) -> &mut Object {
        &mut self.client
    }
}
