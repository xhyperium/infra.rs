//! observex —— L1 tracing/metrics 封装（SPEC 0.1.0 / ADR-005）。
//!
//! [`TracingInstrumentation`] 实现 [`contracts::Instrumentation`]。
//! 另有 [`PrefixedInstrumentation`]、[`CountingInstrumentation`]（本地验证，**非** OTEL）。
//! **非目标**：OTEL exporter / flush / shutdown。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::sync::atomic::{AtomicU64, Ordering};

use contracts::Instrumentation;

mod ops;
mod policy;
mod surface;
pub use ops::{is_friendly_op, join_op_segments, op_depth, op_leaf, sanitize_op, truncate_op};
pub use policy::{
    ObservabilityTier, allows_production_observability_claim, claims_otel_export_complete,
    counting_is_production_metrics, policy_summary, tier_counting, tier_tracing,
};
pub use surface::{PROBE_DOC, probe_counting_circuit, probe_counting_retries, probe_tracing};

#[allow(unused_imports)]
use kernel as _kernel;

/// tracing 实现（ADR-005）。
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingInstrumentation;

impl TracingInstrumentation {
    /// 构造。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// ADR-005 别名。
pub type ObservexInstrumentation = TracingInstrumentation;

impl Instrumentation for TracingInstrumentation {
    fn record_retry(&self, op: &str, attempt: u32) {
        tracing::info!(op = op, attempt = attempt, "retry");
    }
    fn record_circuit_open(&self, op: &str) {
        tracing::info!(op = op, "circuit_open");
    }
    fn record_circuit_close(&self, op: &str) {
        tracing::info!(op = op, "circuit_close");
    }
}

/// op 名前缀包装。
#[derive(Debug, Clone)]
pub struct PrefixedInstrumentation<I> {
    prefix: String,
    inner: I,
}

impl<I> PrefixedInstrumentation<I> {
    /// 构造。
    #[must_use]
    pub fn new(prefix: impl Into<String>, inner: I) -> Self {
        Self { prefix: prefix.into(), inner }
    }
    /// 内层。
    #[must_use]
    pub fn inner(&self) -> &I {
        &self.inner
    }
    fn qualify(&self, op: &str) -> String {
        if self.prefix.is_empty() { op.to_string() } else { format!("{}.{}", self.prefix, op) }
    }
}

impl<I: Instrumentation> Instrumentation for PrefixedInstrumentation<I> {
    fn record_retry(&self, op: &str, attempt: u32) {
        self.inner.record_retry(&self.qualify(op), attempt);
    }
    fn record_circuit_open(&self, op: &str) {
        self.inner.record_circuit_open(&self.qualify(op));
    }
    fn record_circuit_close(&self, op: &str) {
        self.inner.record_circuit_close(&self.qualify(op));
    }
}

/// 进程内计数（单测用，非生产 metrics）。
#[derive(Debug, Default)]
pub struct CountingInstrumentation {
    retries: AtomicU64,
    opens: AtomicU64,
    closes: AtomicU64,
    last_attempt: AtomicU64,
}

impl CountingInstrumentation {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// 重试次数。
    #[must_use]
    pub fn retry_count(&self) -> u64 {
        self.retries.load(Ordering::Relaxed)
    }
    /// 打开次数。
    #[must_use]
    pub fn open_count(&self) -> u64 {
        self.opens.load(Ordering::Relaxed)
    }
    /// 关闭次数。
    #[must_use]
    pub fn close_count(&self) -> u64 {
        self.closes.load(Ordering::Relaxed)
    }
    /// 最近 attempt。
    #[must_use]
    pub fn last_attempt(&self) -> u64 {
        self.last_attempt.load(Ordering::Relaxed)
    }
    /// 清零。
    pub fn reset(&self) {
        self.retries.store(0, Ordering::Relaxed);
        self.opens.store(0, Ordering::Relaxed);
        self.closes.store(0, Ordering::Relaxed);
        self.last_attempt.store(0, Ordering::Relaxed);
    }
}

impl Instrumentation for CountingInstrumentation {
    fn record_retry(&self, _op: &str, attempt: u32) {
        self.retries.fetch_add(1, Ordering::Relaxed);
        self.last_attempt.store(u64::from(attempt), Ordering::Relaxed);
    }
    fn record_circuit_open(&self, _op: &str) {
        self.opens.fetch_add(1, Ordering::Relaxed);
    }
    fn record_circuit_close(&self, _op: &str) {
        self.closes.fetch_add(1, Ordering::Relaxed);
    }
}

impl Instrumentation for &CountingInstrumentation {
    fn record_retry(&self, op: &str, attempt: u32) {
        (*self).record_retry(op, attempt);
    }
    fn record_circuit_open(&self, op: &str) {
        (*self).record_circuit_open(op);
    }
    fn record_circuit_close(&self, op: &str) {
        (*self).record_circuit_close(op);
    }
}

/// 空 op → `"_"`。
#[must_use]
pub fn normalize_op(op: &str) -> &str {
    if op.is_empty() { "_" } else { op }
}

/// normalize 后 retry。
pub fn record_retry_normalized(instr: &dyn Instrumentation, op: &str, attempt: u32) {
    instr.record_retry(normalize_op(op), attempt);
}
/// normalize 后 open。
pub fn record_circuit_open_normalized(instr: &dyn Instrumentation, op: &str) {
    instr.record_circuit_open(normalize_op(op));
}
/// normalize 后 close。
pub fn record_circuit_close_normalized(instr: &dyn Instrumentation, op: &str) {
    instr.record_circuit_close(normalize_op(op));
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::Instrumentation;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Capture {
        fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
        }
    }
    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl<'a> MakeWriter<'a> for Capture {
        type Writer = Capture;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }
    fn with_capture(f: impl FnOnce()) -> String {
        let cap = Capture::default();
        let sub = tracing_subscriber::fmt()
            .with_writer(cap.clone())
            .with_ansi(false)
            .with_level(true)
            .with_target(false)
            .without_time()
            .finish();
        tracing::subscriber::with_default(sub, f);
        let _ = cap.make_writer().flush();
        cap.text()
    }

