//! 行情时间语义纯检查（DM-TIME-001）。

use crate::book::looks_like_unix_millis;
use crate::{Bar, Quote, Tick, Timestamp};
use thiserror::Error;

/// 时间映射校验失败。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeError {
    /// event / received 语义冲突。
    #[error("时间语义违反: {0}")]
    Semantics(String),
    /// Bar 边界不合法。
    #[error("Bar 边界违反: {0}")]
    BarBoundary(String),
    /// 时间单位疑似非毫秒。
    #[error("时间单位可疑: {0}")]
    Unit(String),
}

/// `received_at` 不得早于 `timestamp`（本地收到完整消息不应先于事件时间被写成更早）。
///
/// 网络乱序或时钟漂移可能导致 `received_at < timestamp`；严格模式下拒绝，
/// 以强制 ingestion 注入真实 wall clock 而非复制 event time。
pub fn validate_event_vs_received(
    timestamp: Timestamp,
    received_at: Timestamp,
) -> Result<(), TimeError> {
    if !looks_like_unix_millis(timestamp) {
        return Err(TimeError::Unit(format!("timestamp({timestamp}) 不像 Unix 毫秒")));
    }
    if !looks_like_unix_millis(received_at) {
        return Err(TimeError::Unit(format!("received_at({received_at}) 不像 Unix 毫秒")));
    }
    if received_at < timestamp {
        return Err(TimeError::Semantics(format!(
            "received_at({received_at}) 早于 event timestamp({timestamp})"
        )));
    }
    Ok(())
}

/// Bar：`open_time <= close_time`，且二者均为毫秒。
pub fn validate_bar_bounds(open_time: Timestamp, close_time: Timestamp) -> Result<(), TimeError> {
    if !looks_like_unix_millis(open_time) || !looks_like_unix_millis(close_time) {
        return Err(TimeError::Unit("open_time/close_time 必须为 Unix 毫秒".into()));
    }
    if open_time > close_time {
        return Err(TimeError::BarBoundary(format!(
            "open_time({open_time}) 晚于 close_time({close_time})"
        )));
    }
    Ok(())
}

/// Tick 时间门禁。
pub fn validate_tick_time(tick: &Tick) -> Result<(), TimeError> {
    validate_event_vs_received(tick.timestamp, tick.received_at)
}

/// Quote 时间门禁。
pub fn validate_quote_time(quote: &Quote) -> Result<(), TimeError> {
    validate_event_vs_received(quote.timestamp, quote.received_at)
}

/// Bar 时间门禁。
pub fn validate_bar_time(bar: &Bar) -> Result<(), TimeError> {
    validate_bar_bounds(bar.open_time, bar.close_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BarInterval, InstrumentKey, TickDirection};
    use rust_decimal::Decimal;

    fn ik() -> InstrumentKey {
        InstrumentKey { exchange: "okx".into(), symbol: "BTC-USDT".into() }
    }

    #[test]
    fn time001_tick_accepts_received_after_event() {
        let tick = Tick {
            instrument: ik(),
            price: Decimal::new(1, 0),
            quantity: Decimal::new(1, 0),
            side: Some(TickDirection::Buy),
            trade_id: None,
            timestamp: 1_700_000_000_000,
            received_at: 1_700_000_000_050,
        };
        validate_tick_time(&tick).expect("ok");
    }

    #[test]
    fn time001_rejects_received_before_event() {
        let err = validate_event_vs_received(1_700_000_000_100, 1_700_000_000_000)
            .expect_err("received before event");
        assert!(matches!(err, TimeError::Semantics(_)));
    }

    #[test]
    fn time001_rejects_seconds_unit() {
        let err = validate_event_vs_received(1_700_000_000, 1_700_000_001).expect_err("seconds");
        assert!(matches!(err, TimeError::Unit(_)));
    }

    #[test]
    fn time001_bar_bounds() {
        validate_bar_bounds(1_700_000_000_000, 1_700_000_300_000).expect("ok");
        let err = validate_bar_bounds(1_700_000_300_000, 1_700_000_000_000).expect_err("inverted");
        assert!(matches!(err, TimeError::BarBoundary(_)));

        let bar = Bar {
            instrument: ik(),
            interval: BarInterval::Minutes(5),
            open_time: 1_700_000_000_000,
            close_time: 1_700_000_300_000,
            open: Decimal::new(1, 0),
            high: Decimal::new(2, 0),
            low: Decimal::new(1, 0),
            close: Decimal::new(2, 0),
            volume: Decimal::new(10, 0),
            quote_volume: None,
            trade_count: None,
            taker_buy_volume: None,
            taker_buy_quote_volume: None,
        };
        validate_bar_time(&bar).expect("bar ok");
    }
}
