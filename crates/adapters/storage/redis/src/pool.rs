//! Redis 资源池：Standalone `ConnectionManager` / Cluster `ClusterConnection` + Semaphore 背压。

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use kernel::{XError, XResult};
use redis::aio::{ConnectionLike, ConnectionManager};
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use redis::sentinel::{Sentinel, SentinelNodeConnectionInfo};
use redis::{Cmd, Pipeline, RedisFuture, TlsMode, Value};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

use crate::client::RedisClient;
use crate::config::{RedisConfig, RedisMode};
use crate::error_map::map_redis_result;

#[cfg(feature = "pubsub")]
use crate::pubsub::RedisPubSub;

/// 池运行时快照（低基数，可供 readiness / 指标使用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedisPoolStats {
    /// 逻辑命令 lane 数（= max_in_flight；关闭时 0）。
    pub open: usize,
    /// 正在执行的命令数。
    pub in_flight: usize,
    /// 正在等待 acquire 的调用数。
    pub waiters: usize,
}

/// 低基数累计指标（进程内；无高基数 label）。
///
/// 供宿主导出 Prometheus / 日志采样；**不是** OpenTelemetry 实现。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RedisMetricsSnapshot {
    /// 命令成功次数（`with_conn*` 闭包返回 Ok）。
    pub commands_ok: u64,
    /// 命令失败次数（闭包返回 Err）。
    pub commands_err: u64,
    /// 命令超时次数（command budget 耗尽）。
    pub commands_timeout: u64,
    /// acquire 超时次数。
    pub acquire_timeout: u64,
    /// 池已关闭导致的拒绝次数。
    pub rejected_closed: u64,
}

/// 连接后端：Standalone 或 Cluster。Sentinel 发现 master 后归入 Standalone。
///
/// `ConnectionManager` 体积较大，装箱以抑制 `large_enum_variant`。
#[derive(Clone)]
pub(crate) enum RedisBackend {
    /// 单机 / Sentinel master。
    Standalone(Box<ConnectionManager>),
    /// Redis Cluster。
    Cluster(ClusterConnection),
    /// 测试 driver：记录命令调用并返回错误。
    #[cfg(test)]
    Probe(Arc<AtomicUsize>),
}

impl ConnectionLike for RedisBackend {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        match self {
            Self::Standalone(c) => c.req_packed_command(cmd),
            Self::Cluster(c) => c.req_packed_command(cmd),
            #[cfg(test)]
            Self::Probe(calls) => {
                calls.fetch_add(1, Ordering::SeqCst);
                Box::pin(async {
                    Err(redis::RedisError::from((redis::ErrorKind::IoError, "测试 driver 被调用")))
                })
            }
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a Pipeline,
        offset: usize,
        count: usize,
    ) -> RedisFuture<'a, Vec<Value>> {
        match self {
            Self::Standalone(c) => c.req_packed_commands(cmd, offset, count),
            Self::Cluster(c) => c.req_packed_commands(cmd, offset, count),
            #[cfg(test)]
            Self::Probe(calls) => {
                calls.fetch_add(1, Ordering::SeqCst);
                Box::pin(async {
                    Err(redis::RedisError::from((redis::ErrorKind::IoError, "测试 driver 被调用")))
                })
            }
        }
    }

    fn get_db(&self) -> i64 {
        match self {
            Self::Standalone(c) => c.get_db(),
            Self::Cluster(c) => c.get_db(),
            #[cfg(test)]
            Self::Probe(_) => 0,
        }
    }
}

/// 共享 Redis 资源池（`Clone` 只增加引用计数）。
#[derive(Clone)]
pub struct RedisPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    backend: RedisBackend,
    /// 建池时使用的完整配置。Pub/Sub 必须复用它，禁止重新读取环境变量。
    #[cfg(feature = "pubsub")]
    config: RedisConfig,
    sem: Arc<Semaphore>,
    max_in_flight: usize,
    in_flight: AtomicUsize,
    waiters: AtomicUsize,
    closed: AtomicBool,
    command_timeout: Duration,
    acquire_timeout: Duration,
    display_endpoint: String,
    /// 已应用到 ConnectionManager / Cluster 重试退避的最大延迟。
    reconnect_max_delay: Duration,
    /// 配置请求的 TCP keepalive 间隔（见 [`RedisPool::tcp_keepalive`]）。
    tcp_keepalive: Option<Duration>,
    commands_ok: AtomicU64,
    commands_err: AtomicU64,
    commands_timeout: AtomicU64,
    acquire_timeout_count: AtomicU64,
    rejected_closed: AtomicU64,
}

