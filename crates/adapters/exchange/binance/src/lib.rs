//! `binancex` — binance exchange adapter，生产就绪。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait（`ExecutionVenue`、
//! `MarketDataSource`、`InstrumentCatalog`、`AccountSource`、`VenueTimeSource`）。
//!
//! 注入 [`transportx::HttpDriver`]（`BinanceAdapter::with_http`）走传输边界；
//! 注入 [`BinanceApiKey`]（`BinanceAdapter::with_api_key`）启用已认证端点。
//! 未注入时回退为内存占位。

pub mod auth;
pub mod response;
mod adapter;

pub use adapter::{AdapterState, BinanceAdapter, Candle, Timeframe, parse_binance_server_time};
pub use auth::BinanceApiKey;

#[cfg(test)]
mod public_api_surface {
    use super::*;
    use decimalx::{Decimal, Price};

    #[test]
    fn default_exports_named() {
        let _key = BinanceApiKey::new("k", "s");
        let ts = parse_binance_server_time(br#"{"serverTime":1}"#).expect("ts");
        assert_eq!(ts, 1);
        let a = BinanceAdapter::mainnet();
        assert_eq!(a.state(), AdapterState::Disconnected);
        let _ = Timeframe::M1;
        let c = Candle {
            open_time: 0,
            open: Price::new(Decimal::try_new(1, 0).expect("d")),
            high: Price::new(Decimal::try_new(1, 0).expect("d")),
            low: Price::new(Decimal::try_new(1, 0).expect("d")),
            close: Price::new(Decimal::try_new(1, 0).expect("d")),
            volume: Price::new(Decimal::try_new(1, 0).expect("d")),
            close_time: 1,
        };
        assert_eq!(c.open_time, 0);
    }
}
