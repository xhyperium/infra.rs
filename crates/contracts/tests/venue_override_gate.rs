//! DEFER-8 / CT-10：VenueAdapter structured cancel/query override 门禁。
//!
//! # 合同
//!
//! - **生产推荐入口**：[`contracts::ExecutionVenue`]（**无** additive default）。
//! - [`contracts::VenueAdapter`] 是迁移 facade；默认 `cancel_order_request` /
//!   `query_order_request` 返回中文 `Invalid`（常量
//!   [`VENUE_CANCEL_REQUEST_DEFAULT_MSG`] / [`VENUE_QUERY_REQUEST_DEFAULT_MSG`]）。
//! - **树内** adapter（`binancex` / `okxx`）**必须**覆盖上述方法，不得落入 default。
//! - 本文件同时断言：未 override 的最小实现仍返回上述默认中文错误。
//!
//! # 能力边界
//!
//! - 这是 **运行时** 门禁 + 常量一致性检查，**不是** 编译期 lint。
//! - 全量强制 override 的编译门禁仍属 DEFER-8 的长期项；本测闭合「可机检的最小面」。

use binancex::BinanceAdapter;
use canonical::{CancelOrderRequest, OrderRef};
use contracts::{
    ExecutionVenue, VENUE_CANCEL_REQUEST_DEFAULT_MSG, VENUE_QUERY_REQUEST_DEFAULT_MSG,
    VenueAdapter, is_default_cancel_order_request_error, is_default_query_order_request_error,
};
use okxx::OkxAdapter;

fn sample_req(venue: &str) -> CancelOrderRequest {
    CancelOrderRequest {
        venue: venue.into(),
        instrument: "BTCUSDT".into(),
        id: OrderRef::Exchange("ex-1".into()),
    }
}

/// 断言 adapter 的 structured cancel/query **不是** additive default。
async fn assert_override_not_default(adapter: &dyn VenueAdapter, venue: &str) {
    adapter.connect().await.expect("connect");
    let req = sample_req(venue);

    match adapter.cancel_order_request(&req).await {
        Ok(()) => {}
        Err(e) => {
            assert!(
                !is_default_cancel_order_request_error(&e),
                "{venue} cancel_order_request 仍为 additive default: {}",
                e.context()
            );
        }
    }

    match adapter.query_order_request(&req).await {
        Ok(_) => {}
        Err(e) => {
            assert!(
                !is_default_query_order_request_error(&e),
                "{venue} query_order_request 仍为 additive default: {}",
                e.context()
            );
        }
    }
}

#[test]
fn default_message_constants_are_stable_chinese() {
    // 锁定默认文案：树外实现者与 is_default_* helper 依赖这些子串。
    assert_eq!(
        VENUE_CANCEL_REQUEST_DEFAULT_MSG,
        "cancel_order_request 未实现；请覆盖 VenueAdapter::cancel_order_request（CAN-ID）"
    );
    assert_eq!(
        VENUE_QUERY_REQUEST_DEFAULT_MSG,
        "query_order_request 未实现；请覆盖 VenueAdapter::query_order_request（CAN-ID）"
    );
    assert!(VENUE_CANCEL_REQUEST_DEFAULT_MSG.contains("cancel_order_request 未实现"));
    assert!(VENUE_QUERY_REQUEST_DEFAULT_MSG.contains("query_order_request 未实现"));
}

#[tokio::test]
async fn binancex_overrides_structured_cancel_query() {
    let a = BinanceAdapter::testnet();
    assert_override_not_default(&a, "binance").await;
    let ex: &dyn ExecutionVenue = &a;
    let req = sample_req("binance");
    ex.cancel_order(&req).await.expect("ExecutionVenue::cancel_order");
    let _ = ex.query_order(&req).await.expect("ExecutionVenue::query_order");
    assert_eq!(ex.venue_id(), "binance");
}

#[tokio::test]
async fn okxx_overrides_structured_cancel_query() {
    let a = OkxAdapter::mainnet();
    assert_override_not_default(&a, "okx").await;
    let ex: &dyn ExecutionVenue = &a;
    let req = sample_req("okx");
    ex.cancel_order(&req).await.expect("ExecutionVenue::cancel_order");
    let _ = ex.query_order(&req).await.expect("ExecutionVenue::query_order");
    assert_eq!(ex.venue_id(), "okx");
}

#[tokio::test]
#[allow(deprecated)]
async fn non_override_venue_returns_default_chinese_invalid() {
    /// 未覆盖 request 方法的最小实现：证明 default 仍为中文 Invalid 常量。
    struct BareVenue;
    #[async_trait::async_trait]
    impl VenueAdapter for BareVenue {
        async fn connect(&self) -> kernel::XResult<()> {
            Ok(())
        }
        async fn disconnect(&self) -> kernel::XResult<()> {
            Ok(())
        }
        async fn place_order(
            &self,
            _order: &canonical::Order,
        ) -> kernel::XResult<canonical::OrderAck> {
            Err(kernel::XError::invalid("未实现"))
        }
        async fn cancel_order(&self, _id: &str) -> kernel::XResult<()> {
            Ok(())
        }
        async fn query_order(&self, _id: &str) -> kernel::XResult<canonical::OrderStatus> {
            Ok(canonical::OrderStatus::Pending)
        }
        async fn query_position(&self) -> kernel::XResult<Vec<canonical::Position>> {
            Ok(vec![])
        }
        async fn query_balance(&self) -> kernel::XResult<Vec<canonical::Money>> {
            Ok(vec![])
        }
        async fn subscribe_ticks(
            &self,
            _symbol: &str,
        ) -> kernel::XResult<futures_core::stream::BoxStream<'static, canonical::Tick>> {
            Err(kernel::XError::invalid("未实现"))
        }
        async fn subscribe_orderbook(
            &self,
            _symbol: &str,
        ) -> kernel::XResult<futures_core::stream::BoxStream<'static, canonical::OrderBookSnapshot>>
        {
            Err(kernel::XError::invalid("未实现"))
        }
        async fn subscribe_trades(
            &self,
            _symbol: &str,
        ) -> kernel::XResult<futures_core::stream::BoxStream<'static, canonical::Trade>> {
            Err(kernel::XError::invalid("未实现"))
        }
        async fn server_time(&self) -> kernel::XResult<i64> {
            Ok(0)
        }
        async fn symbol_info(&self, _symbol: &str) -> kernel::XResult<canonical::SymbolMeta> {
            Err(kernel::XError::invalid("未实现"))
        }
        fn venue_id(&self) -> &'static str {
            "bare"
        }
    }

    let v = BareVenue;
    let req = sample_req("bare");
    let e1 = v.cancel_order_request(&req).await.unwrap_err();
    assert!(is_default_cancel_order_request_error(&e1));
    assert_eq!(e1.kind(), kernel::ErrorKind::Invalid);
    assert_eq!(e1.context(), VENUE_CANCEL_REQUEST_DEFAULT_MSG);

    let e2 = v.query_order_request(&req).await.unwrap_err();
    assert!(is_default_query_order_request_error(&e2));
    assert_eq!(e2.kind(), kernel::ErrorKind::Invalid);
    assert_eq!(e2.context(), VENUE_QUERY_REQUEST_DEFAULT_MSG);
}
