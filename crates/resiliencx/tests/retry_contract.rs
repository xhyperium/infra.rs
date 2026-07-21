//! Active SSOT §2 / §4 行为合同：直接驱动 shipped `retry_fn` / `RetryConfig`。

use kernel::{ErrorKind, XError};
use resiliencx::{
    Instrumentation, NoopInstrumentation, RetryConfig, retry_downcast, retry_fn, retry_ok,
};
use std::sync::{Arc, Mutex};

struct MockInstr {
    retries: Mutex<Vec<(String, u32)>>,
    circuit_open: Mutex<Vec<String>>,
    circuit_close: Mutex<Vec<String>>,
}

impl MockInstr {
    fn new() -> Self {
        Self {
            retries: Mutex::new(Vec::new()),
            circuit_open: Mutex::new(Vec::new()),
            circuit_close: Mutex::new(Vec::new()),
        }
    }

    fn retry_events(&self) -> Vec<(String, u32)> {
        self.retries.lock().expect("retry lock").clone()
    }

    fn retry_count(&self) -> usize {
        self.retries.lock().expect("retry lock").len()
    }
}

impl Instrumentation for MockInstr {
    fn record_retry(&self, op: &str, attempt: u32) {
        self.retries.lock().expect("retry lock").push((op.to_string(), attempt));
    }
    fn record_circuit_open(&self, op: &str) {
        self.circuit_open.lock().expect("open lock").push(op.to_string());
    }
    fn record_circuit_close(&self, op: &str) {
        self.circuit_close.lock().expect("close lock").push(op.to_string());
    }
}

#[test]
fn retry_succeeds_first_try_no_retries() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 3, base_delay_ms: 0 };
    let mut op = || Ok(retry_ok(42u32));
    let result = retry_fn(&cfg, &instr, "op", &mut op).expect("ok");
    assert_eq!(retry_downcast::<u32>(result).expect("ty"), 42);
    assert_eq!(instr.retry_count(), 0);
}

#[test]
fn retry_succeeds_after_failures() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 3, base_delay_ms: 0 };
    let counter = Arc::new(Mutex::new(0u32));
    let c = counter.clone();
    let mut op = move || {
        let mut g = c.lock().expect("counter");
        *g += 1;
        if *g < 2 { Err(XError::transient(format!("fail #{}", *g))) } else { Ok(retry_ok(100u32)) }
    };
    let result = retry_fn(&cfg, &instr, "op", &mut op).expect("ok");
    assert_eq!(retry_downcast::<u32>(result).expect("ty"), 100);
    assert_eq!(instr.retry_count(), 1);
    assert_eq!(instr.retry_events()[0], ("op".to_string(), 1));
    assert_eq!(*counter.lock().expect("counter"), 2);
}

#[test]
fn retry_all_failures_returns_last_error() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 2, base_delay_ms: 0 };
    let mut op = || Err(XError::transient("boom"));
    let result = retry_fn(&cfg, &instr, "op", &mut op);
    assert!(result.is_err());
    assert!(result.expect_err("err").to_string().contains("boom"));
    assert_eq!(instr.retry_count(), 1);
}

#[test]
fn non_retryable_errors_are_not_retried() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 5, base_delay_ms: 0 };
    let counter = Arc::new(Mutex::new(0u32));
    let c = counter.clone();
    let mut op = move || {
        *c.lock().expect("counter") += 1;
        Err(XError::invalid("bad input"))
    };
    let result = retry_fn(&cfg, &instr, "op", &mut op);
    assert_eq!(result.expect_err("err").kind(), ErrorKind::Invalid);
    assert_eq!(instr.retry_count(), 0);
    assert_eq!(*counter.lock().expect("counter"), 1);
}

#[test]
fn retry_zero_attempts_returns_invalid() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 0, base_delay_ms: 0 };
    let mut op = || Ok(retry_ok(()));
    let result = retry_fn(&cfg, &instr, "op", &mut op);
    assert!(result.is_err());
    assert_eq!(result.expect_err("err").kind(), ErrorKind::Invalid);
}

#[test]
fn retry_config_default() {
    let cfg = RetryConfig::default();
    assert_eq!(cfg.max_attempts, 3);
    assert_eq!(cfg.base_delay_ms, 0);
}

#[test]
fn retry_config_and_noop_derive_surface() {
    let a = RetryConfig { max_attempts: 1, base_delay_ms: 2 };
    let b = a;
    assert_eq!(a, b);
    assert_ne!(a, RetryConfig::default());
    let _dbg = format!("{a:?}");
    let n = NoopInstrumentation;
    let n2 = n;
    let _n3 = n2;
    let _nd = format!("{n:?}");
}

/// 覆盖 `base_delay_ms > 0` 的 sleep 分支（短延迟，仅为行覆盖；非生产 wait 合同）。
#[test]
fn retry_with_base_delay_sleeps_before_retry() {
    let instr = MockInstr::new();
    let cfg = RetryConfig { max_attempts: 2, base_delay_ms: 1 };
    let counter = Arc::new(Mutex::new(0u32));
    let c = counter.clone();
    let mut op = move || {
        let mut g = c.lock().expect("counter");
        *g += 1;
        if *g < 2 { Err(XError::transient("once")) } else { Ok(retry_ok(7u32)) }
    };
    let result = retry_fn(&cfg, &instr, "delay-op", &mut op).expect("ok");
    assert_eq!(retry_downcast::<u32>(result).expect("ty"), 7);
    assert_eq!(instr.retry_count(), 1);
    assert_eq!(instr.retry_events()[0], ("delay-op".to_string(), 1));
}

#[test]
fn noop_instrumentation_methods_are_callable() {
    let n = NoopInstrumentation;
    n.record_retry("x", 1);
    n.record_circuit_open("x");
    n.record_circuit_close("x");
    let _ = NoopInstrumentation;
}

#[test]
fn mock_circuit_hooks_record_for_future_api() {
    let instr = MockInstr::new();
    instr.record_circuit_open("cb");
    instr.record_circuit_close("cb");
    assert_eq!(instr.circuit_open.lock().expect("l").as_slice(), &["cb".to_string()]);
    assert_eq!(instr.circuit_close.lock().expect("l").as_slice(), &["cb".to_string()]);
}

#[test]
fn retry_downcast_type_mismatch_is_invalid() {
    let v = retry_ok(1u32);
    let err = retry_downcast::<String>(v).expect_err("mismatch");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn retry_ok_roundtrip() {
    let v = retry_ok("hello".to_string());
    assert_eq!(retry_downcast::<String>(v).expect("ty"), "hello");
}
