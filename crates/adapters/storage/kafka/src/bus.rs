//! `contracts::EventBus` facade（**at-most-once**）。
//!
//! # 能力边界
//!
//! - `publish` 等待 broker delivery report（与 `KafkaProducer::publish` 一致）。
//! - `subscribe` 使用独立消费组 + **auto-commit**，流项为已取出消息；
//!   **不**提供 ack/redelivery，故只能承诺 at-most-once。
//! - `BusMessage.id` 固定为 `topic/partition/offset`。
//! - 流错误会结束 stream（合同 `Item=BusMessage` 无法表达 `Result`）。
//!
//! 可靠消费请使用 [`crate::KafkaConsumer`] 专属 API，勿依赖本 facade 做 at-least-once。

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, EventBus};
use futures_core::stream::BoxStream;
use kernel::XResult;
use tokio::sync::mpsc;

use crate::consumer::ConsumerConfig;
use crate::message::encode_bus_id;
use crate::pool::KafkaPool;

/// EventBus facade，持有 [`KafkaPool`]。
#[derive(Clone)]
pub struct KafkaEventBus {
    pool: KafkaPool,
    /// 可选固定 group；默认 `event_bus_group_prefix` + 递增序号。
    group_id: Option<String>,
    sub_seq: std::sync::Arc<AtomicU64>,
}

impl KafkaEventBus {
    /// 从池构造；每次 subscribe 使用独立 group（前缀 + 序号）。
    #[must_use]
    pub fn new(pool: KafkaPool) -> Self {
        Self { pool, group_id: None, sub_seq: std::sync::Arc::new(AtomicU64::new(0)) }
    }

    /// 固定消费组（多实例竞争同一组时使用）。
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
        // EventBus 面：auto-commit 简化 at-most-once 消费。
        let mut cfg = ConsumerConfig::new(group);
        cfg.enable_auto_commit = true;
        cfg.auto_offset_reset = "latest".into();
        let consumer = self.pool.consumer_with(cfg).await?;
        consumer.subscribe(&[topic])?;

        let (tx, rx) = mpsc::channel::<BusMessage>(256);
        tokio::spawn(async move {
            let mut msg_rx = consumer.into_message_stream();
            while let Some(km) = msg_rx.recv().await {
                let bus = BusMessage {
                    id: encode_bus_id(&km.topic, km.partition, km.offset),
                    payload: km.payload,
                };
                if tx.send(bus).await.is_err() {
                    break;
                }
            }
        });

        let stream =
            futures_util::stream::unfold(
                rx,
                |mut rx| async move { rx.recv().await.map(|m| (m, rx)) },
            );
        Ok(Box::pin(stream))
    }
}
