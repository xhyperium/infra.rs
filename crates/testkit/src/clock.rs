//! [`ManualClock`]：`kernel::Clock` 的确定性测试替身（SPEC-TESTKIT-002 §7）。
//!
//! - 墙钟与单调钟独立可控
//! - 控制路径全部 `checked`，失败不修改状态
//! - 一致快照与 fault 注入在同一 `Mutex` 临界区线性化
//! - **无** `Default` / `Clone`；共享请使用 [`std::sync::Arc`]

use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use kernel::{Clock, ClockDomain, ClockError, MonotonicInstant, Timestamp};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ManualClockError {
    /// 墙钟 checked 加减溢出。
    #[error("手动时钟墙钟时间溢出")]
    WallOverflow,
    /// 单调钟推进溢出。
    #[error("手动时钟单调流逝溢出")]
    MonotonicOverflow,
    /// 试图将单调钟回拨或 set 到更小值。
    #[error("手动时钟单调流逝回退")]
    MonotonicRegression,
    /// 状态锁同步失败（poison 在控制路径上报告；`monotonic` 读取走恢复策略）。
    #[error("手动时钟状态锁同步失败")]
    Synchronization,
}

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
    domain: ClockDomain,
}

impl ManualClock {
    fn next_domain() -> ClockDomain {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(100);
        ClockDomain::from_raw(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// 以初始墙钟构造；单调起点为 0。每个实例拥有独立 [`ClockDomain`]。
    pub fn new(initial_wall: Timestamp) -> Self {
        Self::with_monotonic_elapsed(initial_wall, Duration::from_nanos(0))
    }

    /// 以初始墙钟与单调流逝构造。
    pub fn with_monotonic_elapsed(initial_wall: Timestamp, monotonic_elapsed: Duration) -> Self {
        Self {
            state: Mutex::new(State { wall: initial_wall, monotonic_elapsed, wall_fault: None }),
            domain: Self::next_domain(),
        }
    }

    /// 本实例的单调 domain（跨 ManualClock 实例比较间隔为 `None`）。
    pub const fn domain(&self) -> ClockDomain {
        self.domain
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
    ///
    /// # Errors
    ///
    /// 状态锁已中毒时返回 [`ManualClockError::Synchronization`]。
    pub fn set_wall(&self, wall: Timestamp) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall = wall;
        Ok(())
    }

    /// 墙钟前进（checked）；失败不修改状态。
    ///
    /// # Errors
    ///
    /// 结果超出 [`Timestamp`] 范围时返回 [`ManualClockError::WallOverflow`]；状态锁已中毒时返回
    /// [`ManualClockError::Synchronization`]。
    pub fn advance_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError> {
        let mut g = self.lock()?;
        let next = g.wall.checked_add(delta).ok_or(ManualClockError::WallOverflow)?;
        g.wall = next;
        Ok(next)
    }

    /// 墙钟回退（checked）；允许回拨；失败不修改状态。
    ///
    /// # Errors
    ///
    /// 结果超出 [`Timestamp`] 范围时返回 [`ManualClockError::WallOverflow`]；状态锁已中毒时返回
    /// [`ManualClockError::Synchronization`]。
    pub fn rewind_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError> {
        let mut g = self.lock()?;
        let next = g.wall.checked_sub(delta).ok_or(ManualClockError::WallOverflow)?;
        g.wall = next;
        Ok(next)
    }

    /// 设置单调流逝；新值严格小于当前值时返回 [`ManualClockError::MonotonicRegression`]。
    ///
    /// # Errors
    ///
    /// 新值回退时返回 [`ManualClockError::MonotonicRegression`]；状态锁已中毒时返回
    /// [`ManualClockError::Synchronization`]。
    pub fn set_monotonic_elapsed(&self, elapsed: Duration) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        if elapsed < g.monotonic_elapsed {
            return Err(ManualClockError::MonotonicRegression);
        }
        g.monotonic_elapsed = elapsed;
        Ok(())
    }

    /// 单调钟前进；失败不修改状态；不提供 rewind。
    ///
    /// # Errors
    ///
    /// 流逝时间溢出时返回 [`ManualClockError::MonotonicOverflow`]；状态锁已中毒时返回
    /// [`ManualClockError::Synchronization`]。
    pub fn advance_monotonic(&self, delta: Duration) -> Result<MonotonicInstant, ManualClockError> {
        let mut g = self.lock()?;
        let next =
            g.monotonic_elapsed.checked_add(delta).ok_or(ManualClockError::MonotonicOverflow)?;
        g.monotonic_elapsed = next;
        Ok(MonotonicInstant::from_clock_elapsed_in(next, self.domain))
    }

    /// 注入墙钟 fault；不改变已保存的 wall 值；不影响单调钟。
    ///
    /// # Errors
    ///
    /// 状态锁已中毒时返回 [`ManualClockError::Synchronization`]。
    pub fn set_wall_fault(&self, fault: ManualClockFault) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall_fault = Some(fault);
        Ok(())
    }

