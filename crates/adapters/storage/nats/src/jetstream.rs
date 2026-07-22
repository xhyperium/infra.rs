//! JetStream 薄封装（`async_nats::jetstream`）。
//!
//! # 范围
//!
//! - 从 [`NatsPool`] 客户端构造 [`JetStream`] 上下文
//! - `publish`（等待 ack）
//! - pull consumer 创建 / 获取
//! - stream 名校验（离线可测）
//!
//! 完整 Cluster / 跨账户 / 对象存储 **不在** 本版本稳定承诺内。

use bytes::Bytes;
use kernel::{XError, XResult};

use crate::pool::NatsPool;

/// JetStream 上下文包装。
#[derive(Clone)]
pub struct JetStream {
    context: async_nats::jetstream::Context,
}

/// Pull consumer 配置（最小字段）。
#[derive(Debug, Clone)]
pub struct PullConsumerConfig {
    /// durable 名（亦作 consumer name）。
    pub durable_name: String,
    /// 可选 filter subject。
    pub filter_subject: Option<String>,
}

impl PullConsumerConfig {
    /// 仅 durable 名。
    #[must_use]
    pub fn durable(name: impl Into<String>) -> Self {
        Self { durable_name: name.into(), filter_subject: None }
    }
}

/// Stream 创建配置（最小字段）。
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// stream 名。
    pub name: String,
    /// 绑定 subjects。
    pub subjects: Vec<String>,
    /// 最大消息数（0 = 服务端默认）。
    pub max_messages: i64,
}

impl StreamConfig {
    /// 单 subject stream。
    #[must_use]
    pub fn new(name: impl Into<String>, subject: impl Into<String>) -> Self {
        Self { name: name.into(), subjects: vec![subject.into()], max_messages: 10_000 }
    }
}

impl JetStream {
    /// 从已连接的 pool 构造。
    #[must_use]
    pub fn from_pool(pool: &NatsPool) -> Self {
        Self { context: async_nats::jetstream::new(pool.client()) }
    }

    /// 从裸 client 构造。
    #[must_use]
    pub fn from_client(client: async_nats::Client) -> Self {
        Self { context: async_nats::jetstream::new(client) }
    }

    /// 底层 context。
    #[must_use]
    pub fn context(&self) -> &async_nats::jetstream::Context {
        &self.context
    }

    /// 发布并等待 JetStream ack。
    pub async fn publish(&self, subject: &str, payload: Bytes) -> XResult<()> {
        if subject.trim().is_empty() {
            return Err(XError::invalid("natsx jetstream: subject 不能为空"));
        }
        let ack = self
            .context
            .publish(subject.to_string(), payload)
            .await
            .map_err(|e| XError::unavailable(format!("natsx jetstream publish: {e}")))?;
        ack.await.map_err(|e| XError::unavailable(format!("natsx jetstream publish ack: {e}")))?;
        Ok(())
    }

    /// 创建或获取 stream。
    pub async fn get_or_create_stream(&self, cfg: StreamConfig) -> XResult<()> {
        validate_stream_name(&cfg.name)?;
        if cfg.subjects.is_empty() {
            return Err(XError::invalid("natsx jetstream: stream subjects 不能为空"));
        }
        let js_cfg = async_nats::jetstream::stream::Config {
            name: cfg.name,
            subjects: cfg.subjects,
            max_messages: cfg.max_messages,
            ..Default::default()
        };
        self.context.get_or_create_stream(js_cfg).await.map_err(|e| {
            XError::unavailable(format!("natsx jetstream get_or_create_stream: {e}"))
        })?;
        Ok(())
    }

    /// 在 stream 上创建 / 更新 pull consumer（durable）。
    pub async fn create_pull_consumer(&self, stream: &str, cfg: PullConsumerConfig) -> XResult<()> {
        validate_stream_name(stream)?;
        if cfg.durable_name.trim().is_empty() {
            return Err(XError::invalid("natsx jetstream: durable_name 不能为空"));
        }
        let mut pull = async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(cfg.durable_name.clone()),
            ..Default::default()
        };
        if let Some(fs) = cfg.filter_subject {
            pull.filter_subject = fs;
        }
        self.context.create_consumer_on_stream(pull, stream).await.map_err(|e| {
            XError::unavailable(format!("natsx jetstream create_pull_consumer: {e}"))
        })?;
        Ok(())
    }

    /// 获取已有 pull consumer 并返回是否存在（高级调用方自行 messages()）。
    pub async fn get_pull_consumer(
        &self,
        stream: &str,
        consumer: &str,
    ) -> XResult<
        async_nats::jetstream::consumer::Consumer<async_nats::jetstream::consumer::pull::Config>,
    > {
        validate_stream_name(stream)?;
        if consumer.trim().is_empty() {
            return Err(XError::invalid("natsx jetstream: consumer 名不能为空"));
        }
        let stream_handle = self
            .context
            .get_stream(stream)
            .await
            .map_err(|e| XError::unavailable(format!("natsx jetstream get_stream: {e}")))?;
        stream_handle
            .get_consumer(consumer)
            .await
            .map_err(|e| XError::unavailable(format!("natsx jetstream get_consumer: {e}")))
    }
}

