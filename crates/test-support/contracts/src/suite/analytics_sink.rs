//! AnalyticsSink 可调用性 smoke suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::AnalyticsSink;

const C: &str = "AnalyticsSink";

/// 断言一次确定输入可成功调用 `sink`。
///
/// `AnalyticsSink` trait 没有 read/ack interface，因此本函数不证明持久化、可见性、
/// 去重或投递保证；需要这些能力的 adapter 必须提供自身可观察的 live 测试。
pub async fn assert_analytics_sink_callable(
    sink: &dyn AnalyticsSink,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let event = fixture.resource("analytics_event");
    sink.sink(&event, Bytes::from_static(b"contract-testkit-analytics-v1")).await.map_err(|error| {
        ContractFailure::new(C, "sink_callable", format!("sink 调用失败: {error}"))
    })
}

/// 断言一次 sink 调用后，调用方提供的观察函数能看到目标事件与 payload。
///
/// 观察函数是 test-support 内部 seam，不扩展 [`AnalyticsSink`] 的生产合同。本函数只按
/// 包含关系检查，允许额外事件、重复事件和任意顺序。
pub async fn assert_analytics_sink_observed<F>(
    sink: &dyn AnalyticsSink,
    fixture: &FixtureNamespace,
    observe: F,
) -> ContractResult
where
    F: FnOnce() -> kernel::XResult<Vec<(String, Bytes)>>,
{
    let event = fixture.resource("analytics_event");
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
