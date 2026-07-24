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
    println!("policy={}", observex::policy_summary());
    let instr = TracingInstrumentation::new();
    instr.record_retry("demo.op", 1);
    instr.record_circuit_open("demo.op");
    instr.record_circuit_close("demo.op");
    println!("observex_ok");
}
