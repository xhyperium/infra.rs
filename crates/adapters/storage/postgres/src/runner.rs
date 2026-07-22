//! `contracts::TxRunner` 适配（真实 BEGIN/COMMIT/ROLLBACK 边界）。
//!
//! # 诚实限制
//!
//! [`contracts::TxContext`] 只暴露 `commit` / `rollback`，**不**传递 SQL 句柄。
//! 因此本适配器保证的是**事务生命周期边界**可被 `run_tx_lifecycle` 驱动；
//! 若要在同一事务内执行业务 SQL，请使用 [`crate::PostgresPool::with_transaction`]
//! 或 [`crate::PgTransaction`]。

use std::sync::Arc;

use async_trait::async_trait;
use contracts::{TxContext, TxRunner};
use kernel::{XError, XResult};

use crate::pool::PostgresPool;
use crate::tx::{PgTransaction, TxState};

/// 基于 [`PostgresPool`] 的 [`TxRunner`]。
#[derive(Clone)]
pub struct PgTxRunner {
    pool: Arc<PostgresPool>,
}

impl PgTxRunner {
    /// 从池构造。
    #[must_use]
    pub fn new(pool: Arc<PostgresPool>) -> Self {
        Self { pool }
    }

    /// 共享池引用。
    #[must_use]
    pub fn pool(&self) -> &Arc<PostgresPool> {
        &self.pool
    }
}

/// 将 [`PgTransaction`] 适配为 `dyn TxContext`。
///
/// 仅边界语义；不暴露 SQL。
struct PgTxContext {
    inner: Option<PgTransaction>,
    state: TxState,
}

impl PgTxContext {
    fn new(tx: PgTransaction) -> Self {
        Self { inner: Some(tx), state: TxState::Active }
    }
}

#[async_trait]
impl TxContext for PgTxContext {
    async fn commit(&mut self) -> XResult<()> {
        match self.state {
            TxState::Committed => {
                return Err(XError::invariant("TxContext 已 commit".to_string()));
            }
            TxState::RolledBack => {
                return Err(XError::invariant("TxContext 已 rollback，无法 commit".to_string()));
            }
            TxState::Failed => {
                return Err(XError::unavailable("TxContext 已失败，无法 commit"));
            }
            TxState::Active => {}
        }
        let tx = self
            .inner
            .take()
            .ok_or_else(|| XError::invariant("TxContext 无底层事务".to_string()))?;
        self.state = TxState::Failed;
        tx.commit().await?;
        self.state = TxState::Committed;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        match self.state {
            TxState::RolledBack => {
                return Err(XError::invariant("TxContext 已 rollback".to_string()));
            }
            TxState::Committed => {
                return Err(XError::invariant("TxContext 已 commit，无法 rollback".to_string()));
            }
            TxState::Failed => {
                return Err(XError::unavailable("TxContext 已失败且无可用事务句柄"));
            }
            TxState::Active => {}
        }
        let tx = self
            .inner
            .take()
            .ok_or_else(|| XError::invariant("TxContext 无底层事务".to_string()))?;
        self.state = TxState::Failed;
        tx.rollback().await?;
        self.state = TxState::RolledBack;
        Ok(())
    }
}

#[async_trait]
impl TxRunner for PgTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgTxContext::new(tx)))
    }
}
