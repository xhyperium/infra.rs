//! At-least-once 消费：显式 `ack` / `commit` 后才推进 offset。
//!
//! # 语义
//!
//! - 消息交付后进入 pending；在 `ack` 之前不会交付下一条。
//! - `ack` 将 pending 消息 offset 写入 [`OffsetCommitStore`]（next-to-read = offset+1）。
//! - 断线重连时从 store 的 committed next-to-read 用 `StartOffset::At` 重启。
//! - 未 ack 即 drop：store 仍保留上次提交点 → 重连会重投（at-least-once）。

use std::sync::Arc;
use std::time::Duration;

use kernel::{XError, XResult};

use crate::consumer::{ConsumerConfig, KafkaConsumer};
use crate::message::KafkaMessage;
use crate::offset::OffsetCommitStore;
use crate::pool::KafkaPool;

/// 底层消费源：生产路径为 live consumer；单测可注入 unit 后端。
enum ConsumerBackend {
    Live(KafkaConsumer),
    #[cfg(test)]
    Unit,
}

/// At-least-once 分区消费者。
pub struct AtLeastOnceConsumer {
    inner: ConsumerBackend,
    store: Arc<dyn OffsetCommitStore>,
    topic: String,
    partition: i32,
    pending: Option<KafkaMessage>,
    /// `drop_pending_unacked` 后为 true；禁止继续 recv/ack。
    terminated: bool,
}

impl AtLeastOnceConsumer {
    /// 连接：若 store 有 committed next-to-read，则从该 offset 启动。
    pub async fn connect(
        pool: KafkaPool,
        mut cfg: ConsumerConfig,
        store: Arc<dyn OffsetCommitStore>,
    ) -> XResult<Self> {
        if cfg.topic.trim().is_empty() {
            return Err(XError::invalid("kafkax: at-least-once topic 不能为空"));
        }
        let topic = cfg.topic.clone();
        let partition = cfg.partition;
        if let Some(next) = store.committed(&topic, partition).await? {
            cfg.start_offset = Some(next);
            cfg.from_beginning = false;
        }
        let inner = pool.consumer(cfg).await?;
        Ok(Self {
            inner: ConsumerBackend::Live(inner),
            store,
            topic,
            partition,
            pending: None,
            terminated: false,
        })
    }

    /// 单测用：注入 pending / 状态机（**无 broker**，不构造真实 Client）。
    #[cfg(test)]
    pub(crate) fn for_unit_test(
        store: Arc<dyn OffsetCommitStore>,
        topic: impl Into<String>,
        partition: i32,
        pending: Option<KafkaMessage>,
    ) -> Self {
        Self {
            inner: ConsumerBackend::Unit,
            store,
            topic: topic.into(),
            partition,
            pending,
            terminated: false,
        }
    }

    /// topic。
    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// 分区。
    #[must_use]
    pub fn partition(&self) -> i32 {
        self.partition
    }

    /// 当前未 ack 的消息（只读）。
    #[must_use]
    pub fn pending(&self) -> Option<&KafkaMessage> {
        self.pending.as_ref()
    }

    /// 取下一条消息。
    ///
    /// 若已有 pending 未 ack，返回该 pending（不从 broker 再取）。
    pub async fn recv(&mut self) -> Option<XResult<KafkaMessage>> {
        if let Err(e) = self.ensure_active() {
            return Some(Err(e));
        }
        if let Some(m) = &self.pending {
            return Some(Ok(m.clone()));
        }
        match &mut self.inner {
            ConsumerBackend::Live(c) => match c.recv().await {
                Some(Ok(m)) => {
                    self.pending = Some(m.clone());
                    Some(Ok(m))
                }
                Some(Err(e)) => Some(Err(e)),
                None => None,
            },
            #[cfg(test)]
            ConsumerBackend::Unit => None,
        }
    }

    /// 带超时接收（pending 优先，不计时）。
    pub async fn recv_timeout(&mut self, timeout: Duration) -> XResult<Option<KafkaMessage>> {
        self.ensure_active()?;
        if let Some(m) = &self.pending {
            return Ok(Some(m.clone()));
        }
        match &mut self.inner {
            ConsumerBackend::Live(c) => match c.recv_timeout(timeout).await? {
                Some(m) => {
                    self.pending = Some(m.clone());
                    Ok(Some(m))
                }
                None => Ok(None),
            },
            #[cfg(test)]
            ConsumerBackend::Unit => Ok(None),
        }
    }

    /// 确认 pending 消息：写入 store（next = offset+1）并清除 pending。
    ///
    /// **仅当 commit 成功才清除 pending**；store I/O 失败时 pending 保留，调用方可重试 `ack`。
    pub async fn ack(&mut self) -> XResult<()> {
        self.ensure_active()?;
        let msg = self
            .pending
            .as_ref()
            .ok_or_else(|| XError::conflict("kafkax at-least-once: 无 pending 可 ack"))?
            .clone();
        self.store.commit(&msg.topic, msg.partition, msg.offset).await?;
        self.pending = None;
        Ok(())
    }

    /// 显式提交任意 offset（高级兼容面；通常用 [`Self::ack`]）。
    #[deprecated(since = "0.3.2", note = "可绕过 pending 所有权；生产路径请使用 ack")]
    pub async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()> {
        self.store.commit(topic, partition, offset).await
    }

