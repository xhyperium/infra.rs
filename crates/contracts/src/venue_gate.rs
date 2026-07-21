//! VenueAdapter additive default 门禁辅助（生产契约侧常量与检测）。
//!
//! Fake 实现见 `contract-testkit` crate。

use kernel::XError;

/// [`crate::VenueAdapter::cancel_order_request`] 未覆盖时的默认中文错误上下文。
pub const VENUE_CANCEL_REQUEST_DEFAULT_MSG: &str =
    "cancel_order_request 未实现；请覆盖 VenueAdapter::cancel_order_request（CAN-ID）";

/// [`crate::VenueAdapter::query_order_request`] 未覆盖时的默认中文错误上下文。
pub const VENUE_QUERY_REQUEST_DEFAULT_MSG: &str =
    "query_order_request 未实现；请覆盖 VenueAdapter::query_order_request（CAN-ID）";

/// 判断是否为 additive default 的 cancel 未实现错误。
pub fn is_default_cancel_order_request_error(err: &XError) -> bool {
    err.kind() == kernel::ErrorKind::Invalid
        && err.context().contains("cancel_order_request 未实现")
}

/// 判断是否为 additive default 的 query 未实现错误。
pub fn is_default_query_order_request_error(err: &XError) -> bool {
    err.kind() == kernel::ErrorKind::Invalid && err.context().contains("query_order_request 未实现")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::XError;

    #[test]
    fn default_venue_error_helpers() {
        let e = XError::invalid(VENUE_CANCEL_REQUEST_DEFAULT_MSG);
        assert!(is_default_cancel_order_request_error(&e));
        assert!(!is_default_query_order_request_error(&e));
        let e2 = XError::invalid(VENUE_QUERY_REQUEST_DEFAULT_MSG);
        assert!(is_default_query_order_request_error(&e2));
        let e3 = XError::unavailable("未连接");
        assert!(!is_default_cancel_order_request_error(&e3));
    }
}
