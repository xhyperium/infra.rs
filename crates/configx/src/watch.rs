//! 配置变更通知（进程内订阅；不启动自动 watcher）。

use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError};
use std::thread;
use std::time::{Duration, Instant};

use kernel::{XError, XResult};

use crate::ConfigStore;
use crate::layered::LayeredConfig;

const TIMED_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(1);
const WATCH_MUTATION_LOCK_POISONED_CONTEXT: &str = "配置监听变更锁已中毒";
const WATCH_STATE_LOCK_POISONED_CONTEXT: &str = "配置监听状态锁已中毒";
const WATCH_CLOSED_CONTEXT: &str = "配置监听已关闭";
const WATCH_GENERATION_OVERFLOW_CONTEXT: &str = "配置监听 generation 溢出";

/// 一次热更新通知（序号单调递增）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigChange {
    /// 从 1 起的变更序号。
    pub generation: u64,
}

/// 一次订阅等待的显式结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigWaitOutcome {
    /// 观察到新 generation。
    Changed(ConfigChange),
    /// 总 deadline 已到且未观察到新 generation。
    TimedOut,
    /// watch 已关闭。
    Closed,
}

impl ConfigWaitOutcome {
    fn into_change(self) -> Option<ConfigChange> {
        match self {
            Self::Changed(change) => Some(change),
            Self::TimedOut | Self::Closed => None,
        }
    }
}

struct WatchState {
    generation: u64,
    closed: bool,
}

/// 进程内配置变更总线。
///
/// - [`subscribe`](Self::subscribe) 返回订阅句柄
/// - [`notify`](Self::notify) 广播 generation++
/// - [`reload`](Self::reload)：由调用方显式触发，从 [`LayeredConfig`] 重载到 store 后 notify
///
/// watch mutation 由独立 mutex 串行化；state mutex 只保护短暂检查与提交，不跨 store 写锁等待。
pub struct ConfigWatch {
    mutation: Mutex<()>,
    state: Mutex<WatchState>,
    cvar: Condvar,
    #[cfg(test)]
    reload_phase_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl ConfigWatch {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self {
            mutation: Mutex::new(()),
            state: Mutex::new(WatchState { generation: 0, closed: false }),
            cvar: Condvar::new(),
            #[cfg(test)]
            reload_phase_hook: Mutex::new(None),
        }
    }

    /// 当前 generation（未通知或锁中毒时为 0）。
    #[must_use]
    pub fn generation(&self) -> u64 {
        let Ok(_mutation) = self.mutation.lock() else {
            return 0;
        };
        self.state.lock().map(|state| state.generation).unwrap_or(0)
    }

    /// 订阅；返回从当前 generation 之后等待的句柄。
    #[must_use]
    pub fn subscribe(self: &Arc<Self>) -> ConfigSubscription {
        let seen = self.generation();
        ConfigSubscription { watch: Arc::clone(self), seen }
    }

    /// 广播一次变更（generation +1）。
    ///
    /// # Errors
    ///
    /// mutation/state 锁中毒、watch 已关闭或 generation 溢出时返回 [`XError::invalid`]。
    pub fn notify(&self) -> XResult<ConfigChange> {
        let _mutation = self.lock_mutation()?;
        let mut state = self.lock_state()?;
        let generation = next_generation(&state)?;
        state.generation = generation;
        let change = ConfigChange { generation };
        self.cvar.notify_all();
        Ok(change)
    }

