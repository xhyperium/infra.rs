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
}
