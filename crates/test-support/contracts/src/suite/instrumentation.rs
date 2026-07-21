//! Instrumentation 合同 suite。

use crate::failure::{ContractResult, ensure};
use contracts::Instrumentation;

const C: &str = "Instrumentation";

/// 断言三方法可调用且不 panic（对象安全路径）。
pub fn assert_instrumentation(instr: &dyn Instrumentation) -> ContractResult {
    instr.record_retry("place", 1);
    instr.record_circuit_open("place");
    instr.record_circuit_close("place");
    // 无返回值合同：仅要求可调用。
    ensure(C, "callable", true, "")
}
