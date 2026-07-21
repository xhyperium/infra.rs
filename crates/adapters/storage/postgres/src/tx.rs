//! 事务句柄与状态机。
//!
//! 状态：[`TxState::Active`] → [`TxState::Committed`] / [`TxState::RolledBack`]。
//! `Drop` 时若仍为 Active，会异步 best-effort `ROLLBACK`，避免脏连接回池。

use std::sync::atomic::{AtomicBool, Ordering};

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
}

impl PgTransaction {
    /// 在已借出连接上 `BEGIN`。
    pub(crate) async fn begin(client: Object) -> XResult<Self> {
        client.batch_execute("BEGIN").await.map_err(map_tokio_error)?;
        Ok(Self {
            client: Some(client),
            state: TxState::Active,
            drop_scheduled: AtomicBool::new(false),
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
    pub async fn execute(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        self.client()?.execute(sql, params).await.map_err(map_tokio_error)
    }

    /// 参数化查询（恰好一行）。
    pub async fn query_one(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        self.client()?.query_one(sql, params).await.map_err(map_tokio_error)
    }

    /// 参数化查询（0..N 行）。
    pub async fn query(&self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        self.client()?.query(sql, params).await.map_err(map_tokio_error)
    }

    /// 可选单行。
    pub async fn query_opt(
        &self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        self.client()?.query_opt(sql, params).await.map_err(map_tokio_error)
    }

    /// 提交事务。
    pub async fn commit(mut self) -> XResult<()> {
        self.ensure_active()?;
        let client =
            self.client.take().ok_or_else(|| XError::invariant("事务连接已释放".to_string()))?;
        client.batch_execute("COMMIT").await.map_err(map_tokio_error)?;
        self.state = TxState::Committed;
        // 连接在 drop client 时归还池
        drop(client);
        Ok(())
    }

    /// 回滚事务。
    pub async fn rollback(mut self) -> XResult<()> {
        self.ensure_active()?;
        let client =
            self.client.take().ok_or_else(|| XError::invariant("事务连接已释放".to_string()))?;
        client.batch_execute("ROLLBACK").await.map_err(map_tokio_error)?;
        self.state = TxState::RolledBack;
        drop(client);
        Ok(())
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
                handle.spawn(async move {
                    let _ = client.batch_execute("ROLLBACK").await;
                });
            }
            // 无 runtime 时连接 drop 可能导致后端 abort session；仍优于泄漏 open tx
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
