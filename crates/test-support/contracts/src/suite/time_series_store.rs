//! TimeSeriesStore 最小 roundtrip 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};

const C: &str = "TimeSeriesStore";
const SAMPLE_TS: i64 = 1_700_000_000_000_000_000;

/// 断言写入的确定点能在覆盖其时间戳的范围查询中被找回。
///
/// 本 suite 不声明返回顺序、重复写策略、唯一性或查询端点闭合规则。
pub async fn assert_time_series_store(
    store: &dyn TimeSeriesStore,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let table = fixture.resource("time_series_roundtrip");
    let expected = Tick {
        symbol: fixture.resource("symbol"),
        bid: Price::new(Decimal::new(10_001, 2)),
        ask: Price::new(Decimal::new(10_002, 2)),
        ts: SAMPLE_TS,
    };
    store
        .write_series(&table, vec![expected.clone()])
        .await
        .map_err(|error| ContractFailure::new(C, "write", format!("write_series 失败: {error}")))?;
    let rows = store
        .query_series(&table, SAMPLE_TS - 1, SAMPLE_TS + 1)
        .await
        .map_err(|error| ContractFailure::new(C, "query", format!("query_series 失败: {error}")))?;
    ensure(
        C,
        "roundtrip_missing",
        rows.iter().any(|row| row == &expected),
        format!("查询结果未包含写入点: rows={rows:?}"),
    )
}