    /// 从多层源手动重载到 store，成功后 notify。
    ///
    /// 完整 load/key 校验先在锁外完成。随后 mutation mutex 串行化 watch 更新：短暂检查 state 后释放
    /// state 锁，再等待 store 写锁；store 整图替换是配置线性化点，generation 发布在 mutation mutex
    /// 释放前完成。state 锁从不跨 store 写锁等待。
    ///
    /// # Errors
    ///
    /// 配置源加载/校验失败、watch 锁中毒/关闭/generation 溢出或 store 写锁中毒时返回错误。
    pub fn reload(&self, layered: &LayeredConfig, store: &ConfigStore) -> XResult<ConfigChange> {
        let merged = layered.load_merged()?;
        let _mutation = self.lock_mutation()?;
        let generation = {
            let state = self.lock_state()?;
            next_generation(&state)?
        };

        #[cfg(test)]
        if let Some(hook) =
            self.reload_phase_hook.lock().expect("test reload phase hook lock").as_ref()
        {
            hook();
        }
        store.replace_entries(merged)?;

        let mut state = self.lock_state()?;
        state.generation = generation;
        let change = ConfigChange { generation };
        self.cvar.notify_all();
        Ok(change)
    }

    /// 关闭；后续 wait 返回关闭结果。
    ///
    /// # Errors
    ///
    /// mutation/state 锁中毒时返回 [`XError::invalid`]。
    pub fn close(&self) -> XResult<()> {
        let _mutation = self.lock_mutation()?;
        let mut state = self.lock_state()?;
        state.closed = true;
        self.cvar.notify_all();
        Ok(())
    }

    fn lock_mutation(&self) -> XResult<MutexGuard<'_, ()>> {
        self.mutation.lock().map_err(|_| XError::invalid(WATCH_MUTATION_LOCK_POISONED_CONTEXT))
    }

    fn lock_state(&self) -> XResult<MutexGuard<'_, WatchState>> {
        self.state.lock().map_err(|_| XError::invalid(WATCH_STATE_LOCK_POISONED_CONTEXT))
    }

    #[cfg(test)]
    fn set_reload_phase_hook(&self, hook: Arc<dyn Fn() + Send + Sync>) {
        *self.reload_phase_hook.lock().expect("test reload phase hook lock") = Some(hook);
    }
}

fn next_generation(state: &WatchState) -> XResult<u64> {
    if state.closed {
        return Err(XError::invalid(WATCH_CLOSED_CONTEXT));
    }
    state
        .generation
        .checked_add(1)
        .ok_or_else(|| XError::invalid(WATCH_GENERATION_OVERFLOW_CONTEXT))
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

    /// 阻塞直到 generation 增长或 watch 关闭，并返回显式结果。
    ///
    /// # Errors
    ///
    /// watch state 锁中毒时返回 [`XError::invalid`]。
    pub fn wait_outcome(&mut self) -> XResult<ConfigWaitOutcome> {
        let watch = Arc::clone(&self.watch);
        let mut state = watch.lock_state()?;
        loop {
            if let Some(outcome) = observe(&mut self.seen, &state) {
                return Ok(outcome);
            }
            state = watch
                .cvar
                .wait(state)
                .map_err(|_| XError::invalid(WATCH_STATE_LOCK_POISONED_CONTEXT))?;
        }
    }

    /// 兼容等待 API；关闭折叠为 `None`。
    ///
    /// # Errors
    ///
    /// watch state 锁中毒时返回 [`XError::invalid`]。
    pub fn wait(&mut self) -> XResult<Option<ConfigChange>> {
        Ok(self.wait_outcome()?.into_change())
    }

    /// 在总 deadline 内等待，并显式区分变更、超时与关闭。
    ///
    /// state mutex 只通过 `try_lock` 读取；竞争不会造成无界 mutex 等待。轮询 sleep 每次最多 1ms，
    /// 且始终受调用开始时刻计算出的剩余时长约束。
    ///
    /// # Errors
    ///
    /// watch state 锁中毒时返回 [`XError::invalid`]。
    pub fn wait_timeout_outcome(&mut self, timeout: Duration) -> XResult<ConfigWaitOutcome> {
        let started = Instant::now();
        self.wait_timeout_outcome_with(timeout, || started.elapsed(), thread::sleep)
    }

    fn wait_timeout_outcome_with<Elapsed, Sleep>(
        &mut self,
        timeout: Duration,
        mut elapsed: Elapsed,
        mut sleep: Sleep,
    ) -> XResult<ConfigWaitOutcome>
    where
        Elapsed: FnMut() -> Duration,
        Sleep: FnMut(Duration),
    {
        let watch = Arc::clone(&self.watch);
        loop {
            let current_elapsed = elapsed();
            if current_elapsed >= timeout {
                return Ok(ConfigWaitOutcome::TimedOut);
            }
            match watch.state.try_lock() {
                Ok(state) => {
                    if state.closed {
                        return Ok(ConfigWaitOutcome::Closed);
                    }
                    if state.generation > self.seen {
                        if elapsed() >= timeout {
                            return Ok(ConfigWaitOutcome::TimedOut);
                        }
                        self.seen = state.generation;
                        return Ok(ConfigWaitOutcome::Changed(ConfigChange {
                            generation: state.generation,
                        }));
                    }
                }
                Err(TryLockError::Poisoned(_)) => {
                    return Err(XError::invalid(WATCH_STATE_LOCK_POISONED_CONTEXT));
                }
                Err(TryLockError::WouldBlock) => {}
            }

            let remaining = timeout.saturating_sub(current_elapsed);
            sleep(remaining.min(TIMED_WAIT_POLL_INTERVAL));
        }
    }

    /// 兼容限时等待 API；超时与关闭均折叠为 `None`。
    ///
    /// # Errors
    ///
    /// watch state 锁中毒时返回 [`XError::invalid`]。
    pub fn wait_timeout(&mut self, timeout: Duration) -> XResult<Option<ConfigChange>> {
        Ok(self.wait_timeout_outcome(timeout)?.into_change())
    }
}

