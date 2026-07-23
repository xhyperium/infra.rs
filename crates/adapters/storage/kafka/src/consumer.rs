//! Kafka 消费者：分区流式读取。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use kernel::{XError, XResult};
use rskafka::client::consumer::{StartOffset, StreamConsumerBuilder};
use tokio::sync::mpsc;

use crate::error_map::map_kafka_err;
use crate::lifecycle::{send_or_shutdown, wait_for_shutdown};
use crate::message::KafkaMessage;
use crate::pool::KafkaPool;

const CONSUMER_BUFFER_CAPACITY: usize = 64;

/// 消费配置。
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// topic。
    pub topic: String,
    /// 分区（默认 0；纯协议客户端按分区消费）。
    pub partition: i32,
    /// 兼容字段：group id（仅用于 EventBus 文档语义；rskafka 分区消费不依赖 coordinator）。
    pub group_id: String,
    /// 从最早还是最新开始（当 `start_offset` 为 `None` 时生效）。
    pub from_beginning: bool,
    /// 显式启动 offset（`StartOffset::At`）；优先于 `from_beginning`。
    ///
    /// 典型来源：[`crate::OffsetCommitStore::committed`] 的 next-to-read。
    pub start_offset: Option<i64>,
}

impl ConsumerConfig {
    /// 订阅 topic，分区 0。
    pub fn subscribe(topic: impl Into<String>, group_id: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            partition: 0,
            group_id: group_id.into(),
            from_beginning: true,
            start_offset: None,
        }
    }

    /// 手动指定分区。
    pub fn assign(topic: impl Into<String>, partition: i32, group_id: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            partition,
            group_id: group_id.into(),
            from_beginning: true,
            start_offset: None,
        }
    }

    /// 从存储的 next-to-read 启动。
    #[must_use]
    pub fn with_start_offset(mut self, offset: i64) -> Self {
        self.start_offset = Some(offset);
        self.from_beginning = false;
        self
    }

    /// 解析 rskafka [`StartOffset`]。
    #[must_use]
    pub fn resolve_start_offset(&self) -> StartOffset {
        if let Some(off) = self.start_offset {
            StartOffset::At(off)
        } else if self.from_beginning {
            StartOffset::Earliest
        } else {
            StartOffset::Latest
        }
    }
}

/// 消费者会话。
pub struct KafkaConsumer {
    rx: mpsc::Receiver<XResult<KafkaMessage>>,
    pool: KafkaPool,
    task: tokio::task::JoinHandle<()>,
}

impl KafkaConsumer {
    pub(crate) async fn connect(pool: KafkaPool, cfg: ConsumerConfig) -> XResult<Self> {
        validate_consumer_config(&cfg)?;
        let client = Arc::new(pool.partition_client(&cfg.topic, cfg.partition).await?);
        let start = cfg.resolve_start_offset();
        let mut stream = StreamConsumerBuilder::new(client, start).build();
        let (tx, rx) = mpsc::channel(CONSUMER_BUFFER_CAPACITY);
        let topic = cfg.topic.clone();
        let partition = cfg.partition;
        let operation = pool.start_operation()?;
        let mut shutdown = pool.shutdown_receiver();
        let task = tokio::spawn(async move {
            let _operation = operation;
            loop {
                let item = tokio::select! {
                    biased;
                    () = wait_for_shutdown(&mut shutdown) => break,
                    item = stream.next() => item,
                };
                let Some(item) = item else {
                    break;
                };
                let output = match item {
                    Ok((record_offset, _high_watermark)) => {
                        let record = record_offset.record;
                        Ok(KafkaMessage {
                            topic: topic.clone(),
                            partition,
                            offset: record_offset.offset,
                            key: record.key.map(Bytes::from),
                            payload: Bytes::from(record.value.unwrap_or_default()),
                            timestamp: Some(record.timestamp),
                        })
                    }
                    Err(error) => Err(map_kafka_err("kafkax fetch", error)),
                };
                let terminal_error = output.is_err();
                if !send_or_shutdown(&tx, output, &mut shutdown).await || terminal_error {
                    break;
                }
            }
        });
        Ok(Self { rx, pool, task })
    }

    /// 取下一条消息。
    pub async fn recv(&mut self) -> Option<XResult<KafkaMessage>> {
        if let Err(error) = self.pool.ensure_open() {
            return Some(Err(error));
        }
        self.rx.recv().await
    }

    /// 带超时接收。
    pub async fn recv_timeout(&mut self, timeout: Duration) -> XResult<Option<KafkaMessage>> {
        self.pool.ensure_open()?;
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(Some(Ok(m))) => Ok(Some(m)),
            Ok(Some(Err(e))) => Err(e),
            Ok(None) => Ok(None),
            Err(_) => Err(XError::deadline_exceeded("kafkax consumer recv timeout")),
        }
    }
}

impl Drop for KafkaConsumer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// 在发起 broker I/O 前校验消费配置的形状（fail-closed）。
fn validate_consumer_config(cfg: &ConsumerConfig) -> XResult<()> {
    if cfg.topic.trim().is_empty() {
        return Err(XError::invalid("kafkax: consumer topic 不能为空"));
    }
    if cfg.partition < 0 {
        return Err(XError::invalid("kafkax: consumer partition 不能为负"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_start_offset_matrix() {
        let mut c = ConsumerConfig::subscribe("t", "g");
        assert!(matches!(c.resolve_start_offset(), StartOffset::Earliest));
        c.from_beginning = false;
        assert!(matches!(c.resolve_start_offset(), StartOffset::Latest));
        c.start_offset = Some(12);
        match c.resolve_start_offset() {
            StartOffset::At(n) => assert_eq!(n, 12),
            other => panic!("expected At, got {other:?}"),
        }
        let c2 = ConsumerConfig::assign("t", 1, "g").with_start_offset(5);
        assert_eq!(c2.partition, 1);
        match c2.resolve_start_offset() {
            StartOffset::At(n) => assert_eq!(n, 5),
            other => panic!("expected At, got {other:?}"),
        }
    }

    #[test]
    fn consumer_buffer_is_intentionally_bounded() {
        assert_eq!(CONSUMER_BUFFER_CAPACITY, 64);
    }

    #[test]
    fn validate_consumer_config_rejects_empty_topic_before_broker_io() {
        let cfg = ConsumerConfig::subscribe("   ", "g");
        let error = validate_consumer_config(&cfg).expect_err("空 topic 必须在 broker I/O 前失败");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        assert!(error.context().contains("topic"));
    }

    #[test]
    fn validate_consumer_config_rejects_negative_partition_before_broker_io() {
        let cfg = ConsumerConfig::assign("t", -1, "g");
        let error =
            validate_consumer_config(&cfg).expect_err("负 partition 必须在 broker I/O 前失败");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        assert!(error.context().contains("partition"));
    }

    #[test]
    fn validate_consumer_config_accepts_well_formed_config() {
        let cfg = ConsumerConfig::assign("orders", 0, "g");
        validate_consumer_config(&cfg).expect("形状合法的配置必须通过校验");
    }
}