/// 校验 JetStream stream 名（对齐 async-nats：非空、无空白、无 `.` `*` `>`）。
pub fn validate_stream_name(name: &str) -> XResult<()> {
    if name.is_empty() {
        return Err(XError::invalid("natsx jetstream: stream 名不能为空"));
    }
    let ok = name.bytes().all(|c| !c.is_ascii_whitespace() && c != b'.' && c != b'*' && c != b'>');
    if !ok {
        return Err(XError::invalid(format!(
            "natsx jetstream: stream 名非法（禁止空白、点号、*、>）: {name}"
        )));
    }
    Ok(())
}

/// 校验 durable / consumer 名（同 stream 规则）。
pub fn validate_consumer_name(name: &str) -> XResult<()> {
    validate_stream_name(name)
        .map_err(|e| XError::invalid(e.context().replace("stream 名", "consumer 名")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NatsConfig;
    use kernel::ErrorKind;
    use std::time::Duration;

    #[test]
    fn stream_name_validation() {
        assert!(validate_stream_name("events").is_ok());
        assert!(validate_stream_name("EV_1").is_ok());
        assert!(validate_stream_name("").is_err());
        assert!(validate_stream_name("bad.name").is_err());
        assert!(validate_stream_name("bad*name").is_err());
        assert!(validate_stream_name("bad>name").is_err());
        assert!(validate_stream_name("has space").is_err());
    }

    #[test]
    fn config_types_constructible_offline() {
        let sc = StreamConfig::new("ORDERS", "orders.>");
        assert_eq!(sc.name, "ORDERS");
        assert_eq!(sc.subjects, vec!["orders.>".to_string()]);
        assert_eq!(sc.max_messages, 10_000);

        let pc = PullConsumerConfig::durable("worker-1");
        assert_eq!(pc.durable_name, "worker-1");
        assert!(pc.filter_subject.is_none());

        let pc2 = PullConsumerConfig {
            durable_name: "w2".into(),
            filter_subject: Some("orders.created".into()),
        };
        assert_eq!(pc2.filter_subject.as_deref(), Some("orders.created"));
    }

    #[test]
    fn jetstream_flag_on_nats_config() {
        let mut c = NatsConfig::default();
        assert!(!c.jetstream);
        c.jetstream = true;
        assert!(c.jetstream);
        assert!(c.validate().is_ok());
    }

    #[tokio::test]
    async fn jetstream_ops_fail_when_server_missing() {
        // 连接拒绝端口：Core connect 即失败；从失败路径验证类型可调用
        let cfg = NatsConfig {
            url: "nats://127.0.0.1:1".into(),
            connect_timeout: Duration::from_millis(200),
            jetstream: true,
            ..NatsConfig::default()
        };
        let res = tokio::time::timeout(Duration::from_secs(2), NatsPool::connect(cfg)).await;
        match res {
            Ok(Err(err)) => assert_eq!(err.kind(), ErrorKind::Unavailable),
            Ok(Ok(_)) => panic!("must fail without server"),
            Err(_) => {
                // 超时也算连接失败路径已覆盖
            }
        }

        // 离线：validate_stream_name 在无服务器时仍可用
        assert!(validate_stream_name("offline_ok").is_ok());
        assert!(validate_consumer_name("c1").is_ok());
        assert!(validate_consumer_name("").is_err());
    }

    #[test]
    fn from_client_type_constructible_without_io() {
        // 仅证明 JetStream 类型在编译期存在；不发起网络
        // Client 无法无连接构造；此处只检查 StreamConfig/PullConsumerConfig
        let _ = StreamConfig { name: "S".into(), subjects: vec!["s".into()], max_messages: 1 };
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StreamConfig>();
        assert_send_sync::<PullConsumerConfig>();
    }
}
