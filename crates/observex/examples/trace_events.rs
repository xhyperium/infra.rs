//! TracingInstrumentation 三方法最小路径（无 subscriber 也不 panic）。
//!
//! ```bash
//! cargo run -p observex --example trace_events
//! ```
//!
//! # 生产红线
//! - 仅 `tracing::info!` 事件，**不是** OTEL exporter / flush / shutdown。
//! - 不得宣称生产可观测平台完成。

use contracts::Instrumentation;
use observex::TracingInstrumentation;

fn main() {
    let instr = TracingInstrumentation::new();
    // 无 subscriber 时 tracing 为 no-op，调用仍安全
    instr.record_retry("http.fetch", 1);
    instr.record_circuit_open("http.fetch");
    instr.record_circuit_close("http.fetch");
    println!("observex_trace_ok methods=3 (tracing only, not OTEL)");
}
