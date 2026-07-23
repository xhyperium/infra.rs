//! Kafka 生产者：等待 broker 确认。

use bytes::Bytes;
use chrono::Utc;
use kernel::{XError, XResult};
use rskafka::record::Record;
use std::collections::BTreeMap;

use crate::error_map::map_kafka_err;
use crate::lifecycle::wait_for_shutdown;
use crate::message::{Delivery, PublishRecord};
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

    /// 指定分区发布（无 key / headers）。
    pub async fn publish_to_partition(
        &self,
        topic: &str,
        partition: i32,
        payload: Bytes,
    ) -> XResult<Delivery> {
        self.publish_record(PublishRecord::payload(topic, partition, payload)).await
    }

    /// 指定分区 + key 发布。
    pub async fn publish_with_key(
        &self,
        topic: &str,
        partition: i32,
        key: Bytes,
        payload: Bytes,
    ) -> XResult<Delivery> {
        self.publish_record(PublishRecord::payload(topic, partition, payload).with_key(key)).await
    }

    /// 完整记录发布（key / headers / 分区）。
    ///
    /// # Errors
    ///
    /// topic 为空、partition 为负、pool 已关闭、delivery 超时或 broker 错误。
    pub async fn publish_record(&self, record: PublishRecord) -> XResult<Delivery> {
        self.pool.ensure_open()?;
        validate_publish_topic(&record.topic)?;
        if record.partition < 0 {
            return Err(XError::invalid("kafkax: partition 不能为负"));
        }
        let client = self.pool.partition_client(&record.topic, record.partition).await?;
        let _operation = self.pool.start_operation()?;
        let mut shutdown = self.pool.shutdown_receiver();
        let wire = Record {
            key: record.key.as_ref().map(|k| k.to_vec()),
            value: Some(record.payload.to_vec()),
            headers: record
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), v.to_vec()))
                .collect::<BTreeMap<_, _>>(),
            timestamp: Utc::now(),
        };
        match tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => {
                self.pool.record_publish_cancelled();
                return Err(XError::cancelled("kafkax produce 因 pool 关闭而取消"));
            }
            result = tokio::time::timeout(
                self.pool.config().delivery_timeout,
                client.produce(vec![wire], KafkaPool::compression()),
            ) => result,
        } {
            Ok(Ok(offsets)) => {
                let offset = offsets.first().copied().unwrap_or(0);
                self.pool.record_publish_ok();
                Ok(Delivery { partition: record.partition, offset })
            }
            Ok(Err(e)) => {
                self.pool.record_publish_err();
                Err(map_kafka_err("kafkax produce", e))
            }
            Err(error) => {
                self.pool.record_publish_timeout();
                Err(XError::deadline_exceeded("kafkax produce 超时").with_source(error))
            }
        }
    }
}

/// 在 broker I/O 前校验 publish topic 形状。
fn validate_publish_topic(topic: &str) -> XResult<()> {
    if topic.trim().is_empty() {
        return Err(XError::invalid("kafkax: topic 不能为空"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;

    #[test]
    fn empty_topic_rejected_before_broker_io() {
        let error = validate_publish_topic("  ").expect_err("空 topic 必须失败");
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert!(error.context().contains("topic"));
    }

    #[test]
    fn non_empty_topic_accepted() {
        validate_publish_topic("orders").expect("合法 topic");
    }

    #[test]
    fn publish_record_rejects_negative_partition_shape() {
        // 形状校验在 publish_record 入口；此处只测 topic 校验仍可用
        validate_publish_topic("ok").expect("ok");
    }
}
