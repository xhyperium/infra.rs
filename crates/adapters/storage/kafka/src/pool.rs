//! `KafkaPool`：共享生产者 + 消费会话工厂 + 健康/统计/关停。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use kernel::{XError, XResult};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{FutureProducer, Producer};
use rdkafka::util::Timeout;

use crate::config::KafkaConfig;
use crate::consumer::{ConsumerConfig, KafkaConsumer};
use crate::error_map::map_kafka_error;
use crate::producer::KafkaProducer;

/// 池统计快照。
#[derive(Debug, Clone, Copy, Default)]
pub struct KafkaPoolStats {
    /// 成功投递次数。
    pub published: u64,
    /// 投递失败次数。
    pub publish_failed: u64,
    /// 是否已关闭。
    pub closed: bool,
}

/// 健康结果。
#[derive(Debug, Clone)]
pub struct KafkaHealth {
    /// 是否就绪（metadata 可达）。
    pub ready: bool,
    /// 简要说明（无敏感信息）。
    pub detail: String,
}

/// 资源池：一个共享 `FutureProducer` + 配置模板。
#[derive(Clone)]
pub struct KafkaPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    config: KafkaConfig,
    producer: FutureProducer,
    published: Arc<AtomicU64>,
    publish_failed: Arc<AtomicU64>,
    closed: AtomicBool,
}

impl KafkaPool {
    /// 连接并创建共享生产者。
    pub async fn connect(config: KafkaConfig) -> XResult<Self> {
        config.validate()?;
        let producer: FutureProducer = build_client_config(&config, None)
            .create()
            .map_err(|e| map_kafka_error("create_producer", e))?;

        // 触发一次 metadata，尽早暴露认证/连通问题。
        let _ = tokio::task::spawn_blocking({
            let p = producer.clone();
            move || p.client().fetch_metadata(None, Timeout::After(Duration::from_secs(10)))
        })
        .await
        .map_err(|e| XError::internal(format!("kafkax metadata join: {e}")))?
        .map_err(|e| map_kafka_error("fetch_metadata", e))?;

        Ok(Self {
            inner: Arc::new(PoolInner {
                config,
                producer,
                published: Arc::new(AtomicU64::new(0)),
                publish_failed: Arc::new(AtomicU64::new(0)),
                closed: AtomicBool::new(false),
            }),
        })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(KafkaConfig::from_env()).await
    }

    /// 配置快照（密码已脱敏 Debug）。
    #[must_use]
    pub fn config(&self) -> &KafkaConfig {
        &self.inner.config
    }

    /// 共享生产者句柄。
    #[must_use]
    pub fn producer(&self) -> KafkaProducer {
        KafkaProducer {
            inner: self.inner.producer.clone(),
            delivery_timeout: self.inner.config.delivery_timeout,
            published: Arc::clone(&self.inner.published),
            failed: Arc::clone(&self.inner.publish_failed),
        }
    }

    /// 创建消费组会话。
    pub async fn consumer(&self, group: impl Into<String>) -> XResult<KafkaConsumer> {
        self.consumer_with(ConsumerConfig::new(group)).await
    }

    /// 按完整消费配置创建会话。
    pub async fn consumer_with(&self, cfg: ConsumerConfig) -> XResult<KafkaConsumer> {
        self.ensure_open()?;
        if cfg.group_id.trim().is_empty() {
            return Err(XError::invalid("kafkax: group_id 不能为空"));
        }
        let consumer: StreamConsumer = build_client_config(&self.inner.config, Some(&cfg))
            .create()
            .map_err(|e| map_kafka_error("create_consumer", e))?;
        Ok(KafkaConsumer { inner: std::sync::Arc::new(consumer), group_id: cfg.group_id })
    }

