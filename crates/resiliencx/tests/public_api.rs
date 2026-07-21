//! 库外消费者视角：公开 `RetryConfig` / `retry_fn` / `Instrumentation` 可引用。

use kernel::{ErrorKind, XError, XResult};
use resiliencx::{
    Instrumentation, NoopInstrumentation, RetryConfig, retry_downcast, retry_fn, retry_ok,
};
use std::sync::{Arc, Mutex};

struct CountingInstr {
    n: Mutex<u32>,
}

impl Instrumentation for CountingInstr {
    fn record_retry(&self, _op: &str, _attempt: u32) {
        *self.n.lock().expect("lock") += 1;
    }
    fn record_circuit_open(&self, _op: &str) {}
    fn record_circuit_close(&self, _op: &str) {}
}

#[test]
fn consumer_retry_fn_drives_shipped_surface() {
    let instr = CountingInstr { n: Mutex::new(0) };
    let cfg = RetryConfig { max_attempts: 3, base_delay_ms: 0 };
    let hits = Arc::new(Mutex::new(0u32));
    let h = hits.clone();
    let mut op = move || {
        let mut g = h.lock().expect("hits");
        *g += 1;
        if *g == 1 { Err(XError::transient("try again")) } else { Ok(retry_ok("done")) }
    };
    let out: XResult<_> = retry_fn(&cfg, &instr, "consumer-op", &mut op);
    let s = retry_downcast::<&'static str>(out.expect("ok")).expect("ty");
    assert_eq!(s, "done");
    assert_eq!(*instr.n.lock().expect("n"), 1);
    assert_eq!(*hits.lock().expect("hits"), 2);
}

#[test]
fn consumer_non_retryable_stops_immediately() {
    let instr = NoopInstrumentation;
    let cfg = RetryConfig::default();
    let mut op = || Err(XError::invalid("nope"));
    let result = retry_fn(&cfg, &instr, "bad", &mut op);
    assert_eq!(result.expect_err("err").kind(), ErrorKind::Invalid);
}

#[test]
fn consumer_default_config() {
    let cfg = RetryConfig::default();
    assert_eq!(cfg.max_attempts, 3);
    assert_eq!(cfg.base_delay_ms, 0);
}
