//! 最小消费者路径：墙钟读取 + 关停信号 + 错误分类。
//!
//! ```bash
//! cargo run -p kernel --example basic
//! ```

use std::time::Duration;

use kernel::{Clock, ComponentState, ErrorKind, ShutdownSignal, SystemClock, XError};

fn main() {
    let clock = SystemClock::new();
    let now = clock.now().expect("wall clock available");
    assert!(now.as_unix_nanos() > 0, "unix nanos must be positive");
    let mono = clock.monotonic();
    let _ = mono.domain();

    let err = XError::invalid("bad input");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(!err.is_retryable());

    assert!(ComponentState::Created.can_transition_to(ComponentState::Starting));
    let next = ComponentState::Created
        .try_transition(ComponentState::Starting)
        .expect("Created → Starting is legal");
    assert_eq!(next, ComponentState::Starting);

    let (guard, signal) = ShutdownSignal::new();
    assert!(!signal.is_triggered());
    guard.trigger();
    assert!(signal.is_triggered());
    assert!(signal.wait_timeout(Duration::from_millis(10)));

    println!("kernel-consumer: ok now_ns={} shutdown_triggered=true", now.as_unix_nanos());
}
