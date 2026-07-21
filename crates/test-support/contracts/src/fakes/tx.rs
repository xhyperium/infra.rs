//! Tx Fake / Recording。

use async_trait::async_trait;
use contracts::{TxContext, TxRunner};
use kernel::{XError, XResult};
use std::sync::{Arc, Mutex};

/// 内存参考事务：记录 commit/rollback，供合同测试驱动真实 trait 路径。
#[derive(Debug, Default)]
pub struct FakeTxContext {
    /// 是否已 commit。
    pub committed: bool,
    /// 是否已 rollback。
    pub rolled_back: bool,
    fail_commit: bool,
}

impl FakeTxContext {
    /// 新建干净上下文。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入 commit 失败（`ErrorKind::Transient`）。
    pub fn with_commit_failure(mut self) -> Self {
        self.fail_commit = true;
        self
    }
}

#[async_trait]
impl TxContext for FakeTxContext {
    async fn commit(&mut self) -> XResult<()> {
        if self.fail_commit {
            return Err(XError::transient("事务提交失败（注入）"));
        }
        self.committed = true;
        self.rolled_back = false;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        self.rolled_back = true;
        self.committed = false;
        Ok(())
    }
}

/// 内存 [`TxRunner`] 参考实现。
#[derive(Debug, Default)]
pub struct FakeTxRunner;

#[async_trait]
impl TxRunner for FakeTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(FakeTxContext::new()))
    }
}

/// 可观察 commit/rollback 标志的 runner（合同测：证明编排真正驱动 [`TxContext`]）。
#[derive(Debug, Clone)]
pub struct RecordingTxRunner {
    /// 最近一次上下文是否已 commit。
    pub committed: Arc<Mutex<bool>>,
    /// 最近一次上下文是否已 rollback。
    pub rolled_back: Arc<Mutex<bool>>,
}

impl RecordingTxRunner {
    /// 新建，标志初始为 `false`。
    pub fn new() -> Self {
        Self { committed: Arc::new(Mutex::new(false)), rolled_back: Arc::new(Mutex::new(false)) }
    }
}

impl Default for RecordingTxRunner {
    fn default() -> Self {
        Self::new()
    }
}

struct RecordingTxContext {
    inner: FakeTxContext,
    committed: Arc<Mutex<bool>>,
    rolled_back: Arc<Mutex<bool>>,
}

#[async_trait]
impl TxContext for RecordingTxContext {
    async fn commit(&mut self) -> XResult<()> {
        self.inner.commit().await?;
        *self.committed.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.committed;
        *self.rolled_back.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.rolled_back;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        self.inner.rollback().await?;
        *self.committed.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.committed;
        *self.rolled_back.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.rolled_back;
        Ok(())
    }
}

#[async_trait]
impl TxRunner for RecordingTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(RecordingTxContext {
            inner: FakeTxContext::new(),
            committed: Arc::clone(&self.committed),
            rolled_back: Arc::clone(&self.rolled_back),
        }))
    }
}
