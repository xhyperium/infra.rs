//! 组件生命周期状态语言与关停原语。
//!
//! # 设计原则
//!
//! 本模块只提供共同语言和关停原语：
//!
//! - 不提供启动编排；
//! - 不提供健康检查；
//! - 不提供自动重启；
//! - 不依赖 tokio 等异步运行时；
//! - 关停必须一次触发、多方观察、不可逆；
//! - 阻塞等待不得存在 lost wake-up。
//!
//! 并发原语在 `cfg(loom)` 下切换为 `loom::sync`，供模型检验（SPEC §7.6 / §11.2）。
//! **本版本不公开 `Component` trait。**

#[cfg(loom)]
use loom::sync::{Arc, Condvar, Mutex};
#[cfg(not(loom))]
use std::sync::{Arc, Condvar, Mutex};
#[cfg(not(loom))]
use std::time::Duration;

// ---------------------------------------------------------------------------
// ComponentState
// ---------------------------------------------------------------------------

/// 组件的生命周期状态。
///
/// 合法转换：
///
/// ```text
/// Created  → Starting
/// Starting → Running
/// Starting → Failed
/// Running  → Draining
/// Running  → Failed
/// Draining → Stopped
/// Draining → Failed
/// ```
///
/// 其他转换一律非法。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentState {
    /// 组件已创建，尚未开始初始化。
    Created,
    /// 组件正在初始化。
    Starting,
    /// 组件正常运行。
    Running,
    /// 组件正在排空中。
    Draining,
    /// 组件已停止。
    Stopped,
    /// 组件已失败。
    Failed,
}

impl ComponentState {
    /// 检查 `self` 是否可合法转换为 `to`。
    pub const fn can_transition_to(self, to: ComponentState) -> bool {
        matches!(
            (self, to),
            (ComponentState::Created, ComponentState::Starting)
                | (ComponentState::Starting, ComponentState::Running)
                | (ComponentState::Starting, ComponentState::Failed)
                | (ComponentState::Running, ComponentState::Draining)
                | (ComponentState::Running, ComponentState::Failed)
                | (ComponentState::Draining, ComponentState::Stopped)
                | (ComponentState::Draining, ComponentState::Failed)
        )
    }

    /// 尝试从 `self` 转换到 `to`。
    ///
    /// 合法转换返回 `Ok(to)`，非法转换返回 `Err(LifecycleError)`。
    /// 此方法不会 panic。
    pub fn try_transition(self, to: ComponentState) -> Result<ComponentState, LifecycleError> {
        if self.can_transition_to(to) { Ok(to) } else { Err(LifecycleError { from: self, to }) }
    }
}

// ---------------------------------------------------------------------------
// LifecycleError
// ---------------------------------------------------------------------------

/// 非法的组件状态转换错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("非法组件状态转换: {from:?} -> {to:?}")]
pub struct LifecycleError {
    /// 当前状态。
    pub from: ComponentState,
    /// 尝试转换到的目标状态。
    pub to: ComponentState,
}

// ---------------------------------------------------------------------------
// ShutdownInner
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ShutdownInner {
    triggered: Mutex<bool>,
    cv: Condvar,
}

/// 带 deadline 的关停等待失败。
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WaitTimeoutError {
    /// `std::time::Instant` 无法表示请求的 deadline。
    #[error("关停等待 deadline 超出平台可表示范围")]
    DeadlineOverflow,
}

// ---------------------------------------------------------------------------
// ShutdownSignal
// ---------------------------------------------------------------------------

/// 关停信号，可被多个观察者共享。
///
/// 通过 [`ShutdownGuard::trigger`] 触发后，所有已阻塞的 [`ShutdownSignal::wait`]
/// 调用方被唤醒；触发后新创建的观察者也立即看到已触发状态。
///
/// 协议：`Mutex<bool>` + `Condvar`；`trigger` **持锁**写标志并 `notify_all`，避免 lost wake-up。
///
/// `ShutdownSignal` 可 [`Clone`]，`ShutdownGuard` 不可 [`Clone`]。
#[must_use]
#[derive(Clone, Debug)]
pub struct ShutdownSignal {
    inner: Arc<ShutdownInner>,
}

impl ShutdownSignal {
    /// 创建一对 `(ShutdownGuard, ShutdownSignal)`。
    ///
    /// `guard` 是唯一的触发入口；`signal` 可被克隆并分发给观察者。
    pub fn new() -> (ShutdownGuard, ShutdownSignal) {
        let inner = Arc::new(ShutdownInner { triggered: Mutex::new(false), cv: Condvar::new() });
        let signal = ShutdownSignal { inner: Arc::clone(&inner) };
        let guard = ShutdownGuard { inner };
        (guard, signal)
    }

