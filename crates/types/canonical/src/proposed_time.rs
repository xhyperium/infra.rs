//! CAN-TIME-001（**Approved 2026-07-17**）：DTO `ts: i64` = Unix epoch **纳秒**。
//!
//! 与 `kernel::Timestamp` 同刻度。本 crate **不**依赖 `kernel`（分层）。
//! 交易所 REST/WS 常给 **毫秒**：写入 DTO 前必须经 [`ns_from_unix_millis`]。

/// CAN-TIME-001：canonical `ts` 单位标签。
pub const TS_UNIT: &str = "unix_epoch_nanoseconds";

/// 兼容旧名。
#[doc(hidden)]
pub const PROPOSED_TS_UNIT: &str = TS_UNIT;

/// Exchange **毫秒** epoch → DTO **纳秒**。溢出返回 `None`。
#[must_use]
pub fn ns_from_unix_millis(ms: i64) -> Option<i64> {
    ms.checked_mul(1_000_000)
}

/// 兼容旧名。
#[must_use]
pub fn proposed_ns_from_unix_millis(ms: i64) -> Option<i64> {
    ns_from_unix_millis(ms)
}

/// DTO **纳秒** → 毫秒（向 0 截断）；用于只接受 ms 的外部 API。
#[must_use]
pub fn unix_millis_from_ns(ns: i64) -> i64 {
    ns / 1_000_000
}

/// 兼容旧名。
#[must_use]
pub fn proposed_unix_millis_from_ns(ns: i64) -> i64 {
    unix_millis_from_ns(ns)
}

/// Adapter 写入 DTO `ts` 的推荐入口（exchange ms → ns）。
#[must_use]
pub fn dto_ts_from_unix_millis(ms: i64) -> Option<i64> {
    ns_from_unix_millis(ms)
}

/// 兼容旧名。
#[must_use]
pub fn proposed_dto_ts_from_unix_millis(ms: i64) -> Option<i64> {
    dto_ts_from_unix_millis(ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_nanos_round_trip_truncating() {
        let ms = 1_700_000_000_123_i64;
        let ns = proposed_ns_from_unix_millis(ms).expect("mul");
        assert_eq!(ns, ms * 1_000_000);
        assert_eq!(proposed_unix_millis_from_ns(ns), ms);
    }

    #[test]
    fn millis_to_ns_overflow_is_none() {
        assert!(proposed_ns_from_unix_millis(i64::MAX).is_none());
    }

    #[test]
    fn unit_label_is_nanoseconds() {
        assert!(TS_UNIT.contains("nano"));
        assert_eq!(PROPOSED_TS_UNIT, TS_UNIT);
    }
}
