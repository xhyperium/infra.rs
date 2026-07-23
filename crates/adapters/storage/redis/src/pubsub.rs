//! 可选 Redis Pub/Sub（feature `pubsub`）：独占连接，不占用命令 lane。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, PubSub};
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use kernel::{XError, XResult};
use tokio::time::timeout;

use crate::config::{RedisConfig, RedisMode};
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
    /// 旧兼容入口：不再从环境变量重建配置。
    ///
    /// 此签名缺少认证、TLS 与拓扑信息，因此现在安全失败。请迁移到
    /// [`Self::connect_config`] 或 [`RedisPool::subscribe`](crate::RedisPool::subscribe)。
    #[deprecated(
        since = "0.3.3",
        note = "缺少安全配置，改用 RedisPubSub::connect_config 或 RedisPool::subscribe"
    )]
    pub async fn connect(
        _endpoint_display: String,
        _channels: impl IntoIterator<Item = String>,
    ) -> XResult<Self> {
        Err(XError::invalid("RedisPubSub::connect 缺少 ACL/TLS/拓扑配置；请改用 connect_config"))
    }

    /// 旧显式配置入口；`endpoint_display` 被忽略，端点必须由配置安全派生。
    #[deprecated(since = "0.3.3", note = "改用 RedisPubSub::connect_config")]
    pub async fn connect_with_config(
        cfg: RedisConfig,
        _endpoint_display: String,
        channels: impl IntoIterator<Item = String>,
    ) -> XResult<Self> {
        Self::connect_config(cfg, channels).await
    }

    /// 使用显式配置建立 Pub/Sub 并订阅给定频道。
    ///
    /// 认证、TLS 与端点均来自该配置；本方法不会重新读取环境变量。
    /// 当前仅支持 Standalone。Cluster / Sentinel 会在任何网络 I/O 前失败，避免静默
    /// 降级到错误节点或绕过池的拓扑与安全配置。
    pub async fn connect_config(
        cfg: RedisConfig,
        channels: impl IntoIterator<Item = String>,
    ) -> XResult<Self> {
        let endpoint = cfg.display_endpoint();
        let info = pubsub_connection_info(&cfg)?;
        let client = redis::Client::open(info)
            .map_err(|e| XError::unavailable(format!("redis PubSub 客户端创建失败: {e}")))?;

        let manager_config = redis::aio::ConnectionManagerConfig::new()
            .set_connection_timeout(cfg.connect_timeout())
            .set_response_timeout(cfg.command_timeout());
        let publish_conn = timeout(
            cfg.connect_timeout(),
            redis::aio::ConnectionManager::new_with_config(client.clone(), manager_config),
        )
        .await
        .map_err(|_| XError::deadline_exceeded("redis pubsub publish 连接超时"))?
        .map_err(|e| XError::unavailable(format!("redis pubsub publish 连接失败: {e}")))?;

        let mut pubsub = timeout(cfg.connect_timeout(), client.get_async_pubsub())
            .await
            .map_err(|_| XError::deadline_exceeded("redis pubsub 订阅连接超时"))?
            .map_err(|e| XError::unavailable(format!("redis pubsub 连接失败: {e}")))?;

        let channels: Vec<String> = channels.into_iter().collect();
        for ch in &channels {
            let result = timeout(cfg.command_timeout(), pubsub.subscribe(ch))
                .await
                .map_err(|_| XError::deadline_exceeded("redis pubsub 订阅命令超时"))?;
            map_redis_result(result)?;
        }

        let stream = pubsub.into_on_message();
        Ok(Self { publish_conn, stream: Some(stream), seq: Arc::new(AtomicU64::new(0)), endpoint })
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
    ///
    /// 底层连接断开时流**静默结束**（无错误项）。需要感知断线请用
    /// [`Self::into_result_message_stream`]。
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

    /// 取出 `Result` 消息流（只能调用一次）。
    ///
    /// 每条消息为 `Ok(BusMessage)`；底层 Pub/Sub 连接结束（断线 / 对端关闭）时，
    /// 在末尾**恰好一次**产出 `Err(Unavailable)`，避免静默 EOF。
    ///
    /// **不**提供可靠投递或自动重连；断线后调用方应重建会话。
    pub fn into_result_message_stream(
        mut self,
    ) -> XResult<BoxStream<'static, XResult<BusMessage>>> {
        let stream =
            self.stream.take().ok_or_else(|| XError::invariant("PubSub stream 已被取走"))?;
        let seq = Arc::clone(&self.seq);
        let mapped = stream.map(move |msg| {
            let id = seq.fetch_add(1, Ordering::Relaxed).to_string();
            let payload = Bytes::copy_from_slice(msg.get_payload_bytes());
            Ok(BusMessage { id, payload })
        });
        let with_disconnect = mapped.chain(futures_util::stream::once(async {
            Err(XError::unavailable("redis pubsub 连接已断开"))
        }));
        Ok(Box::pin(with_disconnect))
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
        let info = pubsub_connection_info(&cfg)?;
        let client = redis::Client::open(info)
            .map_err(|e| XError::unavailable(format!("redis pubsub facade: {e}")))?;
        let manager_config = redis::aio::ConnectionManagerConfig::new()
            .set_connection_timeout(cfg.connect_timeout())
            .set_response_timeout(cfg.command_timeout());
        let publish = timeout(
            cfg.connect_timeout(),
            redis::aio::ConnectionManager::new_with_config(client, manager_config),
        )
        .await
        .map_err(|_| XError::deadline_exceeded("redis pubsub facade 连接超时"))?
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
        let mut session =
            RedisPubSub::connect_config(self.cfg.clone(), [channel.to_owned()]).await?;
        // 使用 facade 级序号，保证进程内单调
        session.seq = Arc::clone(&self.seq);
        session.into_message_stream()
    }
}

