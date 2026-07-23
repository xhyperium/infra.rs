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

use std::fmt;
use std::future::Future;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use kernel::{XError, XResult};

use crate::pool::NatsPool;

/// JetStream 上下文包装。
#[derive(Clone)]
pub struct JetStream {
    context: async_nats::jetstream::Context,
    operation_timeout: Duration,
}

/// Pull consumer 配置（最小字段）。
#[derive(Debug, Clone)]
pub struct PullConsumerConfig {
    /// durable 名（亦作 consumer name）。
    pub durable_name: String,
    /// 可选 filter subject。
    pub filter_subject: Option<String>,
}

/// 可持久消费的 JetStream consumer 配置。
///
/// 与旧 [`PullConsumerConfig`] 分离，避免给公开结构体追加字段而破坏下游字面量构造。
#[derive(Debug, Clone)]
pub struct JetStreamConsumerConfig {
    /// durable 名（亦作 consumer name）。
    pub durable_name: String,
    /// 可选 filter subject。
    pub filter_subject: Option<String>,
    /// 未确认消息的重投等待时间。
    pub ack_wait: Duration,
    /// 单条消息最多投递次数；达到上限不会自动进入 DLQ。
    pub max_deliver: i64,
    /// consumer 允许的最大未确认消息数。
    pub max_ack_pending: i64,
    /// ack/nak/progress/term 等 broker 指令的调用侧截止时间。
    pub command_timeout: Duration,
}

impl JetStreamConsumerConfig {
    /// 以保守的显式确认默认值创建 durable consumer 配置。
    #[must_use]
    pub fn durable(name: impl Into<String>) -> Self {
        Self {
            durable_name: name.into(),
            filter_subject: None,
            ack_wait: Duration::from_secs(30),
            max_deliver: 5,
            max_ack_pending: 1_024,
            command_timeout: Duration::from_secs(5),
        }
    }

    fn validate(&self) -> XResult<()> {
        validate_consumer_name(&self.durable_name)?;
        if self.ack_wait.is_zero() {
            return Err(XError::invalid("natsx jetstream: ack_wait 必须大于零"));
        }
        if self.max_deliver <= 0 {
            return Err(XError::invalid("natsx jetstream: max_deliver 必须大于零"));
        }
        if self.max_ack_pending <= 0 {
            return Err(XError::invalid("natsx jetstream: max_ack_pending 必须大于零"));
        }
        if self.command_timeout.is_zero() {
            return Err(XError::invalid("natsx jetstream: command_timeout 必须大于零"));
        }
        if self.filter_subject.as_ref().is_some_and(|subject| subject.trim().is_empty()) {
            return Err(XError::invalid("natsx jetstream: filter_subject 不能为空"));
        }
        Ok(())
    }
}

/// JetStream 持久 pull consumer。
#[derive(Clone)]
pub struct JetStreamConsumer {
    inner: async_nats::jetstream::consumer::Consumer<async_nats::jetstream::consumer::pull::Config>,
    command_timeout: Duration,
}

impl fmt::Debug for JetStreamConsumer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JetStreamConsumer")
            .field("stream", &self.inner.cached_info().stream_name)
            .field("name", &self.inner.cached_info().name)
            .finish_non_exhaustive()
    }
}

/// 一次 JetStream 投递的稳定元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JetStreamDeliveryMetadata {
    /// stream 名。
    pub stream: String,
    /// consumer 名。
    pub consumer: String,
    /// stream 内序号；重投时保持不变。
    pub stream_sequence: u64,
    /// consumer 投递序号。
    pub consumer_sequence: u64,
    /// 当前消息已投递次数。
    pub delivery_attempts: u64,
    /// 服务端报告的待投递数。
    pub pending: u64,
}

/// 一次可显式确认的 JetStream 投递。
///
/// `Debug` 故意不输出 payload 和底层消息，避免业务数据进入日志。
pub struct JetStreamDelivery {
    subject: String,
    payload: Bytes,
    metadata: JetStreamDeliveryMetadata,
    raw: async_nats::jetstream::Message,
    command_timeout: Duration,
}

impl fmt::Debug for JetStreamDelivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JetStreamDelivery")
            .field("subject", &self.subject)
            .field("payload_len", &self.payload.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}

