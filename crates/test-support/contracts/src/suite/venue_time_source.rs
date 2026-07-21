//! VenueTimeSource 合同 suite。

use crate::failure::{ContractFailure, ContractResult};
use contracts::VenueTimeSource;

const C: &str = "VenueTimeSource";

/// 断言 server_time 可调用并返回 `i64`。
pub async fn assert_venue_time_source(src: &dyn VenueTimeSource) -> ContractResult {
    src.server_time().await.map_err(|e| ContractFailure::new(C, "server_time", format!("{e}")))?;
    Ok(())
}
