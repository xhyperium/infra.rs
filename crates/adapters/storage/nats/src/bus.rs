//! `contracts::EventBus` 实现（Core NATS，**at-most-once**）。
//!
//! - `publish` → [`NatsPool::publish`]（flush 后返回；非 durable）
//! - `subscribe` → 实时订阅；`BusMessage.id` = `{subject}/{seq}`
//! - 无历史回放、无 ack/redelivery

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use kernel::XResult;

use crate::pool::NatsPool;

/// EventBus facade。
#[derive(Clone)]
pub struct NatsEventBus {
    pool: NatsPool,
}

impl NatsEventBus {
    /// 从池构造。
    #[must_use]
    pub fn new(pool: NatsPool) -> Self {
        Self { pool }
    }

    /// 底层池。
    #[must_use]
    pub fn pool(&self) -> &NatsPool {
        &self.pool
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        self.pool.publish(topic, payload).await
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let sub = self.pool.subscribe(topic).await?;
        let stream = sub
            .into_stream()
            .map(|m| BusMessage { id: format!("{}/{}", m.subject, m.seq), payload: m.payload });
        Ok(Box::pin(stream))
    }
}

// 直接在 NatsPool 上也实现 EventBus，方便 `dyn EventBus` 使用池本身。
#[async_trait]
impl EventBus for NatsPool {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        NatsPool::publish(self, topic, payload).await
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        NatsEventBus::new(self.clone()).subscribe(topic).await
    }
}
