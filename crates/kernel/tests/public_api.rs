//! 公开 API 基本编译测试。
//!
//! 验证所有公开类型和函数签名可被下游正常使用。

use kernel::{
    BoxError, Clock, ClockError, ComponentState, ErrorKind, MonotonicInstant, ShutdownSignal,
    SystemClock, Timestamp, XError, XResult,
};

#[test]
fn test_public_api_basics() {
    // Error
    let err: XError = XError::invalid("test");
    let _kind: ErrorKind = err.kind();
    let _ctx: &str = err.context();
    let _retry: Option<std::time::Duration> = err.retry_after();
    let _boxed: BoxError = Box::new(std::io::Error::other("oops"));
    let _result: XResult<()> = Err(err);

    // Clock
    let clock = SystemClock::new();
    let _ts: Result<Timestamp, ClockError> = clock.now();
    let _mono: MonotonicInstant = clock.monotonic();

    // Lifecycle
    let _state: ComponentState = ComponentState::Created;
    let (_guard, _signal) = ShutdownSignal::new();
}
