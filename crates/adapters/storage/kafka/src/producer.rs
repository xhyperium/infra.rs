//! Kafka 生产者：等待 broker 确认。

use bytes::Bytes;
use chrono::Utc;
use kernel::{XError, XResult};
use rskafka::record::Record;
use std::collections::BTreeMap;

use crate::error_map::map_kafka_err;
use crate::lifecycle::wait_for_shutdown;
use crate::message::Delivery;
use crate::pool::KafkaPool;

/// 可克隆 producer。
#[derive(Clone)]
pub struct KafkaProducer {
    pub(crate) pool: KafkaPool,
}

impl KafkaProducer {
    /// 发布到 topic（默认分区 0），等待 produce 结果。
    pub async fn publish(&self, topic: &str, payload: Bytes) -> XResult<Delivery> {
        self.publish_to_partition(topic, 0, payload).await
    }

    /// 指定分区发布。
    pub async fn publish_to_partition(
        &self,
        topic: &str,
        partition: i32,
        payload: Bytes,
    ) -> XResult<Delivery> {
        self.pool.ensure_open()?;
        if topic.trim().is_empty() {
            return Err(XError::invalid("kafkax: topic 不能为空"));
        }
        let client = self.pool.partition_client(topic, partition).await?;
        let _operation = self.pool.start_operation()?;
        let mut shutdown = self.pool.shutdown_receiver();
        let record = Record {
            key: None,
            value: Some(payload.to_vec()),
            headers: BTreeMap::new(),
            timestamp: Utc::now(),
        };
        match tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => {
                self.pool.record_publish_err();
                return Err(XError::cancelled("kafkax produce 因 pool 关闭而取消"));
            }
            result = tokio::time::timeout(
                self.pool.config().delivery_timeout,
                client.produce(vec![record], KafkaPool::compression()),
            ) => result,
        } {
            Ok(Ok(offsets)) => {
                let offset = offsets.first().copied().unwrap_or(0);
                self.pool.record_publish_ok();
                Ok(Delivery { partition, offset })
            }
            Ok(Err(e)) => {
                self.pool.record_publish_err();
                Err(map_kafka_err("kafkax produce", e))
            }
            Err(error) => {
                self.pool.record_publish_err();
                Err(XError::deadline_exceeded("kafkax produce 超时").with_source(error))
            }
        }
    }
}
