//! 进程内 `MockNatsBus`：实现 [`contracts::EventBus`]。
//!
//! 与 scaffold [`crate::NatsAdapter`] 的差异：
//! - 消息 ID 为全局单调递增序号（非 per-topic 下标）；
//! - 类型名明确标注 **Mock**，避免与生产客户端混淆。
//!
//! **非**真实 NATS 客户端；默认 `cargo test` 离线可跑。

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// 进程内 mock NATS 事件总线。
pub struct MockNatsBus {
    name: String,
    topics: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
}

impl MockNatsBus {
    /// 新建空总线。
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), topics: Mutex::new(HashMap::new()), seq: AtomicU64::new(0) }
    }

    /// 本地命名。
    pub fn local() -> Self {
        Self::new("mock-nats-local")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 当前已分配的消息序号。
    pub fn next_seq(&self) -> u64 {
        self.seq.load(Ordering::Relaxed)
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<BusMessage>>>> {
        self.topics.lock().map_err(|e| XError::internal(format!("topics lock poisoned: {e}")))
    }
}

#[async_trait]
impl EventBus for MockNatsBus {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        let id = self.seq.fetch_add(1, Ordering::Relaxed).to_string();
        let msg = BusMessage { id, payload };
        self.lock()?.entry(topic.to_string()).or_default().push(msg);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = self.lock()?.get(topic).cloned().unwrap_or_default();
        Ok(Box::pin(stream::iter(msgs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn publish_subscribe_with_monotonic_ids() {
        let bus = MockNatsBus::local();
        bus.publish("subj.a", Bytes::from_static(b"1")).await.expect("pub");
        bus.publish("subj.a", Bytes::from_static(b"2")).await.expect("pub");

        let mut s = bus.subscribe("subj.a").await.expect("sub");
        let m1 = s.next().await.expect("m1");
        let m2 = s.next().await.expect("m2");
        assert_eq!(m1.id, "0");
        assert_eq!(m2.id, "1");
        assert_eq!(m1.payload.as_ref(), b"1");
        assert_eq!(bus.next_seq(), 2);
    }

    #[tokio::test]
    async fn dyn_event_bus() {
        let bus = MockNatsBus::local();
        let b: &dyn EventBus = &bus;
        b.publish("t", Bytes::from_static(b"p")).await.expect("pub");
        let mut s = b.subscribe("t").await.expect("sub");
        assert_eq!(s.next().await.expect("msg").payload.as_ref(), b"p");
    }
}
