//! `NatsPool`：共享 `async_nats::Client` + publish/subscribe/health/close。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_nats::Client;
use bytes::Bytes;
use futures_util::StreamExt;
use kernel::{XError, XResult};
use tokio::sync::{Notify, mpsc};
use tokio::task::AbortHandle;

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
    /// 建连事件数（含首次连接）。
    pub connected: u64,
    /// 断线事件数。
    pub disconnected: u64,
    /// 驱动报告的慢消费者事件数。
    pub slow_consumers: u64,
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
    task: Option<SubscriptionTask>,
}

struct SubscriptionTask(AbortHandle);

impl Drop for SubscriptionTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct SubscriptionGuard {
    active: Arc<AtomicU64>,
    drained: Arc<Notify>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
        self.drained.notify_waiters();
    }
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
        futures_util::stream::unfold((self.rx, self.task), |(mut rx, task)| async move {
            rx.recv().await.map(|message| (message, (rx, task)))
        })
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
    connected: Arc<AtomicU64>,
    disconnected: Arc<AtomicU64>,
    slow_consumers: Arc<AtomicU64>,
    subscription_tasks: Mutex<Vec<AbortHandle>>,
    active_subscriptions: Arc<AtomicU64>,
    subscriptions_drained: Arc<Notify>,
}