async fn run_bounded_command<T, F>(
    timeout: Duration,
    operation: &'static str,
    command: F,
) -> XResult<T>
where
    F: Future<Output = XResult<T>>,
{
    tokio::time::timeout(timeout, command).await.map_err(|error| {
        XError::deadline_exceeded(format!("natsx jetstream: {operation} 超时")).with_source(error)
    })?
}

impl JetStreamDelivery {
    fn from_raw(raw: async_nats::jetstream::Message, command_timeout: Duration) -> XResult<Self> {
        let info = raw.info().map_err(|error| {
            XError::unavailable("natsx jetstream: 投递缺少合法的 JetStream 元数据")
                .with_source(error)
        })?;
        let delivery_attempts = u64::try_from(info.delivered).map_err(|error| {
            XError::unavailable("natsx jetstream: delivery attempts 不能为负").with_source(error)
        })?;
        let metadata = JetStreamDeliveryMetadata {
            stream: info.stream.to_string(),
            consumer: info.consumer.to_string(),
            stream_sequence: info.stream_sequence,
            consumer_sequence: info.consumer_sequence,
            delivery_attempts,
            pending: info.pending,
        };
        Ok(Self {
            subject: raw.subject.to_string(),
            payload: raw.payload.clone(),
            metadata,
            raw,
            command_timeout,
        })
    }

    /// 原始 subject。
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// 消息 payload。
    #[must_use]
    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    /// 稳定、已复制的投递元数据。
    #[must_use]
    pub fn metadata(&self) -> &JetStreamDeliveryMetadata {
        &self.metadata
    }

    /// 异步发送确认；消费 `self`，避免同一句柄重复终结。
    ///
    /// # Errors
    ///
    /// broker 连接不可用或确认发送失败时返回 `Unavailable`；超过配置的
    /// `command_timeout` 时返回 `DeadlineExceeded`。
    pub async fn ack(self) -> XResult<()> {
        run_bounded_command(self.command_timeout, "ack", async move {
            self.raw.ack().await.map_err(|error| {
                XError::unavailable("natsx jetstream: ack 发送失败").with_source(error)
            })
        })
        .await
    }

    /// 发送确认并等待服务端确认；消费 `self`。
    ///
    /// # Errors
    ///
    /// broker 未确认或连接不可用时返回 `Unavailable`；超过配置的
    /// `command_timeout` 时返回 `DeadlineExceeded`。
    pub async fn double_ack(self) -> XResult<()> {
        run_bounded_command(self.command_timeout, "double_ack", async move {
            self.raw.double_ack().await.map_err(|error| {
                XError::unavailable("natsx jetstream: double_ack 失败").with_source(error)
            })
        })
        .await
    }

    /// 请求重投，可选延迟；消费 `self`。
    ///
    /// # Errors
    ///
    /// broker 连接不可用或 NAK 发送失败时返回 `Unavailable`；超过配置的
    /// `command_timeout` 时返回 `DeadlineExceeded`。
    pub async fn nak(self, delay: Option<Duration>) -> XResult<()> {
        run_bounded_command(self.command_timeout, "nak", async move {
            self.raw.ack_with(async_nats::jetstream::AckKind::Nak(delay)).await.map_err(|error| {
                XError::unavailable("natsx jetstream: nak 发送失败").with_source(error)
            })
        })
        .await
    }

    /// 通知服务端处理仍在进行，延长 ack wait。
    ///
    /// # Errors
    ///
    /// broker 连接不可用或 progress 发送失败时返回 `Unavailable`；超过配置的
    /// `command_timeout` 时返回 `DeadlineExceeded`。
    pub async fn progress(&self) -> XResult<()> {
        run_bounded_command(self.command_timeout, "progress", async {
            self.raw.ack_with(async_nats::jetstream::AckKind::Progress).await.map_err(|error| {
                XError::unavailable("natsx jetstream: progress 发送失败").with_source(error)
            })
        })
        .await
    }

