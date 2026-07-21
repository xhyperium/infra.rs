//! ExecutionVenue 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use canonical::{CancelOrderRequest, Order, OrderRef};
use contracts::ExecutionVenue;

const C: &str = "ExecutionVenue";

/// 断言 place / cancel / query 与 venue_id 非空。
pub async fn assert_execution_venue(venue: &dyn ExecutionVenue, order: &Order) -> ContractResult {
    ensure(C, "venue_id_nonempty", !venue.venue_id().is_empty(), "venue_id 不得为空")?;

    let ack = venue
        .place_order(order)
        .await
        .map_err(|e| ContractFailure::new(C, "place_order", format!("{e}")))?;
    ensure(C, "ack_id_nonempty", !ack.id.is_empty(), "OrderAck.id 不得为空")?;

    let req = CancelOrderRequest {
        venue: venue.venue_id(),
        instrument: order.symbol.clone(),
        id: OrderRef::Exchange(ack.id.clone()),
    };
    venue
        .cancel_order(&req)
        .await
        .map_err(|e| ContractFailure::new(C, "cancel_order", format!("{e}")))?;
    let _status = venue
        .query_order(&req)
        .await
        .map_err(|e| ContractFailure::new(C, "query_order", format!("{e}")))?;
    Ok(())
}
