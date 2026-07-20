//! 形状级辅助函数（供 **adapter** 入口使用）。
//!
//! - **不是** domain 业务校验（正 qty、状态机、symbol 存在性等仍在 domain/adapter 业务层）。
//! - 规则对应 CAN-ID-001（**Approved 2026-07-17**）；本模块只做形状级防御。
//! - canonical 不因这些检查失败而 panic；由调用方决定如何映射错误。

/// 非空且 trim 后非空。
#[must_use]
pub fn is_nonempty_token(s: &str) -> bool {
    !s.is_empty() && !s.trim().is_empty()
}

/// Venue slug 形状：非空、全 ASCII、仅小写字母/数字/连字符、不以连字符开头或结尾。
///
/// CAN-ID-001 Approved：adapter 入口应使用；仍 **不是** domain 业务校验。
#[must_use]
pub fn is_plausible_venue_slug(s: &str) -> bool {
    if !is_nonempty_token(s) || s.len() > 64 {
        return false;
    }
    let b = s.as_bytes();
    if b[0] == b'-' || b[b.len() - 1] == b'-' {
        return false;
    }
    s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// Instrument 形状：非空、长度上限、无控制字符；**不做**跨所归一。
#[must_use]
pub fn is_plausible_instrument_id(s: &str) -> bool {
    if !is_nonempty_token(s) || s.len() > 128 {
        return false;
    }
    !s.chars().any(|c| c.is_control())
}

/// [`super::OrderRef`] 载荷非空。
#[must_use]
pub fn order_ref_payload_nonempty(r: &super::OrderRef) -> bool {
    match r {
        super::OrderRef::Client(id) | super::OrderRef::Exchange(id) => is_nonempty_token(id),
    }
}

/// [`super::CancelOrderRequest`] 形状级完整性（venue/instrument/id 载荷）。
#[must_use]
pub fn cancel_request_shape_ok(req: &super::CancelOrderRequest) -> bool {
    is_plausible_venue_slug(&req.venue)
        && is_plausible_instrument_id(&req.instrument)
        && order_ref_payload_nonempty(&req.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CancelOrderRequest, OrderRef};

    #[test]
    fn venue_slug_proposal_rules() {
        assert!(is_plausible_venue_slug("okx"));
        assert!(is_plausible_venue_slug("binance"));
        assert!(is_plausible_venue_slug("binance-us"));
        assert!(!is_plausible_venue_slug(""));
        assert!(!is_plausible_venue_slug("OKX"));
        assert!(!is_plausible_venue_slug("-okx"));
        assert!(!is_plausible_venue_slug("okx-"));
        assert!(!is_plausible_venue_slug("okx_1"));
    }

    #[test]
    fn instrument_and_order_ref_shape() {
        assert!(is_plausible_instrument_id("BTC-USDT"));
        assert!(!is_plausible_instrument_id(""));
        assert!(order_ref_payload_nonempty(&OrderRef::Exchange("1".into())));
        assert!(!order_ref_payload_nonempty(&OrderRef::Client(String::new())));
    }

    #[test]
    fn cancel_request_shape_ok_examples() {
        let ok = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        assert!(cancel_request_shape_ok(&ok));
        let bad = CancelOrderRequest {
            venue: "OKX".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("987".into()),
        };
        assert!(!cancel_request_shape_ok(&bad));
    }
}