    /// 终止该消息后续重投；消费 `self`。
    ///
    /// `term` **不是 DLQ**，不会自动把 payload 发布到隔离 subject。
    ///
    /// # Errors
    ///
    /// broker 连接不可用或终止指令发送失败时返回 `Unavailable`；超过配置的
    /// `command_timeout` 时返回 `DeadlineExceeded`。
    pub async fn term(self) -> XResult<()> {
        run_bounded_command(self.command_timeout, "term", async move {
            self.raw.ack_with(async_nats::jetstream::AckKind::Term).await.map_err(|error| {
                XError::unavailable("natsx jetstream: term 发送失败").with_source(error)
            })
        })
        .await
    }
}

impl JetStreamConsumer {
    /// 有限等待下一条消息。
    ///
    /// 服务端 fetch expiry 正常结束返回 `Ok(None)`；broker/协议错误返回 `Err`。
    /// 外层客户端超时比服务端 expiry 多一秒，仅用于阻止连接异常时无限挂起。
    ///
    /// # Errors
    ///
    /// `timeout` 为零、fetch 创建失败、broker 返回错误或服务端未按期终止时返回错误。
    pub async fn next_timeout(&self, timeout: Duration) -> XResult<Option<JetStreamDelivery>> {
        if timeout.is_zero() {
            return Err(XError::invalid("natsx jetstream: fetch timeout 必须大于零"));
        }
        let client_deadline = timeout.saturating_add(Duration::from_secs(1));
        let batch = self.inner.fetch().max_messages(1).expires(timeout).messages();
        let mut batch = tokio::time::timeout(client_deadline, batch)
            .await
            .map_err(|_| XError::deadline_exceeded("natsx jetstream: 创建有限 fetch 超时"))?
            .map_err(|error| {
                XError::unavailable("natsx jetstream: 创建有限 fetch 失败").with_source(error)
            })?;
        match tokio::time::timeout(client_deadline, batch.next()).await {
            Ok(Some(Ok(message))) => {
                JetStreamDelivery::from_raw(message, self.command_timeout).map(Some)
            }
            Ok(Some(Err(error))) => {
                Err(XError::unavailable("natsx jetstream: 拉取消息失败").with_source(error))
            }
            Ok(None) => Ok(None),
            Err(_) => Err(XError::deadline_exceeded(
                "natsx jetstream: 服务端未在 fetch expiry 后终止批次",
            )),
        }
    }
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
        Self {
            context: async_nats::jetstream::new(pool.client()),
            operation_timeout: pool.config().operation_timeout,
        }
    }

    /// 从裸 client 构造。
    #[must_use]
    pub fn from_client(client: async_nats::Client) -> Self {
        Self {
            context: async_nats::jetstream::new(client),
            operation_timeout: Duration::from_secs(5),
        }
    }

    /// 底层 context。
    #[must_use]
    pub fn context(&self) -> &async_nats::jetstream::Context {
        &self.context
    }

    /// 覆盖 JetStream 管理与发布操作的调用侧截止时间。
    ///
    /// # Errors
    ///
    /// `timeout` 为零时返回 `Invalid`。
    pub fn with_operation_timeout(mut self, timeout: Duration) -> XResult<Self> {
        validate_operation_timeout(timeout)?;
        self.operation_timeout = timeout;
        Ok(self)
    }

    /// 发布并等待 JetStream ack。
    pub async fn publish(&self, subject: &str, payload: Bytes) -> XResult<()> {
        if subject.trim().is_empty() {
            return Err(XError::invalid("natsx jetstream: subject 不能为空"));
        }
        let ack = run_bounded_command(self.operation_timeout, "publish", async {
            self.context.publish(subject.to_string(), payload).await.map_err(|error| {
                XError::unavailable("natsx jetstream publish 失败").with_source(error)
            })
        })
        .await?;
        run_bounded_command(self.operation_timeout, "publish ack", async {
            ack.await.map(|_| ()).map_err(|error| {
                XError::unavailable("natsx jetstream publish ack 失败").with_source(error)
            })
        })
        .await?;
        Ok(())
    }

    /// 创建或获取 stream。
    pub async fn get_or_create_stream(&self, cfg: StreamConfig) -> XResult<()> {
        validate_stream_name(&cfg.name)?;
        if cfg.subjects.is_empty() {
            return Err(XError::invalid("natsx jetstream: stream subjects 不能为空"));
        }
        if cfg.max_messages <= 0 {
            return Err(XError::invalid("natsx jetstream: max_messages 必须大于零"));
        }
        let js_cfg = async_nats::jetstream::stream::Config {
            name: cfg.name,
            subjects: cfg.subjects,
            max_messages: cfg.max_messages,
            ..Default::default()
        };
        run_bounded_command(self.operation_timeout, "get_or_create_stream", async {
            self.context.get_or_create_stream(js_cfg).await.map(|_| ()).map_err(|error| {
                XError::unavailable("natsx jetstream get_or_create_stream 失败").with_source(error)
            })
        })
        .await?;
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
        run_bounded_command(self.operation_timeout, "create_pull_consumer", async {
            self.context.create_consumer_on_stream(pull, stream).await.map(|_| ()).map_err(
                |error| {
                    XError::unavailable("natsx jetstream create_pull_consumer 失败")
                        .with_source(error)
                },
            )
        })
        .await?;
        Ok(())
    }

    /// 创建或更新显式确认的 durable consumer，并返回稳定消费面。
    ///
    /// # Errors
    ///
    /// stream/consumer 配置非法或 broker 创建 consumer 失败时返回错误。
    pub async fn consumer(
        &self,
        stream: &str,
        cfg: JetStreamConsumerConfig,
    ) -> XResult<JetStreamConsumer> {
        validate_stream_name(stream)?;
        cfg.validate()?;
        let command_timeout = cfg.command_timeout;
        let pull = async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(cfg.durable_name),
            filter_subject: cfg.filter_subject.unwrap_or_default(),
            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
            ack_wait: cfg.ack_wait,
            max_deliver: cfg.max_deliver,
            max_ack_pending: cfg.max_ack_pending,
            ..Default::default()
        };
        let create = self.context.create_consumer_on_stream(pull, stream);
        let inner = tokio::time::timeout(command_timeout, create)
            .await
            .map_err(|error| {
                XError::deadline_exceeded("natsx jetstream: 创建持久 consumer 超时")
                    .with_source(error)
            })?
            .map_err(|error| {
                XError::unavailable("natsx jetstream: 创建持久 consumer 失败").with_source(error)
            })?;
        Ok(JetStreamConsumer { inner, command_timeout })
    }

    /// 获取已有 pull consumer 的底层句柄（高级逃生口）。
    ///
    /// 普通调用方应使用 [`Self::consumer`]，由稳定包装面统一有限等待和确认语义。
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
        let stream_handle = run_bounded_command(self.operation_timeout, "get_stream", async {
            self.context.get_stream(stream).await.map_err(|error| {
                XError::unavailable("natsx jetstream get_stream 失败").with_source(error)
            })
        })
        .await?;
        run_bounded_command(self.operation_timeout, "get_consumer", async {
            stream_handle.get_consumer(consumer).await.map_err(|error| {
                XError::unavailable("natsx jetstream get_consumer 失败").with_source(error)
            })
        })
        .await
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

