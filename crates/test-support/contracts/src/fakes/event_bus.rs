//! EventBus Fake。

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// 内存 EventBus 参考实现（at-most-once 进程内）。
#[derive(Debug, Default)]
pub struct FakeEventBus {
    inner: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
}

impl FakeEventBus {
    /// 新建空总线。
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EventBus for FakeEventBus {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        let id = self.seq.fetch_add(1, Ordering::Relaxed).to_string();
        let mut g = self.inner.lock().map_err(|_| XError::internal("event bus lock 中毒"))?;
        g.entry(topic.to_string()).or_default().push(BusMessage { id, payload });
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = {
            let g = self.inner.lock().map_err(|_| XError::internal("event bus lock 中毒"))?;
            g.get(topic).cloned().unwrap_or_default()
        };
        Ok(Box::pin(VecBusStream { inner: msgs.into_iter() }))
    }
}

/// 简单的一次性消息流。
struct VecBusStream {
    inner: std::vec::IntoIter<BusMessage>,
}

impl futures_core::Stream for VecBusStream {
    type Item = BusMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.inner.next())
    }
}
