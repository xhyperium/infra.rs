#![forbid(unsafe_code)]

//! # `market_data` — Market Data Kernel
//!
//! market_data.rs L0 内核 crate，提供市场数据类型、
//! 标准化数据模型与核心抽象。
//!
//! **非目标**：交易所 API 对接、数据存储（由上层 crate 负责）。

pub use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 交易产品类型
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstrumentType {
    /// 现货
    Spot,
    /// 永续合约
    Perpetual,
    /// 交割合约
    Future,
    /// 期权
    Option,
}

/// 标准化行情数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketTick {
    /// 交易对符号 (e.g., "BTC-USDT")
    pub symbol: String,
    /// 最新成交价
    pub last_price: Decimal,
    /// 24h 成交量
    pub volume_24h: Decimal,
    /// 数据时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 无效的交易产品类型
    #[error("Invalid instrument: {0}")]
    InvalidInstrument(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_instrument_type_variants() {
        assert_eq!(format!("{:?}", InstrumentType::Spot), "Spot");
        assert_eq!(format!("{:?}", InstrumentType::Perpetual), "Perpetual");
        assert_eq!(format!("{:?}", InstrumentType::Future), "Future");
        assert_eq!(format!("{:?}", InstrumentType::Option), "Option");
    }

    #[test]
    fn test_instrument_type_equality() {
        assert_eq!(InstrumentType::Spot, InstrumentType::Spot);
        assert_ne!(InstrumentType::Spot, InstrumentType::Perpetual);
    }

    #[test]
    fn test_market_tick_creation() {
        let tick = MarketTick {
            symbol: "BTC-USDT".into(),
            last_price: Decimal::new(50000, 0),
            volume_24h: Decimal::new(123456, 2),
            timestamp: Utc::now(),
        };
        assert_eq!(tick.symbol, "BTC-USDT");
        assert_eq!(tick.last_price, Decimal::new(50000, 0));
        assert_eq!(tick.volume_24h, Decimal::new(123456, 2));
    }

    #[test]
    fn test_market_tick_equality() {
        let ts = Utc::now();
        let a = MarketTick {
            symbol: "BTC-USDT".into(),
            last_price: Decimal::new(50000, 0),
            volume_24h: Decimal::new(123456, 2),
            timestamp: ts,
        };
        let b = MarketTick { volume_24h: Decimal::new(999999, 2), ..a.clone() };
        assert_ne!(a, b);
        assert_eq!(a, a.clone());
    }

    #[test]
    fn test_error_display() {
        let err = Error::InvalidInstrument("bad".into());
        assert_eq!(err.to_string(), "Invalid instrument: bad");
    }

    #[test]
    fn test_decimal_precision() {
        let price = Decimal::new(500001234, 6); // 500001234 / 10^6 = 500.001234
        assert_eq!(price.to_string(), "500.001234");
    }
}
