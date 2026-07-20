//! [`ManualClock`]：`kernel::Clock` 的确定性测试替身（SPEC-TESTKIT-002 §7）。
//!
//! - 墙钟与单调钟独立可控
//! - 控制路径全部 `checked`，失败不修改状态
//! - 一致快照与 fault 注入在同一 `Mutex` 临界区线性化
//! - **无** `Default` / `Clone`；共享请使用 [`std::sync::Arc`]

use std::fmt;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use kernel::{Clock, ClockError, MonotonicInstant, Timestamp};

/// 墙钟 fault 注入（映射到 [`ClockError`]）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualClockFault {
    /// 映射 [`ClockError::BeforeUnixEpoch`]。
    BeforeUnixEpoch,
    /// 映射 [`ClockError::Overflow`]。
    Overflow,
    /// 映射 [`ClockError::Unavailable`]。
    Unavailable,
}

impl ManualClockFault {
    fn to_clock_error(self) -> ClockError {
        match self {
            ManualClockFault::BeforeUnixEpoch => ClockError::BeforeUnixEpoch,
            ManualClockFault::Overflow => ClockError::Overflow,
            ManualClockFault::Unavailable => ClockError::Unavailable,
        }
    }
}

/// 控制路径失败（不通过 [`Clock::now`] 返回）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualClockError {
    /// 墙钟 checked 加减溢出。
    WallOverflow,
    /// 单调钟推进溢出。
    MonotonicOverflow,
    /// 试图将单调钟回拨或 set 到更小值。
    MonotonicRegression,
    /// 状态锁同步失败（poison 在控制路径上报告；`monotonic` 读取走恢复策略）。
    Synchronization,
}

impl fmt::Display for ManualClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManualClockError::WallOverflow => write!(f, "manual clock wall time overflow"),
            ManualClockError::MonotonicOverflow => {
                write!(f, "manual clock monotonic elapsed overflow")
            }
            ManualClockError::MonotonicRegression => {
                write!(f, "manual clock monotonic elapsed regression")
            }
            ManualClockError::Synchronization => {
                write!(f, "manual clock state lock synchronization failure")
            }
        }
    }
}

impl std::error::Error for ManualClockError {}

/// 某一时刻的一致快照（同锁临界区读取）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManualClockSnapshot {
    wall: Timestamp,
    monotonic_elapsed: Duration,
    wall_fault: Option<ManualClockFault>,
}

impl ManualClockSnapshot {
    /// 快照中的墙钟。
    pub const fn wall(&self) -> Timestamp {
        self.wall
    }

    /// 快照中的单调流逝。
    pub const fn monotonic_elapsed(&self) -> Duration {
        self.monotonic_elapsed
    }

    /// 快照中的墙钟 fault（若有）。
    pub const fn wall_fault(&self) -> Option<ManualClockFault> {
        self.wall_fault
    }
}

#[derive(Debug)]
struct State {
    wall: Timestamp,
    monotonic_elapsed: Duration,
    wall_fault: Option<ManualClockFault>,
}

/// `Clock` 的确定性测试替身。
///
/// 不实现 [`Clone`]：共享时间线请使用 `Arc<ManualClock>`。
/// 不实现 [`Default`]：禁止把未初始化输入伪装成 epoch 零点。
#[derive(Debug)]
pub struct ManualClock {
    state: Mutex<State>,
}

impl ManualClock {
    /// 以初始墙钟构造；单调起点为 0。
    pub fn new(initial_wall: Timestamp) -> Self {
        Self::with_monotonic_elapsed(initial_wall, Duration::from_nanos(0))
    }

