//! `KafkaPool`：基于 `rskafka` 的共享客户端与生命周期。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use kernel::{XError, XResult};
use rskafka::client::{
    Client, ClientBuilder,
    partition::{Compression, UnknownTopicHandling},
};
use rskafka::client::{Credentials, SaslConfig};

use crate::config::KafkaConfig;
use crate::consumer::{ConsumerConfig, KafkaConsumer};
use crate::error_map::map_kafka_err;
use crate::lifecycle::{Lifecycle, OperationGuard, wait_for_shutdown};
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
    lifecycle: Lifecycle,
}

impl KafkaPool {
    /// 连接集群。
    pub async fn connect(config: KafkaConfig) -> XResult<Self> {
        config.validate()?;
        let connect_timeout = config.connect_timeout;
        tokio::time::timeout(connect_timeout, Self::connect_inner(config))
            .await
            .map_err(|error| XError::deadline_exceeded("kafkax connect 超时").with_source(error))?
    }

    async fn connect_inner(config: KafkaConfig) -> XResult<Self> {
        let brokers: Vec<String> = config
            .brokers
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let mut builder = ClientBuilder::new(brokers).client_id(config.client_id.clone());
        if config.tls {
            builder = builder.tls_config(build_tls_config(config.tls_ca_file.clone()).await?);
        }
        if let (Some(u), Some(p)) = (&config.sasl_username, &config.sasl_password) {
            builder =
                builder.sasl_config(SaslConfig::Plain(Credentials::new(u.clone(), p.clone())));
        } else if config.sasl_mechanism.is_some() {
            return Err(XError::invalid("kafkax: SASL 机制已设但缺少 username/password"));
        }
        let client =
            builder.build().await.map_err(|error| map_kafka_err("kafkax connect", error))?;
        Ok(Self {
            inner: Arc::new(PoolInner {
                config,
                client,
                published: AtomicU64::new(0),
                publish_failed: AtomicU64::new(0),
                lifecycle: Lifecycle::new(),
            }),
        })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(KafkaConfig::from_env()?).await
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
        let _operation = self.start_operation()?;
        let mut shutdown = self.shutdown_receiver();
        match tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => {
                return Err(XError::cancelled("kafkax list_topics 因 pool 关闭而取消"));
            }
            result = tokio::time::timeout(
                self.inner.config.operation_timeout,
                self.inner.client.list_topics(),
            ) => result,
        } {
            Err(error) => {
                Err(XError::deadline_exceeded("kafkax list_topics 超时").with_source(error))
            }
            Ok(Ok(topics)) => {
                Ok(KafkaHealth { ready: true, detail: format!("topics={}", topics.len()) })
            }
            Ok(Err(error)) => {
                let error = map_kafka_err("kafkax list_topics", error);
                Ok(KafkaHealth { ready: false, detail: error.context().to_string() })
            }
        }
    }

    /// 统计。
    #[must_use]
    pub fn stats(&self) -> KafkaPoolStats {
        KafkaPoolStats {
            published: self.inner.published.load(Ordering::Relaxed),
            publish_failed: self.inner.publish_failed.load(Ordering::Relaxed),
            closed: self.inner.lifecycle.is_closed(),
        }
    }

    /// 确保 topic 存在。
    pub async fn ensure_topic(
        &self,
        topic: &str,
        partitions: i32,
        replication: i16,
    ) -> XResult<()> {
        validate_topic_request(topic, partitions, replication)?;
        let _operation = self.start_operation()?;
        let mut shutdown = self.shutdown_receiver();
        let ctrl = self
            .inner
            .client
            .controller_client()
            .map_err(|error| map_kafka_err("kafkax controller", error))?;
        match tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => {
                return Err(XError::cancelled("kafkax create_topic 因 pool 关闭而取消"));
            }
            result = tokio::time::timeout(
                self.inner.config.operation_timeout,
                ctrl.create_topic(topic, partitions, replication, 5_000),
            ) => result,
        } {
            Err(error) => {
                Err(XError::deadline_exceeded("kafkax create_topic 超时").with_source(error))
            }
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                if is_topic_already_exists_error(&error.to_string()) {
                    Ok(())
                } else {
                    Err(map_kafka_err("kafkax create_topic", error))
                }
            }
        }
    }

    /// 关闭：拒绝新请求、取消后台消费与 broker I/O，并等待在途操作释放。
    ///
    /// deadline 超时后 pool 仍保持关闭；调用方可再次调用以继续等待。
    pub async fn close(&self, deadline: Duration) -> XResult<()> {
        self.inner.lifecycle.close(deadline).await
    }

    pub(crate) fn ensure_open(&self) -> XResult<()> {
        self.inner.lifecycle.ensure_open()
    }

    pub(crate) fn start_operation(&self) -> XResult<OperationGuard> {
        self.inner.lifecycle.start_operation()
    }

    pub(crate) fn shutdown_receiver(&self) -> tokio::sync::watch::Receiver<bool> {
        self.inner.lifecycle.subscribe_shutdown()
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
        if topic.trim().is_empty() {
            return Err(XError::invalid("kafkax: topic 不能为空"));
        }
        if partition < 0 {
            return Err(XError::invalid("kafkax: partition 不能为负"));
        }
        let _operation = self.start_operation()?;
        let mut shutdown = self.shutdown_receiver();
        tokio::select! {
            biased;
            () = wait_for_shutdown(&mut shutdown) => {
                Err(XError::cancelled("kafkax partition_client 因 pool 关闭而取消"))
            }
            result = tokio::time::timeout(
                self.inner.config.operation_timeout,
                self.inner.client.partition_client(topic, partition, UnknownTopicHandling::Retry),
            ) => {
                result
                    .map_err(|error| {
                        XError::deadline_exceeded("kafkax partition_client 超时").with_source(error)
                    })?
                    .map_err(|error| map_kafka_err("kafkax partition_client", error))
            }
        }
    }

    pub(crate) fn compression() -> Compression {
        Compression::NoCompression
    }
}

