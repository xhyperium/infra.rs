//! OKX v5 API 响应类型。
//!
//! 所有结构体对应 OKX REST API 的 JSON 响应格式，
//! 通过 `serde::Deserialize` 解析。仅包含适配器需要的字段。

use serde::Deserialize;

/// OKX v5 标准响应信封。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

/// 服务端时间数据（`GET /api/v5/public/time`）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxTimeData {
    /// 毫秒时间戳（字符串）。
    pub ts: String,
}

/// 订单数据（`GET /api/v5/trade/order`、`POST /api/v5/trade/order`）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrderData {
    pub ord_id: String,
    pub cl_ord_id: String,
    pub inst_id: String,
    pub side: String,
    #[serde(rename = "ordType")]
    pub ord_type: String,
    pub px: String,
    pub sz: String,
    pub state: String,
    #[serde(default)]
    pub fill_sz: String,
    #[serde(default)]
    pub fill_px: String,
    #[serde(default)]
    pub fee: String,
    #[serde(default)]
    pub fee_ccy: String,
    #[serde(default)]
    pub c_time: String,
    #[serde(default)]
    pub u_time: String,
}

/// 账户余额（`GET /api/v5/account/balance`）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxBalanceData {
    pub total_eq: String,
    pub details: Vec<OkxBalanceDetail>,
}

/// 币种余额明细。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxBalanceDetail {
    pub ccy: String,
    pub eq: String,
    #[serde(rename = "availEq")]
    pub avail_eq: String,
    #[serde(rename = "frozenBal")]
    pub frozen_bal: String,
}

/// 交易品种信息（`GET /api/v5/public/instruments`）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxInstrumentData {
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "baseCcy")]
    pub base_ccy: String,
    #[serde(rename = "quoteCcy")]
    pub quote_ccy: String,
    pub tick_sz: String,
    pub min_sz: String,
}

/// OKX API 错误响应。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxError {
    pub code: String,
    pub msg: String,
}

impl OkxError {
    pub fn to_error_kind(&self) -> kernel::ErrorKind {
        match self.code.as_str() {
            "50001" => kernel::ErrorKind::Transient,
            "50100" | "50101" | "50102" | "50103" | "50104" | "50105" => kernel::ErrorKind::Invalid,
            "50111" | "50112" | "50113" | "50114" | "50115" => kernel::ErrorKind::Invalid,
            "51000" | "51001" | "51020" => kernel::ErrorKind::Invalid,
            "51006" => kernel::ErrorKind::Transient,
            "51601" | "51602" | "51603" => kernel::ErrorKind::Missing,
            "58350" => kernel::ErrorKind::Transient,
            _ => kernel::ErrorKind::Invalid,
        }
    }
}
