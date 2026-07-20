//! ManualClock property tests（SPEC-TESTKIT-002 §13.3）。

use std::time::Duration;

use kernel::Timestamp;
use proptest::prelude::*;
use testkit::{ManualClock, ManualClockError};

fn ts(n: i64) -> Timestamp {
    Timestamp::from_unix_nanos(n)
}

proptest! {
    // 关闭文件失败种子持久化：Miri isolation 下 cwd/`current_dir` 不可用。
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// 任意可表示的 wall + 小 delta：成功则等于 checked_add；失败则状态不变。
    #[test]
    fn advance_wall_checked(
        start in -1_000_000_000_000_i64..1_000_000_000_000_i64,
        nanos in 0_u64..1_000_000_u64,
    ) {
        let c = ManualClock::new(ts(start));
        let before = c.snapshot().unwrap();
        let delta = Duration::from_nanos(nanos);
        match c.advance_wall(delta) {
            Ok(next) => {
                let expect = ts(start).checked_add(delta).expect("ok path implies checked_add");
                assert_eq!(next, expect);
                assert_eq!(c.snapshot().unwrap().wall(), expect);
            }
            Err(ManualClockError::WallOverflow) => {
                assert!(ts(start).checked_add(delta).is_none());
                assert_eq!(c.snapshot().unwrap(), before);
            }
            Err(e) => panic!("unexpected {e:?}"),
        }
    }

    #[test]
    fn rewind_wall_checked(
        start in -1_000_000_000_000_i64..1_000_000_000_000_i64,
        nanos in 0_u64..1_000_000_u64,
    ) {
        let c = ManualClock::new(ts(start));
        let before = c.snapshot().unwrap();
        let delta = Duration::from_nanos(nanos);
        match c.rewind_wall(delta) {
            Ok(next) => {
                let expect = ts(start).checked_sub(delta).expect("ok path");
                assert_eq!(next, expect);
                assert_eq!(c.snapshot().unwrap().wall(), expect);
            }
            Err(ManualClockError::WallOverflow) => {
                assert!(ts(start).checked_sub(delta).is_none());
                assert_eq!(c.snapshot().unwrap(), before);
            }
            Err(e) => panic!("unexpected {e:?}"),
        }
    }

    #[test]
    fn mono_advance_then_reject_regression(start in 0_u64..1_000_000_u64, step in 1_u64..10_000_u64) {
        let c = ManualClock::with_monotonic_elapsed(ts(0), Duration::from_nanos(start));
        c.advance_monotonic(Duration::from_nanos(step)).unwrap();
        let after = start + step;
        let before = c.snapshot().unwrap();
        assert!(matches!(
            c.set_monotonic_elapsed(Duration::from_nanos(after.saturating_sub(1))),
            Err(ManualClockError::MonotonicRegression)
        ));
        assert_eq!(c.snapshot().unwrap(), before);
    }

    /// 任意可表示的 mono elapsed + delta：成功则 equals checked_add；失败则 snapshot 不变。
    #[test]
    fn mono_advance_checked(
        start in 0_u64..1_000_000_000_u64,
        nanos in 0_u64..1_000_000_u64,
    ) {
        let c = ManualClock::with_monotonic_elapsed(ts(0), Duration::from_nanos(start));
        let before = c.snapshot().unwrap();
        let delta = Duration::from_nanos(nanos);
        match c.advance_monotonic(delta) {
            Ok(inst) => {
                let expect = Duration::from_nanos(start)
                    .checked_add(delta)
                    .expect("ok path implies checked_add");
                let origin = kernel::MonotonicInstant::from_clock_elapsed(Duration::ZERO);
                assert_eq!(
                    inst.checked_duration_since(origin).expect("duration since origin"),
                    expect
                );
                assert_eq!(c.snapshot().unwrap().monotonic_elapsed(), expect);
                // 墙钟不得被 mono 控制副作用改写
                assert_eq!(c.snapshot().unwrap().wall(), before.wall());
            }
            Err(ManualClockError::MonotonicOverflow) => {
                assert!(Duration::from_nanos(start).checked_add(delta).is_none());
                assert_eq!(c.snapshot().unwrap(), before);
            }
            Err(e) => panic!("unexpected {e:?}"),
        }
    }

    /// 任意 fault set/clear sequence：wall 与 mono 值保持；仅 wall_fault 字段变化。
    #[test]
    fn fault_set_clear_sequence_preserves_clock_values(
        wall in -1_000_000_i64..1_000_000_i64,
        mono_nanos in 0_u64..1_000_000_u64,
        fault_kind in 0_u8..3_u8,
        clear_first in any::<bool>(),
    ) {
        use testkit::ManualClockFault;
        let fault = match fault_kind {
            0 => ManualClockFault::BeforeUnixEpoch,
            1 => ManualClockFault::Overflow,
            _ => ManualClockFault::Unavailable,
        };
        let c = ManualClock::with_monotonic_elapsed(ts(wall), Duration::from_nanos(mono_nanos));
        let baseline = c.snapshot().unwrap();
        assert!(baseline.wall_fault().is_none());

        if clear_first {
            // clear 在无 fault 时为幂等 no-op
            c.clear_wall_fault().unwrap();
            assert_eq!(c.snapshot().unwrap(), baseline);
        }

        c.set_wall_fault(fault).unwrap();
        let with_fault = c.snapshot().unwrap();
        assert_eq!(with_fault.wall(), baseline.wall());
        assert_eq!(with_fault.monotonic_elapsed(), baseline.monotonic_elapsed());
        assert_eq!(with_fault.wall_fault(), Some(fault));

        // 再 set 另一 fault 仍不改 wall/mono
        let other = match fault {
            ManualClockFault::BeforeUnixEpoch => ManualClockFault::Unavailable,
            ManualClockFault::Overflow => ManualClockFault::BeforeUnixEpoch,
            ManualClockFault::Unavailable => ManualClockFault::Overflow,
            _ => ManualClockFault::Overflow,
        };
        c.set_wall_fault(other).unwrap();
        let swapped = c.snapshot().unwrap();
        assert_eq!(swapped.wall(), baseline.wall());
        assert_eq!(swapped.monotonic_elapsed(), baseline.monotonic_elapsed());
        assert_eq!(swapped.wall_fault(), Some(other));

        c.clear_wall_fault().unwrap();
        let cleared = c.snapshot().unwrap();
        assert_eq!(cleared.wall(), baseline.wall());
        assert_eq!(cleared.monotonic_elapsed(), baseline.monotonic_elapsed());
        assert!(cleared.wall_fault().is_none());
    }
}
