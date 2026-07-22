//! OKX v5 公共 WS / 行情帧解析（纯逻辑，离线可测）。

use canonical::{OrderBookSnapshot, PriceLevel, Tick, Trade, ns_from_unix_millis};
use decimalx::{Decimal, Price, Qty};
use kernel::{XError, XResult};

fn parse_decimal(s: &str, field: &str) -> XResult<Decimal> {
    s.parse::<Decimal>().map_err(|e| XError::invalid(format!("okx market {field}: {e}")))
}

fn parse_price(s: &str, field: &str) -> XResult<Price> {
    Ok(Price::new(parse_decimal(s, field)?))
}

fn parse_qty(s: &str, field: &str) -> XResult<Qty> {
    Ok(Qty::new(parse_decimal(s, field)?))
}

fn ms_str_to_ns(s: &str) -> XResult<i64> {
    let ms: i64 = s.parse().map_err(|e| XError::invalid(format!("okx ts parse: {e}")))?;
    ns_from_unix_millis(ms).ok_or_else(|| XError::invalid(format!("okx ts overflow: {ms}")))
}

/// 解析 OKX `tickers` 频道推送 → [`Tick`]。
///
/// 形如：`{"arg":{"channel":"tickers","instId":"BTC-USDT"},"data":[{...}]}`
pub fn parse_okx_ticker(body: &[u8]) -> XResult<Tick> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("okx ticker json: {e}")))?;
    let row = first_data_row(&v)?;
    let symbol = row
        .get("instId")
        .and_then(|x| x.as_str())
        .ok_or_else(|| XError::invalid("okx ticker missing instId"))?
        .to_string();
    let bid = parse_price(
        row.get("bidPx")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx ticker missing bidPx"))?,
        "bidPx",
    )?;
    let ask = parse_price(
        row.get("askPx")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx ticker missing askPx"))?,
        "askPx",
    )?;
    let ts = ms_str_to_ns(
        row.get("ts")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx ticker missing ts"))?,
    )?;
    Ok(Tick { symbol, bid, ask, ts })
}

/// 解析 OKX `trades` 频道推送 → [`Trade`]。
pub fn parse_okx_trade(body: &[u8]) -> XResult<Trade> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("okx trade json: {e}")))?;
    let row = first_data_row(&v)?;
    let symbol = row
        .get("instId")
        .and_then(|x| x.as_str())
        .ok_or_else(|| XError::invalid("okx trade missing instId"))?
        .to_string();
    let price = parse_price(
        row.get("px")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx trade missing px"))?,
        "px",
    )?;
    let qty = parse_qty(
        row.get("sz")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx trade missing sz"))?,
        "sz",
    )?;
    let ts = ms_str_to_ns(
        row.get("ts")
            .and_then(|x| x.as_str())
            .ok_or_else(|| XError::invalid("okx trade missing ts"))?,
    )?;
    Ok(Trade { symbol, price, qty, ts })
}

/// 解析 OKX `books5` / `books` 频道推送 → [`OrderBookSnapshot`]。
pub fn parse_okx_orderbook(body: &[u8], symbol_fallback: &str) -> XResult<OrderBookSnapshot> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("okx books json: {e}")))?;
    let row = first_data_row(&v)?;
    let symbol = row
        .get("instId")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("arg").and_then(|a| a.get("instId")).and_then(|x| x.as_str()))
        .unwrap_or(symbol_fallback)
        .to_string();
    let bids = parse_levels(
        row.get("bids")
            .and_then(|x| x.as_array())
            .ok_or_else(|| XError::invalid("okx books missing bids"))?,
    )?;
    let asks = parse_levels(
        row.get("asks")
            .and_then(|x| x.as_array())
            .ok_or_else(|| XError::invalid("okx books missing asks"))?,
    )?;
    let ts = match row.get("ts").and_then(|x| x.as_str()) {
        Some(s) => ms_str_to_ns(s)?,
        None => 0,
    };
    Ok(OrderBookSnapshot { symbol, bids, asks, ts })
}

fn first_data_row(v: &serde_json::Value) -> XResult<&serde_json::Value> {
    v.get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| XError::invalid("okx frame missing data[0]"))
}

fn parse_levels(arr: &[serde_json::Value]) -> XResult<Vec<PriceLevel>> {
    let mut out = Vec::with_capacity(arr.len());
    for level in arr {
        let pair = level.as_array().ok_or_else(|| XError::invalid("okx level not array"))?;
        if pair.len() < 2 {
            return Err(XError::invalid("okx level needs price,qty"));
        }
        let p = pair[0].as_str().ok_or_else(|| XError::invalid("okx level price not str"))?;
        let q = pair[1].as_str().ok_or_else(|| XError::invalid("okx level qty not str"))?;
        out.push(PriceLevel {
            price: parse_price(p, "level.price")?,
            qty: parse_qty(q, "level.qty")?,
        });
    }
    Ok(out)
}

/// 构造 OKX 公共 WS 基址 URL。
#[must_use]
pub fn okx_public_ws_url(base: &str) -> String {
    base.trim_end_matches('/').to_string()
}

/// 构造 OKX 订阅消息 JSON。
#[must_use]
pub fn okx_subscribe_message(channel: &str, inst_id: &str) -> String {
    format!(r#"{{"op":"subscribe","args":[{{"channel":"{channel}","instId":"{inst_id}"}}]}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ticker_fixture() {
        let body = br#"{"arg":{"channel":"tickers","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","bidPx":"42000.1","askPx":"42000.2","ts":"1700000000123"}]}"#;
        let tick = parse_okx_ticker(body).expect("tick");
        assert_eq!(tick.symbol, "BTC-USDT");
        assert_eq!(tick.bid.as_decimal(), "42000.1".parse().unwrap());
        assert_eq!(tick.ask.as_decimal(), "42000.2".parse().unwrap());
        assert_eq!(tick.ts, 1_700_000_000_123 * 1_000_000);
    }

    #[test]
    fn parse_trade_fixture() {
        let body = br#"{"arg":{"channel":"trades","instId":"ETH-USDT"},"data":[{"instId":"ETH-USDT","px":"2500.5","sz":"0.01","side":"buy","ts":"1700000000456"}]}"#;
        let t = parse_okx_trade(body).expect("trade");
        assert_eq!(t.symbol, "ETH-USDT");
        assert_eq!(t.price.as_decimal(), "2500.5".parse().unwrap());
        assert_eq!(t.qty.as_decimal(), "0.01".parse().unwrap());
    }

    #[test]
    fn parse_books_fixture() {
        let body = br#"{"arg":{"channel":"books5","instId":"BTC-USDT"},"data":[{"asks":[["1.1","3","0","1"]],"bids":[["1.0","2","0","1"]],"ts":"1700000000789"}]}"#;
        let book = parse_okx_orderbook(body, "BTC-USDT").expect("book");
        assert_eq!(book.symbol, "BTC-USDT");
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks[0].price.as_decimal(), "1.1".parse().unwrap());
        assert_eq!(book.ts, 1_700_000_000_789 * 1_000_000);
    }

    #[test]
    fn subscribe_message_shape() {
        let m = okx_subscribe_message("tickers", "BTC-USDT");
        assert!(m.contains("subscribe"));
        assert!(m.contains("tickers"));
        assert!(m.contains("BTC-USDT"));
    }
}
