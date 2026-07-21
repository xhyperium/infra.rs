//! SPEC-KERNEL-002 §11 — Clock / Timestamp / ErrorKind / ComponentState 合同。
//!
//! 注意：`MonotonicInstant::from_clock_elapsed` 仅允许出现在
//! `crates/kernel/src/clock.rs` 与 `crates/testkit/*`（SPEC §6.3 / TIME-004）。
//! 双通道 ControlledClock 与依赖该构造器的单调属性测位于 `src/clock.rs` 单元测试。

use kernel::{Clock, ClockError, ComponentState, ErrorKind, SystemClock, Timestamp, XError};
use proptest::prelude::*;
use std::time::Duration;

#[test]
fn system_clock_now_and_mono_contract() {
    let c = SystemClock::new();
    let _timestamp = c.now().expect("SystemClock::now 必须可表示");
    let a = c.monotonic();
    let b = c.monotonic();
    // 单调非递减（同进程内 elapsed 不回退）
    assert!(b.checked_duration_since(a).is_some());
}

/// SystemClock 只对单调通道承诺非递减；墙钟读数仅验证可表示。
#[test]
fn system_clock_monotonic_series_non_decreasing() {
    let c = SystemClock::new();
    let mut mono = c.monotonic();
    for _ in 0..32 {
        let _wall = c.now().expect("墙钟必须可表示");
        let m = c.monotonic();
        assert!(m.checked_duration_since(mono).is_some());
        mono = m;
    }
}

#[test]
fn test_system_clock_now_returns_valid_timestamp() {
    let clock = SystemClock::new();
    let ts = clock.now().expect("墙钟应可用");
    // 应晚于 2000-01-01
    let y2k_nanos: i64 = 946_684_800_000_000_000;
    assert!(ts.as_unix_nanos() > y2k_nanos);
}

#[test]
fn test_system_clock_default_works() {
    let clock = SystemClock::new();
    let ts = clock.now().expect("墙钟应可用");
    assert!(ts.as_unix_nanos() > 0);
}

/// 时间错误映射为 `XError` Unavailable（§5.7 / §11.1）。
#[test]
fn clock_error_maps_to_xerror_unavailable() {
    for err in [ClockError::BeforeUnixEpoch, ClockError::Overflow, ClockError::Unavailable] {
        let xe: XError = err.into();
        assert_eq!(xe.kind(), ErrorKind::Unavailable);
        assert!(!xe.is_retryable());
        assert!(!xe.is_bug());
    }
}

/// Timestamp 边界：MIN/MAX 与 u64 Duration 溢出语义。
#[test]
fn timestamp_min_max_and_u64_edges() {
    let min = Timestamp::from_unix_nanos(i64::MIN);
    let max = Timestamp::from_unix_nanos(i64::MAX);
    assert_eq!(min.as_unix_nanos(), i64::MIN);
    assert_eq!(max.as_unix_nanos(), i64::MAX);
    let full_i64_span = Duration::from_nanos(u64::MAX);
    assert_eq!(min.checked_add(full_i64_span), Some(max));
    assert_eq!(max.checked_sub(full_i64_span), Some(min));
    assert_eq!(max.checked_duration_since(min), Some(full_i64_span));
    assert!(min.checked_duration_since(max).is_none());
    // MAX + 1ns 溢出
    assert!(max.checked_add(Duration::from_nanos(1)).is_none());
    // MIN - 1ns 下溢
    assert!(min.checked_sub(Duration::from_nanos(1)).is_none());
    // MAX 相对自身 since = 0
    assert_eq!(max.checked_duration_since(max), Some(Duration::ZERO));
    // 大 Duration：超过 i64 可表示差分
    let huge = Duration::from_secs(u64::from(u32::MAX));
    // 从 0 加 huge 可能成功或失败，取决于 i64 范围；不得 panic
    let _ = Timestamp::from_unix_nanos(0).checked_add(huge);
    let near_max = Timestamp::from_unix_nanos(i64::MAX - 10);
    assert!(near_max.checked_add(Duration::from_nanos(20)).is_none());
    assert_eq!(near_max.checked_add(Duration::from_nanos(10)).unwrap().as_unix_nanos(), i64::MAX);
}

/// ComponentState 合法边全枚举 + 非法边全矩阵。
#[test]
fn component_state_transition_matrix() {
    use ComponentState::*;
    let legal = [
        (Created, Starting),
        (Starting, Running),
        (Starting, Failed),
        (Running, Draining),
        (Running, Failed),
        (Draining, Stopped),
        (Draining, Failed),
    ];
    let all = [Created, Starting, Running, Draining, Stopped, Failed];
    for (from, to) in legal {
        assert!(from.can_transition_to(to), "{from:?}->{to:?}");
        assert_eq!(from.try_transition(to).unwrap(), to);
    }
    for from in all {
        for to in all {
            let allowed = legal.contains(&(from, to));
            assert_eq!(from.can_transition_to(to), allowed, "矩阵不一致 {from:?}->{to:?}");
            assert_eq!(from.try_transition(to).is_ok(), allowed);
        }
    }
}

