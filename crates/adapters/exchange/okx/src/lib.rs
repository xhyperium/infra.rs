//! `okxx` — okx exchange adapter��生产就绪。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait（`ExecutionVenue`、
//! `MarketDataSource`、`InstrumentCatalog`、`AccountSource`、`VenueTimeSource`）。
//!
//! 注入 [`transportx::HttpDriver`]（`OkxAdapter::with_http`）走传输边界；
//! 注入 [`OkxApiKey`]（`OkxAdapter::with_api_key`）启用已认证端点。
//! 未注入时回退为内存占位。

mod adapter;
pub mod auth;
pub mod response;

pub use adapter::{AdapterState, OkxAdapter, parse_okx_server_time};
pub use auth::OkxApiKey;

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
    }
}