    /// 健康检查：拉取集群 metadata。
    pub async fn health(&self) -> XResult<KafkaHealth> {
        self.ensure_open()?;
        let producer = self.inner.producer.clone();
        let meta = tokio::task::spawn_blocking(move || {
            producer.client().fetch_metadata(None, Timeout::After(Duration::from_secs(5)))
        })
        .await
        .map_err(|e| XError::internal(format!("kafkax health join: {e}")))?
        .map_err(|e| map_kafka_error("health_metadata", e))?;
        let brokers = meta.brokers().len();
        Ok(KafkaHealth {
            ready: brokers > 0,
            detail: format!("brokers={brokers} topics={}", meta.topics().len()),
        })
    }

    /// 统计快照。
    #[must_use]
    pub fn stats(&self) -> KafkaPoolStats {
        KafkaPoolStats {
            published: self.inner.published.load(Ordering::Relaxed),
            publish_failed: self.inner.publish_failed.load(Ordering::Relaxed),
            closed: self.inner.closed.load(Ordering::Relaxed),
        }
    }

    /// 尽力创建 topic（幂等：已存在则忽略）。
    pub async fn ensure_topic(
        &self,
        topic: &str,
        partitions: i32,
        replication: i32,
    ) -> XResult<()> {
        self.ensure_open()?;
        if topic.is_empty() {
            return Err(XError::invalid("kafkax: topic 不能为空"));
        }
        let admin: AdminClient<_> = build_client_config(&self.inner.config, None)
            .create()
            .map_err(|e| map_kafka_error("create_admin", e))?;
        let new_topic = NewTopic::new(topic, partitions, TopicReplication::Fixed(replication));
        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(15)));
        let results = admin
            .create_topics(std::iter::once(&new_topic), &opts)
            .await
            .map_err(|e| map_kafka_error("create_topics", e))?;
        for r in results {
            match r {
                Ok(_) => {}
                Err((_, rdkafka::types::RDKafkaErrorCode::TopicAlreadyExists)) => {}
                Err((name, code)) => {
                    return Err(XError::unavailable(format!(
                        "kafkax create_topic {name}: {code:?}"
                    )));
                }
            }
        }
        Ok(())
    }

    /// 关停：标记 closed 并 flush producer。
    pub async fn close(&self, _deadline: Duration) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        let producer = self.inner.producer.clone();
        let _ = tokio::task::spawn_blocking(move || {
            producer.flush(Timeout::After(Duration::from_secs(5)))
        })
        .await;
        Ok(())
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            Err(XError::cancelled("kafkax: pool 已关闭"))
        } else {
            Ok(())
        }
    }
}

fn build_client_config(config: &KafkaConfig, consumer: Option<&ConsumerConfig>) -> ClientConfig {
    let mut c = ClientConfig::new();
    c.set("bootstrap.servers", &config.brokers);
    c.set("client.id", &config.client_id);
    c.set("security.protocol", config.security_protocol());
    c.set_log_level(RDKafkaLogLevel::Warning);

    if let Some(mech) = &config.sasl_mechanism {
        c.set("sasl.mechanisms", mech);
        if let Some(u) = &config.sasl_username {
            c.set("sasl.username", u);
        }
        if let Some(p) = &config.sasl_password {
            c.set("sasl.password", p);
        }
    }

    if let Some(cc) = consumer {
        // 仅 consumer 属性；禁止混入 acks/idempotence（会导致消费会话异常）
        c.set("group.id", &cc.group_id);
        c.set("auto.offset.reset", &cc.auto_offset_reset);
        c.set("enable.auto.commit", if cc.enable_auto_commit { "true" } else { "false" });
        c.set("session.timeout.ms", "45000");
        c.set("heartbeat.interval.ms", "3000");
        c.set("max.poll.interval.ms", "300000");
        c.set("enable.partition.eof", "false");
        c.set("api.version.request", "true");
    } else {
        // 仅 producer/admin
        c.set("acks", "all");
        c.set("enable.idempotence", "true");
        c.set("message.timeout.ms", config.delivery_timeout.as_millis().to_string());
    }

    c
}