fn pubsub_connection_info(cfg: &RedisConfig) -> XResult<redis::ConnectionInfo> {
    match cfg.mode() {
        RedisMode::Standalone => cfg.to_connection_info(),
        RedisMode::Cluster => {
            Err(XError::invalid("Redis Pub/Sub 尚不支持 Cluster；拒绝降级到 Standalone 节点"))
        }
        RedisMode::Sentinel => Err(XError::invalid(
            "Redis Pub/Sub 尚不支持 Sentinel master 跟随；拒绝使用静态种子节点",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;

    #[test]
    fn standalone_pubsub_reuses_acl_and_tls_config() {
        let cfg = RedisConfig::builder()
            .addr("redis.example:6380")
            .username("pubsub-user")
            .password(String::from_utf8(vec![b's', b'e', b'c', b'r', b'e', b't']).expect("utf8"))
            .db(4)
            .tls(true)
            .build()
            .expect("cfg");

        let info = pubsub_connection_info(&cfg).expect("standalone info");
        assert_eq!(info.redis.username.as_deref(), Some("pubsub-user"));
        assert!(info.redis.password.is_some());
        assert_eq!(info.redis.db, 4);
        assert!(matches!(info.addr, redis::ConnectionAddr::TcpTls { insecure: false, .. }));
    }

    #[test]
    fn cluster_pubsub_fails_closed_before_connect() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Cluster)
            .nodes(["127.0.0.1:7000"])
            .build()
            .expect("cfg");
        let err = pubsub_connection_info(&cfg).expect_err("cluster must not downgrade");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert!(err.to_string().contains("Cluster"));
    }

    #[test]
    fn sentinel_pubsub_fails_closed_before_connect() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Sentinel)
            .nodes(["127.0.0.1:26379"])
            .sentinel_master("mymaster")
            .build()
            .expect("cfg");
        let err = pubsub_connection_info(&cfg).expect_err("sentinel must not use seed as master");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert!(err.to_string().contains("Sentinel"));
    }

    #[test]
    fn pubsub_deadlines_come_from_same_config() {
        let connect = Duration::from_millis(17);
        let command = Duration::from_millis(23);
        let cfg = RedisConfig::builder()
            .connect_timeout(connect)
            .command_timeout(command)
            .build()
            .expect("cfg");
        assert_eq!(cfg.connect_timeout(), connect);
        assert_eq!(cfg.command_timeout(), command);
    }

    #[tokio::test]
    #[allow(deprecated, reason = "验证 0.3.2 兼容入口仍可编译且安全失败")]
    async fn legacy_entrypoints_compile_and_fail_closed() {
        let err = RedisPubSub::connect(String::from("redis://redacted.invalid"), Vec::new())
            .await
            .expect_err("legacy endpoint lacks security config");
        assert_eq!(err.kind(), ErrorKind::Invalid);

        let cfg = RedisConfig::builder()
            .mode(RedisMode::Cluster)
            .nodes(["127.0.0.1:7000"])
            .build()
            .expect("cfg");
        let err = RedisPubSub::connect_with_config(
            cfg,
            String::from("ignored diagnostic endpoint"),
            Vec::new(),
        )
        .await
        .expect_err("legacy explicit config must preserve fail-closed topology");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn result_stream_api_is_named_on_type() {
        // 编译期存在性 + 文档约束：双入口语义在 rustdoc 中区分
        fn _assert_methods(_: fn(RedisPubSub) -> XResult<BoxStream<'static, BusMessage>>) {}
        fn _assert_result(_: fn(RedisPubSub) -> XResult<BoxStream<'static, XResult<BusMessage>>>) {}
        _assert_methods(RedisPubSub::into_message_stream);
        _assert_result(RedisPubSub::into_result_message_stream);
    }
}
