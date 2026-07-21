//! 可选 Redis Pub/Sub（feature `pubsub`）：独占连接，不占用命令 lane。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, PubSub};
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use kernel::{XError, XResult};

use crate::config::RedisConfig;
use crate::error_map::map_redis_result;

/// 专用 Pub/Sub 会话。
///
/// Drop 时底层 stream 任务随之结束；**不**提供可靠投递保证。
pub struct RedisPubSub {
    /// 用于 publish 的独立 ConnectionManager（与 subscribe 连接分离）。
    publish_conn: redis::aio::ConnectionManager,
    /// 已订阅 channel 的消息流（split 后）。
    stream: Option<redis::aio::PubSubStream>,
    seq: Arc<AtomicU64>,
    endpoint: String,
}

impl std::fmt::Debug for RedisPubSub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPubSub").field("endpoint", &self.endpoint).finish_non_exhaustive()
    }
}

impl RedisPubSub {
    /// 从环境/默认配置建立 PubSub 并订阅给定频道。
    pub async fn connect(
        endpoint_display: String,
        channels: impl IntoIterator<Item = String>,
    ) -> XResult<Self> {
        let cfg = RedisConfig::from_env().or_else(|_| RedisConfig::builder().build())?;
        Self::connect_with_config(cfg, endpoint_display, channels).await
    }

    /// 使用显式配置连接。
    pub async fn connect_with_config(
        cfg: RedisConfig,
        endpoint_display: String,
        channels: impl IntoIterator<Item = String>,
    ) -> XResult<Self> {
        if cfg.tls() {
            return Err(XError::invalid("PubSub TLS 在当前构建不可用"));
        }
        let info = cfg.to_connection_info()?;
        let client = redis::Client::open(info)
            .map_err(|e| XError::unavailable(format!("redis pubsub client: {e}")))?;

        let publish_conn = redis::aio::ConnectionManager::new(client.clone())
            .await
            .map_err(|e| XError::unavailable(format!("redis pubsub publish 连接失败: {e}")))?;

        let mut pubsub = client
            .get_async_pubsub()
            .await
            .map_err(|e| XError::unavailable(format!("redis pubsub 连接失败: {e}")))?;

        let channels: Vec<String> = channels.into_iter().collect();
        for ch in &channels {
            map_redis_result(pubsub.subscribe(ch).await)?;
        }

        let stream = pubsub.into_on_message();
        Ok(Self {
            publish_conn,
            stream: Some(stream),
            seq: Arc::new(AtomicU64::new(0)),
            endpoint: endpoint_display,
        })
    }

    /// 脱敏端点。
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// 发布消息。
    pub async fn publish(&self, channel: &str, payload: &[u8]) -> XResult<()> {
        let mut conn = self.publish_conn.clone();
        let _: i64 = map_redis_result(
            redis::cmd("PUBLISH").arg(channel).arg(payload).query_async(&mut conn).await,
        )?;
        Ok(())
    }

    /// 取出消息流（只能调用一次）。
    pub fn into_message_stream(mut self) -> XResult<BoxStream<'static, BusMessage>> {
        let stream =
            self.stream.take().ok_or_else(|| XError::invariant("PubSub stream 已被取走"))?;
        let seq = Arc::clone(&self.seq);
        let mapped = stream.filter_map(move |msg| {
            let seq = Arc::clone(&seq);
            async move {
                let id = seq.fetch_add(1, Ordering::Relaxed).to_string();
                let payload = Bytes::copy_from_slice(msg.get_payload_bytes());
                Some(BusMessage { id, payload })
            }
        });
        Ok(Box::pin(mapped))
    }
}

/// 轻量 `PubSub` 适配：每次 `sub_channel` 新建会话；`pub_message` 复用 publish 连接。
///
/// 适用于合同层接线；高扇出场景应直接持有 [`RedisPubSub`]。
pub struct RedisPubSubFacade {
    cfg: RedisConfig,
    publish: redis::aio::ConnectionManager,
    endpoint: String,
    seq: Arc<AtomicU64>,
}

impl RedisPubSubFacade {
    /// 从配置建立 facade。
    pub async fn connect(cfg: RedisConfig) -> XResult<Self> {
        let endpoint = cfg.display_endpoint();
        let info = cfg.to_connection_info()?;
        let client = redis::Client::open(info)
            .map_err(|e| XError::unavailable(format!("redis pubsub facade: {e}")))?;
        let publish = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| XError::unavailable(format!("redis pubsub publish: {e}")))?;
        Ok(Self { cfg, publish, endpoint, seq: Arc::new(AtomicU64::new(0)) })
    }

    /// 脱敏端点。
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[async_trait]
impl PubSub for RedisPubSubFacade {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()> {
        let mut conn = self.publish.clone();
        let _: i64 = map_redis_result(
            redis::cmd("PUBLISH").arg(channel).arg(msg.as_ref()).query_async(&mut conn).await,
        )?;
        Ok(())
    }

    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let mut session = RedisPubSub::connect_with_config(
            self.cfg.clone(),
            self.endpoint.clone(),
            [channel.to_owned()],
        )
        .await?;
        // 使用 facade 级序号，保证进程内单调
        session.seq = Arc::clone(&self.seq);
        session.into_message_stream()
    }
}
