//! `binancex` — binance exchange adapter，生产默认 REST+WS 路径。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait（`ExecutionVenue`、
//! `MarketDataSource`、`InstrumentCatalog`、`AccountSource`、`VenueTimeSource`）。
//!
//! - 注入 [`transportx::HttpDriver`]（`BinanceAdapter::with_http`）走 HTTP 边界
//! - 注入 [`BinanceApiKey`]（`BinanceAdapter::with_api_key`）启用已认证端点
//! - 注入 [`transportx::WsConnector`]（`BinanceAdapter::with_ws`）启用公共行情
//!
//! 未注入时回退为明确内存占位 / 空流（不静默假成交）。

mod adapter;
pub mod auth;
pub mod market;
pub mod response;

pub use adapter::{AdapterState, BinanceAdapter, Candle, Timeframe, parse_binance_server_time};
pub use auth::BinanceApiKey;
pub use market::{
    binance_ws_stream_url, parse_binance_book_ticker, parse_binance_orderbook, parse_binance_trade,
};

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
        assert!(!a.has_ws());
        let _ = Timeframe::M1.to_api_str();
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
        // 公开解析入口（consumer 路径）
        let tick = parse_binance_book_ticker(br#"{"s":"BTCUSDT","b":"1.0","a":"1.1","E":1000}"#)
            .expect("tick");
        assert_eq!(tick.symbol, "BTCUSDT");
        assert_eq!(tick.ts, 1000 * 1_000_000);
    }
}
