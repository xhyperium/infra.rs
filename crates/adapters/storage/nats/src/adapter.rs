//! nats 内存 scaffold：`EventBus`。

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// nats 适配器（进程内；非真实客户端）。
pub struct NatsAdapter {
    name: String,
    endpoint: String,
    topics: Mutex<HashMap<String, Vec<BusMessage>>>,
}

impl NatsAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), topics: Mutex::new(HashMap::new()) }
    }

    pub fn local() -> Self {
        Self::new("nats-local", "nats://127.0.0.1:4222")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<BusMessage>>>> {
        self.topics.lock().map_err(|e| XError::internal(format!("topics lock poisoned: {e}")))
    }
}

#[async_trait]
impl EventBus for NatsAdapter {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        let n = self.lock()?.get(topic).map(|v| v.len()).unwrap_or(0);
        let msg = BusMessage { id: n.to_string(), payload };
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
    async fn publish_subscribe() {
        let a = NatsAdapter::local();
        a.publish("t", Bytes::from_static(b"p")).await.expect("pub");
        let mut s = a.subscribe("t").await.expect("sub");
        let msg = s.next().await.expect("msg");
        assert_eq!(msg.payload, Bytes::from_static(b"p"));
        assert!(!msg.id.is_empty());
    }

    #[test]
    fn name_endpoint() {
        let a = NatsAdapter::local();
        assert_eq!(a.name(), "nats-local");
        assert_eq!(a.endpoint(), "nats://127.0.0.1:4222");
    }
}
