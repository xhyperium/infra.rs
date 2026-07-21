//! Kafka 消费者会话：每 group 独占 `StreamConsumer`。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use kernel::{XError, XResult};
use rdkafka::Message;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use tokio::sync::mpsc;

use crate::error_map::map_kafka_error;
use crate::message::KafkaMessage;

/// 消费组配置。
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// group.id
    pub group_id: String,
    /// auto.offset.reset：`earliest` / `latest`
    pub auto_offset_reset: String,
    /// 是否自动提交（默认 false；EventBus at-most-once 面可开启简化）。
    pub enable_auto_commit: bool,
}

impl ConsumerConfig {
    /// 以 group_id 构造默认配置（earliest、auto-commit 关）。
    pub fn new(group_id: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            auto_offset_reset: "earliest".into(),
            enable_auto_commit: false,
        }
    }
}

/// 独占消费会话。
pub struct KafkaConsumer {
    pub(crate) inner: Arc<StreamConsumer>,
    pub(crate) group_id: String,
}

impl KafkaConsumer {
    /// 消费组 id。
    #[must_use]
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    /// 订阅 topic 列表（消费组 rebalance）。
    ///
    /// 需要 broker 上 **group coordinator 可用**。若 `FindCoordinator`
    /// 返回 `COORDINATOR_NOT_AVAILABLE`，请改用 [`Self::assign`]。
    pub fn subscribe(&self, topics: &[&str]) -> XResult<()> {
        if topics.is_empty() {
            return Err(XError::invalid("kafkax: subscribe topics 不能为空"));
        }
        self.inner.subscribe(topics).map_err(|e| map_kafka_error("subscribe", e))
    }

    /// 手动分配分区（不依赖 group coordinator）。
    ///
    /// `partitions` 为 `(partition, Offset::Beginning|End|Offset(...))` 列表。
    pub fn assign(&self, topic: &str, partitions: &[(i32, i64)]) -> XResult<()> {
        if topic.is_empty() || partitions.is_empty() {
            return Err(XError::invalid("kafkax: assign topic/partitions 不能为空"));
        }
        let mut tpl = TopicPartitionList::new();
        for (p, off) in partitions {
            let offset = if *off < 0 {
                // -1=end, -2=beginning（与 librdkafka 特殊值对齐）
                if *off == -1 { Offset::End } else { Offset::Beginning }
            } else {
                Offset::Offset(*off)
            };
            tpl.add_partition_offset(topic, *p, offset)
                .map_err(|e| map_kafka_error("assign_add", e))?;
        }
        self.inner.assign(&tpl).map_err(|e| map_kafka_error("assign", e))
    }

    /// 当前分区分配数量。
    pub fn assignment_count(&self) -> XResult<usize> {
        let tpl = self.inner.assignment().map_err(|e| map_kafka_error("assignment", e))?;
        Ok(tpl.count())
    }

    /// 将底层 stream 转为有界 mpsc 消息流（`'static`）。
    ///
    /// 错误会结束流；因 `contracts::EventBus` 流项不能表达 `Result`，
    /// 可靠业务请直接使用本类型 + 指标/日志观察错误。
    pub fn into_message_stream(self) -> mpsc::Receiver<KafkaMessage> {
        let (tx, rx) = mpsc::channel(256);
        let consumer = Arc::clone(&self.inner);
        tokio::spawn(async move {
            loop {
                match consumer.recv().await {
                    Ok(msg) => {
                        let km = borrowed_to_owned(&msg);
                        if tx.send(km).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "kafkax consumer stream error; ending stream");
                        break;
                    }
                }
            }
        });
        rx
    }

    /// 有 deadline 的单次接收。
    pub async fn recv_timeout(&self, timeout: Duration) -> XResult<Option<KafkaMessage>> {
        match tokio::time::timeout(timeout, self.inner.recv()).await {
            Ok(Ok(msg)) => Ok(Some(borrowed_to_owned(&msg))),
            Ok(Err(e)) => Err(map_kafka_error("recv", e)),
            Err(_) => Ok(None),
        }
    }
}

fn borrowed_to_owned(msg: &BorrowedMessage<'_>) -> KafkaMessage {
    let payload = msg.payload().map(Bytes::copy_from_slice).unwrap_or_default();
    let key = msg.key().map(Bytes::copy_from_slice);
    KafkaMessage {
        topic: msg.topic().to_string(),
        partition: msg.partition(),
        offset: msg.offset(),
        payload,
        key,
    }
}
