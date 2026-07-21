//! 进程内 `MockKafkaBus`：实现 [`contracts::EventBus`]。
//!
//! 与 scaffold [`crate::KafkaAdapter`] 的差异：
//! - 消息 ID 为全局单调递增序号（非 per-topic 下标）；
//! - 类型名明确标注 **Mock**，避免与生产客户端混淆。
//!
//! **非**真实 Kafka 客户端；默认 `cargo test` 离线可跑。

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// 进程内 mock Kafka 事件总线。
pub struct MockKafkaBus {
    name: String,
    topics: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
}

impl MockKafkaBus {
    /// 新建空总线。
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), topics: Mutex::new(HashMap::new()), seq: AtomicU64::new(0) }
    }

    /// 本地命名。
    pub fn local() -> Self {
        Self::new("mock-kafka-local")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 当前已分配的消息序号（下一 id 的起点）。
    pub fn next_seq(&self) -> u64 {
        self.seq.load(Ordering::Relaxed)
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<BusMessage>>>> {
        self.topics.lock().map_err(|e| XError::internal(format!("topics lock poisoned: {e}")))
    }
}

#[async_trait]
impl EventBus for MockKafkaBus {
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
        let bus = MockKafkaBus::local();
        bus.publish("orders", Bytes::from_static(b"a")).await.expect("pub");
        bus.publish("orders", Bytes::from_static(b"b")).await.expect("pub");
        bus.publish("fills", Bytes::from_static(b"c")).await.expect("pub");

        let mut s = bus.subscribe("orders").await.expect("sub");
        let m1 = s.next().await.expect("m1");
        let m2 = s.next().await.expect("m2");
        assert_eq!(m1.id, "0");
        assert_eq!(m2.id, "1");
        assert_eq!(m1.payload.as_ref(), b"a");
        assert_eq!(m2.payload.as_ref(), b"b");
        assert!(s.next().await.is_none());

        let mut s2 = bus.subscribe("fills").await.expect("sub fills");
        let m3 = s2.next().await.expect("m3");
        assert_eq!(m3.id, "2");
        assert_eq!(bus.next_seq(), 3);
    }

    #[tokio::test]
    async fn empty_topic_stream() {
        let bus = MockKafkaBus::local();
        let mut s = bus.subscribe("none").await.expect("sub");
        assert!(s.next().await.is_none());
    }

    #[tokio::test]
    async fn dyn_event_bus() {
        let bus = MockKafkaBus::local();
        let b: &dyn EventBus = &bus;
        b.publish("t", Bytes::from_static(b"p")).await.expect("pub");
        let mut s = b.subscribe("t").await.expect("sub");
        assert_eq!(s.next().await.expect("msg").payload.as_ref(), b"p");
    }
}
