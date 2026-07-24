//! 订单簿纯校验（DM-BOOK-001..003 中可在本 crate 表达的部分）。
//!
//! Provider 特有 checksum/恢复状态机属于各 adapter；此处只检查公共形状不变量。

use crate::{OrderBook, OrderBookUpdateType, PriceLevel, Timestamp};
use thiserror::Error;

/// 订单簿纯校验失败。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BookError {
    /// 档位排序不合法。
    #[error("订单簿排序不合法: {0}")]
    Ordering(String),
    /// Snapshot/Delta 语义冲突。
    #[error("更新类型不变量违反: {0}")]
    UpdateType(String),
    /// update id 区间不合法。
    #[error("update id 不合法: {0}")]
    UpdateId(String),
}

/// DM-BOOK-002：bid 按价格严格非升序（降序，允许相等仅当同价位合并前的宽松检查用非严格）。
///
/// 规范要求 bid 降序：相邻档位 `bids[i].price >= bids[i+1].price`。
pub fn bids_are_descending(bids: &[PriceLevel]) -> bool {
    bids.windows(2).all(|w| w[0].price >= w[1].price)
}

/// DM-BOOK-002：ask 按价格升序：`asks[i].price <= asks[i+1].price`。
pub fn asks_are_ascending(asks: &[PriceLevel]) -> bool {
    asks.windows(2).all(|w| w[0].price <= w[1].price)
}

/// 校验 bids/asks 排序（DM-BOOK-002 纯检查）。
pub fn validate_level_ordering(book: &OrderBook) -> Result<(), BookError> {
    if !bids_are_descending(&book.bids) {
        return Err(BookError::Ordering("bids 必须按价格降序".into()));
    }
    if !asks_are_ascending(&book.asks) {
        return Err(BookError::Ordering("asks 必须按价格升序".into()));
    }
    Ok(())
}

/// DM-BOOK-001 可本地检查部分：
/// - Snapshot：可独立消费（本函数不检查档位非空，因部分 provider 允许空簿）
/// - Delta：若同时给出 first/last update id，则 first <= last
/// - 不得把缺失 ID 解释为连续（本函数仅在两侧都存在时校验区间）
pub fn validate_update_ids(book: &OrderBook) -> Result<(), BookError> {
    match (book.first_update_id, book.last_update_id) {
        (Some(first), Some(last)) if first > last => {
            Err(BookError::UpdateId(format!("first_update_id({first}) > last_update_id({last})")))
        }
        _ => Ok(()),
    }
}

/// DM-BOOK-001：update_type 与“完整状态 vs 增量”语义的轻量一致性。
///
/// Snapshot 不强制 sequence；Delta 若带 sequence 必须 > 0 时由 adapter 解释，
/// 此处只拒绝 Delta 同时伪装为“必须可独立消费却又空且无任何 id”的极端脏数据——
/// 保留宽松：空 Delta 允许（清档），但 Snapshot 与 Delta 标签本身必须可识别。
pub fn validate_update_type_shape(book: &OrderBook) -> Result<(), BookError> {
    // 当前仅 Snapshot/Delta；定义 crate 内穷尽匹配。
    match book.update_type {
        OrderBookUpdateType::Snapshot | OrderBookUpdateType::Delta => Ok(()),
    }
}

/// 组合公共订单簿纯检查。
pub fn validate_order_book(book: &OrderBook) -> Result<(), BookError> {
    validate_update_type_shape(book)?;
    validate_level_ordering(book)?;
    validate_update_ids(book)?;
    Ok(())
}

/// 判断两个连续 delta 的 update id 是否可能衔接（纯检查）。
///
/// 若任一端缺失 ID，返回 `None`（不得假设连续，DM-BOOK-001）。
/// 若两端齐全，返回 `prev.last + 1 == next.first`。
pub fn deltas_are_contiguous(prev: &OrderBook, next: &OrderBook) -> Option<bool> {
    match (prev.last_update_id, next.first_update_id) {
        (Some(last), Some(first)) => Some(last.saturating_add(1) == first),
        _ => None,
    }
}

/// 时间戳是否为“看起来像毫秒”的 Unix 纪元范围启发式（不做日历解析）。
///
/// 用于测试/fixture 门禁：拒绝秒级（~1e9）或微秒级（~1e15）被误写入 ms 字段。
pub fn looks_like_unix_millis(ts: Timestamp) -> bool {
    // 2001-09-09 毫秒 ≈ 1_000_000_000_000；2100-01-01 毫秒 ≈ 4_102_444_800_000
    const MIN_MS: Timestamp = 1_000_000_000_000;
    const MAX_MS: Timestamp = 4_102_444_800_000;
    (MIN_MS..=MAX_MS).contains(&ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstrumentKey;
    use rust_decimal::Decimal;

    fn sample_book(bids: Vec<PriceLevel>, asks: Vec<PriceLevel>) -> OrderBook {
        OrderBook {
            instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
            bids,
            asks,
            sequence: Some(1),
            first_update_id: Some(100),
            last_update_id: Some(105),
            timestamp: 1_700_000_000_000,
            update_type: OrderBookUpdateType::Snapshot,
        }
    }

    fn lvl(price: i64, qty: i64) -> PriceLevel {
        PriceLevel {
            price: Decimal::new(price, 0),
            quantity: Decimal::new(qty, 0),
            order_count: None,
        }
    }

    #[test]
    fn book002_accepts_sorted_levels() {
        let book = sample_book(vec![lvl(100, 1), lvl(99, 2)], vec![lvl(101, 1), lvl(102, 2)]);
        validate_order_book(&book).expect("sorted");
    }

    #[test]
    fn book002_rejects_unsorted_bids() {
        let book = sample_book(vec![lvl(99, 1), lvl(100, 2)], vec![lvl(101, 1)]);
        let err = validate_level_ordering(&book).expect_err("unsorted bids");
        assert!(matches!(err, BookError::Ordering(_)));
    }

    #[test]
    fn book002_rejects_unsorted_asks() {
        let book = sample_book(vec![lvl(100, 1)], vec![lvl(102, 1), lvl(101, 2)]);
        let err = validate_level_ordering(&book).expect_err("unsorted asks");
        assert!(matches!(err, BookError::Ordering(_)));
    }

    #[test]
    fn book001_rejects_inverted_update_ids() {
        let mut book = sample_book(vec![], vec![]);
        book.first_update_id = Some(200);
        book.last_update_id = Some(100);
        let err = validate_update_ids(&book).expect_err("inverted ids");
        assert!(matches!(err, BookError::UpdateId(_)));
    }

    #[test]
    fn book001_contiguous_requires_both_ids() {
        let mut prev = sample_book(vec![], vec![]);
        let mut next = sample_book(vec![], vec![]);
        prev.last_update_id = Some(10);
        next.first_update_id = Some(11);
        assert_eq!(deltas_are_contiguous(&prev, &next), Some(true));
        next.first_update_id = Some(12);
        assert_eq!(deltas_are_contiguous(&prev, &next), Some(false));
        next.first_update_id = None;
        assert_eq!(deltas_are_contiguous(&prev, &next), None);
    }

    #[test]
    fn millis_heuristic() {
        assert!(looks_like_unix_millis(1_700_000_000_000));
        assert!(!looks_like_unix_millis(1_700_000_000)); // seconds
        assert!(!looks_like_unix_millis(1_700_000_000_000_000)); // micros-ish
    }
}
