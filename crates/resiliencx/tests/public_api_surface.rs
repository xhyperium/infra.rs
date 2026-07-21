//! resiliencx 公开面补全：Wait 实现、jitter、bulkhead 查询、rate capacity。

use kernel::{ErrorKind, XError};
use resiliencx::{
    Backoff, Bulkhead, BulkheadConfig, CircuitBreaker, CircuitConfig, CircuitState, NoWait,
    NoopInstrumentation, RateLimitConfig, RateLimiter, RecordingWait, RetryConfig, ThreadSleepWait,
    Wait, apply_deterministic_jitter, retry_delay_ms, retry_fn_with_wait, retry_ok,
};
use std::sync::Arc;

#[test]
fn wait_impls_and_delay_helpers() {
    ThreadSleepWait.wait_ms(0);
    NoWait.wait_ms(99_000);
    let rec = RecordingWait::new();
    rec.wait_ms(7);
    assert_eq!(rec.delays(), vec![7]);

    let fixed = RetryConfig::fixed(3, 10);
    assert_eq!(retry_delay_ms(&fixed, 1), 10);
    let exp = RetryConfig {
        max_attempts: 5,
        base_delay_ms: 10,
        backoff: Backoff::Exponential { factor: 2, max_delay_ms: 100 },
        jitter_bps: 0,
    };
    assert_eq!(retry_delay_ms(&exp, 3), 40);
    let j = apply_deterministic_jitter(100, 0, 1);
    assert_eq!(j, 100);
    let j2 = apply_deterministic_jitter(100, 10_000, 2);
    let _ = j2;
}

#[test]
fn bulkhead_and_rate_queries() {
    let b = Arc::new(Bulkhead::new(BulkheadConfig { max_concurrent: 2 }).unwrap());
    assert_eq!(b.max_concurrent(), 2);
    assert_eq!(b.in_flight(), 0);
    let p = b.try_enter().unwrap();
    assert_eq!(b.in_flight(), 1);
    drop(p);
    assert_eq!(b.in_flight(), 0);

    let mut lim = RateLimiter::new(RateLimitConfig { capacity: 3 }).unwrap();
    assert_eq!(lim.capacity(), 3);
    assert_eq!(lim.available(), 3);
    lim.try_acquire(2).unwrap();
    assert_eq!(lim.available(), 1);
    assert_eq!(lim.try_acquire(5).unwrap_err().kind(), ErrorKind::Unavailable);
}

#[test]
fn circuit_config_accessor_and_noop_instr() {
    let cfg = CircuitConfig {
        failure_threshold: 2,
        success_threshold: 1,
        open_to_half_open_after_rejects: 1,
    };
    let mut cb = CircuitBreaker::new(cfg).unwrap();
    assert_eq!(cb.config().failure_threshold, 2);
    assert_eq!(cb.state(), CircuitState::Closed);
    let instr = NoopInstrumentation;
    let _ = format!("{instr:?}");
    cb.call(&instr, "op", || Err::<(), _>(XError::invalid("x"))).unwrap_err();
    cb.call(&instr, "op", || Err::<(), _>(XError::invalid("x"))).unwrap_err();
    assert_eq!(cb.state(), CircuitState::Open);

    let rcfg = RetryConfig::fixed(1, 0);
    let mut op = || Ok(retry_ok(1u8));
    let out = retry_fn_with_wait(&rcfg, &instr, "n", &NoWait, &mut op).unwrap();
    let _ = out;
}
