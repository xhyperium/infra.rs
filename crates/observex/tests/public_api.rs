//! 消费者侧导入面：crate 外使用 `observex::` 与 `infra_contracts::Instrumentation`。

use infra_infra_contracts::Instrumentation;
use observex::{ObservexInstrumentation, TracingInstrumentation};

#[test]
fn consumer_can_construct_and_call_via_trait() {
    let instr = TracingInstrumentation::new();
    let obj: &dyn Instrumentation = &instr;
    obj.record_retry("consumer_op", 1);
    obj.record_circuit_open("consumer_op");
    obj.record_circuit_close("consumer_op");
}

#[test]
fn consumer_can_use_adr_alias() {
    let instr = ObservexInstrumentation::default();
    instr.record_retry("alias_op", 9);
}

#[test]
fn consumer_arc_dyn_works() {
    use std::sync::Arc;
    let instr: Arc<dyn Instrumentation> = Arc::new(TracingInstrumentation::new());
    instr.record_retry("arc", 1);
}
