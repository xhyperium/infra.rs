//! `contracts::EventBus` facade（**at-most-once**）。
//!
//! # 能力边界
//!
//! - `publish` 等待 broker produce 确认。
//! - `subscribe` 使用分区 0 流式消费；**不**提供 ack/redelivery。
//! - `BusMessage.id` 固定为 `topic/partition/offset`。
//!
//! 可靠消费请使用 [`crate::KafkaConsumer`]。

use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::{Stream, stream::BoxStream};
use kernel::XResult;
use tokio::sync::mpsc;

use crate::consumer::ConsumerConfig;
use crate::lifecycle::{send_or_shutdown, wait_for_shutdown};
use crate::message::encode_bus_id;
use crate::pool::KafkaPool;

const EVENT_BUS_BUFFER_CAPACITY: usize = 256;

struct BusSubscription {
    rx: mpsc::Receiver<BusMessage>,
    task: tokio::task::JoinHandle<()>,
}

impl Stream for BusSubscription {
    type Item = BusMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl Drop for BusSubscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// EventBus facade，持有 [`KafkaPool`]。
#[derive(Clone)]
pub struct KafkaEventBus {
    pool: KafkaPool,
    /// 可选固定 group 标签（仅文档/日志语义；分区消费不依赖 coordinator）。
    group_id: Option<String>,
    sub_seq: std::sync::Arc<AtomicU64>,
}

impl KafkaEventBus {
    /// 从池构造。
    #[must_use]
    pub fn new(pool: KafkaPool) -> Self {
        Self { pool, group_id: None, sub_seq: std::sync::Arc::new(AtomicU64::new(0)) }
    }

    /// 带 group 标签。
    #[must_use]
    pub fn with_group(pool: KafkaPool, group_id: impl Into<String>) -> Self {
        Self {
            pool,
            group_id: Some(group_id.into()),
            sub_seq: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    /// 底层池。
    #[must_use]
    pub fn pool(&self) -> &KafkaPool {
        &self.pool
    }
}

#[async_trait]
impl EventBus for KafkaEventBus {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        let _ = self.pool.producer().publish(topic, payload).await?;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let group = if let Some(g) = &self.group_id {
            g.clone()
        } else {
            let n = self.sub_seq.fetch_add(1, Ordering::Relaxed);
            format!("{}-{n}-{}", self.pool.config().event_bus_group_prefix, std::process::id())
        };
        let mut cfg = ConsumerConfig::subscribe(topic, group);
        cfg.from_beginning = false;
        let mut consumer = self.pool.consumer(cfg).await?;
        let (tx, rx) = mpsc::channel::<BusMessage>(EVENT_BUS_BUFFER_CAPACITY);
        let mut shutdown = self.pool.shutdown_receiver();
        let task = tokio::spawn(async move {
            loop {
                let item = tokio::select! {
                    biased;
                    () = wait_for_shutdown(&mut shutdown) => break,
                    item = consumer.recv() => item,
                };
                let Some(item) = item else {
                    break;
                };
                match item {
                    Ok(m) => {
                        let id = encode_bus_id(&m.topic, m.partition, m.offset);
                        let bus = BusMessage { id, payload: m.payload };
                        if !send_or_shutdown(&tx, bus, &mut shutdown).await {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Box::pin(BusSubscription { rx, task }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_buffer_is_intentionally_bounded() {
        assert_eq!(EVENT_BUS_BUFFER_CAPACITY, 256);
    }
}
