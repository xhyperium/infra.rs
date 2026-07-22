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

    /// 校验有界池配置。
    pub fn validate(self) -> Result<(), TransportError> {
        if self.max_pool_size == 0 {
            return Err(TransportError::ProtocolViolation(
                "http client pool max_pool_size 必须大于 0".into(),
            ));
        }
        if self.max_idle > self.max_pool_size {
            return Err(TransportError::ProtocolViolation(format!(
                "http client pool max_idle {} 不得超过 max_pool_size {}",
                self.max_idle, self.max_pool_size
            )));
        }
        Ok(())
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

/// HTTP 客户端池 RAII 借用；离开作用域时自动归还对象并释放许可。
pub struct HttpClientLease<'a, T> {
    pool: &'a HttpClientPool<T>,
    item: Option<T>,
}

impl<T> HttpClientLease<'_, T> {
    /// 只读访问借出的对象。
    #[must_use]
    pub fn get(&self) -> Option<&T> {
        self.item.as_ref()
    }

    /// 可变访问借出的对象。
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.item.as_mut()
    }

    /// 取走对象并释放池许可；对象不再归还 idle，Drop 不会重复释放。
    #[must_use]
    pub fn into_inner(mut self) -> Option<T> {
        let item = self.item.take();
        if item.is_some() {
            self.pool.release_permit();
        }
        item
    }
}

impl<T> Drop for HttpClientLease<'_, T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            self.pool.return_client(item);
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for HttpClientLease<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientLease").field("item", &self.item).finish_non_exhaustive()
    }
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

    /// 构造已校验的有界池。
    pub fn try_new(config: PoolConfig) -> Result<Self, TransportError> {
        config.validate()?;
        Ok(Self::new(config))
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

    fn release_permit(&self) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        if state.checked_out > 0 {
            state.checked_out -= 1;
        }
        self.cvar.notify_one();
    }

    /// 借出：优先空闲；否则若未达 `max_pool_size` 用 `factory` 创建。
    ///
    /// `factory` 失败时会回滚 `checked_out`，避免槽位永久泄漏。
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
            match factory() {
                Ok(item) => Ok(item),
                Err(e) => {
                    // factory 失败：归还许可，否则 size=1 池会永久耗尽
                    self.release_permit();
                    Err(e)
                }
            }
        } else {
            Err(TransportError::ProtocolViolation(format!(
                "http client pool exhausted (max {})",
                self.config.max_pool_size
            )))
        }
    }

    /// 借出 RAII lease；调用方即使忘记显式归还，Drop 也会回收对象与许可。
    pub fn checkout_lease_with<F>(
        &self,
        factory: F,
    ) -> Result<HttpClientLease<'_, T>, TransportError>
    where
        F: FnOnce() -> Result<T, TransportError>,
    {
        self.checkout_with(factory).map(|item| HttpClientLease { pool: self, item: Some(item) })
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

    #[test]
    fn factory_err_releases_slot_so_pool_not_exhausted() {
        let pool = HttpClientPool::new(PoolConfig::new(1, 1));
        // size-1 池：factory 失败不得永久占满
        let err = pool
            .checkout_with(|| Err(TransportError::ProtocolViolation("factory boom".into())))
            .unwrap_err();
        assert!(matches!(err, TransportError::ProtocolViolation(_)));
        assert_eq!(pool.checked_out(), 0, "factory Err must rollback checked_out");
        // 随后成功 checkout 证明槽位已释放
        let item = pool.checkout_with(|| Ok(99)).expect("slot free after factory err");
        assert_eq!(item, 99);
        assert_eq!(pool.checked_out(), 1);
        pool.return_client(item);
        assert_eq!(pool.checked_out(), 0);
    }

    #[test]
    fn lease_debug_and_into_inner_recover_from_poison() {
        let pool = HttpClientPool::try_new(PoolConfig::new(1, 1)).unwrap();
        let lease = pool.checkout_lease_with(|| Ok(7)).unwrap();
        assert!(format!("{lease:?}").contains('7'));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = pool.state.lock().expect("lock");
            panic!("poison pool before into_inner");
        }));
        assert_eq!(lease.into_inner(), Some(7));
        assert_eq!(pool.checked_out(), 0);
    }
}
