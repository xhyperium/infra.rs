//! `binancex` — binance exchange adapter scaffold。
//!
//! 实现 [`contracts::VenueAdapter`] 及能力拆分 trait。
//! 可选注入 [`transportx::HttpDriver`]（`BinanceAdapter::with_http`）走传输边界；
//! 默认仍为内存占位（非真实交易所协议）。

mod adapter;

pub use adapter::{AdapterState, BinanceAdapter, Candle, Timeframe, parse_binance_server_time};

#[cfg(test)]
mod public_api_surface {
    use super::*;
    use decimalx::{Decimal, Price};

    /// 默认 crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
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
