//! AnalyticsSink 可移植核心与可观察 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::AnalyticsSink;

const C: &str = "AnalyticsSink";

/// 断言 sink 接受调用方提供的唯一事件。
///
/// trait 没有 read/ack interface，因此本函数不证明持久化、可见性、去重或投递保证；
/// 真实 adapter 必须用后端查询单独验证落盘结果。
///
/// # Errors
///
/// 事件名为空或 `sink` 调用失败时返回 [`ContractFailure`]。
pub async fn assert_analytics_sink(
    sink: &dyn AnalyticsSink,
    unique_event: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_event", !unique_event.is_empty(), "测试事件名不得为空")?;
    sink.sink(unique_event, payload)
        .await
        .map_err(|error| ContractFailure::new(C, "sink", format!("sink 调用失败: {error}")))
}

/// 使用确定性 fixture 运行 [`assert_analytics_sink`]。
///
/// # Errors
///
/// 资源名派生失败或核心 suite 失败时返回 [`ContractFailure`]。
pub async fn assert_analytics_sink_callable(
    sink: &dyn AnalyticsSink,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let event = fixture.resource("analytics_event")?;
    assert_analytics_sink(sink, &event, Bytes::from_static(b"contract-testkit-analytics-v1")).await
}

/// 断言一次 sink 调用后，调用方提供的观察函数能看到目标事件与 payload。
///
/// 观察函数是 test-support 内部 seam，不扩展 [`AnalyticsSink`] 的生产合同。本函数只按
/// 包含关系检查，允许额外事件、重复事件和任意顺序。
///
/// # Errors
///
/// sink、资源名、观察函数失败，或观察结果缺少目标事件时返回 [`ContractFailure`]。
pub async fn assert_analytics_sink_observed<F>(
    sink: &dyn AnalyticsSink,
    fixture: &FixtureNamespace,
    observe: F,
) -> ContractResult
where
    F: FnOnce() -> kernel::XResult<Vec<(String, Bytes)>>,
{
    let event = fixture.resource("analytics_event")?;
    let payload = Bytes::from_static(b"contract-testkit-analytics-v1");
    assert_analytics_sink_callable(sink, fixture).await?;
    let observed = observe().map_err(|error| {
        ContractFailure::new(C, "observe", format!("观察 analytics sink 失败: {error}"))
    })?;
    ensure(
        C,
        "observed_event_missing",
        observed.iter().any(|(name, body)| name == &event && body == &payload),
        format!("观察结果缺少目标事件: {observed:?}"),
    )
}