    /// 清除墙钟 fault。
    ///
    /// # Errors
    ///
    /// 状态锁已中毒时返回 [`ManualClockError::Synchronization`]。
    pub fn clear_wall_fault(&self) -> Result<(), ManualClockError> {
        let mut g = self.lock()?;
        g.wall_fault = None;
        Ok(())
    }

    /// 当前墙钟 fault。
    ///
    /// # Errors
    ///
    /// 状态锁已中毒时返回 [`ManualClockError::Synchronization`]。
    pub fn wall_fault(&self) -> Result<Option<ManualClockFault>, ManualClockError> {
        let g = self.lock()?;
        Ok(g.wall_fault)
    }

    /// 一致快照（同锁读取全部字段）。
    ///
    /// # Errors
    ///
    /// 状态锁已中毒时返回 [`ManualClockError::Synchronization`]。
    pub fn snapshot(&self) -> Result<ManualClockSnapshot, ManualClockError> {
        let g = self.lock()?;
        Ok(ManualClockSnapshot {
            wall: g.wall,
            monotonic_elapsed: g.monotonic_elapsed,
            wall_fault: g.wall_fault,
        })
    }

    #[cfg(test)]
    pub(crate) fn poison_state_for_test(&self) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = self.state.lock().expect("测试毒化前必须取得状态锁");
            panic!("测试专用 ManualClock 状态锁毒化");
        }));
        assert!(result.is_err(), "测试专用毒化必须捕获内部 panic");
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
        MonotonicInstant::from_clock_elapsed_in(g.monotonic_elapsed, self.domain)
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
                .checked_duration_since(MonotonicInstant::from_clock_elapsed_in(
                    Duration::ZERO,
                    c.domain()
                ))
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

    /// §13.1 / §7.8：monotonic advance 溢出返回 `MonotonicOverflow` 且不修改状态。
    #[test]
    fn mono_overflow_does_not_mutate() {
        let c = ManualClock::with_monotonic_elapsed(ts(0), Duration::MAX);
        let before = c.snapshot().unwrap();
        assert!(matches!(
            c.advance_monotonic(Duration::from_nanos(1)),
            Err(ManualClockError::MonotonicOverflow)
        ));
        assert_eq!(c.snapshot().unwrap(), before);
        // 墙钟与 fault 也不得被副作用改写
        assert_eq!(c.now().unwrap().as_unix_nanos(), 0);
        assert_eq!(c.wall_fault().unwrap(), None);
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
                .checked_duration_since(MonotonicInstant::from_clock_elapsed_in(
                    Duration::ZERO,
                    c.domain()
                ))
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
        // 两阶段 barrier：启动同步 + reader 完成首次读后再允许 controller 推进
        let start = Arc::new(std::sync::Barrier::new(5)); // 4 readers + controller
        let primed = Arc::new(std::sync::Barrier::new(5));
        let mut handles = vec![];
        for _ in 0..4 {
            let c = Arc::clone(&c);
            let start = Arc::clone(&start);
            let primed = Arc::clone(&primed);
            handles.push(thread::spawn(move || {
                start.wait();
                // 首次读路径（覆盖 now/monotonic/snapshot）
                let _ = c.now();
                let _ = c.monotonic();
                let _ = c.snapshot();
                primed.wait();
                // 固定次数并发读，避免 stop 标志与 controller 的竞态漏计覆盖率
                for _ in 0..50 {
                    let _ = c.now();
                    let _ = c.monotonic();
                    let _ = c.snapshot();
                }
            }));
        }
        start.wait();
        primed.wait();
        for i in 0..200u64 {
            c.advance_wall(Duration::from_nanos(1)).unwrap();
            c.advance_monotonic(Duration::from_nanos(1)).unwrap();
            if i % 50 == 0 {
                c.set_wall_fault(ManualClockFault::Unavailable).unwrap();
                c.clear_wall_fault().unwrap();
            }
        }
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
        assert!(wall.contains("墙钟") || wall.contains("溢出"), "{wall}");
        assert!(mono_ov.contains("单调") || mono_ov.contains("溢出"), "{mono_ov}");
        assert!(mono_reg.contains("回退") || mono_reg.contains("单调"), "{mono_reg}");
        assert!(sync.contains("同步") || sync.contains("锁"), "{sync}");
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

    /// 毒化 state mutex 后：`monotonic` 走 `lock_recover`；控制路径报 Synchronization；
    /// `Clock::now` 映射 Unavailable。
    #[test]
    fn poison_recovery_and_control_path_errors() {
        let c = ManualClock::new(ts(7));
        c.advance_monotonic(Duration::from_nanos(3)).unwrap();

        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = c.state.lock().expect("lock before poison");
            panic!("intentional ManualClock state poison");
        }));
        assert!(poison.is_err());

        // lock_recover：不 panic、不伪造零值
        assert_eq!(
            c.monotonic()
                .checked_duration_since(MonotonicInstant::from_clock_elapsed_in(
                    Duration::ZERO,
                    c.domain()
                ))
                .unwrap(),
            Duration::from_nanos(3)
        );

        // 控制路径：poison → Synchronization，状态保持
        assert!(matches!(c.set_wall(ts(1)), Err(ManualClockError::Synchronization)));
        assert!(matches!(
            c.advance_wall(Duration::from_nanos(1)),
            Err(ManualClockError::Synchronization)
        ));
        assert!(matches!(
            c.rewind_wall(Duration::from_nanos(1)),
            Err(ManualClockError::Synchronization)
        ));
        assert!(matches!(
            c.set_monotonic_elapsed(Duration::from_nanos(9)),
            Err(ManualClockError::Synchronization)
        ));
        assert!(matches!(
            c.advance_monotonic(Duration::from_nanos(1)),
            Err(ManualClockError::Synchronization)
        ));
        assert!(matches!(
            c.set_wall_fault(ManualClockFault::Unavailable),
            Err(ManualClockError::Synchronization)
        ));
        assert!(matches!(c.clear_wall_fault(), Err(ManualClockError::Synchronization)));
        assert!(matches!(c.wall_fault(), Err(ManualClockError::Synchronization)));
        assert!(matches!(c.snapshot(), Err(ManualClockError::Synchronization)));

        // Clock::now 毒锁 → Unavailable
        assert!(matches!(c.now(), Err(ClockError::Unavailable)));
    }

    #[test]
    fn cross_manual_clock_domain_duration_is_none() {
        let a = ManualClock::new(Timestamp::from_unix_nanos(0));
        let b = ManualClock::new(Timestamp::from_unix_nanos(0));
        assert_ne!(a.domain(), b.domain());
        let ia = a.monotonic();
        let ib = b.monotonic();
        assert!(ia.checked_duration_since(ib).is_none());
        assert!(ib.checked_duration_since(ia).is_none());
        // 同实例 OK
        assert_eq!(ia.checked_duration_since(ia), Some(Duration::ZERO));
    }

    #[test]
    fn poison_contract_documented_entries() {
        // poison 后：控制 API → Synchronization；now → Unavailable；monotonic 恢复 inner
        // （既有 poison 测试覆盖；此处锁定 domain 仍可用）
        let c = ManualClock::new(Timestamp::from_unix_nanos(1));
        let d = c.domain();
        assert_eq!(c.monotonic().domain(), d);
    }
}
