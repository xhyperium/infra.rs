//! AnalyticsSink 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use bytes::Bytes;
use contracts::AnalyticsSink;

const C: &str = "AnalyticsSink";

/// 断言 sink 接受调用方提供的唯一事件。
///
/// trait 没有读取面，因此本 suite 不宣称持久化可见；真实 adapter 必须用后端查询
/// 单独验证落盘结果。
pub async fn assert_analytics_sink(
    sink: &dyn AnalyticsSink,
    unique_event: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_event", !unique_event.is_empty(), "测试事件名不得为空")?;
    sink.sink(unique_event, payload)
        .await
        .map_err(|e| ContractFailure::new(C, "sink", format!("sink 失败: {e}")))
}