    /// 以初始墙钟与单调流逝构造。
    pub fn with_monotonic_elapsed(initial_wall: Timestamp, monotonic_elapsed: Duration) -> Self {
        Self {
            state: Mutex::new(State { wall: initial_wall, monotonic_elapsed, wall_fault: None }),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, State>, ManualClockError> {
        self.state.lock().map_err(|_| ManualClockError::Synchronization)
    }

    /// 锁中毒时恢复 inner（仅用于 [`Clock::monotonic`] 无错误通道）。
    ///
    /// 恢复语义：返回 poison 前的 `State`；不伪造零值；不 panic。
    fn lock_recover(&self) -> MutexGuard<'_, State> {
        match self.state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// 直接设置墙钟；失败不修改状态。
    pub fn set_wall(&self, wall: Timestamp) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall = wall;
        Ok(())
    }

    /// 墙钟前进（checked）；失败不修改状态。
    pub fn advance_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError> {
        let mut g = self.lock()?;
        let next = g.wall.checked_add(delta).ok_or(ManualClockError::WallOverflow)?;
        g.wall = next;
        Ok(next)
    }

    /// 墙钟回退（checked）；允许回拨；失败不修改状态。
    pub fn rewind_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError> {
        let mut g = self.lock()?;
        let next = g.wall.checked_sub(delta).ok_or(ManualClockError::WallOverflow)?;
        g.wall = next;
        Ok(next)
    }

    /// 设置单调流逝；新值严格小于当前值时返回 [`ManualClockError::MonotonicRegression`]。
    pub fn set_monotonic_elapsed(&self, elapsed: Duration) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        if elapsed < g.monotonic_elapsed {
            return Err(ManualClockError::MonotonicRegression);
        }
        g.monotonic_elapsed = elapsed;
        Ok(())
    }

    /// 单调钟前进；失败不修改状态；不提供 rewind。
    pub fn advance_monotonic(&self, delta: Duration) -> Result<MonotonicInstant, ManualClockError> {
        let mut g = self.lock()?;
        let next =
            g.monotonic_elapsed.checked_add(delta).ok_or(ManualClockError::MonotonicOverflow)?;
        g.monotonic_elapsed = next;
        Ok(MonotonicInstant::from_clock_elapsed(next))
    }

