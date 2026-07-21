//! `KafkaPool`：基于 `rskafka` 的共享客户端与生命周期。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use kernel::{XError, XResult};
use rskafka::client::{
    Client, ClientBuilder,
    partition::{Compression, UnknownTopicHandling},
};
use rskafka::client::{Credentials, SaslConfig};

use crate::config::KafkaConfig;
use crate::consumer::{ConsumerConfig, KafkaConsumer};
use crate::producer::KafkaProducer;

/// 池统计。
#[derive(Debug, Clone, Copy, Default)]
pub struct KafkaPoolStats {
    /// 成功 publish 次数。
    pub published: u64,
    /// publish 失败次数。
    pub publish_failed: u64,
    /// 是否已关闭。
    pub closed: bool,
}

/// 健康结果。
#[derive(Debug, Clone)]
pub struct KafkaHealth {
    /// 是否就绪。
    pub ready: bool,
    /// 说明。
    pub detail: String,
}

/// 资源池（可克隆）。
#[derive(Clone)]
pub struct KafkaPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    config: KafkaConfig,
    client: Client,
    published: AtomicU64,
    publish_failed: AtomicU64,
    closed: AtomicBool,
}

impl KafkaPool {
    /// 连接集群。
    pub async fn connect(config: KafkaConfig) -> XResult<Self> {
        config.validate()?;
        let brokers: Vec<String> = config
            .brokers
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let mut builder = ClientBuilder::new(brokers).client_id(config.client_id.clone());
        if let (Some(u), Some(p)) = (&config.sasl_username, &config.sasl_password) {
            builder =
                builder.sasl_config(SaslConfig::Plain(Credentials::new(u.clone(), p.clone())));
        } else if config.sasl_mechanism.is_some() {
            return Err(XError::invalid("kafkax: SASL 机制已设但缺少 username/password"));
        }
        let client = builder
            .build()
            .await
            .map_err(|e| XError::unavailable(format!("kafkax connect: {e}")))?;
        Ok(Self {
            inner: Arc::new(PoolInner {
                config,
                client,
                published: AtomicU64::new(0),
                publish_failed: AtomicU64::new(0),
                closed: AtomicBool::new(false),
            }),
        })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(KafkaConfig::from_env()).await
    }

    /// 配置。
    #[must_use]
    pub fn config(&self) -> &KafkaConfig {
        &self.inner.config
    }

    /// 底层 client。
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.inner.client
    }

    /// 共享 producer 句柄。
    #[must_use]
    pub fn producer(&self) -> KafkaProducer {
        KafkaProducer { pool: self.clone() }
    }

    /// 创建 consumer（分区客户端 + 流）。
    pub async fn consumer(&self, cfg: ConsumerConfig) -> XResult<KafkaConsumer> {
        self.ensure_open()?;
        KafkaConsumer::connect(self.clone(), cfg).await
    }

    /// 健康：列出 topics。
    pub async fn health(&self) -> XResult<KafkaHealth> {
        self.ensure_open()?;
        match self.inner.client.list_topics().await {
            Ok(topics) => {
                Ok(KafkaHealth { ready: true, detail: format!("topics={}", topics.len()) })
            }
            Err(e) => Ok(KafkaHealth { ready: false, detail: format!("list_topics: {e}") }),
        }
    }

    /// 统计。
    #[must_use]
    pub fn stats(&self) -> KafkaPoolStats {
        KafkaPoolStats {
            published: self.inner.published.load(Ordering::Relaxed),
            publish_failed: self.inner.publish_failed.load(Ordering::Relaxed),
            closed: self.inner.closed.load(Ordering::Relaxed),
        }
    }

    /// 确保 topic 存在。
    pub async fn ensure_topic(
        &self,
        topic: &str,
        partitions: i32,
        replication: i16,
    ) -> XResult<()> {
        self.ensure_open()?;
        let ctrl = self
            .inner
            .client
            .controller_client()
            .map_err(|e| XError::unavailable(format!("kafkax controller: {e}")))?;
        match ctrl.create_topic(topic, partitions, replication, 5_000).await {
            Ok(()) => Ok(()),
            Err(e) => {
                let s = e.to_string().to_ascii_lowercase();
                if s.contains("exist") || s.contains("already") || s.contains("topic_already") {
                    Ok(())
                } else {
                    Err(XError::unavailable(format!("kafkax create_topic: {e}")))
                }
            }
        }
    }

    /// 关闭：拒绝新请求。
    pub async fn close(&self, _deadline: Duration) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub(crate) fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            Err(XError::cancelled("kafkax: pool closed"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn record_publish_ok(&self) {
        self.inner.published.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_publish_err(&self) {
        self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) async fn partition_client(
        &self,
        topic: &str,
        partition: i32,
    ) -> XResult<rskafka::client::partition::PartitionClient> {
        self.ensure_open()?;
        self.inner
            .client
            .partition_client(topic, partition, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| XError::unavailable(format!("kafkax partition_client: {e}")))
    }

    pub(crate) fn compression() -> Compression {
        Compression::NoCompression
    }
}
