//! 简单连接/客户端池（配置 + 信号量式 checkout）。

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::TransportError;

/// 池配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolConfig {
    /// 池内最大对象数。
    pub max_pool_size: usize,
    /// 最大空闲保留（语义标记；本实现用同一上限）。
    pub max_idle: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self { max_pool_size: 8, max_idle: 4 }
    }
}

impl PoolConfig {
    /// 构造。
    #[must_use]
    pub const fn new(max_pool_size: usize, max_idle: usize) -> Self {
        Self { max_pool_size, max_idle }
    }
}

/// 可 checkout 的 HTTP 客户端池（泛型槽位；通常放 `ReqwestHttpDriver` 或连接 id）。
///
/// 使用许可计数实现上限：`checkout` 在无许可时阻塞（可选超时）。
pub struct HttpClientPool<T> {
    config: PoolConfig,
    state: Mutex<PoolState<T>>,
    cvar: Condvar,
}

struct PoolState<T> {
    idle: Vec<T>,
    /// 已借出数量。
    checked_out: usize,
}

impl<T> HttpClientPool<T> {
    /// 空池。
    #[must_use]
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            state: Mutex::new(PoolState { idle: Vec::new(), checked_out: 0 }),
            cvar: Condvar::new(),
        }
    }

    /// 配置。
    #[must_use]
    pub fn config(&self) -> PoolConfig {
        self.config
    }

    /// 当前空闲数。
    #[must_use]
    pub fn idle_len(&self) -> usize {
        self.state.lock().map(|g| g.idle.len()).unwrap_or(0)
    }

    /// 已借出数。
    #[must_use]
    pub fn checked_out(&self) -> usize {
        self.state.lock().map(|g| g.checked_out).unwrap_or(0)
    }

    /// 归还。
    pub fn checkin(&self, item: T) {
        if let Ok(mut g) = self.state.lock() {
            if g.checked_out > 0 {
                g.checked_out -= 1;
            }
            if g.idle.len() < self.config.max_idle {
                g.idle.push(item);
            }
            // else drop item (over max_idle)
            self.cvar.notify_one();
        }
    }

    /// 借出：优先空闲；否则若未达 `max_pool_size` 用 `factory` 创建。
    pub fn checkout_with<F>(&self, factory: F) -> Result<T, TransportError>
    where
        F: FnOnce() -> Result<T, TransportError>,
    {
        let mut g = self
            .state
            .lock()
            .map_err(|_| TransportError::Io(Box::new(std::io::Error::other("pool lock"))))?;
        if let Some(item) = g.idle.pop() {
            g.checked_out += 1;
            return Ok(item);
        }
        if g.checked_out < self.config.max_pool_size {
            g.checked_out += 1;
            drop(g);
            return factory();
        }
        Err(TransportError::ProtocolViolation(format!(
            "http client pool exhausted (max {})",
            self.config.max_pool_size
        )))
    }

    /// 限时等待空闲槽（无 factory：仅从 idle 取）。
    pub fn checkout_idle_timeout(&self, timeout: Duration) -> Result<Option<T>, TransportError> {
        let mut g = self
            .state
            .lock()
            .map_err(|_| TransportError::Io(Box::new(std::io::Error::other("pool lock"))))?;
        let start = std::time::Instant::now();
        loop {
            if let Some(item) = g.idle.pop() {
                g.checked_out += 1;
                return Ok(Some(item));
            }
            let left = timeout.saturating_sub(start.elapsed());
            if left.is_zero() {
                return Ok(None);
            }
            let (next, res) = self
                .cvar
                .wait_timeout(g, left)
                .map_err(|_| TransportError::Io(Box::new(std::io::Error::other("pool lock"))))?;
            g = next;
            if res.timed_out() && g.idle.is_empty() {
                return Ok(None);
            }
        }
    }
}

impl<T> std::fmt::Debug for HttpClientPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientPool")
            .field("config", &self.config)
            .field("idle", &self.idle_len())
            .field("checked_out", &self.checked_out())
            .finish()
    }
}

/// `Arc` 包装便利类型。
pub type SharedHttpClientPool<T> = Arc<HttpClientPool<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_checkout_limits() {
        let pool = HttpClientPool::new(PoolConfig::new(2, 2));
        let a = pool.checkout_with(|| Ok(1)).unwrap();
        let b = pool.checkout_with(|| Ok(2)).unwrap();
        assert_eq!(pool.checked_out(), 2);
        assert!(pool.checkout_with(|| Ok(3)).is_err());
        pool.checkin(a);
        pool.checkin(b);
        assert_eq!(pool.idle_len(), 2);
        let c = pool.checkout_with(|| Ok(9)).unwrap();
        // LIFO idle stack：后 checkin 的先出
        assert!(c == 1 || c == 2, "expected idle item, got {c}");
        assert_eq!(pool.idle_len(), 1);
        let _ = format!("{:?}", pool);
        let _ = PoolConfig::default();
    }
}
