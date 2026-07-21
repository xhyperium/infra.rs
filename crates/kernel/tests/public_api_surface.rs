//! 公开消费面全量驱动：每个 crate 根 re-export 类型/构造器/方法至少调用一次并断言结果。

use std::error::Error;
use std::time::Duration;

use kernel::{
    BoxError, Clock, ClockDomain, ClockError, ComponentState, ErrorKind, LifecycleError,
    MonotonicInstant, ShutdownSignal, SystemClock, Timestamp, XError, XResult,
};

#[test]
fn error_constructors_and_queries() {
    // is_retryable 仅 Transient；is_bug 仅 Invariant（库实现权威）
    for (e, kind) in [
        (XError::invalid("i"), ErrorKind::Invalid),
        (XError::missing("m"), ErrorKind::Missing),
        (XError::conflict("c"), ErrorKind::Conflict),
        (XError::transient("t"), ErrorKind::Transient),
        (XError::unavailable("u"), ErrorKind::Unavailable),
        (XError::cancelled("x"), ErrorKind::Cancelled),
        (XError::deadline_exceeded("d"), ErrorKind::DeadlineExceeded),
        (XError::invariant("v"), ErrorKind::Invariant),
        (XError::internal("n"), ErrorKind::Internal),
    ] {
        assert_eq!(e.kind(), kind);
        assert_eq!(e.is_retryable(), kind == ErrorKind::Transient);
        assert_eq!(e.is_bug(), kind == ErrorKind::Invariant);
        assert!(!e.context().is_empty());
        assert!(!e.to_string().is_empty());
    }
    let e = XError::transient_after("a", Duration::from_millis(5));
    assert_eq!(e.retry_after(), Some(Duration::from_millis(5)));
    let e2 = e.with_source(std::io::Error::other("src"));
    assert_eq!(e2.kind(), ErrorKind::Transient);
    assert!(e2.source().is_some());
    let _boxed: BoxError = Box::new(std::io::Error::other("b"));
    let r: XResult<u8> = Err(XError::invalid("r"));
    assert!(r.is_err());

    // ClockError → XError：库约定全部映射为 Unavailable，并保留 source 链
    for ce in [ClockError::Unavailable, ClockError::Overflow, ClockError::BeforeUnixEpoch] {
        let xe: XError = ce.into();
        assert_eq!(xe.kind(), ErrorKind::Unavailable);
        assert!(xe.source().is_some());
        assert!(!xe.context().is_empty());
    }
}

#[test]
fn timestamp_and_clock_domain_surface() {
    let t = Timestamp::from_unix_nanos(100);
    assert_eq!(t.as_unix_nanos(), 100);
    assert_eq!(t.checked_add(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 103);
    assert_eq!(t.checked_sub(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 97);
    assert_eq!(
        Timestamp::from_unix_nanos(110).checked_duration_since(t).unwrap(),
        Duration::from_nanos(10)
    );
    // 允许早于 epoch 的负 nanos；溢出才 None
    assert_eq!(t.checked_sub(Duration::from_nanos(200)).unwrap().as_unix_nanos(), -100);
    assert!(Timestamp::from_unix_nanos(i64::MIN).checked_sub(Duration::from_nanos(1)).is_none());
    assert!(t < Timestamp::from_unix_nanos(101));
    assert_eq!(t.cmp(&Timestamp::from_unix_nanos(100)), std::cmp::Ordering::Equal);

    let d = ClockDomain::from_raw(42);
    assert_eq!(d.as_raw(), 42);
    assert_eq!(ClockDomain::PROCESS.as_raw(), 1);
    assert_eq!(d, ClockDomain::from_raw(42));

    let m = MonotonicInstant::from_clock_elapsed(Duration::from_millis(1));
    assert_eq!(m.domain(), ClockDomain::PROCESS);
    let m2 = MonotonicInstant::from_clock_elapsed_in(Duration::from_millis(2), d);
    assert_eq!(m2.domain(), d);
    assert!(m2.checked_duration_since(m).is_none()); // cross-domain
    let m3 = MonotonicInstant::from_clock_elapsed_in(Duration::from_millis(5), d);
    assert_eq!(m3.checked_duration_since(m2).unwrap(), Duration::from_millis(3));
    assert!(m3 > m2);
}

#[test]
fn system_clock_and_clock_error_display() {
    let c = SystemClock::new();
    let now = c.now().unwrap();
    assert!(now.as_unix_nanos() > 0);
    let a = c.monotonic();
    let b = c.monotonic();
    assert!(b >= a);

    // Default 与 Clone 路径（Default trait 入口）
    let d: SystemClock = Default::default();
    assert!(d.now().unwrap().as_unix_nanos() > 0);
    let cloned = c.clone();
    assert!(cloned.now().unwrap().as_unix_nanos() > 0);

    // Clock trait object
    let as_clock: &dyn Clock = &c;
    assert!(as_clock.now().unwrap().as_unix_nanos() > 0);
    let _ = as_clock.monotonic();

    assert!(!ClockError::Unavailable.to_string().is_empty());
    assert!(!ClockError::Overflow.to_string().is_empty());
    assert!(!ClockError::BeforeUnixEpoch.to_string().is_empty());
    assert_eq!(ClockError::Unavailable, ClockError::Unavailable);
    assert_ne!(ClockError::Unavailable, ClockError::Overflow);
}

#[test]
fn lifecycle_states_and_shutdown_timeout() {
    // 全状态变体 + 合法链
    let chain = [
        (ComponentState::Created, ComponentState::Starting),
        (ComponentState::Starting, ComponentState::Running),
        (ComponentState::Running, ComponentState::Draining),
        (ComponentState::Draining, ComponentState::Stopped),
    ];
    for (from, to) in chain {
        assert!(from.can_transition_to(to));
        assert_eq!(from.try_transition(to).unwrap(), to);
    }
    assert!(ComponentState::Starting.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Running.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Draining.can_transition_to(ComponentState::Failed));
    assert!(!ComponentState::Stopped.can_transition_to(ComponentState::Created));
    assert!(!ComponentState::Failed.can_transition_to(ComponentState::Running));

    let err = ComponentState::Created.try_transition(ComponentState::Running).unwrap_err();
    let le: LifecycleError = err;
    assert_eq!(le.from, ComponentState::Created);
    assert_eq!(le.to, ComponentState::Running);
    assert!(le.to_string().contains("非法"));

    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    assert!(!s.wait_timeout(Duration::from_millis(1)));
    let observer = s.clone();
    g.trigger();
    assert!(s.is_triggered());
    assert!(observer.is_triggered());
    assert!(s.wait_timeout(Duration::from_millis(50)));
    // wait 在已触发时立即返回
    s.wait();
}

#[test]
fn modules_are_reachable_via_crate_paths() {
    // 公开模块路径可被外部集成测试使用
    let _ = kernel::clock::SystemClock::new();
    let _ = kernel::error::XError::invalid("mod");
    let (g, s) = kernel::lifecycle::ShutdownSignal::new();
    g.trigger();
    assert!(s.is_triggered());
}
