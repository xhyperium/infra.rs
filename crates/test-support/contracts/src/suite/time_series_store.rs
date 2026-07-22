//! TimeSeriesStore 可移植核心 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};

const C: &str = "TimeSeriesStore";
const SAMPLE_TS: i64 = 1_700_000_000_000_000_000;

/// 写入调用方按后端精度对齐的点，并验证覆盖其时间戳的范围内可读。
///
/// 本 suite 不声明返回顺序、重复写策略、唯一性、清理或查询端点闭合规则。
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
        .map_err(|error| ContractFailure::new(C, "write", format!("write_series 失败: {error}")))?;
    let rows = store
        .query_series(unique_table, ts, ts)
        .await
        .map_err(|error| ContractFailure::new(C, "query", format!("query_series 失败: {error}")))?;
    ensure(
        C,
        "read_after_write",
        rows.iter().any(|row| row == &point),
        format!("查询结果未包含写入点: rows={rows:?}"),
    )
}

/// 使用确定性 fixture 派生表名和样例点并运行 [`assert_time_series_store`]。
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
    assert_time_series_store(store, &table, point).await
}
