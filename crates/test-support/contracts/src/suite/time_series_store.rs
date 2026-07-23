//! TimeSeriesStore 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};

const C: &str = "TimeSeriesStore";
const SAMPLE_TS: i64 = 1_700_000_000_000_000_000;

/// ClosedPoint profile：写入调用方按后端精度对齐的点，并以 `[ts, ts]` 查询。
///
/// 本函数保留 0.1.2 API，只适用于明确支持闭区间零宽查询的实现。可移植验证使用
/// [`assert_time_series_store_in_window`]，由调用方提供后端语义正确的查询窗口。
///
/// # Errors
///
/// 表名为空、后端调用失败或查询结果缺少写入点时返回 [`ContractFailure`]。
pub async fn assert_time_series_store(
    store: &dyn TimeSeriesStore,
    unique_table: &str,
    point: Tick,
) -> ContractResult {
    assert_time_series_store_in_window(store, unique_table, point.clone(), point.ts, point.ts).await
}

/// 写入一点，并验证调用方给定的查询窗口返回该点。
///
/// 本函数不推断端点闭合、排序、重复写、唯一性或清理语义。
///
/// # Errors
///
/// 表名为空、后端调用失败或查询结果缺少写入点时返回 [`ContractFailure`]。
pub async fn assert_time_series_store_in_window(
    store: &dyn TimeSeriesStore,
    unique_table: &str,
    point: Tick,
    query_start: i64,
    query_end: i64,
) -> ContractResult {
    ensure(C, "unique_table", !unique_table.is_empty(), "测试表名不得为空")?;
    store
        .write_series(unique_table, vec![point.clone()])
        .await
        .map_err(|error| ContractFailure::new(C, "write", format!("write_series 失败: {error}")))?;
    let rows = store
        .query_series(unique_table, query_start, query_end)
        .await
        .map_err(|error| ContractFailure::new(C, "query", format!("query_series 失败: {error}")))?;
    ensure(
        C,
        "read_after_write",
        rows.iter().any(|row| row == &point),
        format!("查询结果未包含写入点: rows={rows:?}"),
    )
}

/// 使用确定性 fixture 派生表名和样例点，并以兼容闭/半开语义的窗口运行 suite。
///
/// # Errors
///
/// 资源名派生失败或窗口 suite 失败时返回 [`ContractFailure`]。
pub async fn assert_time_series_store_with_fixture(
    store: &dyn TimeSeriesStore,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let table = fixture.resource("time_series_roundtrip")?;
    let point = Tick {
        symbol: fixture.resource("symbol")?,
        bid: Price::new(Decimal::new(10_001, 2)),
        ask: Price::new(Decimal::new(10_002, 2)),
        ts: SAMPLE_TS,
    };
    assert_time_series_store_in_window(store, &table, point, SAMPLE_TS, SAMPLE_TS + 1).await
}