impl NatsPool {
    /// 连接 NATS。
    ///
    /// TLS：按 [`NatsConfig::effective_tls_policy`] 设置 `require_tls`。
    pub async fn connect(config: NatsConfig) -> XResult<Self> {
        config.validate()?;
        let policy = config.effective_tls_policy();
        let connected = Arc::new(AtomicU64::new(0));
        let disconnected = Arc::new(AtomicU64::new(0));
        let slow_consumers = Arc::new(AtomicU64::new(0));
        let event_connected = Arc::clone(&connected);
        let event_disconnected = Arc::clone(&disconnected);
        let event_slow_consumers = Arc::clone(&slow_consumers);
        let reconnect_max_delay = config.reconnect_max_delay;
        let mut opts = async_nats::ConnectOptions::new()
            .name(config.name.clone())
            .connection_timeout(config.connect_timeout)
            .require_tls(policy.require_tls())
            .request_timeout(Some(config.operation_timeout))
            .subscription_capacity(config.subscription_capacity)
            .client_capacity(config.client_capacity)
            .max_reconnects(Some(config.max_reconnects))
            .reconnect_delay_callback(move |attempt| {
                let exponent = u32::try_from(attempt.min(16)).unwrap_or(16);
                let factor = 1u32.checked_shl(exponent).unwrap_or(u32::MAX);
                Duration::from_millis(100).saturating_mul(factor).min(reconnect_max_delay)
            })
            .event_callback(move |event| {
                let connected = Arc::clone(&event_connected);
                let disconnected = Arc::clone(&event_disconnected);
                let slow_consumers = Arc::clone(&event_slow_consumers);
                async move {
                    match event {
                        async_nats::Event::Connected => {
                            connected.fetch_add(1, Ordering::Relaxed);
                        }
                        async_nats::Event::Disconnected => {
                            disconnected.fetch_add(1, Ordering::Relaxed);
                        }
                        async_nats::Event::SlowConsumer(_) => {
                            slow_consumers.fetch_add(1, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            });
        if config.ignore_discovered_servers {
            opts = opts.ignore_discovered_servers().retain_servers_order();
        }
        if let (Some(u), Some(p)) = (&config.user, &config.password) {
            opts = opts.user_and_password(u.clone(), p.clone());
        }
        let client =
            tokio::time::timeout(config.connect_timeout, opts.connect(config.url.as_str()))
                .await
                .map_err(|error| {
                    XError::deadline_exceeded("natsx connect 超时").with_source(error)
                })?
                .map_err(|e| XError::unavailable(format!("natsx connect: {e}")).with_source(e))?;
        // 连通性：flush 一次
        tokio::time::timeout(config.operation_timeout, client.flush())
            .await
            .map_err(|error| {
                XError::deadline_exceeded("natsx initial flush 超时").with_source(error)
            })?
            .map_err(|e| XError::unavailable(format!("natsx flush: {e}")).with_source(e))?;

        Ok(Self {
            inner: Arc::new(PoolInner {
                config,
                client,
                published: AtomicU64::new(0),
                publish_failed: AtomicU64::new(0),
                closed: AtomicBool::new(false),
                sub_seq: AtomicU64::new(0),
                connected,
                disconnected,
                slow_consumers,
                subscription_tasks: Mutex::new(Vec::new()),
                active_subscriptions: Arc::new(AtomicU64::new(0)),
                subscriptions_drained: Arc::new(Notify::new()),
            }),
        })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(NatsConfig::from_env()?).await
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
        match tokio::time::timeout(
            self.inner.config.operation_timeout,
            self.inner.client.publish(subject.to_string(), payload),
        )
        .await
        {
            Err(error) => {
                self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
                Err(XError::deadline_exceeded("natsx publish 超时").with_source(error))
            }
            Ok(Ok(())) => {
                // 尽量 flush，使调用方在返回时消息已离开客户端缓冲
                match tokio::time::timeout(
                    self.inner.config.operation_timeout,
                    self.inner.client.flush(),
                )
                .await
                {
                    Err(error) => {
                        self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
                        return Err(XError::deadline_exceeded("natsx publish flush 超时")
                            .with_source(error));
                    }
                    Ok(Err(error)) => {
                        self.inner.publish_failed.fetch_add(1, Ordering::Relaxed);
                        return Err(XError::unavailable(format!(
                            "natsx flush after publish: {error}"
                        ))
                        .with_source(error));
                    }
                    Ok(Ok(())) => {}
                }
                self.inner.published.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Ok(Err(e)) => {
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
        let mut sub = tokio::time::timeout(
            self.inner.config.operation_timeout,
            self.inner.client.subscribe(subject.to_string()),
        )
        .await
        .map_err(|error| XError::deadline_exceeded("natsx subscribe 超时").with_source(error))?
        .map_err(|e| XError::unavailable(format!("natsx subscribe: {e}")).with_source(e))?;

        let (tx, rx) = mpsc::channel(self.inner.config.subscription_capacity);
        let seq_base = self.inner.sub_seq.fetch_add(1, Ordering::Relaxed) << 32;
        let counter = Arc::new(AtomicU64::new(0));
        let slow_consumers = Arc::clone(&self.inner.slow_consumers);
        let operation_timeout = self.inner.config.operation_timeout;
        let mut tasks = self
            .inner
            .subscription_tasks
            .lock()
            .map_err(|_| XError::invariant("natsx 订阅任务注册表锁已中毒"))?;
        self.ensure_open()?;
        tasks.retain(|handle| !handle.is_finished());

        self.inner.active_subscriptions.fetch_add(1, Ordering::AcqRel);
        let guard = SubscriptionGuard {
            active: Arc::clone(&self.inner.active_subscriptions),
            drained: Arc::clone(&self.inner.subscriptions_drained),
        };
        let task = tokio::spawn(async move {
            let _guard = guard;
            while let Some(msg) = sub.next().await {
                let n = counter.fetch_add(1, Ordering::Relaxed);
                let out = NatsMessage {
                    subject: msg.subject.to_string(),
                    payload: msg.payload,
                    seq: seq_base | n,
                };
                match tokio::time::timeout(operation_timeout, tx.send(out)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => break,
                    Err(_) => {
                        slow_consumers.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }
        });
        let abort_handle = task.abort_handle();
        tasks.push(abort_handle.clone());
        drop(tasks);
        drop(task);
        Ok(NatsSubscription { rx, task: Some(SubscriptionTask(abort_handle)) })
    }

    /// ping/flush 健康检查。
    pub async fn ping(&self) -> XResult<Duration> {
        self.ensure_open()?;
        let start = std::time::Instant::now();
        tokio::time::timeout(self.inner.config.operation_timeout, self.inner.client.flush())
            .await
            .map_err(|error| XError::deadline_exceeded("natsx ping/flush 超时").with_source(error))?
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
            connected: self.inner.connected.load(Ordering::Relaxed),
            disconnected: self.inner.disconnected.load(Ordering::Relaxed),
            slow_consumers: self.inner.slow_consumers.load(Ordering::Relaxed),
        }
    }

    /// 关停：拒绝新请求，取消并等待订阅转发任务，再刷新连接缓冲。
    pub async fn close(&self) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        let handles = {
            let mut tasks = self
                .inner
                .subscription_tasks
                .lock()
                .map_err(|_| XError::invariant("natsx 订阅任务注册表锁已中毒"))?;
            tasks.drain(..).collect::<Vec<_>>()
        };
        for handle in handles {
            handle.abort();
        }
        let drain = async {
            loop {
                if self.inner.active_subscriptions.load(Ordering::Acquire) == 0 {
                    return;
                }
                let notified = self.inner.subscriptions_drained.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if self.inner.active_subscriptions.load(Ordering::Acquire) == 0 {
                    return;
                }
                notified.await;
            }
        };
        tokio::time::timeout(self.inner.config.operation_timeout, drain).await.map_err(
            |error| {
                XError::deadline_exceeded("natsx close 等待订阅任务退出超时").with_source(error)
            },
        )?;
        // async-nats Client 无显式 close；flush 后丢弃引用
        tokio::time::timeout(self.inner.config.operation_timeout, self.inner.client.flush())
            .await
            .map_err(|error| {
                XError::deadline_exceeded("natsx close flush 超时").with_source(error)
            })?
            .map_err(|error| XError::unavailable("natsx close flush 失败").with_source(error))
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            Err(XError::cancelled("natsx: pool 已关闭"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NatsConfig;
    use futures_util::StreamExt;
    use kernel::ErrorKind;

    #[tokio::test]
    async fn connect_refused_returns_unavailable() {
        let cfg = NatsConfig {
            url: "nats://127.0.0.1:1".into(),
            connect_timeout: Duration::from_millis(300),
            name: "natsx-test".into(),
            ..NatsConfig::default()
        };
        let res = tokio::time::timeout(Duration::from_secs(2), NatsPool::connect(cfg)).await;
        match res {
            Ok(Err(err)) => assert!(
                matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
                "kind={:?}",
                err.kind()
            ),
            Ok(Ok(_)) => panic!("must fail"),
            Err(_) => panic!("NatsPool::connect 必须受内部截止时间约束"),
        }
    }

    #[tokio::test]
    async fn dropping_subscription_aborts_owned_forwarder_task() {
        let (_tx, rx) = mpsc::channel(1);
        let task = tokio::spawn(std::future::pending::<()>());
        let subscription =
            NatsSubscription { rx, task: Some(SubscriptionTask(task.abort_handle())) };
        drop(subscription);
        let error = task.await.expect_err("订阅 drop 必须取消转发任务");
        assert!(error.is_cancelled());
    }

    #[tokio::test]
    async fn require_tls_on_plain_remote_fails_closed() {
        // 非 loopback + Require + 明文 nats:// → require_tls(true)，无 TLS 服务时连接失败
        let cfg = NatsConfig {
            url: "nats://203.0.113.10:4222".into(), // TEST-NET-3，不应有真实监听
            connect_timeout: Duration::from_millis(300),
            tls_policy: Some(crate::config::TlsPolicy::Require),
            name: "natsx-tls-test".into(),
            ..NatsConfig::default()
        };
        assert!(cfg.effective_tls_policy().require_tls());
        let res = tokio::time::timeout(Duration::from_secs(2), NatsPool::connect(cfg)).await;
        match res {
            Ok(Err(err)) => assert!(
                matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
                "kind={:?}",
                err.kind()
            ),
            Ok(Ok(_)) => panic!("must fail without TLS endpoint"),
            Err(_) => panic!("NatsPool::connect 必须受内部截止时间约束"),
        }
    }

    /// 离线构造公共消息/健康/统计类型（不依赖 NATS 进程）。
    #[test]
    fn offline_message_health_stats_types() {
        let msg = NatsMessage {
            subject: "infra.test".into(),
            payload: Bytes::from_static(b"payload"),
            seq: 42,
        };
        assert_eq!(msg.subject, "infra.test");
        assert_eq!(msg.seq, 42);
        assert_eq!(msg.payload.as_ref(), b"payload");

        let health = NatsHealth { ready: false, detail: "offline".into() };
        assert!(!health.ready);
        assert!(health.detail.contains("offline"));

        let stats = NatsPoolStats {
            published: 1,
            publish_failed: 0,
            closed: false,
            connected: 1,
            disconnected: 0,
            slow_consumers: 0,
        };
        assert_eq!(stats.published, 1);
        assert!(!stats.closed);
        let _default = NatsPoolStats::default();
    }

    /// `NatsSubscription::recv` / `into_stream` 离线路径。
    #[tokio::test]
    async fn subscription_recv_and_into_stream() {
        let (tx, rx) = mpsc::channel(2);
        let mut sub = NatsSubscription { rx, task: None };
        tx.send(NatsMessage { subject: "s1".into(), payload: Bytes::from_static(b"a"), seq: 1 })
            .await
            .expect("send");
        let got = sub.recv().await.expect("recv");
        assert_eq!(got.seq, 1);
        assert_eq!(got.subject, "s1");

        let (tx2, rx2) = mpsc::channel(1);
        let sub2 = NatsSubscription { rx: rx2, task: None };
        tx2.send(NatsMessage { subject: "s2".into(), payload: Bytes::from_static(b"b"), seq: 2 })
            .await
            .expect("send");
        drop(tx2);
        let mut stream = Box::pin(sub2.into_stream());
        let next = stream.next().await.expect("stream item");
        assert_eq!(next.seq, 2);
        assert!(stream.next().await.is_none());
    }
}
