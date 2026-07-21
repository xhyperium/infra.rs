//! `contracts::EventBus` facade（**at-most-once**）。
//!
//! # 能力边界
//!
//! - `publish` 等待 broker produce 确认。
//! - `subscribe` 使用分区 0 流式消费；**不**提供 ack/redelivery。
//! - `BusMessage.id` 固定为 `topic/partition/offset`。
//!
//! 可靠消费请使用 [`crate::KafkaConsumer`]。

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::XResult;
use tokio::sync::mpsc;

use crate::consumer::ConsumerConfig;
use crate::message::encode_bus_id;
use crate::pool::KafkaPool;

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
        let (tx, rx) = mpsc::channel::<BusMessage>(256);
        tokio::spawn(async move {
            while let Some(item) = consumer.recv().await {
                match item {
                    Ok(m) => {
                        let id = encode_bus_id(&m.topic, m.partition, m.offset);
                        let bus = BusMessage { id, payload: m.payload };
                        if tx.send(bus).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let s = stream::unfold(rx, |mut rx| async move { rx.recv().await.map(|m| (m, rx)) });
        Ok(Box::pin(s))
    }
}
