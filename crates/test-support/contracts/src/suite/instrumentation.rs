//! Instrumentation 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fakes::InstrEvent;
use crate::fixture::FixtureNamespace;
use contracts::Instrumentation;

const C: &str = "Instrumentation";

/// Smoke：断言三方法可调用且不 panic（对象安全路径）。
///
/// 本函数不能识别 no-op；需要行为验证时使用 [`assert_instrumentation_observed`]。
///
/// # Errors
///
/// 当前 smoke 只返回成功；保留 `ContractResult` 以维持兼容接口。该函数不计入
/// broken implementation kill rate，行为合同只由 [`assert_instrumentation_observed`] 验证。
pub fn assert_instrumentation(instr: &dyn Instrumentation) -> ContractResult {
    instr.record_retry("place", 1);
    instr.record_circuit_open("place");
    instr.record_circuit_close("place");
    // 无返回值合同：仅要求可调用，不伪造永真断言。
    Ok(())
}

/// 断言三类 instrumentation 调用能被调用方提供的观察函数看到。
///
/// 仅按包含关系检查，允许额外事件、重复事件和任意顺序；不证明生产 exporter 已发送。
///
/// # Errors
///
/// 资源名派生、观察函数失败，或观察结果缺少目标事件时返回 [`ContractFailure`]。
pub fn assert_instrumentation_observed<F>(
    instr: &dyn Instrumentation,
    fixture: &FixtureNamespace,
    observe: F,
) -> ContractResult
where
    F: FnOnce() -> kernel::XResult<Vec<InstrEvent>>,
{
    let op = fixture.resource("instrumentation_op")?;
    instr.record_retry(&op, 1);
    instr.record_circuit_open(&op);
    instr.record_circuit_close(&op);
    let events = observe().map_err(|error| {
        ContractFailure::new(C, "observe", format!("观察 instrumentation 失败: {error}"))
    })?;
    ensure(
        C,
        "retry_missing",
        events.contains(&InstrEvent::Retry { op: op.clone(), attempt: 1 }),
        format!("观察结果缺少 retry: {events:?}"),
    )?;
    ensure(
        C,
        "circuit_open_missing",
        events.contains(&InstrEvent::CircuitOpen { op: op.clone() }),
        format!("观察结果缺少 circuit open: {events:?}"),
    )?;
    ensure(
        C,
        "circuit_close_missing",
        events.contains(&InstrEvent::CircuitClose { op }),
        format!("观察结果缺少 circuit close: {events:?}"),
    )
}