    /// 检查关停是否已触发。
    ///
    /// 锁中毒时按 SPEC §10.2 用 `into_inner` 恢复，不把 poison 当作对外 panic 合同。
    pub fn is_triggered(&self) -> bool {
        let triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        *triggered
    }

    /// 阻塞等待直到关停被触发。若已触发则立即返回。
    ///
    /// 使用 `while !triggered` 循环配合 [`Condvar::wait`]，确保不会出现
    /// lost wake-up。锁中毒时按 SPEC §10.2 用 `into_inner` 恢复。
    pub fn wait(&self) {
        let mut triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        while !*triggered {
            triggered = self.inner.cv.wait(triggered).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// 阻塞直到关停被触发或超时。
    ///
    /// 返回 `Ok(true)` 表示已触发；`Ok(false)` 表示超时仍未触发。
    /// 请求的 deadline 超出平台可表示范围时返回 [`WaitTimeoutError`]，不得把该错误
    /// 伪装为普通超时。
    /// 若调用前已触发，则完成状态优先，在构造 deadline 前立即返回 `Ok(true)`。
    /// 组合根应在丢弃 [`ShutdownGuard`] 前设定 deadline；超时后升级（告警/强制退出）。
    ///
    /// # Errors
    ///
    /// 信号尚未触发，且 `timeout` 无法与当前 [`std::time::Instant`] 组成可表示的
    /// deadline 时返回 [`WaitTimeoutError::DeadlineOverflow`]。
    ///
    /// 在 `cfg(loom)` 下不可用（loom Condvar 无 `wait_timeout`）。
    #[cfg(not(loom))]
    pub fn wait_timeout(&self, timeout: Duration) -> Result<bool, WaitTimeoutError> {
        let mut triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        if *triggered {
            return Ok(true);
        }
        let now0 = std::time::Instant::now();
        let deadline = now0.checked_add(timeout).ok_or(WaitTimeoutError::DeadlineOverflow)?;
        while !*triggered {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            let remaining = deadline.saturating_duration_since(now);
            let (guard, result) =
                self.inner.cv.wait_timeout(triggered, remaining).unwrap_or_else(|e| e.into_inner());
            triggered = guard;
            if result.timed_out() && !*triggered {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

// ---------------------------------------------------------------------------
// ShutdownGuard
// ---------------------------------------------------------------------------

/// 关停守卫，唯一的触发入口。
///
/// 消费 `guard` 即触发关停。直接 drop guard 不会自动触发关停、不会 panic、
/// 不会记录日志——由组合根保证 guard 生命周期。
#[must_use]
#[derive(Debug)]
pub struct ShutdownGuard {
    inner: Arc<ShutdownInner>,
}

impl ShutdownGuard {
    /// 触发关停信号，唤醒所有阻塞在 [`ShutdownSignal::wait`] 的观察者。
    ///
    /// 此方法消费 `guard`，触发后不可重置。
    /// SPEC §7.6 顺序：持同一 mutex → 设 true → `notify_all` → 释放锁。
    pub fn trigger(self) {
        let mut triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        *triggered = true;
        self.inner.cv.notify_all();
    }
}

// ---------------------------------------------------------------------------
// 单元测试（标准 std 路径；loom 模型见 tests/lifecycle_concurrency_loom.rs）
// ---------------------------------------------------------------------------

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;

    // -- ComponentState 合法转换 ------------------------------------------

    #[test]
    fn test_legal_transitions() {
        assert!(ComponentState::Created.can_transition_to(ComponentState::Starting));
        assert!(ComponentState::Starting.can_transition_to(ComponentState::Running));
        assert!(ComponentState::Starting.can_transition_to(ComponentState::Failed));
        assert!(ComponentState::Running.can_transition_to(ComponentState::Draining));
        assert!(ComponentState::Running.can_transition_to(ComponentState::Failed));
        assert!(ComponentState::Draining.can_transition_to(ComponentState::Stopped));
        assert!(ComponentState::Draining.can_transition_to(ComponentState::Failed));
    }

    #[test]
    fn test_illegal_transitions() {
        // Created can only go to Starting
        assert!(!ComponentState::Created.can_transition_to(ComponentState::Running));
        assert!(!ComponentState::Created.can_transition_to(ComponentState::Failed));
        assert!(!ComponentState::Created.can_transition_to(ComponentState::Draining));
        assert!(!ComponentState::Created.can_transition_to(ComponentState::Stopped));

        // Stopped is terminal
        assert!(!ComponentState::Stopped.can_transition_to(ComponentState::Created));
        assert!(!ComponentState::Stopped.can_transition_to(ComponentState::Running));

        // Failed is terminal
        assert!(!ComponentState::Failed.can_transition_to(ComponentState::Created));
        assert!(!ComponentState::Failed.can_transition_to(ComponentState::Running));

        // Running cannot go backwards
        assert!(!ComponentState::Running.can_transition_to(ComponentState::Starting));
        assert!(!ComponentState::Running.can_transition_to(ComponentState::Created));

        // Draining cannot go to anything except Stopped or Failed
        assert!(!ComponentState::Draining.can_transition_to(ComponentState::Running));
        assert!(!ComponentState::Draining.can_transition_to(ComponentState::Created));
    }

    #[test]
    fn test_try_transition_legal() {
        assert_eq!(
            ComponentState::Created.try_transition(ComponentState::Starting).unwrap(),
            ComponentState::Starting
        );
    }

    #[test]
    fn test_try_transition_illegal() {
        let err = ComponentState::Created.try_transition(ComponentState::Running).unwrap_err();
        assert_eq!(err.from, ComponentState::Created);
        assert_eq!(err.to, ComponentState::Running);
    }

    #[test]
    fn test_all_transition_pairs() {
        let states = [
            ComponentState::Created,
            ComponentState::Starting,
            ComponentState::Running,
            ComponentState::Draining,
            ComponentState::Stopped,
            ComponentState::Failed,
        ];
        for &from in &states {
            for &to in &states {
                let result = from.try_transition(to);
                match result {
                    Ok(_) => assert!(from.can_transition_to(to)),
                    Err(e) => {
                        assert!(!from.can_transition_to(to));
                        assert_eq!(e.from, from);
                        assert_eq!(e.to, to);
                    }
                }
            }
        }
    }

    // -- Shutdown: trigger before wait ------------------------------------

    #[test]
    fn test_trigger_before_wait() {
        let (guard, signal) = ShutdownSignal::new();
        guard.trigger();
        assert!(signal.is_triggered());
        // wait should return immediately
        signal.wait();
    }

    // -- Shutdown: wait before trigger ------------------------------------

    #[test]
    fn test_wait_before_trigger() {
        let (guard, signal) = ShutdownSignal::new();
        let signal_clone = signal.clone();

        let handle = thread::spawn(move || {
            signal_clone.wait();
            assert!(signal_clone.is_triggered());
        });

        // Small delay to ensure waiter is parked
        thread::sleep(std::time::Duration::from_millis(10));
        guard.trigger();
        handle.join().unwrap();
    }

    // -- Shutdown: multi-observer -----------------------------------------

    #[test]
    fn test_multi_observer() {
        let (guard, signal) = ShutdownSignal::new();
        let n = 5;
        let barrier = Arc::new(Barrier::new(n + 1));
        let mut handles = Vec::new();

        for _ in 0..n {
            let sig = signal.clone();
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait(); // all threads ready
                sig.wait();
            }));
        }

        // Wait for all threads to be ready
        barrier.wait();
        guard.trigger();

        for h in handles {
            h.join().unwrap();
        }
    }

    // -- Shutdown: trigger 后新 observer 立即可见 ------------------------

    #[test]
    fn test_new_observer_sees_triggered() {
        let (guard, signal) = ShutdownSignal::new();
        guard.trigger();
        assert!(signal.is_triggered());

        let cloned = signal.clone();
        assert!(cloned.is_triggered());
        cloned.wait(); // should return immediately
    }

    // -- Shutdown: signal 可 Clone ---------------------------------------

    #[test]
    fn test_signal_cloneable() {
        let (_guard, signal) = ShutdownSignal::new();
        let _s2 = signal.clone();
        let _s3 = signal.clone();
    }

    // -- Shutdown: guard drop 不触发 -------------------------------------

    #[test]
    fn test_guard_drop_does_not_trigger() {
        let (guard, signal) = ShutdownSignal::new();
        drop(guard);
        assert!(!signal.is_triggered());
    }

    // -- Shutdown: poison recovery（真实注入 mutex 毒锁） ------------------

    #[test]
    fn test_poison_recovery_into_inner() {
        let (guard, signal) = ShutdownSignal::new();

        // 同模块可触及 `ShutdownInner.triggered`，注入真实 poison。
        // 首次 lock 必成功，用 unwrap（避免测试里未覆盖的 poison 分支污染 line 覆盖率）。
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = signal.inner.triggered.lock().expect("lock before poison");
            panic!("intentional mutex poison for §11.1 poison recovery");
        }));
        assert!(poison.is_err(), "setup must poison the mutex");