    /// 当前 store 中的 committed next-to-read。
    pub async fn committed(&self) -> XResult<Option<i64>> {
        self.store.committed(&self.topic, self.partition).await
    }

    /// 丢弃 pending 但不提交（下次 `recv` 仍返回同一 pending；用于本地重试）。
    pub fn nack_keep_pending(&mut self) {
        // pending 保留；调用方再次 ack 或 drop
    }

    /// 丢弃 pending 且 **不** 提交。
    ///
    /// **会话终止**：随后 `recv`/`ack` 返回 `Cancelled`，避免在未 ack 的情况下继续消费并越过位点。
    /// 重连后会从 last committed 重投。
    pub fn drop_pending_unacked(&mut self) {
        self.pending = None;
        self.terminated = true;
    }

    /// 会话是否已因 `drop_pending_unacked` 终止。
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        self.terminated
    }

    fn ensure_active(&self) -> XResult<()> {
        if self.terminated {
            Err(XError::cancelled(
                "kafkax at-least-once: 会话已因 drop_pending_unacked 终止；请重连",
            ))
        } else {
            Ok(())
        }
    }
}

/// 持有 pool + store 的 at-least-once 入口（便于构造多个消费者）。
#[derive(Clone)]
pub struct KafkaAtLeastOnceBus {
    pool: KafkaPool,
    store: Arc<dyn OffsetCommitStore>,
}

impl KafkaAtLeastOnceBus {
    /// 构造。
    #[must_use]
    pub fn new(pool: KafkaPool, store: Arc<dyn OffsetCommitStore>) -> Self {
        Self { pool, store }
    }

    /// 底层池。
    #[must_use]
    pub fn pool(&self) -> &KafkaPool {
        &self.pool
    }

    /// 打开 at-least-once 消费者。
    pub async fn consumer(&self, cfg: ConsumerConfig) -> XResult<AtLeastOnceConsumer> {
        AtLeastOnceConsumer::connect(self.pool.clone(), cfg, Arc::clone(&self.store)).await
    }

    /// 共享 store。
    #[must_use]
    pub fn store(&self) -> Arc<dyn OffsetCommitStore> {
        Arc::clone(&self.store)
    }
}

/// 纯逻辑：根据 store 状态解析启动 offset（离线单测用）。
pub async fn resolve_start_offset(
    store: &dyn OffsetCommitStore,
    topic: &str,
    partition: i32,
) -> XResult<Option<i64>> {
    store.committed(topic, partition).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offset::MemoryOffsetStore;
    use bytes::Bytes;
    use kernel::ErrorKind;

    fn sample_msg(offset: i64) -> KafkaMessage {
        KafkaMessage {
            topic: "orders".into(),
            partition: 0,
            offset,
            payload: Bytes::from_static(b"a"),
            key: None,
            headers: Default::default(),
            timestamp: None,
        }
    }

    #[tokio::test]
    async fn commit_advances_without_commit_stays() {
        let store = MemoryOffsetStore::new().shared();
        assert!(resolve_start_offset(store.as_ref(), "t", 0).await.expect("r").is_none());
        store.commit("t", 0, 7).await.expect("ack");
        assert_eq!(resolve_start_offset(store.as_ref(), "t", 0).await.expect("r"), Some(8));
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(8));
    }

    #[tokio::test]
    async fn nack_keep_pending_preserves_pending_and_store() {
        let store = MemoryOffsetStore::new().shared();
        let msg = sample_msg(9);
        let mut c = AtLeastOnceConsumer::for_unit_test(
            Arc::clone(&store) as Arc<dyn OffsetCommitStore>,
            "orders",
            0,
            Some(msg),
        );
        assert_eq!(c.topic(), "orders");
        assert_eq!(c.partition(), 0);
        assert!(!c.is_terminated());
        assert_eq!(c.pending().map(|m| m.offset), Some(9));
        c.nack_keep_pending();
        assert!(!c.is_terminated());
        assert_eq!(c.pending().map(|m| m.offset), Some(9));
        assert!(store.committed("orders", 0).await.expect("c").is_none());
        let again = c.recv().await.expect("some").expect("ok");
        assert_eq!(again.offset, 9);
        c.ack().await.expect("ack after nack");
        assert!(c.pending().is_none());
        assert_eq!(store.committed("orders", 0).await.expect("c"), Some(10));
    }

    #[tokio::test]
    async fn drop_pending_unacked_terminates_session() {
        let store = MemoryOffsetStore::new().shared();
        let mut c = AtLeastOnceConsumer::for_unit_test(
            Arc::clone(&store) as Arc<dyn OffsetCommitStore>,
            "orders",
            0,
            Some(sample_msg(3)),
        );
        c.drop_pending_unacked();
        assert!(c.is_terminated());
        assert!(c.pending().is_none());
        assert!(store.committed("orders", 0).await.expect("c").is_none());
        let err = c.recv().await.expect("err some").expect_err("terminated");
        assert_eq!(err.kind(), ErrorKind::Cancelled);
        let ack_err = c.ack().await.expect_err("ack after drop");
        assert_eq!(ack_err.kind(), ErrorKind::Cancelled);
        let to = c.recv_timeout(Duration::from_millis(10)).await.expect_err("to");
        assert_eq!(to.kind(), ErrorKind::Cancelled);
    }
}