fn observe(seen: &mut u64, state: &WatchState) -> Option<ConfigWaitOutcome> {
    if state.closed {
        return Some(ConfigWaitOutcome::Closed);
    }
    if state.generation > *seen {
        *seen = state.generation;
        return Some(ConfigWaitOutcome::Changed(ConfigChange { generation: state.generation }));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;
    use kernel::ErrorKind;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Barrier, mpsc};

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
    fn explicit_wait_outcome_reports_changed_and_closed() {
        let watch = Arc::new(ConfigWatch::new());
        let mut changed = watch.subscribe();
        watch.notify().unwrap();
        assert_eq!(
            changed.wait_outcome().unwrap(),
            ConfigWaitOutcome::Changed(ConfigChange { generation: 1 })
        );

        let mut closed = watch.subscribe();
        watch.close().unwrap();
        assert_eq!(closed.wait_outcome().unwrap(), ConfigWaitOutcome::Closed);
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
    fn reload_releases_state_lock_while_waiting_for_store_and_serializes_notify() {
        let watch = Arc::new(ConfigWatch::new());
        let store = Arc::new(ConfigStore::new());
        store.set("old", "value").unwrap();
        let store_guard = store.data.write().unwrap();
        let phase_entered = Arc::new(Barrier::new(2));
        let release_phase = Arc::new(Barrier::new(2));
        let hook_entered = Arc::clone(&phase_entered);
        let hook_release = Arc::clone(&release_phase);
        watch.set_reload_phase_hook(Arc::new(move || {
            hook_entered.wait();
            hook_release.wait();
        }));
        let layered = LayeredConfig::new()
            .with_source(Arc::new(MemorySource::from_pairs([("new", "value")])));
        let reload_watch = Arc::clone(&watch);
        let reload_store = Arc::clone(&store);
        let reload_thread = thread::spawn(move || reload_watch.reload(&layered, &reload_store));

        phase_entered.wait();
        assert!(watch.state.try_lock().is_ok(), "reload 等待 store 时不得持有 state 锁");
        assert!(matches!(watch.mutation.try_lock(), Err(TryLockError::WouldBlock)));

        let notify_watch = Arc::clone(&watch);
        let notify_started = Arc::new(Barrier::new(2));
        let notify_ready = Arc::clone(&notify_started);
        let notify_thread = thread::spawn(move || {
            notify_ready.wait();
            notify_watch.notify()
        });
        notify_started.wait();
        assert!(matches!(watch.mutation.try_lock(), Err(TryLockError::WouldBlock)));

        release_phase.wait();
        assert!(watch.state.try_lock().is_ok(), "reload 进入 store 等待前 state 锁必须已释放");
        drop(store_guard);
        assert_eq!(reload_thread.join().unwrap().unwrap().generation, 1);
        assert_eq!(notify_thread.join().unwrap().unwrap().generation, 2);
        assert_eq!(store.try_get("new").unwrap().as_deref(), Some("value"));
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
        let outcome = sub.wait_timeout_outcome(Duration::from_millis(50)).unwrap();
        assert_eq!(outcome, ConfigWaitOutcome::Changed(ConfigChange { generation: 1 }));
        assert_eq!(sub.seen(), 1);
        assert_eq!(
            sub.wait_timeout_outcome(Duration::from_millis(20)).unwrap(),
            ConfigWaitOutcome::TimedOut
        );
        assert!(sub.wait_timeout(Duration::from_millis(1)).unwrap().is_none());
    }

    #[test]
    fn notify_after_close_errors() {
        let watch = ConfigWatch::new();
        watch.close().unwrap();
        assert_invalid_context(watch.notify().unwrap_err(), WATCH_CLOSED_CONTEXT);
    }

    #[test]
    fn wait_timeout_sees_close() {
        let watch = Arc::new(ConfigWatch::new());
        let mut explicit = watch.subscribe();
        let mut compatible = watch.subscribe();
        let w2 = Arc::clone(&watch);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            w2.close().unwrap();
        });
        assert_eq!(
            explicit.wait_timeout_outcome(Duration::from_secs(1)).unwrap(),
            ConfigWaitOutcome::Closed
        );
        assert!(compatible.wait_timeout(Duration::from_secs(1)).unwrap().is_none());
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

    #[test]
    fn wait_timeout_rejects_generation_arriving_at_deadline() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let late_watch = Arc::clone(&watch);
        let timeout = Duration::from_millis(10);
        let mut elapsed_calls = 0usize;
        let outcome = sub
            .wait_timeout_outcome_with(
                timeout,
                || {
                    elapsed_calls += 1;
                    if elapsed_calls == 1 { Duration::ZERO } else { timeout }
                },
                move |_| {
                    late_watch.notify().unwrap();
                },
            )
            .unwrap();
        assert_eq!(outcome, ConfigWaitOutcome::TimedOut);
        assert_eq!(sub.seen(), 0);
        assert_eq!(watch.generation(), 1);

        let visible_watch = Arc::new(ConfigWatch::new());
        let mut visible_sub = visible_watch.subscribe();
        visible_watch.notify().unwrap();
        let mut acceptance_checks = 0usize;
        let outcome = visible_sub
            .wait_timeout_outcome_with(
                timeout,
                || {
                    acceptance_checks += 1;
                    if acceptance_checks == 1 { Duration::ZERO } else { timeout }
                },
                drop,
            )
            .unwrap();
        assert_eq!(outcome, ConfigWaitOutcome::TimedOut);
        assert_eq!(visible_sub.seen(), 0);
    }

    #[test]
    fn wait_timeout_is_bounded_when_state_mutex_is_held() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let state_guard = watch.state.lock().unwrap();
        let (tx, rx) = mpsc::channel();
        let waiter = thread::spawn(move || {
            tx.send(sub.wait_timeout_outcome(Duration::from_millis(30))).unwrap();
        });

        let outcome =
            rx.recv_timeout(Duration::from_millis(150)).expect("等待必须在 deadline 附近返回");
        assert_eq!(outcome.unwrap(), ConfigWaitOutcome::TimedOut);
        drop(state_guard);
        waiter.join().unwrap();
    }

    #[test]
    fn notify_reports_generation_overflow() {
        let watch = ConfigWatch::new();
        watch.state.lock().unwrap().generation = u64::MAX;
        let err = watch.notify().unwrap_err();
        assert_invalid_context(err, WATCH_GENERATION_OVERFLOW_CONTEXT);
        assert_eq!(watch.generation(), u64::MAX);
    }

    #[test]
    fn reload_does_not_replace_store_when_generation_overflows() {
        let watch = ConfigWatch::new();
        watch.state.lock().unwrap().generation = u64::MAX;
        let store = ConfigStore::new();
        store.set("old", "value").unwrap();
        let layered = LayeredConfig::new()
            .with_source(Arc::new(MemorySource::from_pairs([("new", "value")])));
        assert_invalid_context(
            watch.reload(&layered, &store).unwrap_err(),
            WATCH_GENERATION_OVERFLOW_CONTEXT,
        );
        assert_eq!(store.try_get("old").unwrap().as_deref(), Some("value"));
        assert_eq!(store.try_get("new").unwrap(), None);
    }

    #[test]
    fn wait_timeout_deadline_survives_actual_spurious_notifications() {
        let watch = Arc::new(ConfigWatch::new());
        let mut sub = watch.subscribe();
        let ready = Arc::new(Barrier::new(2));
        let done = Arc::new(AtomicBool::new(false));
        let notifications = Arc::new(AtomicUsize::new(0));
        let waiter_ready = Arc::clone(&ready);
        let waiter_done = Arc::clone(&done);
        let waiter = thread::spawn(move || {
            waiter_ready.wait();
            let started = Instant::now();
            let outcome = sub.wait_timeout_outcome(Duration::from_millis(40)).unwrap();
            waiter_done.store(true, Ordering::Release);
            (outcome, started.elapsed())
        });

        ready.wait();
        watch.cvar.notify_all();
        notifications.fetch_add(1, Ordering::Relaxed);
        while !done.load(Ordering::Acquire) {
            watch.cvar.notify_all();
            notifications.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(2));
        }

        let (outcome, elapsed) = waiter.join().unwrap();
        assert_eq!(outcome, ConfigWaitOutcome::TimedOut);
        assert!(notifications.load(Ordering::Relaxed) > 1, "必须实际发送多次伪通知");
        assert!(elapsed < Duration::from_millis(120), "实际等待 {elapsed:?}");
    }

    #[test]
    fn poisoned_mutation_and_state_locks_return_errors() {
        let mutation_watch = ConfigWatch::new();
        let mutation_poisoned = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = mutation_watch.mutation.lock().unwrap();
            panic!("故意毒化 mutation lock");
        }));
        assert!(mutation_poisoned.is_err());
        assert_invalid_context(
            mutation_watch.notify().unwrap_err(),
            WATCH_MUTATION_LOCK_POISONED_CONTEXT,
        );
        assert_eq!(mutation_watch.generation(), 0);

        let state_watch = Arc::new(ConfigWatch::new());
        let mut sub = state_watch.subscribe();
        let state_poisoned = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = state_watch.state.lock().unwrap();
            panic!("故意毒化 state lock");
        }));
        assert!(state_poisoned.is_err());
        assert_invalid_context(
            state_watch.notify().unwrap_err(),
            WATCH_STATE_LOCK_POISONED_CONTEXT,
        );
        assert_invalid_context(
            sub.wait_timeout_outcome(Duration::from_millis(1)).unwrap_err(),
            WATCH_STATE_LOCK_POISONED_CONTEXT,
        );
        assert_eq!(state_watch.generation(), 0);
    }

    fn assert_invalid_context(error: XError, expected: &str) {
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert_eq!(error.context(), expected);
    }
}