    #[test]
    fn tracing_and_alias() {
        let t = TracingInstrumentation::new();
        t.record_retry("f", 1);
        t.record_circuit_open("f");
        t.record_circuit_close("f");
        let a: ObservexInstrumentation = ObservexInstrumentation::new();
        let b: TracingInstrumentation = a;
        b.record_retry("x", 1);
        let d = TracingInstrumentation;
        let _ = format!("{d:?}");
        let _ = d;
    }

    #[test]
    fn counting_and_prefix() {
        let c = CountingInstrumentation::new();
        let p = PrefixedInstrumentation::new("m", &c);
        p.record_retry("op", 3);
        p.record_circuit_open("op");
        p.record_circuit_close("op");
        assert_eq!(c.retry_count(), 1);
        assert_eq!(c.open_count(), 1);
        assert_eq!(c.close_count(), 1);
        assert_eq!(c.last_attempt(), 3);
        assert_eq!(p.inner().retry_count(), 1);
        let p0 = PrefixedInstrumentation::new("", &c);
        p0.record_retry("z", 1);
        assert_eq!(c.retry_count(), 2);
        c.reset();
        assert_eq!(c.retry_count(), 0);
    }

    #[test]
    fn normalize_helpers() {
        assert_eq!(normalize_op(""), "_");
        assert_eq!(normalize_op("a"), "a");
        let c = CountingInstrumentation::new();
        record_retry_normalized(&c, "", 1);
        record_circuit_open_normalized(&c, "");
        record_circuit_close_normalized(&c, "z");
        assert_eq!(c.retry_count() + c.open_count() + c.close_count(), 3);
        assert!(is_friendly_op("ok"));
        assert!(!is_friendly_op(""));
    }

    #[test]
    fn tracing_fields_captured() {
        let out = with_capture(|| {
            let p = PrefixedInstrumentation::new("api", TracingInstrumentation::new());
            p.record_retry("get", 2);
            p.record_circuit_open("get");
            p.record_circuit_close("get");
        });
        assert!(out.contains("retry"));
        assert!(out.contains("circuit_open"));
        assert!(out.contains("circuit_close"));
    }

    #[test]
    fn policy_is_honest() {
        assert!(!claims_otel_export_complete());
        assert!(!counting_is_production_metrics());
        assert!(policy_summary().contains("DEFER"));
    }

    #[test]
    fn dyn_and_ops() {
        let t = TracingInstrumentation::new();
        let o: &dyn Instrumentation = &t;
        o.record_retry("d", 1);
        assert_eq!(join_op_segments(&["a", "b"]), "a.b");
        assert!(truncate_op("abcdef", 3).len() <= 3);
    }
}