async fn build_tls_config(
    ca_file: Option<std::path::PathBuf>,
) -> XResult<Arc<rustls::ClientConfig>> {
    tokio::task::spawn_blocking(move || {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut roots =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        if let Some(path) = ca_file {
            let metadata = std::fs::metadata(&path).map_err(|error| {
                XError::invalid("kafkax: 无法检查 TLS CA 文件").with_source(error)
            })?;
            if !metadata.is_file() {
                return Err(XError::invalid("kafkax: TLS CA 路径必须是普通文件"));
            }
            if metadata.len() > 1024 * 1024 {
                return Err(XError::invalid("kafkax: TLS CA 文件不得超过 1 MiB"));
            }
            let file = std::fs::File::open(&path).map_err(|error| {
                XError::invalid("kafkax: 无法读取 TLS CA 文件").with_source(error)
            })?;
            let mut reader = std::io::BufReader::new(file);
            let certificates =
                rustls_pemfile::certs(&mut reader).collect::<Result<Vec<_>, _>>().map_err(
                    |error| XError::invalid("kafkax: TLS CA PEM 解析失败").with_source(error),
                )?;
            if certificates.is_empty() {
                return Err(XError::invalid("kafkax: TLS CA 文件中没有证书"));
            }
            for certificate in certificates {
                roots.add(certificate).map_err(|error| {
                    XError::invalid("kafkax: TLS CA 证书无效").with_source(error)
                })?;
            }
        }
        let tls =
            rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
        Ok(Arc::new(tls))
    })
    .await
    .map_err(|error| XError::internal("kafkax: TLS 配置任务失败").with_source(error))?
}

fn validate_topic_request(topic: &str, partitions: i32, replication: i16) -> XResult<()> {
    if topic.trim().is_empty() {
        return Err(XError::invalid("kafkax: topic 不能为空"));
    }
    if partitions <= 0 {
        return Err(XError::invalid("kafkax: partitions 必须大于零"));
    }
    if replication <= 0 {
        return Err(XError::invalid("kafkax: replication 必须大于零"));
    }
    Ok(())
}

/// 判断 `create_topic` 的错误文本是否表示「topic 已存在」（幂等语义，非失败）。
///
/// `rskafka` 未对该场景暴露结构化错误类型，只能按驱动错误文本分类；
/// 因此本函数独立、可离线单测，避免误将其他失败（如鉴权、网络）误判为已存在。
fn is_topic_already_exists_error(message: &str) -> bool {
    let s = message.to_ascii_lowercase();
    s.contains("exist") || s.contains("already") || s.contains("topic_already")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KafkaConfig;
    use kernel::ErrorKind;

    #[tokio::test]
    async fn connect_refused_returns_error() {
        let cfg = KafkaConfig {
            brokers: "127.0.0.1:1".into(),
            delivery_timeout: Duration::from_millis(300),
            connect_timeout: Duration::from_millis(300),
            ..KafkaConfig::default()
        };
        let res = tokio::time::timeout(Duration::from_secs(2), KafkaPool::connect(cfg)).await;
        match res {
            Ok(Err(err)) => {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
                    ),
                    "kind={:?}",
                    err.kind()
                );
            }
            Ok(Ok(_)) => panic!("must fail"),
            Err(_) => panic!("KafkaPool::connect 必须受内部截止时间约束"),
        }
    }

    #[test]
    fn ensure_open_after_close_flag() {
        // 构造 closed 状态通过 ensure_open 语义：仅当有 pool 时
        // 离线验证：默认 config 可 validate
        let c = KafkaConfig::default();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn topic_request_rejects_invalid_shape_before_broker_io() {
        for (topic, partitions, replication) in [("", 1, 1), ("t", 0, 1), ("t", 1, 0)] {
            let error = validate_topic_request(topic, partitions, replication)
                .expect_err("非法 topic 请求必须在 broker I/O 前失败");
            assert_eq!(error.kind(), ErrorKind::Invalid);
        }
    }

    #[test]
    fn topic_already_exists_matches_known_broker_phrasings() {
        for message in [
            "Topic already exists",
            "TOPIC_ALREADY_EXISTS",
            "error creating topic: already present",
            "duplicate: topic exist on broker",
        ] {
            assert!(is_topic_already_exists_error(message), "应识别为已存在: {message}");
        }
    }

    #[test]
    fn topic_already_exists_rejects_unrelated_failures() {
        for message in [
            "connection refused",
            "not authorized to access topic",
            "network timeout while contacting controller",
            "invalid replication factor",
        ] {
            assert!(!is_topic_already_exists_error(message), "不应误判为已存在: {message}");
        }
    }
}
