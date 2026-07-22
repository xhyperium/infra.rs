//! 配置热更新通知（进程内订阅）。

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use kernel::{XError, XResult};

use crate::ConfigStore;
use crate::layered::LayeredConfig;

/// 一次热更新通知（序号单调递增）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigChange {
    /// 从 1 起的变更序号。
    pub generation: u64,
}

struct WatchState {
    generation: u64,
    closed: bool,
}

/// 进程内配置变更总线。
///
/// - [`subscribe`](Self::subscribe) 返回订阅句柄
/// - [`notify`](Self::notify) 广播 generation++
/// - [`reload`](Self::reload) 可选：从 [`LayeredConfig`] 重载到 store 后自动 notify
pub struct ConfigWatch {
    state: Mutex<WatchState>,
    cvar: Condvar,
}

impl ConfigWatch {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WatchState { generation: 0, closed: false }),
            cvar: Condvar::new(),
        }
    }

    /// 当前 generation（未通知时为 0）。
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.state.lock().map(|g| g.generation).unwrap_or(0)
    }

    /// 订阅；返回从当前 generation 之后等待的句柄。
    #[must_use]
    pub fn subscribe(self: &Arc<Self>) -> ConfigSubscription {
        let seen = self.generation();
        ConfigSubscription { watch: Arc::clone(self), seen }
    }

    /// 广播一次变更（generation +1）。
    pub fn notify(&self) -> XResult<ConfigChange> {
        let mut g = self.state.lock().map_err(|_| XError::invalid("config watch lock poisoned"))?;
        if g.closed {
            return Err(XError::invalid("config watch closed"));
        }
        g.generation = g.generation.saturating_add(1);
        let change = ConfigChange { generation: g.generation };
        self.cvar.notify_all();
        Ok(change)
    }

    /// 从多层源重载到 store，成功后 notify。
    pub fn reload(&self, layered: &LayeredConfig, store: &ConfigStore) -> XResult<ConfigChange> {
        layered.reload_into(store)?;
        self.notify()
    }

    /// 关闭；后续 wait 返回 `None`。
    pub fn close(&self) -> XResult<()> {
        let mut g = self.state.lock().map_err(|_| XError::invalid("config watch lock poisoned"))?;
        g.closed = true;
        self.cvar.notify_all();
        Ok(())
    }
}

impl Default for ConfigWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConfigWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigWatch").field("generation", &self.generation()).finish()
    }
}

/// 订阅句柄：阻塞等待下一次 generation 增长。
pub struct ConfigSubscription {
    watch: Arc<ConfigWatch>,
    seen: u64,
}

impl ConfigSubscription {
    /// 上次观察到的 generation。
    #[must_use]
    pub fn seen(&self) -> u64 {
        self.seen
    }

    /// 阻塞直到 generation > seen 或 watch 关闭；关闭返回 `None`。
    pub fn wait(&mut self) -> XResult<Option<ConfigChange>> {
        let mut g =
            self.watch.state.lock().map_err(|_| XError::invalid("config watch lock poisoned"))?;
        loop {
            if g.closed {
                return Ok(None);
            }
            if g.generation > self.seen {
                self.seen = g.generation;
                return Ok(Some(ConfigChange { generation: g.generation }));
            }
            g = self
                .watch
                .cvar
                .wait(g)
                .map_err(|_| XError::invalid("config watch lock poisoned"))?;
        }
    }

    /// 限时等待；超时返回 `Ok(None)`（与关闭区分：可结合 `watch.generation()`）。
    pub fn wait_timeout(&mut self, timeout: Duration) -> XResult<Option<ConfigChange>> {
        let mut g =
            self.watch.state.lock().map_err(|_| XError::invalid("config watch lock poisoned"))?;
        loop {
            if g.closed {
                return Ok(None);
            }
            if g.generation > self.seen {
                self.seen = g.generation;
                return Ok(Some(ConfigChange { generation: g.generation }));
            }
            let (next, result) = self
                .watch
                .cvar
                .wait_timeout(g, timeout)
                .map_err(|_| XError::invalid("config watch lock poisoned"))?;
            g = next;
            if result.timed_out() && g.generation <= self.seen {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn notify_wakes_subscriber() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let w2 = Arc::clone(&watch);
        let h = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            w2.notify().unwrap()
        });
        let change = sub.wait().unwrap().expect("notified");
        assert_eq!(change.generation, 1);
        assert_eq!(h.join().unwrap().generation, 1);
    }

    #[test]
    fn reload_notifies_and_updates_store() {
        let watch = Arc::new(ConfigWatch::new());
        let store = ConfigStore::new();
        store.set("old", "1").unwrap();
        let layered =
            LayeredConfig::new().with_source(Arc::new(MemorySource::from_pairs([("k", "v")])));
        let change = watch.reload(&layered, &store).unwrap();
        assert_eq!(change.generation, 1);
        assert_eq!(store.get("k").as_deref(), Some("v"));
        assert_eq!(store.get("old"), None);
    }

    #[test]
    fn close_ends_wait() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let w2 = Arc::clone(&watch);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            w2.close().unwrap();
        });
        assert!(sub.wait().unwrap().is_none());
    }

    #[test]
    fn default_debug_and_seen() {
        let watch = Arc::new(ConfigWatch::default());
        let _ = format!("{:?}", watch);
        let mut sub = watch.subscribe();
        assert_eq!(sub.seen(), 0);
        watch.notify().unwrap();
        // 非阻塞限时等待应立刻拿到 generation
        let change = sub.wait_timeout(Duration::from_millis(50)).unwrap().expect("gen");
        assert_eq!(change.generation, 1);
        assert_eq!(sub.seen(), 1);
        // 已追上 generation 时超时返回 None
        assert!(sub.wait_timeout(Duration::from_millis(20)).unwrap().is_none());
    }

    #[test]
    fn notify_after_close_errors() {
        let watch = ConfigWatch::new();
        watch.close().unwrap();
        assert!(watch.notify().is_err());
    }

    #[test]
    fn wait_timeout_sees_close() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let w2 = Arc::clone(&watch);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            w2.close().unwrap();
        });
        assert!(sub.wait_timeout(Duration::from_secs(1)).unwrap().is_none());
    }

    #[test]
    fn wait_timeout_wakes_on_notify() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let w2 = Arc::clone(&watch);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(15));
            w2.notify().unwrap();
        });
        let change = sub.wait_timeout(Duration::from_secs(1)).unwrap().expect("notified");
        assert_eq!(change.generation, 1);
    }
}
