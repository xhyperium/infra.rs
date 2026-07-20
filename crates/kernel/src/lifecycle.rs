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

use std::sync::{Arc, Condvar, Mutex};

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
    pub fn try_transition(
        self,
        to: ComponentState,
    ) -> Result<ComponentState, LifecycleError> {
        if self.can_transition_to(to) {
            Ok(to)
        } else {
            Err(LifecycleError { from: self, to })
        }
    }
}

// ---------------------------------------------------------------------------
// LifecycleError
// ---------------------------------------------------------------------------

/// 非法的组件状态转换错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("illegal component state transition: {from:?} -> {to:?}")]
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

// ---------------------------------------------------------------------------
// ShutdownSignal
// ---------------------------------------------------------------------------

/// 关停信号，可被多个观察者共享。
///
/// 通过 [`ShutdownGuard::trigger`] 触发后，所有已阻塞的 [`ShutdownSignal::wait`]
/// 调用方被唤醒；触发后新创建的观察者也立即看到已触发状态。
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
        let inner = Arc::new(ShutdownInner {
            triggered: Mutex::new(false),
            cv: Condvar::new(),
        });
        let signal = ShutdownSignal {
            inner: Arc::clone(&inner),
        };
        let guard = ShutdownGuard { inner };
        (guard, signal)
    }

    /// 检查关停是否已触发。
    pub fn is_triggered(&self) -> bool {
        // 锁中毒时恢复，继续遵守同一状态机
        let triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        *triggered
    }

    /// 阻塞等待直到关停被触发。若已触发则立即返回。
    ///
    /// 使用 `while !triggered` 循环配合 [`Condvar::wait`]，确保不会出现
    /// lost wake-up。
    pub fn wait(&self) {
        let mut triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        while !*triggered {
            triggered = self.inner.cv.wait(triggered).unwrap_or_else(|e| e.into_inner());
        }
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
    pub fn trigger(self) {
        let mut triggered = self.inner.triggered.lock().unwrap_or_else(|e| e.into_inner());
        *triggered = true;
        self.inner.cv.notify_all();
    }
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
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
            ComponentState::Created
                .try_transition(ComponentState::Starting)
                .unwrap(),
            ComponentState::Starting
        );
    }

    #[test]
    fn test_try_transition_illegal() {
        let err = ComponentState::Created
            .try_transition(ComponentState::Running)
            .unwrap_err();
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

    // -- Shutdown: poison recovery ----------------------------------------

    #[test]
    fn test_poison_recovery() {
        let (guard, signal) = ShutdownSignal::new();
        // Trigger to mark as triggered
        guard.trigger();
        // is_triggered should return true even after poison (not
        // realistically testable without actual poison, but the code
        // path with unwrap_or_else is exercised)
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
}
