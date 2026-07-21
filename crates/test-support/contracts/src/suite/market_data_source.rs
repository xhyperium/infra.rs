//! MarketDataSource 合同 suite。

use crate::failure::{ContractFailure, ContractResult};
use contracts::MarketDataSource;

const C: &str = "MarketDataSource";

/// 断言三条订阅入口均可调用（允许空流；错误必须是类型化 XError）。
pub async fn assert_market_data_source(src: &dyn MarketDataSource, symbol: &str) -> ContractResult {
    let ticks = src
        .subscribe_ticks(symbol)
        .await
        .map_err(|e| ContractFailure::new(C, "subscribe_ticks", format!("{e}")))?;
    let book = src
        .subscribe_orderbook(symbol)
        .await
        .map_err(|e| ContractFailure::new(C, "subscribe_orderbook", format!("{e}")))?;
    let trades = src
        .subscribe_trades(symbol)
        .await
        .map_err(|e| ContractFailure::new(C, "subscribe_trades", format!("{e}")))?;
    // 持有流以证明 subscribe 返回可用 Stream（允许立即结束）。
    let _ = (ticks, book, trades);
    Ok(())
}
