//! DE-ERR-001 / DE-CAP-001 / DE-PAGE-001 契约测试。

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, InstrumentMeta, OrderAmend, Page, PageRequest, RateLimitDetail,
    RateLimitScope, StreamType, VenueAdapter, VenueCapabilities,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{ExecutionReport, Order, OrderId};

struct CapMock;

#[async_trait]
impl VenueAdapter for CapMock {
    fn exchange_id(&self) -> &str {
        "mock-venue"
    }

    fn capabilities(&self) -> VenueCapabilities {
        VenueCapabilities {
            product_lines: vec!["spot".into()],
            streams: vec![StreamType::Ticker, StreamType::Trade],
            can_trade: true,
            can_subscribe_ws: true,
            can_query_account: true,
            supports_amend: false,
            supports_pagination: true,
        }
    }

    async fn connect(&self) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn disconnect(&self) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn subscribe_ticker(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn subscribe_order_book(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn subscribe_trades(&self, _: &InstrumentKey) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn place_order(&self, _: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Unsupported("n/a".into()))
    }
    async fn cancel_order(&self, _: &OrderId, _: &InstrumentKey) -> Result<(), AdapterError> {
        Ok(())
    }
    async fn amend_order(&self, _: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Unsupported("no amend".into()))
    }
    async fn get_order(&self, _: &OrderId, _: &InstrumentKey) -> Result<Order, AdapterError> {
        Err(AdapterError::Internal("n/a".into()))
    }
    async fn get_open_orders(&self, _: &InstrumentKey) -> Result<Vec<Order>, AdapterError> {
        Ok(vec![])
    }
    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Internal("n/a".into()))
    }
    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Ok(vec![])
    }
    async fn get_order_book(
        &self,
        _: &InstrumentKey,
        _: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal("n/a".into()))
    }
}

#[test]
fn de_cap_exchange_id_and_capabilities() {
    let m = CapMock;
    assert_eq!(m.exchange_id(), "mock-venue");
    let caps = m.capabilities();
    assert!(caps.can_trade);
    assert!(caps.can_subscribe_ws);
    assert!(!caps.supports_amend);
    assert_eq!(caps.streams.len(), 2);

    let json = serde_json::to_value(&caps).unwrap();
    assert!(json.get("canTrade").is_some());
    assert!(json.get("productLines").is_some());
}

#[test]
fn de_err_rate_limit_detailed_fields() {
    let detail = RateLimitDetail {
        message: "rate exceeded".into(),
        retry_after_ms: Some(500),
        scope: Some(RateLimitScope::Account),
        provider_code: Some("TOO_MANY".into()),
        http_status: Some(429),
        request_id: Some("r-9".into()),
    };
    let err = AdapterError::rate_limit_detailed(detail.clone());
    assert_eq!(err.retry_after_ms(), Some(500));
    match err {
        AdapterError::RateLimitDetailed(d) => {
            assert_eq!(d.provider_code.as_deref(), Some("TOO_MANY"));
            assert_eq!(d.scope, Some(RateLimitScope::Account));
            assert_eq!(d.http_status, Some(429));
        }
        other => panic!("unexpected {other}"),
    }
    // serde on detail
    let v = serde_json::to_value(&detail).unwrap();
    assert_eq!(v["retryAfterMs"], 500);
    assert_eq!(v["httpStatus"], 429);
}

#[test]
fn de_page_request_and_response_round_trip() {
    let req = PageRequest {
        cursor: Some("c1".into()),
        limit: Some(100),
        start_time: Some(1_700_000_000_000),
        end_time: Some(1_700_000_100_000),
    };
    let v = serde_json::to_value(&req).unwrap();
    assert!(v.get("startTime").is_some());
    let again: PageRequest = serde_json::from_value(v).unwrap();
    assert_eq!(req, again);

    let page = Page {
        items: vec!["a".to_string(), "b".to_string()],
        next_cursor: Some("c2".into()),
        has_more: true,
    };
    assert!(page.has_more);
    let full = Page::single_page(vec![1, 2, 3]);
    assert!(!full.has_more);
    assert!(full.next_cursor.is_none());
}

#[test]
fn rest_only_capabilities_preset() {
    let caps = VenueCapabilities::rest_only_public();
    assert!(!caps.can_trade);
    assert!(!caps.can_subscribe_ws);
    assert!(caps.supports_pagination);
}

#[tokio::test]
async fn de_page_default_trait_methods_single_page() {
    let m = CapMock;
    let ik = InstrumentKey { exchange: "mock".into(), symbol: "BTCUSDT".into() };
    let page = m
        .get_open_orders_page(
            &ik,
            PageRequest {
                cursor: Some("ignored".into()),
                limit: Some(10),
                start_time: None,
                end_time: None,
            },
        )
        .await
        .expect("orders page");
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());
    assert!(domain_exchange::page_cursor_is_consistent(&page));

    let page = m
        .get_instruments_page(PageRequest {
            cursor: None,
            limit: Some(1),
            start_time: None,
            end_time: None,
        })
        .await
        .expect("instruments page");
    assert!(page.items.len() <= 1);
    assert!(domain_exchange::page_cursor_is_consistent(&page));
}

#[test]
fn de_page_cursor_consistency_and_limit() {
    use domain_exchange::{Page, apply_page_limit, page_cursor_is_consistent};

    let ok = Page::from_cursor(vec![1, 2], Some("n".into()));
    assert!(ok.has_more);
    assert!(page_cursor_is_consistent(&ok));

    let bad = Page { items: vec![1], next_cursor: None, has_more: true };
    assert!(!page_cursor_is_consistent(&bad));

    let limited = apply_page_limit(vec![1, 2, 3, 4], Some(2));
    assert_eq!(limited, vec![1, 2]);
}