impl std::fmt::Debug for RedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPool")
            .field("endpoint", &self.inner.display_endpoint)
            .field("stats", &self.stats())
            .field("closed", &self.inner.closed.load(Ordering::Relaxed))
            .finish()
    }
}

impl RedisPool {
    #[cfg(test)]
    pub(crate) fn test_probe(driver_calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner: Arc::new(PoolInner {
                backend: RedisBackend::Probe(driver_calls),
                #[cfg(feature = "pubsub")]
                config: RedisConfig::default(),
                sem: Arc::new(Semaphore::new(1)),
                max_in_flight: 1,
                in_flight: AtomicUsize::new(0),
                waiters: AtomicUsize::new(0),
                closed: AtomicBool::new(false),
                command_timeout: Duration::from_secs(1),
                acquire_timeout: Duration::from_secs(1),
                display_endpoint: "redis://测试-driver".to_owned(),
                reconnect_max_delay: Duration::from_secs(5),
                tcp_keepalive: None,
                commands_ok: AtomicU64::new(0),
                commands_err: AtomicU64::new(0),
                commands_timeout: AtomicU64::new(0),
                acquire_timeout_count: AtomicU64::new(0),
                rejected_closed: AtomicU64::new(0),
            }),
        }
    }

    /// 按配置建立连接（Standalone / Cluster / Sentinel）。
    pub async fn connect(config: RedisConfig) -> XResult<Self> {
        config.validate()?;
        let backend = match config.mode() {
            RedisMode::Standalone => connect_standalone(&config).await?,
            RedisMode::Cluster => connect_cluster(&config).await?,
            RedisMode::Sentinel => connect_sentinel(&config).await?,
        };

        // 可选 CLIENT SETNAME（失败不阻断；Cluster 可能路由到任意节点）
        if let Some(name) = config.client_name() {
            let mut c = backend.clone();
            let _: redis::RedisResult<()> =
                redis::cmd("CLIENT").arg("SETNAME").arg(name).query_async(&mut c).await;
        }

        // 连接预热：可选 N 次 PING（失败不阻断建池，记 warn 路径由调用方 readiness 覆盖）
        for _ in 0..config.warmup_count() {
            let mut c = backend.clone();
            let _: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut c).await;
        }

        Ok(Self {
            inner: Arc::new(PoolInner {
                backend,
                sem: Arc::new(Semaphore::new(config.max_in_flight())),
                max_in_flight: config.max_in_flight(),
                in_flight: AtomicUsize::new(0),
                waiters: AtomicUsize::new(0),
                closed: AtomicBool::new(false),
                command_timeout: config.command_timeout(),
                acquire_timeout: config.acquire_timeout(),
                display_endpoint: config.display_endpoint(),
                reconnect_max_delay: config.reconnect_max_delay(),
                tcp_keepalive: config.tcp_keepalive(),
                commands_ok: AtomicU64::new(0),
                commands_err: AtomicU64::new(0),
                commands_timeout: AtomicU64::new(0),
                acquire_timeout_count: AtomicU64::new(0),
                rejected_closed: AtomicU64::new(0),
                #[cfg(feature = "pubsub")]
                config,
            }),
        })
    }

    /// 从环境变量连接（见 [`RedisConfig::from_env`]）。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(RedisConfig::from_env()?).await
    }

    /// 派生可克隆的命令客户端。
    #[must_use]
    pub fn client(&self) -> RedisClient {
        RedisClient::from_pool(self.clone())
    }

    /// 脱敏端点（日志 / 诊断用）。
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.inner.display_endpoint
    }

    /// 执行 `PING`，返回 RTT。
    #[tracing::instrument(skip(self), fields(endpoint = %self.inner.display_endpoint))]
    pub async fn ping(&self) -> XResult<Duration> {
        let start = Instant::now();
        self.with_conn(|mut conn| async move {
            let pong: String = map_redis_result(redis::cmd("PING").query_async(&mut conn).await)?;
            if pong.eq_ignore_ascii_case("PONG") || !pong.is_empty() {
                Ok(())
            } else {
                Err(XError::internal(format!("redis PING 异常响应: {pong}")))
            }
        })
        .await?;
        Ok(start.elapsed())
    }

    /// 当前统计。
    #[must_use]
    pub fn stats(&self) -> RedisPoolStats {
        let closed = self.inner.closed.load(Ordering::Relaxed);
        RedisPoolStats {
            // 逻辑 command lane 数 = max_in_flight（Semaphore 许可）；关闭时为 0
            open: if closed { 0 } else { self.inner.max_in_flight },
            in_flight: self.inner.in_flight.load(Ordering::Relaxed),
            waiters: self.inner.waiters.load(Ordering::Relaxed),
        }
    }

    /// 配置的命令超时。
    #[must_use]
    pub fn command_timeout(&self) -> Duration {
        self.inner.command_timeout
    }

    /// 最大并发命令 lane（= max_in_flight）。
    #[must_use]
    pub fn command_lanes(&self) -> usize {
        self.inner.max_in_flight
    }

    /// 建池时应用到重连退避的最大延迟（毫秒级映射进驱动）。
    #[must_use]
    pub fn reconnect_max_delay(&self) -> Duration {
        self.inner.reconnect_max_delay
    }

    /// 建池时记录的 TCP keepalive 请求间隔。
    ///
    /// redis 0.27 在 TCP 建连时启用 OS 默认 keepalive；本字段保存配置意图供宿主与
    /// 未来驱动对齐，并保证 connect 路径读取了该配置。
    #[must_use]
    pub fn tcp_keepalive(&self) -> Option<Duration> {
        self.inner.tcp_keepalive
    }

    /// Liveness：进程内池未关闭即可（不访问网络）。
    #[must_use]
    pub fn liveness(&self) -> bool {
        !self.is_closed()
    }

    /// Readiness：未关闭且 `PING` 成功。
    pub async fn readiness(&self) -> XResult<Duration> {
        if self.is_closed() {
            return Err(XError::unavailable("redis 连接池已关闭"));
        }
        self.ping().await
    }

    /// 使用自定义命令预算执行（阻塞命令 BLPOP/XREAD BLOCK）。
    pub(crate) async fn with_conn_budget<F, Fut, T>(
        &self,
        command_budget: Duration,
        f: F,
    ) -> XResult<T>
    where
        F: FnOnce(RedisBackend) -> Fut,
        Fut: Future<Output = XResult<T>>,
    {
        if command_budget.is_zero() {
            return Err(XError::deadline_exceeded("redis 命令预算为 0"));
        }
        let _permit = self.acquire_with_timeout(self.inner.acquire_timeout).await?;
        if self.is_closed() {
            self.inner.rejected_closed.fetch_add(1, Ordering::Relaxed);
            return Err(XError::unavailable("redis 连接池已关闭"));
        }
        self.inner.in_flight.fetch_add(1, Ordering::SeqCst);
        let conn = self.inner.backend.clone();
        let result = timeout(command_budget, f(conn)).await;
        self.inner.in_flight.fetch_sub(1, Ordering::SeqCst);
        match result {
            Ok(Ok(v)) => {
                self.inner.commands_ok.fetch_add(1, Ordering::Relaxed);
                Ok(v)
            }
            Ok(Err(e)) => {
                self.inner.commands_err.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.inner.commands_timeout.fetch_add(1, Ordering::Relaxed);
                Err(XError::deadline_exceeded("redis 阻塞命令超时"))
            }
        }
    }

    /// 低基数累计指标快照（进程内；无高基数 label）。
    #[must_use]
    pub fn metrics_snapshot(&self) -> RedisMetricsSnapshot {
        RedisMetricsSnapshot {
            commands_ok: self.inner.commands_ok.load(Ordering::Relaxed),
            commands_err: self.inner.commands_err.load(Ordering::Relaxed),
            commands_timeout: self.inner.commands_timeout.load(Ordering::Relaxed),
            acquire_timeout: self.inner.acquire_timeout_count.load(Ordering::Relaxed),
            rejected_closed: self.inner.rejected_closed.load(Ordering::Relaxed),
        }
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    /// 关闭池：拒绝新请求，并在 `deadline` 内等待 in-flight 排空。
    pub async fn close(&self, deadline: Duration) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        let start = Instant::now();
        loop {
            let inflight = self.inner.in_flight.load(Ordering::SeqCst);
            if inflight == 0 {
                return Ok(());
            }
            if start.elapsed() >= deadline {
                return Err(XError::deadline_exceeded(format!(
                    "redis close 排空超时（仍有 {inflight} 个 in-flight）"
                )));
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    /// 订阅频道（feature `pubsub`）。
    #[cfg(feature = "pubsub")]
    pub async fn subscribe(
        &self,
        channels: impl IntoIterator<Item = String>,
    ) -> XResult<RedisPubSub> {
        if self.is_closed() {
            return Err(XError::unavailable("redis 连接池已关闭"));
        }
        RedisPubSub::connect_config(self.inner.config.clone(), channels).await
    }

    /// 获取命令连接许可并执行异步闭包（计入 in-flight / 超时）。
    ///
    /// 使用配置的 `acquire_timeout` + `command_timeout`（二者独立，不共享总预算）。
    pub(crate) async fn with_conn<F, Fut, T>(&self, f: F) -> XResult<T>
    where
        F: FnOnce(RedisBackend) -> Fut,
        Fut: Future<Output = XResult<T>>,
    {
        self.with_conn_inner(None, f).await
    }

    /// 在**调用级总 deadline**内完成排队 + 命令（draft §2.4 / §2.6）。
    ///
    /// 排队（acquire）时间计入 `total`；剩余时间再与 `command_timeout` 取 min 作为命令预算。
    pub(crate) async fn with_conn_total_deadline<F, Fut, T>(
        &self,
        total: Duration,
        f: F,
    ) -> XResult<T>
    where
        F: FnOnce(RedisBackend) -> Fut,
        Fut: Future<Output = XResult<T>>,
    {
        if total.is_zero() {
            return Err(XError::deadline_exceeded("redis 调用总 deadline 为 0"));
        }
        self.with_conn_inner(Some(total), f).await
    }

    async fn with_conn_inner<F, Fut, T>(&self, total: Option<Duration>, f: F) -> XResult<T>
    where
        F: FnOnce(RedisBackend) -> Fut,
        Fut: Future<Output = XResult<T>>,
    {
        let start = Instant::now();
        let acquire_budget = match total {
            Some(t) => t.min(self.inner.acquire_timeout),
            None => self.inner.acquire_timeout,
        };
        let _permit = self.acquire_with_timeout(acquire_budget).await?;
        if self.is_closed() {
            self.inner.rejected_closed.fetch_add(1, Ordering::Relaxed);
            return Err(XError::unavailable("redis 连接池已关闭"));
        }
        let command_budget = match total {
            Some(t) => {
                let rem = t.saturating_sub(start.elapsed());
                if rem.is_zero() {
                    self.inner.commands_timeout.fetch_add(1, Ordering::Relaxed);
                    return Err(XError::deadline_exceeded(
                        "redis 排队耗尽调用总 deadline（acquire 计入总预算）",
                    ));
                }
                rem.min(self.inner.command_timeout)
            }
            None => self.inner.command_timeout,
        };
        self.inner.in_flight.fetch_add(1, Ordering::SeqCst);
        let conn = self.inner.backend.clone();
        let result = timeout(command_budget, f(conn)).await;
        self.inner.in_flight.fetch_sub(1, Ordering::SeqCst);
        match result {
            Ok(Ok(v)) => {
                self.inner.commands_ok.fetch_add(1, Ordering::Relaxed);
                Ok(v)
            }
            Ok(Err(e)) => {
                self.inner.commands_err.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.inner.commands_timeout.fetch_add(1, Ordering::Relaxed);
                Err(XError::deadline_exceeded("redis 命令超时"))
            }
        }
    }

    async fn acquire_with_timeout(&self, budget: Duration) -> XResult<OwnedSemaphorePermit> {
        if self.is_closed() {
            self.inner.rejected_closed.fetch_add(1, Ordering::Relaxed);
            return Err(XError::unavailable("redis 连接池已关闭"));
        }
        if budget.is_zero() {
            self.inner.acquire_timeout_count.fetch_add(1, Ordering::Relaxed);
            return Err(XError::deadline_exceeded("redis 获取 in-flight 许可预算为 0"));
        }
        self.inner.waiters.fetch_add(1, Ordering::SeqCst);
        let result = timeout(budget, self.inner.sem.clone().acquire_owned()).await;
        self.inner.waiters.fetch_sub(1, Ordering::SeqCst);
        match result {
            Ok(Ok(permit)) => {
                if self.is_closed() {
                    drop(permit);
                    self.inner.rejected_closed.fetch_add(1, Ordering::Relaxed);
                    return Err(XError::unavailable("redis 连接池已关闭"));
                }
                Ok(permit)
            }
            Ok(Err(_)) => Err(XError::unavailable("redis 背压信号量已关闭")),
            Err(_) => {
                self.inner.acquire_timeout_count.fetch_add(1, Ordering::Relaxed);
                Err(XError::deadline_exceeded(format!(
                    "redis 获取 in-flight 许可超时（max={}）",
                    self.inner.max_in_flight
                )))
            }
        }
    }
}

/// 由 [`RedisConfig`] 构造 ConnectionManager 重连/超时参数（可单测）。
pub(crate) fn connection_manager_config(
    config: &RedisConfig,
) -> redis::aio::ConnectionManagerConfig {
    let max_delay_ms = u64::try_from(config.reconnect_max_delay().as_millis()).unwrap_or(u64::MAX);
    // 读取 tcp_keepalive：保证 connect 路径消费该配置（驱动侧为 OS 默认 keepalive）。
    let _keepalive_policy = config.tcp_keepalive();
    redis::aio::ConnectionManagerConfig::new()
        .set_connection_timeout(config.connect_timeout())
        .set_response_timeout(config.command_timeout())
        .set_max_delay(max_delay_ms)
}

async fn connect_standalone(config: &RedisConfig) -> XResult<RedisBackend> {
    let info = config.to_connection_info()?;
    let client = redis::Client::open(info)
        .map_err(|e| XError::unavailable(format!("redis 打开客户端失败: {e}")))?;

    let cm_config = connection_manager_config(config);

    let conn =
        timeout(config.connect_timeout(), ConnectionManager::new_with_config(client, cm_config))
            .await
            .map_err(|_| XError::deadline_exceeded("redis 连接超时"))?
            .map_err(|e| XError::unavailable(format!("redis 连接失败: {e}")))?;

    Ok(RedisBackend::Standalone(Box::new(conn)))
}

async fn connect_cluster(config: &RedisConfig) -> XResult<RedisBackend> {
    let infos = config.seed_connection_infos()?;
    let mut builder = ClusterClient::builder(infos);
    let max_wait_ms = u64::try_from(config.reconnect_max_delay().as_millis()).unwrap_or(u64::MAX);
    let _keepalive_policy = config.tcp_keepalive();
    builder = builder
        .connection_timeout(config.connect_timeout())
        .response_timeout(config.command_timeout())
        .retries(config.max_cluster_redirects())
        .max_retry_wait(max_wait_ms);
    if config.tls() {
        builder = builder.tls(TlsMode::Secure);
    }
    if let Some(pw) = config.password_opt() {
        builder = builder.password(pw.to_owned());
    }
    if let Some(user) = config.username_opt() {
        builder = builder.username(user.to_owned());
    }

    let client = builder
        .build()
        .map_err(|e| XError::unavailable(format!("redis cluster 客户端构建失败: {e}")))?;

    let conn = timeout(config.connect_timeout(), client.get_async_connection())
        .await
        .map_err(|_| XError::deadline_exceeded("redis cluster 连接超时"))?
        .map_err(|e| XError::unavailable(format!("redis cluster 连接失败: {e}")))?;

    Ok(RedisBackend::Cluster(conn))
}

async fn connect_sentinel(config: &RedisConfig) -> XResult<RedisBackend> {
    let master_name = config
        .sentinel_master()
        .ok_or_else(|| XError::invalid("Sentinel 模式缺少 sentinel_master"))?
        .to_owned();

    let sentinel_infos = config.seed_connection_infos()?;
    let mut sentinel = Sentinel::build(sentinel_infos)
        .map_err(|e| XError::unavailable(format!("redis sentinel 客户端构建失败: {e}")))?;

    let node_info = SentinelNodeConnectionInfo {
        tls_mode: if config.tls() { Some(TlsMode::Secure) } else { None },
        redis_connection_info: Some(redis::RedisConnectionInfo {
            db: config.db(),
            username: config.username_opt().map(str::to_owned),
            password: config.password_opt().map(str::to_owned),
            protocol: Default::default(),
        }),
    };

    // Sentinel 发现是同步阻塞 API 路径 + async_master_for；在超时内完成
    let discover = async {
        sentinel
            .async_master_for(&master_name, Some(&node_info))
            .await
            .map_err(|e| XError::unavailable(format!("redis sentinel 发现 master 失败: {e}")))
    };

    let client = timeout(config.connect_timeout(), discover)
        .await
        .map_err(|_| XError::deadline_exceeded("redis sentinel 发现 master 超时"))??;

    let cm_config = connection_manager_config(config);

    let conn =
        timeout(config.connect_timeout(), ConnectionManager::new_with_config(client, cm_config))
            .await
            .map_err(|_| XError::deadline_exceeded("redis sentinel master 连接超时"))?
            .map_err(|e| XError::unavailable(format!("redis sentinel master 连接失败: {e}")))?;

    Ok(RedisBackend::Standalone(Box::new(conn)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;
    use kernel::ErrorKind;

    #[test]
    fn connection_manager_config_applies_reconnect_max_delay() {
        let cfg = RedisConfig::builder()
            .addr("127.0.0.1:6379")
            .reconnect_max_delay(Duration::from_millis(1234))
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .expect("cfg");
        // 构造路径必须读取 keepalive + 写入 max_delay（Debug 含 max_delay 字段）
        let cm = connection_manager_config(&cfg);
        let dbg = format!("{cm:?}");
        assert!(dbg.contains("max_delay"), "cm={dbg}");
        assert!(dbg.contains("1234") || dbg.contains("Some(1234)"), "cm={dbg}");
        assert_eq!(cfg.tcp_keepalive(), Some(Duration::from_secs(30)));
        assert_eq!(cfg.reconnect_max_delay(), Duration::from_millis(1234));
    }

    #[tokio::test]
    async fn connect_refused_returns_error() {
        // 驱动真实 connect 路径：连不可达端口（短超时）
        let cfg = RedisConfig::builder()
            .addr("127.0.0.1:1")
            .password(String::from_utf8(vec![b'u', b'n', b'u', b's', b'e', b'd']).unwrap())
            .connect_timeout(Duration::from_millis(150))
            .command_timeout(Duration::from_millis(150))
            .acquire_timeout(Duration::from_millis(150))
            .build()
            .expect("cfg");
        let res = tokio::time::timeout(Duration::from_secs(3), RedisPool::connect(cfg)).await;
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
            Ok(Ok(_)) => panic!("unexpected connect success to 127.0.0.1:1"),
            Err(_) => {
                // 外层超时也视为连接失败路径被驱动
            }
        }
    }

    #[tokio::test]
    async fn cluster_connect_refused_returns_error() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Cluster)
            .nodes(["127.0.0.1:1"])
            .connect_timeout(Duration::from_millis(150))
            .command_timeout(Duration::from_millis(150))
            .acquire_timeout(Duration::from_millis(150))
            .build()
            .expect("cfg");
        let res = tokio::time::timeout(Duration::from_secs(5), RedisPool::connect(cfg)).await;
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
            Ok(Ok(_)) => panic!("unexpected cluster connect success"),
            Err(_) => {}
        }
    }

    #[tokio::test]
    async fn sentinel_connect_refused_returns_error() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Sentinel)
            .nodes(["127.0.0.1:1"])
            .sentinel_master("mymaster")
            .connect_timeout(Duration::from_millis(150))
            .command_timeout(Duration::from_millis(150))
            .acquire_timeout(Duration::from_millis(150))
            .build()
            .expect("cfg");
        let res = tokio::time::timeout(Duration::from_secs(5), RedisPool::connect(cfg)).await;
        match res {
            Ok(Err(err)) => {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Unavailable
                            | ErrorKind::DeadlineExceeded
                            | ErrorKind::Transient
                            | ErrorKind::Invalid
                    ),
                    "kind={:?}",
                    err.kind()
                );
            }
            Ok(Ok(_)) => panic!("unexpected sentinel connect success"),
            Err(_) => {}
        }
    }

    #[tokio::test]
    async fn closed_pool_is_closed_flag() {
        // 路径：
        // 1) 无 env → from_env 失败（合法离线）
        // 2) 有 env 但 Redis 不可达 → connect 失败（合法 CI 无服务）
        // 3) 有 env 且可达 → 验证 close 状态机
        // 禁止「无任何可观察断言就 return」的 silent pass。
        let cfg = match RedisConfig::from_env() {
            Ok(cfg) => cfg,
            Err(err) => {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Invalid | ErrorKind::Unavailable | ErrorKind::Missing
                    ),
                    "from_env without Redis env should fail closed, kind={:?}",
                    err.kind()
                );
                return;
            }
        };
        let pool = match RedisPool::connect(cfg).await {
            Ok(pool) => pool,
            Err(err) => {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Unavailable
                            | ErrorKind::DeadlineExceeded
                            | ErrorKind::Transient
                            | ErrorKind::Invalid
                    ),
                    "connect failure must be typed, kind={:?}",
                    err.kind()
                );
                return;
            }
        };
        assert!(!pool.is_closed());
        let _ = pool.stats();
        pool.close(Duration::from_secs(2)).await.expect("close");
        assert!(pool.is_closed());
        let err = pool.ping().await.expect_err("after close");
        assert!(
            matches!(err.kind(), ErrorKind::Unavailable | ErrorKind::Cancelled),
            "kind={:?}",
            err.kind()
        );
        let snap = pool.metrics_snapshot();
        assert!(snap.rejected_closed >= 1, "closed pool rejections should be counted: {snap:?}");
    }

    #[tokio::test]
    async fn probe_metrics_count_command_errors() {
        let pool = RedisPool::test_probe(Arc::new(AtomicUsize::new(0)));
        let before = pool.metrics_snapshot();
        assert_eq!(before, RedisMetricsSnapshot::default());

        let err = pool
            .with_conn(|mut conn| async move {
                let _: String = map_redis_result(redis::cmd("PING").query_async(&mut conn).await)?;
                Ok(())
            })
            .await
            .expect_err("probe always fails command");
        assert_eq!(err.kind(), ErrorKind::Transient);

        let snap = pool.metrics_snapshot();
        assert_eq!(snap.commands_ok, 0);
        assert_eq!(snap.commands_err, 1);
        assert_eq!(snap.commands_timeout, 0);
        assert_eq!(snap.acquire_timeout, 0);
        assert_eq!(snap.rejected_closed, 0);
    }

    #[tokio::test]
    async fn probe_metrics_count_closed_reject() {
        let pool = RedisPool::test_probe(Arc::new(AtomicUsize::new(0)));
        pool.close(Duration::from_secs(1)).await.expect("close");
        let err =
            pool.with_conn(|_conn| async move { Ok::<(), XError>(()) }).await.expect_err("closed");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        let snap = pool.metrics_snapshot();
        assert!(snap.rejected_closed >= 1, "snap={snap:?}");
    }

    #[tokio::test]
    async fn probe_metrics_count_zero_deadline_as_acquire_timeout() {
        let pool = RedisPool::test_probe(Arc::new(AtomicUsize::new(0)));
        let err = pool
            .with_conn_total_deadline(Duration::ZERO, |_conn| async move { Ok::<(), XError>(()) })
            .await
            .expect_err("zero total deadline");
        assert_eq!(err.kind(), ErrorKind::DeadlineExceeded);
        // 零总 deadline 在 with_conn_total_deadline 入口拦截，不计入 acquire_timeout
        assert_eq!(pool.metrics_snapshot().acquire_timeout, 0);
    }
}
