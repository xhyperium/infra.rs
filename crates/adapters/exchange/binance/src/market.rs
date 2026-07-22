//! Binance 公共 WS / 行情帧解析（纯逻辑，离线可测）。
//!
//! 支持：
//! - `bookTicker` → [`Tick`]
//! - `trade` / `aggTrade` → [`Trade`]
//! - partial `depth` 快照 → [`OrderBookSnapshot`]

use canonical::{OrderBookSnapshot, PriceLevel, Tick, Trade, ns_from_unix_millis};
use decimalx::{Decimal, Price, Qty};
use kernel::{XError, XResult};

fn parse_decimal(s: &str, field: &str) -> XResult<Decimal> {
    s.parse::<Decimal>().map_err(|e| XError::invalid(format!("binance market {field}: {e}")))
}

fn parse_price(s: &str, field: &str) -> XResult<Price> {
    Ok(Price::new(parse_decimal(s, field)?))
}

fn parse_qty(s: &str, field: &str) -> XResult<Qty> {
    Ok(Qty::new(parse_decimal(s, field)?))
}

fn ms_to_ns(ms: i64) -> XResult<i64> {
    ns_from_unix_millis(ms).ok_or_else(|| XError::invalid(format!("binance ts overflow: {ms}")))
}

/// 解析 Binance `bookTicker` 帧为 [`Tick`]。
///
/// 形如：`{"u":…,"s":"BTCUSDT","b":"…","B":"…","a":"…","A":"…"}`  
/// 无事件时间时 `ts = 0`。
pub fn parse_binance_book_ticker(body: &[u8]) -> XResult<Tick> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("bookTicker json: {e}")))?;
    let symbol = v
        .get("s")
        .and_then(|x| x.as_str())
        .ok_or_else(|| XError::invalid("bookTicker missing s"))?
        .to_string();
    let bid = parse_price(
        v.get("b")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("bookTicker missing b"))?,
        "bid",
    )?;
    let ask = parse_price(
        v.get("a")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("bookTicker missing a"))?,
        "ask",
    )?;
    // bookTicker 无独立事件时间；若有 E 则用 E。
    let ts = match v.get("E").and_then(|x| x.as_i64()) {
        Some(ms) => ms_to_ns(ms)?,
        None => 0,
    };
    Ok(Tick { symbol, bid, ask, ts })
}

/// 解析 Binance `trade` / `aggTrade` 帧为 [`Trade`]。
pub fn parse_binance_trade(body: &[u8]) -> XResult<Trade> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| XError::invalid(format!("trade json: {e}")))?;
    let symbol = v
        .get("s")
        .and_then(|x| x.as_str())
        .ok_or_else(|| XError::invalid("trade missing s"))?
        .to_string();
    let price = parse_price(
        v.get("p").and_then(|x| x.as_str()).ok_or_else(|| XError::invalid("trade missing p"))?,
        "price",
    )?;
    let qty = parse_qty(
        v.get("q").and_then(|x| x.as_str()).ok_or_else(|| XError::invalid("trade missing q"))?,
        "qty",
    )?;
    let ms = v
        .get("T")
        .or_else(|| v.get("E"))
        .and_then(|x| x.as_i64())
        .ok_or_else(|| XError::invalid("trade missing T/E"))?;
    Ok(Trade { symbol, price, qty, ts: ms_to_ns(ms)? })
}

