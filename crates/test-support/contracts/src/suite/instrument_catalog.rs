//! InstrumentCatalog 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use contracts::InstrumentCatalog;

const C: &str = "InstrumentCatalog";

/// 断言已知 symbol 返回匹配元数据。
pub async fn assert_instrument_catalog(
    catalog: &dyn InstrumentCatalog,
    symbol: &str,
) -> ContractResult {
    let meta = catalog
        .symbol_info(symbol)
        .await
        .map_err(|e| ContractFailure::new(C, "symbol_info", format!("{e}")))?;
    ensure(
        C,
        "symbol_match",
        meta.symbol == symbol,
        format!("meta.symbol={} 期望 {symbol}", meta.symbol),
    )
}
