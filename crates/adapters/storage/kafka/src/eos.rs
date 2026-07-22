//! 应用层 Exactly-Once 语义（EOS）协调。
//!
//! # 能力边界
//!
//! `rskafka` **没有** transactional producer / 幂等 producer 的完整 EOS 协议。
//! 本模块提供 **应用级 dual-write 模式**：
//!
//! 1. 消费业务消息（或本地副作用）
//! 2. **先** produce 到 side-effect / 日志 topic
//! 3. **仅当 produce 成功** 才允许 commit 消费 offset
//! 4. produce 失败 → **fail-closed**：绝不 commit（at-least-once 重投）
//!
//! 这保证「副作用已写出」与「消费位点前进」的单向依赖，避免「提交了 offset 但副作用丢失」。
//! 反向（副作用写出但 commit 失败）会在重启后重投 → 业务需幂等。

use std::sync::Arc;

use bytes::Bytes;
use kernel::{XError, XResult};

use crate::message::Delivery;
use crate::offset::OffsetCommitStore;
use crate::producer::KafkaProducer;

/// EOS 协调器：绑定 offset store。
#[derive(Clone)]
pub struct EosCoordinator {
    store: Arc<dyn OffsetCommitStore>,
}

impl EosCoordinator {
    /// 构造。
    #[must_use]
    pub fn new(store: Arc<dyn OffsetCommitStore>) -> Self {
        Self { store }
    }

    /// 共享 store。
    #[must_use]
    pub fn store(&self) -> Arc<dyn OffsetCommitStore> {
        Arc::clone(&self.store)
    }

    /// 开启会话（绑定待提交的消费位点）。
    #[must_use]
    pub fn begin(
        &self,
        commit_topic: impl Into<String>,
        commit_partition: i32,
        commit_offset: i64,
    ) -> EosSession {
        EosSession {
            store: Arc::clone(&self.store),
            commit_topic: commit_topic.into(),
            commit_partition,
            commit_offset,
            produce_ok: false,
            committed: false,
            aborted: false,
        }
    }

    /// Dual-write 一步完成：produce → 成功才 commit。
    ///
    /// Fail-closed：`produce` 失败时 **不** 调用 store.commit。
    pub async fn produce_then_commit(
        &self,
        producer: &KafkaProducer,
        side_topic: &str,
        payload: Bytes,
        commit_topic: &str,
        commit_partition: i32,
        commit_offset: i64,
    ) -> XResult<Delivery> {
        match producer.publish(side_topic, payload).await {
            Ok(delivery) => {
                self.store.commit(commit_topic, commit_partition, commit_offset).await?;
                Ok(delivery)
            }
            Err(e) => {
                // fail-closed：明确不 commit
                Err(e)
            }
        }
    }

    /// 对任意 produce 结果应用 fail-closed 规则（离线可测）。
    ///
    /// - `Ok(delivery)` → commit 后返回 delivery
    /// - `Err(_)` → **不** commit，原样返回错误
    pub async fn after_produce_result(
        &self,
        produce: XResult<Delivery>,
        commit_topic: &str,
        commit_partition: i32,
        commit_offset: i64,
    ) -> XResult<Delivery> {
        match produce {
            Ok(delivery) => {
                self.store.commit(commit_topic, commit_partition, commit_offset).await?;
                Ok(delivery)
            }
            Err(e) => Err(e),
        }
    }
}

/// 单次 EOS 会话：标记 produce 成功后才允许 commit。
pub struct EosSession {
    store: Arc<dyn OffsetCommitStore>,
    commit_topic: String,
    commit_partition: i32,
    commit_offset: i64,
    produce_ok: bool,
    committed: bool,
    /// `rollback` 后为 true；会话终结，不可再 mark/commit。
    aborted: bool,
}

impl EosSession {
    /// 消费位点。
    #[must_use]
    pub fn commit_offset(&self) -> i64 {
        self.commit_offset
    }

    /// produce 是否已标记成功。
    #[must_use]
    pub fn produce_ok(&self) -> bool {
        self.produce_ok
    }