/// 解析 Binance partial depth 快照为 [`OrderBookSnapshot`]。
///
/// 支持有 `e":"depthUpdate"` 的增量帧（取 b/a）与 REST 形 `bids`/`asks` 快照。
pub fn parse_binance_orderbook(body: &[u8], symbol_fallback: &str) -> XResult<OrderBookSnapshot> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| XError::invalid(format!("depth json: {e}")))?;
    let symbol = v.get("s").and_then(|x| x.as_str()).unwrap_or(symbol_fallback).to_string();

    let bids_raw = v
        .get("bids")
        .or_else(|| v.get("b"))
        .and_then(|x| x.as_array())
        .ok_or_else(|| XError::invalid("depth missing bids/b"))?;
    let asks_raw = v
        .get("asks")
        .or_else(|| v.get("a"))
        .and_then(|x| x.as_array())
        .ok_or_else(|| XError::invalid("depth missing asks/a"))?;

    let bids = parse_levels(bids_raw)?;
    let asks = parse_levels(asks_raw)?;
    let ts = match v.get("E").or_else(|| v.get("T")).and_then(|x| x.as_i64()) {
        Some(ms) => ms_to_ns(ms)?,
        None => 0,
    };
    Ok(OrderBookSnapshot { symbol, bids, asks, ts })
}

fn parse_levels(arr: &[serde_json::Value]) -> XResult<Vec<PriceLevel>> {
    let mut out = Vec::with_capacity(arr.len());
    for level in arr {
        let pair = level.as_array().ok_or_else(|| XError::invalid("depth level not array"))?;
        if pair.len() < 2 {
            return Err(XError::invalid("depth level needs price,qty"));
        }
        let p = pair[0].as_str().ok_or_else(|| XError::invalid("depth price not str"))?;
        let q = pair[1].as_str().ok_or_else(|| XError::invalid("depth qty not str"))?;
        out.push(PriceLevel {
            price: parse_price(p, "level.price")?,
            qty: parse_qty(q, "level.qty")?,
        });
    }
    Ok(out)
}

/// 构造公共 WS stream URL（combined 单流）。
#[must_use]
pub fn binance_ws_stream_url(base: &str, symbol: &str, stream: &str) -> String {
    let base = base.trim_end_matches('/');
    let sym = symbol.to_ascii_lowercase();
    format!("{base}/ws/{sym}@{stream}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_book_ticker_fixture() {
        let body = br#"{"u":400900217,"s":"BTCUSDT","b":"42000.10","B":"1.5","a":"42000.20","A":"2.0","E":1700000000123}"#;
        let tick = parse_binance_book_ticker(body).expect("tick");
        assert_eq!(tick.symbol, "BTCUSDT");
        assert_eq!(tick.bid.as_decimal(), "42000.10".parse().unwrap());
        assert_eq!(tick.ask.as_decimal(), "42000.20".parse().unwrap());
        assert_eq!(tick.ts, 1_700_000_000_123 * 1_000_000);
    }

    #[test]
    fn parse_trade_fixture() {
        let body = br#"{"e":"trade","E":1700000000456,"s":"ETHUSDT","t":1,"p":"2500.5","q":"0.01","T":1700000000456,"m":true}"#;
        let t = parse_binance_trade(body).expect("trade");
        assert_eq!(t.symbol, "ETHUSDT");
        assert_eq!(t.price.as_decimal(), "2500.5".parse().unwrap());
        assert_eq!(t.qty.as_decimal(), "0.01".parse().unwrap());
        assert_eq!(t.ts, 1_700_000_000_456 * 1_000_000);
    }

    #[test]
    fn parse_depth_snapshot_fixture() {
        let body = br#"{"lastUpdateId":160,"bids":[["0.0024","10"],["0.0023","5"]],"asks":[["0.0026","100"]]}"#;
        let book = parse_binance_orderbook(body, "BNBBTC").expect("book");
        assert_eq!(book.symbol, "BNBBTC");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids[0].price.as_decimal(), "0.0024".parse().unwrap());
        assert_eq!(book.bids[0].qty.as_decimal(), "10".parse().unwrap());
    }

    #[test]
    fn ws_url_lowercases_symbol() {
        let u = binance_ws_stream_url("wss://stream.binance.com:9443", "BTCUSDT", "bookTicker");
        assert_eq!(u, "wss://stream.binance.com:9443/ws/btcusdt@bookTicker");
    }

    #[test]
    fn bad_json_is_invalid() {
        assert!(parse_binance_book_ticker(b"{").is_err());
        assert!(parse_binance_trade(br#"{"s":"X"}"#).is_err());
    }
}
