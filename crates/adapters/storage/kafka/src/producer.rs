//! Kafka 生产者：共享 `FutureProducer`，`publish` 等待 delivery report。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bytes::Bytes;
use kernel::XResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use crate::error_map::map_kafka_error;
use crate::message::Delivery;

/// 可克隆的生产者句柄（共享底层 FutureProducer）。
#[derive(Clone)]
pub struct KafkaProducer {
    pub(crate) inner: FutureProducer,
    pub(crate) delivery_timeout: Duration,
    pub(crate) published: Arc<AtomicU64>,
    pub(crate) failed: Arc<AtomicU64>,
}

impl KafkaProducer {
    /// 发布消息并等待 broker delivery report。
    ///
    /// 成功仅表示 broker 已按 `acks` 确认；**不**承诺跨系统事务。
    pub async fn publish(&self, topic: &str, payload: Bytes) -> XResult<Delivery> {
        if topic.is_empty() {
            return Err(kernel::XError::invalid("kafkax: topic 不能为空"));
        }
        let record: FutureRecord<'_, (), [u8]> = FutureRecord::to(topic).payload(payload.as_ref());
        match self.inner.send(record, Timeout::After(self.delivery_timeout)).await {
            Ok((partition, offset)) => {
                self.published.fetch_add(1, Ordering::Relaxed);
                Ok(Delivery { partition, offset })
            }
            Err((err, _owned)) => {
                self.failed.fetch_add(1, Ordering::Relaxed);
                Err(map_kafka_error("publish", err))
            }
        }
    }

    /// 带 key 的发布。
    pub async fn publish_with_key(
        &self,
        topic: &str,
        key: &[u8],
        payload: Bytes,
    ) -> XResult<Delivery> {
        if topic.is_empty() {
            return Err(kernel::XError::invalid("kafkax: topic 不能为空"));
        }
        let record: FutureRecord<'_, [u8], [u8]> =
            FutureRecord::to(topic).payload(payload.as_ref()).key(key);
        match self.inner.send(record, Timeout::After(self.delivery_timeout)).await {
            Ok((partition, offset)) => {
                self.published.fetch_add(1, Ordering::Relaxed);
                Ok(Delivery { partition, offset })
            }
            Err((err, _owned)) => {
                self.failed.fetch_add(1, Ordering::Relaxed);
                Err(map_kafka_error("publish_with_key", err))
            }
        }
    }
}