    /// 注入墙钟 fault；不改变已保存的 wall 值；不影响单调钟。
    pub fn set_wall_fault(&self, fault: ManualClockFault) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall_fault = Some(fault);
        Ok(())
    }

    /// 清除墙钟 fault。
    pub fn clear_wall_fault(&self) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall_fault = None;
        Ok(())
    }

    /// 当前墙钟 fault。
    pub fn wall_fault(&self) -> Result<Option<ManualClockFault>, ManualClockError> {
        let g = self.lock()?;
        Ok(g.wall_fault)
    }

    /// 一致快照（同锁读取全部字段）。
    pub fn snapshot(&self) -> Result<ManualClockSnapshot, ManualClockError> {
        let g = self.lock()?;
        Ok(ManualClockSnapshot {
            wall: g.wall,
            monotonic_elapsed: g.monotonic_elapsed,
            wall_fault: g.wall_fault,
        })
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Result<Timestamp, ClockError> {
        let g = self.state.lock().map_err(|_| ClockError::Unavailable)?;
        if let Some(fault) = g.wall_fault {
            return Err(fault.to_clock_error());
        }
        Ok(g.wall)
    }

    fn monotonic(&self) -> MonotonicInstant {
        // poison 恢复：见 `lock_recover` 文档语义（I-CLK-POISON）。
        let g = self.lock_recover();
        MonotonicInstant::from_clock_elapsed(g.monotonic_elapsed)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn ts(n: i64) -> Timestamp {
        Timestamp::from_unix_nanos(n)
    }

    #[test]
    fn construct_initial_wall_and_zero_mono() {
        let c = ManualClock::new(ts(1_000));
        assert_eq!(c.now().unwrap().as_unix_nanos(), 1_000);
        let snap = c.snapshot().unwrap();
        assert_eq!(snap.monotonic_elapsed(), Duration::from_nanos(0));
        assert!(snap.wall_fault().is_none());
    }

    #[test]
    fn with_monotonic_elapsed_initial() {
        let c = ManualClock::with_monotonic_elapsed(ts(5), Duration::from_nanos(9));
        assert_eq!(
            c.monotonic()
                .checked_duration_since(MonotonicInstant::from_clock_elapsed(Duration::ZERO))
                .unwrap(),
            Duration::from_nanos(9)
        );
    }

    #[test]
    fn wall_advance_and_rewind() {
        let c = ManualClock::new(ts(100));
        assert_eq!(c.advance_wall(Duration::from_nanos(50)).unwrap().as_unix_nanos(), 150);
        assert_eq!(c.rewind_wall(Duration::from_nanos(20)).unwrap().as_unix_nanos(), 130);
        c.set_wall(ts(200)).unwrap();
        assert_eq!(c.now().unwrap().as_unix_nanos(), 200);
    }

    #[test]
    fn wall_overflow_does_not_mutate() {
        let c = ManualClock::new(ts(i64::MAX));
        let before = c.snapshot().unwrap();
        assert!(matches!(
            c.advance_wall(Duration::from_nanos(1)),
            Err(ManualClockError::WallOverflow)
        ));
        assert_eq!(c.snapshot().unwrap(), before);
    }

    #[test]
    fn wall_rewind_underflow_does_not_mutate() {
        let c = ManualClock::new(ts(i64::MIN));
        let before = c.snapshot().unwrap();
        assert!(matches!(
            c.rewind_wall(Duration::from_nanos(1)),
            Err(ManualClockError::WallOverflow)
        ));
        assert_eq!(c.snapshot().unwrap(), before);
    }

    #[test]
    fn mono_advance_and_regression() {
        let c = ManualClock::new(ts(0));
        c.advance_monotonic(Duration::from_nanos(10)).unwrap();
        let before = c.snapshot().unwrap();
        assert!(matches!(
            c.set_monotonic_elapsed(Duration::from_nanos(5)),
            Err(ManualClockError::MonotonicRegression)
        ));
        assert_eq!(c.snapshot().unwrap(), before);
        c.set_monotonic_elapsed(Duration::from_nanos(10)).unwrap();
        c.set_monotonic_elapsed(Duration::from_nanos(11)).unwrap();
    }

    #[test]
    fn fault_injection_preserves_wall_and_mono() {
        let c = ManualClock::new(ts(42));
        c.advance_monotonic(Duration::from_nanos(7)).unwrap();
        c.set_wall_fault(ManualClockFault::Unavailable).unwrap();
        assert!(matches!(c.now(), Err(ClockError::Unavailable)));
        let snap = c.snapshot().unwrap();
        assert_eq!(snap.wall().as_unix_nanos(), 42);
        assert_eq!(snap.monotonic_elapsed(), Duration::from_nanos(7));
        assert_eq!(snap.wall_fault(), Some(ManualClockFault::Unavailable));
        // mono 不受 wall fault 影响
        assert_eq!(
            c.monotonic()
                .checked_duration_since(MonotonicInstant::from_clock_elapsed(Duration::ZERO))
                .unwrap(),
            Duration::from_nanos(7)
        );
        c.clear_wall_fault().unwrap();
        assert_eq!(c.now().unwrap().as_unix_nanos(), 42);
    }

    #[test]
    fn fault_variants_map_to_clock_error() {
        let c = ManualClock::new(ts(1));
        c.set_wall_fault(ManualClockFault::BeforeUnixEpoch).unwrap();
        assert!(matches!(c.now(), Err(ClockError::BeforeUnixEpoch)));
        c.set_wall_fault(ManualClockFault::Overflow).unwrap();
        assert!(matches!(c.now(), Err(ClockError::Overflow)));
    }

    #[test]
    fn snapshot_consistent_under_control() {
        let c = ManualClock::new(ts(10));
        c.advance_wall(Duration::from_nanos(5)).unwrap();
        c.advance_monotonic(Duration::from_nanos(3)).unwrap();
        c.set_wall_fault(ManualClockFault::Overflow).unwrap();
        let s = c.snapshot().unwrap();
        assert_eq!(s.wall().as_unix_nanos(), 15);
        assert_eq!(s.monotonic_elapsed(), Duration::from_nanos(3));
        assert_eq!(s.wall_fault(), Some(ManualClockFault::Overflow));
    }

    #[test]
    fn wall_and_mono_independent() {
        let c = ManualClock::new(ts(100));
        c.advance_monotonic(Duration::from_nanos(1_000)).unwrap();
        c.rewind_wall(Duration::from_nanos(50)).unwrap();
        assert_eq!(c.now().unwrap().as_unix_nanos(), 50);
        assert_eq!(c.snapshot().unwrap().monotonic_elapsed(), Duration::from_nanos(1_000));
    }

    #[test]
    fn determinism_without_control_call() {
        let c = ManualClock::new(ts(9));
        let a = c.now().unwrap();
        let b = c.now().unwrap();
        assert_eq!(a, b);
        let m1 = c.monotonic();
        let m2 = c.monotonic();
        assert_eq!(m1, m2);
    }

    #[test]
    fn send_sync_and_arc_share() {
        fn assert_bounds<T: Clock + Send + Sync>(_: &T) {}
        let c = Arc::new(ManualClock::new(ts(1)));
        assert_bounds(c.as_ref());
        let c2 = Arc::clone(&c);
        c2.advance_wall(Duration::from_nanos(1)).unwrap();
        assert_eq!(c.now().unwrap().as_unix_nanos(), 2);
    }

    #[test]
    fn concurrent_readers_and_controller() {
        let c = Arc::new(ManualClock::new(ts(0)));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut handles = vec![];
        for _ in 0..4 {
            let c = Arc::clone(&c);
            let stop = Arc::clone(&stop);
            handles.push(thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = c.now();
                    let _ = c.monotonic();
                    let _ = c.snapshot();
                }
            }));
        }
        for i in 0..200u64 {
            c.advance_wall(Duration::from_nanos(1)).unwrap();
            c.advance_monotonic(Duration::from_nanos(1)).unwrap();
            if i % 50 == 0 {
                c.set_wall_fault(ManualClockFault::Unavailable).unwrap();
                c.clear_wall_fault().unwrap();
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        for h in handles {
            h.join().unwrap();
        }
        let s = c.snapshot().unwrap();
        assert_eq!(s.wall().as_unix_nanos(), 200);
        assert_eq!(s.monotonic_elapsed(), Duration::from_nanos(200));
    }

    #[test]
    fn error_display_is_nonempty_and_distinct() {
        let wall = ManualClockError::WallOverflow.to_string();
        let mono_ov = ManualClockError::MonotonicOverflow.to_string();
        let mono_reg = ManualClockError::MonotonicRegression.to_string();
        let sync = ManualClockError::Synchronization.to_string();
        assert!(wall.contains("wall"), "{wall}");
        assert!(wall.contains("overflow"), "{wall}");
        assert!(mono_ov.contains("monotonic"), "{mono_ov}");
        assert!(mono_ov.contains("overflow"), "{mono_ov}");
        assert!(mono_reg.contains("regression"), "{mono_reg}");
        assert!(sync.contains("synchronization") || sync.contains("lock"), "{sync}");
        // 禁止 Display 静默变成空串（杀 Display::fmt -> Ok(()) 类突变）
        assert!(
            !wall.is_empty() && !mono_ov.is_empty() && !mono_reg.is_empty() && !sync.is_empty()
        );
        assert_ne!(wall, mono_ov);
        assert_ne!(wall, mono_reg);
        assert_ne!(wall, sync);
    }

    #[test]
    fn snapshot_getters_return_stored_fields() {
        let c = ManualClock::new(ts(99));
        c.advance_monotonic(Duration::from_nanos(5)).unwrap();
        c.set_wall_fault(ManualClockFault::Overflow).unwrap();
        let s = c.snapshot().unwrap();
        // 必须经 getter 断言（杀 wall()/monotonic_elapsed()/wall_fault() 返回 Default/None）
        assert_eq!(s.wall().as_unix_nanos(), 99);
        assert_eq!(s.monotonic_elapsed(), Duration::from_nanos(5));
        assert_eq!(s.wall_fault(), Some(ManualClockFault::Overflow));
    }

    #[test]
    fn wall_fault_accessor_reports_current_fault() {
        let c = ManualClock::new(ts(1));
        assert_eq!(c.wall_fault().unwrap(), None);
        c.set_wall_fault(ManualClockFault::BeforeUnixEpoch).unwrap();
        // 直接调用 ManualClock::wall_fault（非 snapshot）
        assert_eq!(c.wall_fault().unwrap(), Some(ManualClockFault::BeforeUnixEpoch));
        c.clear_wall_fault().unwrap();
        assert_eq!(c.wall_fault().unwrap(), None);
    }
}
