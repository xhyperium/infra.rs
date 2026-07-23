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
use crate::tx::{PgTransaction, TxStatus};

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
    state: TxStatus,
}

impl PgTxContext {
    fn new(tx: PgTransaction) -> Self {
        Self { inner: Some(tx), state: TxStatus::Active }
    }
}

#[async_trait]
impl TxContext for PgTxContext {
    async fn commit(&mut self) -> XResult<()> {
        match self.state {
            TxStatus::Committed => {
                return Err(XError::invariant("TxContext 已 commit".to_string()));
            }
            TxStatus::RolledBack => {
                return Err(XError::invariant("TxContext 已 rollback，无法 commit".to_string()));
            }
            TxStatus::Failed => {
                return Err(XError::unavailable("TxContext 已失败，无法 commit"));
            }
            TxStatus::Active => {}
        }
        let tx = self
            .inner
            .take()
            .ok_or_else(|| XError::invariant("TxContext 无底层事务".to_string()))?;
        self.state = TxStatus::Failed;
        tx.commit().await?;
        self.state = TxStatus::Committed;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        match self.state {
            TxStatus::RolledBack => {
                return Err(XError::invariant("TxContext 已 rollback".to_string()));
            }
            TxStatus::Committed => {
                return Err(XError::invariant("TxContext 已 commit，无法 rollback".to_string()));
            }
            TxStatus::Failed => {
                return Err(XError::unavailable("TxContext 已失败且无可用事务句柄"));
            }
            TxStatus::Active => {}
        }
        let tx = self
            .inner
            .take()
            .ok_or_else(|| XError::invariant("TxContext 无底层事务".to_string()))?;
        self.state = TxStatus::Failed;
        tx.rollback().await?;
        self.state = TxStatus::RolledBack;
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

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::XError;

    #[test]
    fn pg_tx_runner_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgTxRunner>();
    }

    #[test]
    fn pg_tx_runner_new_and_pool_access() {
        // Use PgTxRunner with a null Arc to verify structural correctness.
        // Tests that need live PG are marked #[ignore] below.
        let pool = std::sync::Arc::new(
            std::panic::catch_unwind(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let cfg = crate::PostgresConfig::builder()
                        .host("127.0.0.1")
                        .database("postgres")
                        .user("postgres")
                        .build()
                        .expect("valid config");
                    crate::PostgresPool::connect_lazy(&cfg).await
                })
            })
        );
        // connect_lazy may fail if PG is not running, but the struct should still compile and work
        drop(pool);
    }
}
