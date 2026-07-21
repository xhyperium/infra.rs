//! AccountSource 合同 suite。

use crate::failure::{ContractFailure, ContractResult};
use contracts::AccountSource;

const C: &str = "AccountSource";

/// 断言 position / balance 查询可调用（允许空列表）。
pub async fn assert_account_source(src: &dyn AccountSource) -> ContractResult {
    src.query_position()
        .await
        .map_err(|e| ContractFailure::new(C, "query_position", format!("{e}")))?;
    src.query_balance()
        .await
        .map_err(|e| ContractFailure::new(C, "query_balance", format!("{e}")))?;
    Ok(())
}
