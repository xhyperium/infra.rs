//! TimeSeriesStore 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use canonical::Tick;
use contracts::TimeSeriesStore;

const C: &str = "TimeSeriesStore";

/// 写入一个调用方已按后端精度对齐的点，并验证给定闭区间内可读。
///
/// 本 suite 不假定返回顺序、唯一性或测试表中没有其他历史数据。
pub async fn assert_time_series_store(
    store: &dyn TimeSeriesStore,
    unique_table: &str,
    point: Tick,
) -> ContractResult {
    ensure(C, "unique_table", !unique_table.is_empty(), "测试表名不得为空")?;
    let ts = point.ts;
    store
        .write_series(unique_table, vec![point.clone()])
        .await
        .map_err(|e| ContractFailure::new(C, "write", format!("write_series 失败: {e}")))?;
    let rows = store
        .query_series(unique_table, ts, ts)
        .await
        .map_err(|e| ContractFailure::new(C, "query", format!("query_series 失败: {e}")))?;
    ensure(C, "read_after_write", rows.contains(&point), "查询结果不含刚写入的点")
}
