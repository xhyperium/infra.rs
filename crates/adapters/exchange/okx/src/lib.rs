//! `okxx` — okx exchange adapter，生产默认 REST+WS 路径。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait（`ExecutionVenue`、
//! `MarketDataSource`、`InstrumentCatalog`、`AccountSource`、`VenueTimeSource`）。
//!
//! - 注入 [`transportx::HttpDriver`]（`OkxAdapter::with_http`）走 HTTP 边界
//! - 注入 [`OkxApiKey`]（`OkxAdapter::with_api_key`）启用四头鉴权 REST
//! - 注入 [`transportx::WsConnector`]（`OkxAdapter::with_ws`）启用公共行情
//!
//! 未注入时回退为明确内存占位 / 空流（不静默假成交）。

mod adapter;
pub mod auth;
pub mod market;
pub mod response;

pub use adapter::{AdapterState, OkxAdapter, parse_okx_server_time};
pub use auth::OkxApiKey;
pub use market::{
    okx_public_ws_url, okx_subscribe_message, parse_okx_orderbook, parse_okx_ticker,
    parse_okx_trade,
};

#[cfg(test)]
mod public_api_surface {
    use super::*;

    #[test]
    fn default_exports_named() {
        let _key = OkxApiKey::new("k", "s", "p");
        let ts = parse_okx_server_time(br#"{"code":"0","data":[{"ts":"1"}]}"#).expect("ts");
        assert_eq!(ts, 1);
        let a = OkxAdapter::mainnet();
        assert_eq!(a.state(), AdapterState::Disconnected);
        assert!(!a.has_ws());
        let tick = parse_okx_ticker(
            br#"{"data":[{"instId":"BTC-USDT","bidPx":"1","askPx":"2","ts":"1000"}]}"#,
        )
        .expect("tick");
        assert_eq!(tick.symbol, "BTC-USDT");
        assert_eq!(tick.ts, 1000 * 1_000_000);
    }
}
