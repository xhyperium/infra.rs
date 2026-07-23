//! 进程内 mock 后端：带 **commit 边界** 的 Repository + TxRunner。
//!
//! 与 scaffold [`crate::PostgresAdapter`]（直接写 durable + 本地 ScaffoldTxContext）不同：
//! - 事务内写入进入 staged 区，**仅**在 `commit` 后可见于 durable；
//! - `rollback` 丢弃 staged，不触碰 durable；
//! - 可观察 commit/rollback 计数，证明 `run_tx_lifecycle` 驱动真实路径。
//!
//! **非**真实 Postgres 客户端；默认 `cargo test` 离线可跑。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use contracts::{Repository, TxContext, TxRunner};
use kernel::{XError, XResult};

use crate::Record;

/// 可观察的事务结果计数。
#[derive(Debug, Default)]
pub struct TxObservability {
    /// 成功 commit 次数。
    pub commits: AtomicU64,
    /// 成功 rollback 次数。
    pub rollbacks: AtomicU64,
}

impl TxObservability {
    /// 新建计数器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前 commit 次数。
    pub fn commit_count(&self) -> u64 {
        self.commits.load(Ordering::SeqCst)
    }

    /// 当前 rollback 次数。
    pub fn rollback_count(&self) -> u64 {
        self.rollbacks.load(Ordering::SeqCst)
    }
}

/// 进程内 mock Postgres（commit 边界版）。
///
/// 与 scaffold [`super::PostgresAdapter`] 的差异：
/// - `begin_tx` 返回 [`MockTxContext`]（staged 写入；非 scaffold 空事务）
/// - 事务写入必须 `stage_save`，commit 后才进入 durable
/// - 可观察 commit/rollback 次数
#[derive(Debug, Clone)]
pub struct ObservingPostgresAdapter {
    name: String,
    durable: Arc<Mutex<HashMap<String, Record>>>,
    obs: Arc<TxObservability>,
}

/// 与 [`ObservingPostgresAdapter`] 同义（mock 验证入口主名）。
pub type MockPostgresBackend = ObservingPostgresAdapter;

impl ObservingPostgresAdapter {
    /// 新建空适配器。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: Arc::new(Mutex::new(HashMap::new())),
            obs: Arc::new(TxObservability::new()),
        }
    }

    /// 本地命名。
    pub fn local() -> Self {
        Self::new("observing-postgres-local")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 可观察计数。
    pub fn observability(&self) -> Arc<TxObservability> {
        Arc::clone(&self.obs)
    }

    /// durable 行数。
    pub fn durable_len(&self) -> XResult<usize> {
        Ok(self
            .durable
            .lock()
            .map_err(|e| XError::internal(format!("持久状态锁已污染: {e}")))?
            .len())
    }

    /// 开启事务并返回具体类型（可 `stage_save`）。
    pub async fn begin_mock_tx(&self) -> XResult<MockTxContext> {
        Ok(MockTxContext {
            durable: Arc::clone(&self.durable),
            staged: HashMap::new(),
            finished: false,
            obs: Arc::clone(&self.obs),
        })
    }
}

/// Mock 事务上下文：staged 写入 + commit/rollback 边界。
pub struct MockTxContext {
    durable: Arc<Mutex<HashMap<String, Record>>>,
    staged: HashMap<String, Record>,
    finished: bool,
    obs: Arc<TxObservability>,
}

impl MockTxContext {
    /// 事务内暂存写入（commit 前对 Repository 不可见）。
    pub fn stage_save(&mut self, entity: Record) -> XResult<()> {
        if self.finished {
            return Err(XError::conflict("事务已终结，禁止再 stage"));
        }
        self.staged.insert(entity.id.clone(), entity);
        Ok(())
    }

    /// staged 条数。
    pub fn staged_len(&self) -> usize {
        self.staged.len()
    }
}