/// 校验 JetStream 操作截止时间：必须严格大于零（fail-closed）。
pub fn validate_operation_timeout(timeout: Duration) -> XResult<()> {
    if timeout.is_zero() {
        return Err(XError::invalid("natsx jetstream: operation_timeout 必须大于零"));
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
        for invalid in ["", "bad.name", "bad*name", "bad>name", "has space"] {
            let error = validate_stream_name(invalid).expect_err("非法 stream 名必须失败");
            assert_eq!(error.kind(), ErrorKind::Invalid);
        }
    }

    #[test]
    fn operation_timeout_zero_is_rejected_offline() {
        let err = validate_operation_timeout(Duration::ZERO).expect_err("零 timeout 必须拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert!(err.context().contains("operation_timeout"));
        validate_operation_timeout(Duration::from_millis(1)).expect("正 timeout 必须通过");
    }

    #[test]
    fn validate_consumer_name_rejects_wildcards_like_stream() {
        for invalid in ["", "a.b", "x*y", "a>b", "has space"] {
            let err = validate_consumer_name(invalid).expect_err("非法 consumer 名");
            assert_eq!(err.kind(), ErrorKind::Invalid);
        }
        validate_consumer_name("worker_1").expect("合法 consumer");
    }

    #[test]
    fn config_types_constructible_offline() {
        let sc = StreamConfig::new("ORDERS", "orders.>");
        assert_eq!(sc.name, "ORDERS");
        assert_eq!(sc.subjects, vec!["orders.>".to_string()]);
        assert_eq!(sc.max_messages, 10_000);

        let invalid_stream = StreamConfig { max_messages: 0, ..sc.clone() };
        // 无 client 时无法调用管理面；结构值在调用前由 get_or_create_stream 拒绝。
        assert_eq!(invalid_stream.max_messages, 0);

        let pc = PullConsumerConfig::durable("worker-1");
        assert_eq!(pc.durable_name, "worker-1");
        assert!(pc.filter_subject.is_none());

        let pc2 = PullConsumerConfig {
            durable_name: "w2".into(),
            filter_subject: Some("orders.created".into()),
        };
        assert_eq!(pc2.filter_subject.as_deref(), Some("orders.created"));

        let durable = JetStreamConsumerConfig::durable("worker-3");
        assert_eq!(durable.max_deliver, 5);
        assert_eq!(durable.max_ack_pending, 1_024);
        assert_eq!(durable.command_timeout, Duration::from_secs(5));
        durable.validate().expect("默认持久消费者配置应有效");
    }

    #[test]
    fn durable_consumer_config_rejects_unbounded_values() {
        let mut cfg = JetStreamConsumerConfig::durable("worker");
        cfg.ack_wait = Duration::ZERO;
        let ack_wait = cfg.validate().expect_err("零 ack_wait 必须失败");
        assert_eq!(ack_wait.kind(), ErrorKind::Invalid);
        cfg.ack_wait = Duration::from_secs(1);
        cfg.max_deliver = 0;
        let max_deliver = cfg.validate().expect_err("零 max_deliver 必须失败");
        assert_eq!(max_deliver.kind(), ErrorKind::Invalid);
        cfg.max_deliver = 1;
        cfg.max_ack_pending = 0;
        let max_ack_pending = cfg.validate().expect_err("零 max_ack_pending 必须失败");
        assert_eq!(max_ack_pending.kind(), ErrorKind::Invalid);
        cfg.max_ack_pending = 1;
        cfg.command_timeout = Duration::ZERO;
        let command_timeout = cfg.validate().expect_err("零 command_timeout 必须失败");
        assert_eq!(command_timeout.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn broker_command_timeout_maps_to_deadline_exceeded() {
        let pending = std::future::pending::<XResult<()>>();
        let error = run_bounded_command(Duration::from_millis(1), "测试指令", pending)
            .await
            .expect_err("挂起指令必须按截止时间失败");
        assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
        assert!(std::error::Error::source(&error).is_some());
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
            Ok(Err(err)) => assert!(
                matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
                "kind={:?}",
                err.kind()
            ),
            Ok(Ok(_)) => panic!("无服务端时必须连接失败"),
            Err(_) => panic!("连接拒绝路径不得超出测试硬超时"),
        }

        // 离线：validate_stream_name 在无服务器时仍可用
        assert!(validate_stream_name("offline_ok").is_ok());
        assert!(validate_consumer_name("c1").is_ok());
        let consumer_error = validate_consumer_name("").expect_err("空 consumer 名必须失败");
        assert_eq!(consumer_error.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn from_client_type_constructible_without_io() {
        // 仅证明 JetStream 类型在编译期存在；不发起网络
        // Client 无法无连接构造；此处只检查 StreamConfig/PullConsumerConfig
        let _ = StreamConfig { name: "S".into(), subjects: vec!["s".into()], max_messages: 1 };
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StreamConfig>();
        assert_send_sync::<PullConsumerConfig>();
        assert_send_sync::<JetStreamConsumerConfig>();
        assert_send_sync::<JetStreamConsumer>();
        assert_send_sync::<JetStreamDelivery>();
    }

    #[tokio::test]
    async fn generic_bounded_command_preserves_value_and_timeout_source() {
        let value = run_bounded_command(Duration::from_secs(1), "返回值", async {
            Ok::<_, XError>(42u8)
        })
        .await
        .expect("返回值");
        assert_eq!(value, 42);

        let error = run_bounded_command(
            Duration::from_millis(1),
            "挂起返回值",
            std::future::pending::<XResult<u8>>(),
        )
        .await
        .expect_err("必须超时");
        assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
        assert!(std::error::Error::source(&error).is_some());
    }
}