    /// 是否已提交。
    #[must_use]
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 是否已 abort。
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    /// 标记 side-effect produce 成功（调用方在真实 produce Ok 后调用）。
    pub fn mark_produce_ok(&mut self) -> XResult<()> {
        if self.aborted {
            return Err(XError::conflict("kafkax EOS: 会话已 abort，禁止 mark_produce_ok"));
        }
        if self.committed {
            return Err(XError::conflict("kafkax EOS: 会话已提交，禁止 mark_produce_ok"));
        }
        self.produce_ok = true;
        Ok(())
    }

    /// 标记 produce 失败并回滚会话意图（清除 produce_ok）。
    pub fn mark_produce_failed(&mut self) {
        self.produce_ok = false;
    }

    /// 尝试 commit：仅当 `produce_ok` 且尚未 committed / abort。
    ///
    /// Fail-closed：produce 未成功 → `Conflict`，store 不变。
    pub async fn try_commit(&mut self) -> XResult<()> {
        if self.aborted {
            return Err(XError::conflict("kafkax EOS: 会话已 abort，拒绝 commit"));
        }
        if self.committed {
            return Err(XError::conflict("kafkax EOS: 会话已提交"));
        }
        if !self.produce_ok {
            return Err(XError::conflict("kafkax EOS: produce 未成功，拒绝 commit（fail-closed）"));
        }
        self.store.commit(&self.commit_topic, self.commit_partition, self.commit_offset).await?;
        self.committed = true;
        Ok(())
    }

    /// 显式回滚：会话终结，后续 mark/commit 一律拒绝。
    pub fn rollback(&mut self) {
        self.produce_ok = false;
        self.committed = false;
        self.aborted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offset::MemoryOffsetStore;
    use kernel::ErrorKind;

    #[tokio::test]
    async fn produce_ok_allows_commit() {
        let store = MemoryOffsetStore::new().shared();
        let eos = EosCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        let delivery = Delivery { partition: 0, offset: 42 };
        let out = eos.after_produce_result(Ok(delivery), "consume-topic", 0, 7).await.expect("ok");
        assert_eq!(out.offset, 42);
        // commit next-to-read = 8
        assert_eq!(store.committed("consume-topic", 0).await.expect("c"), Some(8));
    }

    #[tokio::test]
    async fn produce_fail_no_commit() {
        let store = MemoryOffsetStore::new().shared();
        let eos = EosCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        let err = eos
            .after_produce_result(Err(XError::unavailable("broker down")), "consume-topic", 0, 7)
            .await
            .expect_err("must fail");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        // fail-closed：store 无记录
        assert!(store.committed("consume-topic", 0).await.expect("c").is_none());
    }

    #[tokio::test]
    async fn session_fail_closed_and_rollback() {
        let store = MemoryOffsetStore::new().shared();
        let eos = EosCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        let mut session = eos.begin("t", 0, 3);

        // 未 produce → commit 拒绝
        let e = session.try_commit().await.expect_err("closed");
        assert_eq!(e.kind(), ErrorKind::Conflict);
        assert!(store.committed("t", 0).await.expect("c").is_none());

        // produce ok → commit 成功
        session.mark_produce_ok().expect("mark");
        session.try_commit().await.expect("commit");
        assert!(session.is_committed());
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(4));

        // 新会话 + rollback
        let mut s2 = eos.begin("t", 0, 10);
        s2.mark_produce_ok().expect("mark");
        s2.rollback();
        let e2 = s2.try_commit().await.expect_err("rolled back");
        assert_eq!(e2.kind(), ErrorKind::Conflict);
        // 仍为 4，未前进到 11
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(4));
    }

    #[tokio::test]
    async fn session_produce_fail_path() {
        let store = MemoryOffsetStore::new().shared();
        let eos = EosCoordinator::new(Arc::clone(&store) as Arc<dyn OffsetCommitStore>);
        let mut session = eos.begin("orders", 1, 100);
        session.mark_produce_ok().expect("mark");
        session.mark_produce_failed();
        assert!(!session.produce_ok());
        let e = session.try_commit().await.expect_err("fail closed");
        assert!(
            e.context().contains("fail-closed")
                || e.to_string().contains("fail-closed")
                || e.context().contains("拒绝")
        );
        assert!(store.committed("orders", 1).await.expect("c").is_none());
    }
}