#[async_trait]
impl TxContext for MockTxContext {
    async fn commit(&mut self) -> XResult<()> {
        if self.finished {
            return Err(XError::conflict("事务已终结，重复 commit"));
        }
        let staged = std::mem::take(&mut self.staged);
        {
            let mut g = self
                .durable
                .lock()
                .map_err(|e| XError::internal(format!("持久状态锁已污染: {e}")))?;
            for (k, v) in staged {
                g.insert(k, v);
            }
        }
        self.finished = true;
        self.obs.commits.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        if self.finished {
            return Err(XError::conflict("事务已终结，重复 rollback"));
        }
        self.staged.clear();
        self.finished = true;
        self.obs.rollbacks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl Repository<Record, String> for ObservingPostgresAdapter {
    async fn find(&self, id: String) -> XResult<Option<Record>> {
        Ok(self
            .durable
            .lock()
            .map_err(|e| XError::internal(format!("持久状态锁已污染: {e}")))?
            .get(&id)
            .cloned())
    }

    async fn save(&self, entity: &Record) -> XResult<()> {
        // 直接写 durable（非事务路径）；事务写入请用 MockTxContext::stage_save
        self.durable
            .lock()
            .map_err(|e| XError::internal(format!("持久状态锁已污染: {e}")))?
            .insert(entity.id.clone(), entity.clone());
        Ok(())
    }
}

#[async_trait]
impl TxRunner for ObservingPostgresAdapter {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(MockTxContext {
            durable: Arc::clone(&self.durable),
            staged: HashMap::new(),
            finished: false,
            obs: Arc::clone(&self.obs),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::run_tx_lifecycle;

    #[tokio::test]
    async fn staged_write_visible_only_after_commit() {
        let a = ObservingPostgresAdapter::local();
        let mut tx = a.begin_mock_tx().await.expect("begin");
        tx.stage_save(Record { id: "1".into(), data: b"x".to_vec() }).expect("stage");
        assert_eq!(tx.staged_len(), 1);
        // commit 前 durable 不可见
        assert!(a.find("1".into()).await.expect("find").is_none());
        tx.commit().await.expect("commit");
        assert_eq!(
            a.find("1".into()).await.expect("find"),
            Some(Record { id: "1".into(), data: b"x".to_vec() })
        );
        assert_eq!(a.observability().commit_count(), 1);
        assert_eq!(a.observability().rollback_count(), 0);
    }

    #[tokio::test]
    async fn rollback_discards_staged() {
        let a = ObservingPostgresAdapter::local();
        let mut tx = a.begin_mock_tx().await.expect("begin");
        tx.stage_save(Record { id: "2".into(), data: b"y".to_vec() }).expect("stage");
        tx.rollback().await.expect("rollback");
        assert!(a.find("2".into()).await.expect("find").is_none());
        assert_eq!(a.observability().rollback_count(), 1);
        assert_eq!(a.observability().commit_count(), 0);
        assert_eq!(a.durable_len().expect("len"), 0);
    }

    #[tokio::test]
    async fn run_tx_lifecycle_observes_commit() {
        let a = ObservingPostgresAdapter::local();
        let out =
            run_tx_lifecycle(&a, || async move { Ok::<_, XError>(9u8) }).await.expect("事务成功");
        assert_eq!(out, 9);
        assert_eq!(a.observability().commit_count(), 1);
    }

    #[tokio::test]
    async fn run_tx_err_observes_rollback() {
        let a = ObservingPostgresAdapter::local();
        let err = run_tx_lifecycle(&a, || async move { Err::<(), _>(XError::invalid("业务失败")) })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            contracts::TxRunError::Business { source }
                if source.kind() == kernel::ErrorKind::Invalid
        ));
        assert_eq!(a.observability().rollback_count(), 1);
        assert_eq!(a.observability().commit_count(), 0);
    }

    #[tokio::test]
    async fn double_commit_is_conflict() {
        let a = ObservingPostgresAdapter::local();
        let mut tx = a.begin_mock_tx().await.expect("begin");
        tx.commit().await.expect("first");
        let e = tx.commit().await.expect_err("second");
        assert_eq!(e.kind(), kernel::ErrorKind::Conflict);
    }

    #[tokio::test]
    async fn repository_direct_save_skips_tx() {
        let a = ObservingPostgresAdapter::local();
        let r = Record { id: "d".into(), data: b"z".to_vec() };
        a.save(&r).await.expect("save");
        assert_eq!(a.find("d".into()).await.expect("find"), Some(r));
    }

    #[tokio::test]
    async fn dyn_tx_runner_object_safe() {
        let a = ObservingPostgresAdapter::local();
        let runner: &dyn TxRunner = &a;
        let mut ctx = runner.begin_tx().await.expect("begin");
        ctx.commit().await.expect("commit");
        assert_eq!(a.observability().commit_count(), 1);
    }
}
