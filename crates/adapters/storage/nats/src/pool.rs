//! `NatsPool`：共享 `async_nats::Client` + publish/subscribe/health/close。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use async_nats::Client;
use bytes::Bytes;
use futures_util::StreamExt;
use kernel::{XError, XResult};
use tokio::sync::mpsc;

use crate::config::NatsConfig;

/// 池统计。
#[derive(Debug, Clone, Copy, Default)]
pub struct NatsPoolStats {
    /// 成功 publish 次数。
    pub published: u64,
    /// publish 失败次数。
    pub publish_failed: u64,
    /// 是否已关闭。
    pub closed: bool,
}

/// 健康结果。
#[derive(Debug, Clone)]
pub struct NatsHealth {
    /// flush/ping 是否成功。
    pub ready: bool,
    /// 说明。
    pub detail: String,
}

/// 订阅句柄：后台任务将消息推入 mpsc。
pub struct NatsSubscription {
    rx: mpsc::Receiver<NatsMessage>,
}

/// 收到的 Core NATS 消息。
#[derive(Debug, Clone)]
pub struct NatsMessage {
    /// subject。
    pub subject: String,
    /// 载荷。
    pub payload: Bytes,
    /// 会话内单调序号（跨重连不可用于去重）。
    pub seq: u64,
}

impl NatsSubscription {
    /// 取下一条消息。
    pub async fn recv(&mut self) -> Option<NatsMessage> {
        self.rx.recv().await
    }

    /// 转为 `'static` 流（消费 self）。
    pub fn into_stream(self) -> impl futures_core::Stream<Item = NatsMessage> + Send {
        futures_util::stream::unfold(
            self.rx,
            |mut rx| async move { rx.recv().await.map(|m| (m, rx)) },
        )
    }
}

/// 资源池（可克隆，共享连接句柄）。
#[derive(Clone)]
pub struct NatsPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    config: NatsConfig,
    client: Client,
    published: AtomicU64,
    publish_failed: AtomicU64,
    closed: AtomicBool,
    sub_seq: AtomicU64,
}

impl NatsPool {
    /// 连接 NATS。
    pub async fn connect(config: NatsConfig) -> XResult<Self> {
        config.validate()?;
        let mut opts = async_nats::ConnectOptions::new()
            .name(config.name.clone())
            .connection_timeout(config.connect_timeout);
        if let (Some(u), Some(p)) = (&config.user, &config.password) {
            opts = opts.user_and_password(u.clone(), p.clone());
        }
        let client = opts
            .connect(config.url.as_str())
            .await
            .map_err(|e| XError::unavailable(format!("natsx connect: {e}")).with_source(e))?;

        // 连通性：flush 一次
        client
            .flush()
            .await
            .map_err(|e| XError::unavailable(format!("natsx flush: {e}")).with_source(e))?;

        Ok(Self {
            inner: Arc::new(PoolInner {
                config,
                client,
                published: AtomicU64::new(0),
                publish_failed: AtomicU64::new(0),
                closed: AtomicBool::new(false),
                sub_seq: AtomicU64::new(0),
            }),
        })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(NatsConfig::from_env()).await
    }

    /// 配置。
    #[must_use]
    pub fn config(&self) -> &NatsConfig {
        &self.inner.config
    }

    /// 底层 client（高级用法）。
    #[must_use]
    pub fn client(&self) -> Client {
        self.inner.client.clone()
    }

    /// 发布（客户端 accept + flush 语义由驱动决定；**非** durable）。
    pub async fn publish(&self, subject: &str, payload: Bytes) -> XResult<()> {
        self.ensure_open()?;
        if subject.is_empty() {
            return Err(XError::invalid("natsx: subject 不能为空"));
        }
        match self.inner.client.publish(subject.to_string(), payload).await {
            Ok(()) => {
                // 尽量 flush，使调用方在返回时消息已离开客户端缓冲
                if let Err(e) = self.inner.client.flush().await {
                    self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
                    return Err(XError::unavailable(format!("natsx flush after publish: {e}"))
                        .with_source(e));
                }
                self.inner.published.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
                Err(XError::unavailable(format!("natsx publish: {e}")).with_source(e))
            }
        }
    }

    /// 订阅 subject（实时；无历史回放）。
    pub async fn subscribe(&self, subject: &str) -> XResult<NatsSubscription> {
        self.ensure_open()?;
        if subject.is_empty() {
            return Err(XError::invalid("natsx: subject 不能为空"));
        }
        let mut sub = self
            .inner
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| XError::unavailable(format!("natsx subscribe: {e}")).with_source(e))?;

        let (tx, rx) = mpsc::channel(256);
        let seq_base = self.inner.sub_seq.fetch_add(1, Ordering::Relaxed) << 32;
        let counter = Arc::new(AtomicU64::new(0));
        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                let n = counter.fetch_add(1, Ordering::Relaxed);
                let out = NatsMessage {
                    subject: msg.subject.to_string(),
                    payload: msg.payload,
                    seq: seq_base | n,
                };
                if tx.send(out).await.is_err() {
                    break;
                }
            }
        });
        Ok(NatsSubscription { rx })
    }

    /// ping/flush 健康检查。
    pub async fn ping(&self) -> XResult<Duration> {
        self.ensure_open()?;
        let start = std::time::Instant::now();
        self.inner
            .client
            .flush()
            .await
            .map_err(|e| XError::unavailable(format!("natsx ping/flush: {e}")).with_source(e))?;
        Ok(start.elapsed())
    }

    /// 健康。
    pub async fn health(&self) -> XResult<NatsHealth> {
        match self.ping().await {
            Ok(d) => Ok(NatsHealth { ready: true, detail: format!("flush_ok rtt={d:?}") }),
            Err(e) => Ok(NatsHealth { ready: false, detail: e.context().to_string() }),
        }
    }

    /// 统计。
    #[must_use]
    pub fn stats(&self) -> NatsPoolStats {
        NatsPoolStats {
            published: self.inner.published.load(Ordering::Relaxed),
            publish_failed: self.inner.publish_failed.load(Ordering::Relaxed),
            closed: self.inner.closed.load(Ordering::Relaxed),
        }
    }

    /// 关停（标记 closed；连接在 drop 时释放）。
    pub async fn close(&self) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        // async-nats Client 无显式 close；flush 后丢弃引用
        let _ = self.inner.client.flush().await;
        Ok(())
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            Err(XError::cancelled("natsx: pool 已关闭"))
        } else {
            Ok(())
        }
    }
}
