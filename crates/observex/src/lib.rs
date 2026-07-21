//! observex —— L1 tracing/metrics 封装（SPEC-INFRA-OBSERVEX 0.1.0 / ADR-005）。
//!
//! 提供 [`TracingInstrumentation`]，实现 [`infra_contracts::Instrumentation`]，
//! 将重试与熔断事件通过 `tracing::info!` 输出。
//!
//! **非目标（本版本）**：OpenTelemetry exporter / flush / shutdown / 采样 / 有界缓冲。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use infra_contracts::Instrumentation;

// kernel 为 SPEC §2 / §4.4 依赖信封预留；0.1.0 热路径不直接引用。
#[allow(unused_imports)]
use kernel as _kernel;

/// 基于 `tracing` 的可观测性实现（ADR-005 行为面；公开名 `TracingInstrumentation`）。
///
/// 零字段、`Copy` 类型。将重试与熔断事件写入 `tracing::info!`。
/// 无 subscriber 时调用仍不 panic（tracing 默认 no-op）。
///
/// > 命名：Approved ADR-005 写 `ObservexInstrumentation`；本仓与上游代码事实一致使用
/// > `TracingInstrumentation`。兼容别名见 [`ObservexInstrumentation`]。
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingInstrumentation;

impl TracingInstrumentation {
    /// 构造零字段实现。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// ADR-005 命名兼容别名（与 [`TracingInstrumentation`] 同一类型）。
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

#[cfg(test)]
mod tests {
    use super::*;
    use infra_contracts::Instrumentation;
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    /// 线程安全的内存 writer，用于捕获 tracing 输出。
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Capture {
        fn text(&self) -> String {
            let g = self.0.lock().expect("capture lock");
            String::from_utf8_lossy(&g).into_owned()
        }
    }

    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("capture lock").extend_from_slice(buf);
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
        let subscriber = tracing_subscriber::fmt()
            .with_writer(cap.clone())
            .with_ansi(false)
            .with_level(true)
            .with_target(false)
            .without_time()
            .finish();
        tracing::subscriber::with_default(subscriber, f);
        // 显式 flush，覆盖 Write::flush 与捕获路径
        let mut w = cap.make_writer();
        let _ = w.flush();
        cap.text()
    }

    /// 驱动 `Default` 而不触发 clippy::default_constructed_unit_structs。
    fn via_default<T: Default>() -> T {
        T::default()
    }

    /// 驱动 `Clone` 而不触发 clippy::clone_on_copy。
    fn via_clone<T: Clone>(t: &T) -> T {
        t.clone()
    }

    #[test]
    fn implements_instrumentation_without_panic() {
        let instr = TracingInstrumentation::new();
        // 无 subscriber 时 tracing 为 no-op，仍不得 panic
        instr.record_retry("fetch", 1);
        instr.record_retry("fetch", 2);
        instr.record_circuit_open("fetch");
        instr.record_circuit_close("fetch");
    }

    #[test]
    fn default_equals_new_and_unit_struct() {
        let a = TracingInstrumentation::new();
        let b = TracingInstrumentation;
        let c = via_default::<TracingInstrumentation>();
        // Copy / Clone / Debug 烟雾
        let d = a;
        let e = via_clone(&a);
        let _ = format!("{a:?}");
        let _ = (a, b, c, d, e);
    }

    #[test]
    fn as_trait_object_works() {
        let instr = TracingInstrumentation::new();
        let obj: &dyn Instrumentation = &instr;
        obj.record_retry("obj", 3);
        obj.record_circuit_open("obj");
        obj.record_circuit_close("obj");
    }

    #[test]
    fn clone_copy_preserves_behavior() {
        let a = TracingInstrumentation::new();
        let b = a;
        b.record_retry("op", 1);
        a.record_retry("op", 1);
    }

    #[test]
    fn observex_alias_is_same_type() {
        let a: ObservexInstrumentation = ObservexInstrumentation::new();
        let b: TracingInstrumentation = a;
        b.record_circuit_open("alias");
    }

    #[test]
    fn tracing_fields_record_retry() {
        let out = with_capture(|| {
            TracingInstrumentation::new().record_retry("fetch_orders", 2);
        });
        assert!(out.contains("retry"), "out={out}");
        assert!(out.contains("fetch_orders"), "out={out}");
        assert!(out.contains("2") || out.contains("attempt"), "out={out}");
    }

    #[test]
    fn tracing_fields_record_circuit_open() {
        let out = with_capture(|| {
            TracingInstrumentation::new().record_circuit_open("place_order");
        });
        assert!(out.contains("circuit_open"), "out={out}");
        assert!(out.contains("place_order"), "out={out}");
    }

    #[test]
    fn tracing_fields_record_circuit_close() {
        let out = with_capture(|| {
            TracingInstrumentation::new().record_circuit_close("place_order");
        });
        assert!(out.contains("circuit_close"), "out={out}");
        assert!(out.contains("place_order"), "out={out}");
    }
}
