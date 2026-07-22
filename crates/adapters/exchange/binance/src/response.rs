//! Binance v3 API 响应类型。
//!
//! 所有结构体对应 Binance REST API 的 JSON 响应格式，
//! 通过 `serde::Deserialize` 解析。仅包含适配器需要的字段。

use serde::Deserialize;

/// `GET /api/v3/time` 响应。
#[derive(Debug, Clone, Deserialize)]
pub struct ServerTime {
    #[serde(rename = "serverTime")]
    pub server_time: i64,
}

/// `GET /api/v3/exchangeInfo` 中的单个 symbol 信息。
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInfoSymbol {
    pub symbol: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    pub filters: Vec<serde_json::Value>,
}

/// `GET /api/v3/exchangeInfo` 响应。
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<ExchangeInfoSymbol>,
}

/// `POST /api/v3/order`、`GET /api/v3/order`、`DELETE /api/v3/order` 响应。
#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty", default)]
    pub cumulative_quote_qty: String,
    pub status: String,
    pub side: String,
    pub price: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "timeInForce", default)]
    pub time_in_force: String,
    #[serde(rename = "updateTime", default)]
    pub update_time: i64,
    #[serde(default)]
    pub time: i64,
}

/// `DELETE /api/v3/order` 撤单响应。
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    pub symbol: String,
    #[serde(rename = "origClientOrderId")]
    pub orig_client_order_id: String,
    #[serde(rename = "orderId")]
    pub order_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub status: String,
}

/// `GET /api/v3/account` 中的单个余额条目。
#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

/// `GET /api/v3/account` 响应。
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub balances: Vec<Balance>,
}

/// Binance API 错误响应。
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceError {
    pub code: i32,
    pub msg: String,
}

impl BinanceError {
    /// 将 Binance 错误码映射为 kernel [`ErrorKind`](kernel::ErrorKind)。
    pub fn to_error_kind(&self) -> kernel::ErrorKind {
        match self.code {
            -1003 | -1015 | -1016 => kernel::ErrorKind::Transient,
            -1013 => kernel::ErrorKind::Invalid,
            -1021 => kernel::ErrorKind::Invalid,
            -2011 => kernel::ErrorKind::Missing,
            -2010 => kernel::ErrorKind::Invalid,
            -2014 | -2015 => kernel::ErrorKind::Invalid,
            _ => kernel::ErrorKind::Invalid,
        }
    }
}

/// `GET /api/v3/klines` 单根 K 线。
#[derive(Debug, Clone)]
pub struct KlineData {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_asset_volume: String,
    pub number_of_trades: i64,
    pub taker_buy_base_volume: String,
    pub taker_buy_quote_volume: String,
}

impl KlineData {
    /// 从 Binance klines 数组中的 `Vec<Value>` 解析。
    pub fn from_json_array(arr: &[serde_json::Value]) -> Option<Self> {
        if arr.len() < 12 {
            return None;
        }
        Some(Self {
            open_time: arr[0].as_i64()?,
            open: arr[1].as_str()?.to_string(),
            high: arr[2].as_str()?.to_string(),
            low: arr[3].as_str()?.to_string(),
            close: arr[4].as_str()?.to_string(),
            volume: arr[5].as_str()?.to_string(),
            close_time: arr[6].as_i64()?,
            quote_asset_volume: arr[7].as_str()?.to_string(),
            number_of_trades: arr[8].as_i64()?,
            taker_buy_base_volume: arr[9].as_str()?.to_string(),
            taker_buy_quote_volume: arr[10].as_str()?.to_string(),
        })
    }
}
