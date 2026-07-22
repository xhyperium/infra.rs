//! 事务句柄与状态机。
//!
//! 状态：[`TxState::Active`] → [`TxState::Committed`] / [`TxState::RolledBack`] /
//! [`TxState::Failed`]。
//! `Drop` 时若仍为 Active，会把连接永久移出池并关闭 session，由 PostgreSQL 回滚事务。

use std::time::Duration;

use deadpool_postgres::Object;
use kernel::{XError, XResult};
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::conn::PooledObjectGuard;
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
    /// 操作失败；仅持有可回滚连接时允许 `ROLLBACK`，否则连接已移出池。
    Failed,
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
    operation_timeout: Duration,
}

impl PgTransaction {
    /// 在已借出连接上 `BEGIN`。
    pub(crate) async fn begin(client: Object, operation_timeout: Duration) -> XResult<Self> {
        let guard = PooledObjectGuard::new(client);
        match tokio::time::timeout(operation_timeout, guard.object()?.batch_execute("BEGIN")).await
        {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(map_tokio_error(error)),
            Err(error) => {
                return Err(
                    XError::deadline_exceeded("Postgres BEGIN 超时；连接已丢弃").with_source(error)
                );
            }
        }
        let client = guard.release()?;
        Ok(Self { client: Some(client), state: TxState::Active, operation_timeout })
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

    fn take_guard(&mut self) -> XResult<PooledObjectGuard> {
        self.ensure_active()?;
        self.take_guard_after_validation()
    }

    fn take_guard_after_validation(&mut self) -> XResult<PooledObjectGuard> {
        let guard = self
            .client
            .take()
            .map(PooledObjectGuard::new)
            .ok_or_else(|| XError::invariant("事务连接已释放"))?;
        // 先进入失败态，再把连接交给可被取消的 await。只有操作明确完成且连接
        // 恢复到事务句柄后才回到 Active；future drop/任务 abort 会保留 Failed。
        self.state = TxState::Failed;
        Ok(guard)
    }

    fn ensure_active(&self) -> XResult<()> {
        match self.state {
            TxState::Active => Ok(()),
            TxState::Committed => Err(XError::invariant("事务已 COMMIT，禁止再操作".to_string())),
            TxState::RolledBack => {
                Err(XError::invariant("事务已 ROLLBACK，禁止再操作".to_string()))
            }
            TxState::Failed => Err(XError::invariant("事务已失败，仅允许在连接可用时 ROLLBACK")),
        }
    }

    fn ensure_rollbackable(&self) -> XResult<()> {
        match self.state {
            TxState::Active | TxState::Failed => Ok(()),
            TxState::Committed => {
                Err(XError::invariant("事务已 COMMIT，禁止再 ROLLBACK".to_string()))
            }
            TxState::RolledBack => {
                Err(XError::invariant("事务已 ROLLBACK，禁止重复操作".to_string()))
            }
        }
    }

    /// 参数化 `EXECUTE`。
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.execute(sql, params))
            .await
        {
            Ok(Ok(affected)) => {
                self.client = Some(guard.release()?);
                self.state = TxState::Active;
                Ok(affected)
            }
            Ok(Err(error)) => {
                self.client = Some(guard.release()?);
                Err(map_tokio_error(error))
            }
            Err(error) => Err(XError::deadline_exceeded("Postgres 事务 execute 超时；连接已丢弃")
                .with_source(error)),
        }
    }

    /// 参数化查询（恰好一行）。
    pub async fn query_one(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_one(sql, params))
            .await
        {
            Ok(Ok(row)) => {
                self.client = Some(guard.release()?);
                self.state = TxState::Active;
                Ok(row)
            }
            Ok(Err(error)) => {
                self.client = Some(guard.release()?);
                Err(map_tokio_error(error))
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres 事务 query_one 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询（0..N 行）。
    pub async fn query(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query(sql, params)).await
        {
            Ok(Ok(rows)) => {
                self.client = Some(guard.release()?);
                self.state = TxState::Active;
                Ok(rows)
            }
            Ok(Err(error)) => {
                self.client = Some(guard.release()?);
                Err(map_tokio_error(error))
            }
            Err(error) => Err(XError::deadline_exceeded("Postgres 事务 query 超时；连接已丢弃")
                .with_source(error)),
        }
    }

    /// 可选单行。
    pub async fn query_opt(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_opt(sql, params))
            .await
        {
            Ok(Ok(row)) => {
                self.client = Some(guard.release()?);
                self.state = TxState::Active;
                Ok(row)
            }
            Ok(Err(error)) => {
                self.client = Some(guard.release()?);
                Err(map_tokio_error(error))
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres 事务 query_opt 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 提交事务。
    pub async fn commit(mut self) -> XResult<()> {
        self.ensure_active()?;
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.batch_execute("COMMIT"))
            .await
        {
            Ok(Ok(())) => {
                self.state = TxState::Committed;
                drop(guard.release()?);
                Ok(())
            }
            Ok(Err(error)) => {
                Err(XError::unavailable("Postgres COMMIT 失败且结果未知；连接已丢弃")
                    .with_source(error))
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres COMMIT 超时且结果未知；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 回滚事务。
    pub async fn rollback(mut self) -> XResult<()> {
        self.ensure_rollbackable()?;
        let guard = self.take_guard_after_validation()?;
        match tokio::time::timeout(
            self.operation_timeout,
            guard.object()?.batch_execute("ROLLBACK"),
        )
        .await
        {
            Ok(Ok(())) => {
                self.state = TxState::RolledBack;
                drop(guard.release()?);
                Ok(())
            }
            Ok(Err(error)) => {
                Err(XError::unavailable("Postgres ROLLBACK 失败；连接已丢弃").with_source(error))
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres ROLLBACK 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }
}

impl Drop for PgTransaction {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            // Active/Failed 均可能仍持有 open/aborted transaction。Drop 不能监督异步
            // rollback；直接永久脱离池并关闭 session。禁止 fire-and-forget 任务。
            drop(Object::take(client));
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
