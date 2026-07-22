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
        match self.state.lock() {
            Ok(g) => g.idle.len(),
            Err(p) => p.into_inner().idle.len(),
        }
    }

    /// 已借出数。
    #[must_use]
    pub fn checked_out(&self) -> usize {
        match self.state.lock() {
            Ok(g) => g.checked_out,
            Err(p) => p.into_inner().checked_out,
        }
    }

    /// 归还。
    pub fn return_client(&self, item: T) {
        let mut g = match self.state.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if g.checked_out > 0 {
            g.checked_out -= 1;
        }
        if g.idle.len() < self.config.max_idle {
            g.idle.push(item);
        }
        // else drop item (over max_idle)
        self.cvar.notify_one();
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
        assert_eq!(pool.config(), PoolConfig::new(2, 2));
        let a = pool.checkout_with(|| Ok(1)).unwrap();
        let b = pool.checkout_with(|| Ok(2)).unwrap();
        assert_eq!(pool.checked_out(), 2);
        assert!(pool.checkout_with(|| Ok(3)).is_err());
        pool.return_client(a);
        pool.return_client(b);
        assert_eq!(pool.idle_len(), 2);
        let c = pool.checkout_with(|| Ok(9)).unwrap();
        // LIFO idle stack：后 return_client 的先出
        assert!(c == 1 || c == 2, "expected idle item, got {c}");
        assert_eq!(pool.idle_len(), 1);
        let _ = format!("{:?}", pool);
        let _ = PoolConfig::default();
    }

    #[test]
    fn return_client_drops_over_max_idle() {
        let pool = HttpClientPool::new(PoolConfig::new(4, 1));
        let a = pool.checkout_with(|| Ok(1)).unwrap();
        let b = pool.checkout_with(|| Ok(2)).unwrap();
        pool.return_client(a);
        assert_eq!(pool.idle_len(), 1);
        // 超过 max_idle：丢弃 item，不增长 idle
        pool.return_client(b);
        assert_eq!(pool.idle_len(), 1);
    }

    #[test]
    fn checkout_idle_timeout_paths() {
        use std::sync::Arc;
        use std::thread;
        let pool = Arc::new(HttpClientPool::new(PoolConfig::new(2, 2)));
        // zero timeout：首轮 left.is_zero
        assert!(pool.checkout_idle_timeout(Duration::ZERO).unwrap().is_none());
        // 空 idle → wait_timeout 后超时
        assert!(pool.checkout_idle_timeout(Duration::from_millis(15)).unwrap().is_none());

        // 确定性：先放入 idle，再超时等待应立即拿到
        let item = pool.checkout_with(|| Ok(7)).unwrap();
        pool.return_client(item);
        assert_eq!(pool.checkout_idle_timeout(Duration::from_millis(10)).unwrap(), Some(7));

        // 后台 return_client 唤醒等待中的 checkout_idle_timeout
        let p2 = Arc::clone(&pool);
        let h = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            let item = p2.checkout_with(|| Ok(42)).unwrap();
            p2.return_client(item);
        });
        let got = pool.checkout_idle_timeout(Duration::from_secs(2)).unwrap();
        h.join().unwrap();
        // 线程 return_client 后应被唤醒；若主线程先拿到则值为 42
        assert_eq!(got, Some(42));
    }

    #[test]
    fn return_client_recovers_from_poison() {
        let pool = HttpClientPool::new(PoolConfig::new(1, 1));
        let item = pool.checkout_with(|| Ok(1)).unwrap();
        assert_eq!(pool.checked_out(), 1);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = pool.state.lock().expect("lock");
            panic!("poison pool");
        }));
        // poison 后 accessors / return_client 均 via into_inner 恢复
        assert_eq!(pool.checked_out(), 1);
        assert_eq!(pool.idle_len(), 0);
        pool.return_client(item);
        assert_eq!(pool.idle_len(), 1);
        assert_eq!(pool.checked_out(), 0);
    }
}