/// 任意 `Duration`：由秒 + 亚秒纳秒合成，覆盖大秒数与亚秒分量（§11.3）。
fn arb_duration() -> impl Strategy<Value = Duration> {
    (
        prop_oneof![
            Just(0u64),
            0u64..10_000,
            10_000u64..1_000_000,
            any::<u32>().prop_map(u64::from),
            Just(u64::MAX),
        ],
        0u32..1_000_000_000,
    )
        .prop_map(|(secs, nanos)| Duration::new(secs, nanos))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 任意 i64 Timestamp × 任意 Duration 的 checked 运算不 panic，且可逆时一致。
    #[test]
    fn timestamp_checked_add_sub_agree(
        nanos in any::<i64>(),
        d in arb_duration(),
    ) {
        let t = Timestamp::from_unix_nanos(nanos);
        if let Some(t2) = t.checked_add(d) {
            // 可表示 i64 差分时，since 必须等于 d
            if let (Ok(delta_i64), Some(back)) = (
                i64::try_from(d.as_nanos()),
                t2.checked_duration_since(t),
            ) {
                if t2.as_unix_nanos().checked_sub(t.as_unix_nanos()) == Some(delta_i64) {
                    prop_assert_eq!(back, d);
                }
            }
            // sub 往返
            if let Some(t0) = t2.checked_sub(d) {
                prop_assert_eq!(t0.as_unix_nanos(), t.as_unix_nanos());
            }
        } else {
            // 溢出路径：不得 panic；对 MAX 基座 + 正 duration 必为 None
            if nanos == i64::MAX && d > Duration::ZERO {
                prop_assert!(t.checked_add(d).is_none());
            }
        }
        // sub 独立路径也不 panic
        let _ = t.checked_sub(d);
    }

    #[test]
    fn timestamp_reverse_since_is_none(
        a in any::<i64>(),
        b in any::<i64>(),
    ) {
        let ta = Timestamp::from_unix_nanos(a);
        let tb = Timestamp::from_unix_nanos(b);
        if a < b {
            prop_assert!(ta.checked_duration_since(tb).is_none());
        }
        if a > b {
            prop_assert!(tb.checked_duration_since(ta).is_none());
        }
        if a == b {
            prop_assert_eq!(ta.checked_duration_since(tb), Some(Duration::ZERO));
        }
    }

    /// 靠近 i64 边界的 Timestamp：加减不 panic，溢出为 None。
    #[test]
    fn timestamp_near_i64_bounds(
        base in prop_oneof![Just(i64::MIN), Just(i64::MAX), Just(0i64), any::<i64>()],
        d in arb_duration(),
    ) {
        let t = Timestamp::from_unix_nanos(base);
        let _ = t.checked_add(d);
        let _ = t.checked_sub(d);
        if base == i64::MAX && d > Duration::ZERO {
            prop_assert!(t.checked_add(d).is_none());
        }
        if base == i64::MIN && d > Duration::ZERO {
            prop_assert!(t.checked_sub(d).is_none());
        }
        prop_assert_eq!(t.as_unix_nanos(), base);
    }

    /// ErrorKind 构造器 → kind / is_retryable / is_bug 一致。
    #[test]
    fn error_kind_constructor_matrix(kind_idx in 0usize..9) {
        let (err, expect) = match kind_idx {
            0 => (XError::invalid("x"), ErrorKind::Invalid),
            1 => (XError::missing("x"), ErrorKind::Missing),
            2 => (XError::conflict("x"), ErrorKind::Conflict),
            3 => (XError::transient("x"), ErrorKind::Transient),
            4 => (XError::unavailable("x"), ErrorKind::Unavailable),
            5 => (XError::cancelled("x"), ErrorKind::Cancelled),
            6 => (XError::deadline_exceeded("x"), ErrorKind::DeadlineExceeded),
            7 => (XError::invariant("x"), ErrorKind::Invariant),
            _ => (XError::internal("x"), ErrorKind::Internal),
        };
        prop_assert_eq!(err.kind(), expect);
        prop_assert_eq!(err.is_retryable(), matches!(expect, ErrorKind::Transient));
        prop_assert_eq!(err.is_bug(), matches!(expect, ErrorKind::Invariant));
        prop_assert_eq!(err.context(), "x");
    }

    /// §11.3：任意 ComponentState 二元组合与合法边表一致（有限枚举，抽样 + 穷举互补）。
    #[test]
    fn component_state_pair_matrix(from_idx in 0usize..6, to_idx in 0usize..6) {
        use ComponentState::*;
        let all = [Created, Starting, Running, Draining, Stopped, Failed];
        let legal = [
            (Created, Starting),
            (Starting, Running),
            (Starting, Failed),
            (Running, Draining),
            (Running, Failed),
            (Draining, Stopped),
            (Draining, Failed),
        ];
        let from = all[from_idx];
        let to = all[to_idx];
        let allowed = legal.contains(&(from, to));
        prop_assert_eq!(from.can_transition_to(to), allowed);
        prop_assert_eq!(from.try_transition(to).is_ok(), allowed);
        if allowed {
            prop_assert_eq!(from.try_transition(to).unwrap(), to);
        } else {
            let err = from.try_transition(to).unwrap_err();
            prop_assert_eq!(err.from, from);
            prop_assert_eq!(err.to, to);
        }
    }
}
