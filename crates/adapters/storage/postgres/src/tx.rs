//! 事务句柄与状态机。
//!
//! 状态：[`TxState::Active`] → [`TxState::Committed`] / [`TxState::RolledBack`]。
//! `Drop` 时若仍为 Active，会异步 best-effort `ROLLBACK`，避免脏连接回池。

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use deadpool_postgres::Object;
use kernel::{XError, XResult};
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::error::map_tokio_error;

/// 事务生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxState {
    /// 已 `BEGIN`，尚未终结。
    Active,
    /// 已成功 `COMMIT`。
    Committed,
    /// 已成功 `ROLLBACK`。
    RolledBack,
}

/// Postgres 事务。
///
/// 限制（诚实声明）：
/// - 使用显式 `BEGIN`/`COMMIT`/`ROLLBACK` SQL（非 `tokio_postgres::Transaction` 借用 API），
///   以便跨 `async` 边界持有连接；
/// - 终结后再次 `commit`/`rollback` 返回 [`ErrorKind::Invariant`](kernel::ErrorKind::Invariant)。
pub struct PgTransaction {
    client: Option<Object>,
    state: TxState,
    /// Drop 时是否已调度 rollback（避免重复）。
    drop_scheduled: AtomicBool,
    operation_timeout: Duration,
}

impl PgTransaction {
    /// 在已借出连接上 `BEGIN`。
    pub(crate) async fn begin(client: Object, operation_timeout: Duration) -> XResult<Self> {
        match tokio::time::timeout(operation_timeout, client.batch_execute("BEGIN")).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                drop(Object::take(client));
                return Err(map_tokio_error(error));
            }
            Err(error) => {
                drop(Object::take(client));
                return Err(
                    XError::deadline_exceeded("Postgres BEGIN 超时；连接已丢弃").with_source(error)
                );
            }
        }
        Ok(Self {
            client: Some(client),
            state: TxState::Active,
            drop_scheduled: AtomicBool::new(false),
            operation_timeout,
        })
    }

    /// 当前状态。
    #[must_use]
    pub fn state(&self) -> TxState {
        self.state
    }

    /// 是否仍可执行 SQL。
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state == TxState::Active
    }

    fn client(&self) -> XResult<&Object> {
        self.ensure_active()?;
        self.client.as_ref().ok_or_else(|| XError::invariant("事务连接已释放".to_string()))
    }

    fn ensure_active(&self) -> XResult<()> {
        match self.state {
            TxState::Active => Ok(()),
            TxState::Committed => Err(XError::invariant("事务已 COMMIT，禁止再操作".to_string())),
            TxState::RolledBack => {
                Err(XError::invariant("事务已 ROLLBACK，禁止再操作".to_string()))
            }
        }
    }

    /// 参数化 `EXECUTE`。
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        match tokio::time::timeout(self.operation_timeout, self.client()?.execute(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres 事务 execute 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询（恰好一行）。
    pub async fn query_one(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        match tokio::time::timeout(self.operation_timeout, self.client()?.query_one(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres 事务 query_one 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询（0..N 行）。
    pub async fn query(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        match tokio::time::timeout(self.operation_timeout, self.client()?.query(sql, params)).await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres 事务 query 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 可选单行。
    pub async fn query_opt(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        match tokio::time::timeout(self.operation_timeout, self.client()?.query_opt(sql, params))
            .await
        {
            Ok(result) => result.map_err(map_tokio_error),
            Err(error) => {
                self.discard();
                Err(XError::deadline_exceeded("Postgres 事务 query_opt 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 提交事务。
    pub async fn commit(mut self) -> XResult<()> {
        self.ensure_active()?;
        let client =
            self.client.take().ok_or_else(|| XError::invariant("事务连接已释放".to_string()))?;
        match tokio::time::timeout(self.operation_timeout, client.batch_execute("COMMIT")).await {
            Ok(Ok(())) => {
                self.state = TxState::Committed;
                drop(client);
                Ok(())
            }
            Ok(Err(error)) => {
                drop(Object::take(client));
                Err(map_tokio_error(error))
            }
            Err(error) => {
                drop(Object::take(client));
                Err(XError::deadline_exceeded("Postgres COMMIT 超时且结果未知；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 回滚事务。
    pub async fn rollback(mut self) -> XResult<()> {
        self.ensure_active()?;
        let client =
            self.client.take().ok_or_else(|| XError::invariant("事务连接已释放".to_string()))?;
        match tokio::time::timeout(self.operation_timeout, client.batch_execute("ROLLBACK")).await {
            Ok(Ok(())) => {
                self.state = TxState::RolledBack;
                drop(client);
                Ok(())
            }
            Ok(Err(error)) => {
                drop(Object::take(client));
                Err(map_tokio_error(error))
            }
            Err(error) => {
                drop(Object::take(client));
                Err(XError::deadline_exceeded("Postgres ROLLBACK 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    fn discard(&mut self) {
        if let Some(client) = self.client.take() {
            drop(Object::take(client));
        }
    }
}

impl Drop for PgTransaction {
    fn drop(&mut self) {
        if self.state != TxState::Active {
            return;
        }
        if self
            .drop_scheduled
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }
        if let Some(client) = self.client.take() {
            // best-effort：避免未终结事务回到连接池
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let operation_timeout = self.operation_timeout;
                handle.spawn(async move {
                    match tokio::time::timeout(operation_timeout, client.batch_execute("ROLLBACK"))
                        .await
                    {
                        Ok(Ok(())) => drop(client),
                        Ok(Err(_)) | Err(_) => drop(Object::take(client)),
                    }
                });
            } else {
                // 无 runtime 时无法安全 rollback；必须从池分离，禁止归还 open transaction。
                drop(Object::take(client));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_state_copy() {
        assert_eq!(TxState::Active, TxState::Active);
        assert_ne!(TxState::Active, TxState::Committed);
    }
}