        // 毒锁后仍可读；标志未改写（走 is_triggered 的 into_inner 恢复）
        assert!(!signal.is_triggered());

        // 毒锁后仍可 trigger，并与 is_triggered / wait 一致
        guard.trigger();
        assert!(signal.is_triggered());
        signal.wait();
    }

    /// 覆盖 `wait` 中 `Condvar::wait` 返回后的 poison 恢复路径（§10.2）。
    #[test]
    fn test_wait_poison_recovery_on_condvar_reacquire() {
        let (guard, signal) = ShutdownSignal::new();
        let waiter = {
            let s = signal.clone();
            std::thread::spawn(move || s.wait())
        };
        // 等到 waiter 进入 Condvar::wait（已释放 mutex）
        std::thread::sleep(std::time::Duration::from_millis(50));

        // 在 wait 阻塞期间毒化 mutex
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = signal.inner.triggered.lock().expect("lock to poison");
            panic!("poison while waiter blocked");
        }));
        assert!(poison.is_err());

        // trigger 经 into_inner 写标志 + notify；waiter 在 reacquire 时走 poison 恢复
        guard.trigger();
        waiter.join().expect("waiter");
        assert!(signal.is_triggered());
    }

    // -- Shutdown: 并发回归测试 ------------------------------------------

    #[test]
    fn test_shutdown_concurrency_stress() {
        let n_observers = 10;
        let n_iterations = 100;

        for _ in 0..n_iterations {
            let (g, s) = ShutdownSignal::new();
            let mut handles = Vec::new();

            for _ in 0..n_observers {
                let sig = s.clone();
                handles.push(thread::spawn(move || {
                    sig.wait();
                }));
            }

            g.trigger();

            for h in handles {
                h.join().unwrap();
            }
        }
    }

    #[test]
    fn wait_timeout_returns_false_before_trigger() {
        let (_guard, signal) = ShutdownSignal::new();
        assert!(!signal.wait_timeout(Duration::from_millis(20)).unwrap());
    }

    #[test]
    fn wait_timeout_returns_true_after_trigger() {
        let (guard, signal) = ShutdownSignal::new();
        guard.trigger();
        assert!(signal.wait_timeout(Duration::from_secs(1)).unwrap());
    }

    #[test]
    fn wait_timeout_completed_state_precedes_deadline_validation() {
        let (guard, signal) = ShutdownSignal::new();
        guard.trigger();
        assert_eq!(signal.wait_timeout(Duration::MAX), Ok(true));
    }

    #[test]
    fn composition_root_deadline_upgrade_path() {
        let (guard, signal) = ShutdownSignal::new();
        let observer = signal.clone();
        let done = thread::spawn(move || observer.wait_timeout(Duration::from_millis(30)).unwrap());
        let timed_out = !done.join().expect("join");
        assert!(timed_out, "deadline exceeded without trigger");
        guard.trigger();
        assert!(signal.is_triggered());
    }

    #[test]
    fn lifecycle_error_display_is_chinese() {
        let err = LifecycleError { from: ComponentState::Running, to: ComponentState::Starting };
        assert!(err.to_string().contains("非法"));
    }

    #[test]
    fn wait_timeout_huge_timeout_checked_add_no_panic() {
        let (_g, s) = ShutdownSignal::new();
        let error = s
            .wait_timeout(Duration::MAX)
            .expect_err("不可表示的 deadline 必须显式报错，不能伪装成普通超时");
        assert_eq!(error, WaitTimeoutError::DeadlineOverflow);
    }

    #[test]
    fn wait_timeout_zero_deadline_branch() {
        let (_g, s) = ShutdownSignal::new();
        // 零超时：立即走 deadline 分支
        assert!(!s.wait_timeout(Duration::from_millis(0)).unwrap());
    }

    #[test]
    fn wait_timeout_spuriously_not_triggered_until_timeout() {
        let (_g, s) = ShutdownSignal::new();
        assert!(!s.wait_timeout(Duration::from_millis(5)).unwrap());
    }

    #[test]
    fn wait_timeout_true_after_trigger_from_other_thread() {
        let (guard, signal) = ShutdownSignal::new();
        let s2 = signal.clone();
        let h = thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            guard.trigger();
        });
        assert!(s2.wait_timeout(Duration::from_secs(2)).unwrap());
        h.join().unwrap();
    }
}
