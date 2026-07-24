#![forbid(unsafe_code)]
#![allow(dead_code)]

//! # exchange_okx — OKX Exchange Adapter
//!
//! OKX 交易所适配器，覆盖 Spot、Swap（永续合约）、Futures（交割合约）三大产品线。
//!
//! 规格 SSOT：`.agents/ssot/market_data/okx/spec/spec.md`

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, ExecutionReport, InstrumentMeta, OrderAmend, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Order, OrderId};
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
//  OKX WebSocket 频道类型（§2.1）
// ──────────────────────────────────────────────

/// OKX WebSocket 可订阅的频道
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OkxStream {
    /// Ticker 数据（最优买卖价 + 24hr 统计）
    Tickers { inst_id: String },
    /// 1 分钟 K 线
    Candle1m { inst_id: String },
    /// 5 分钟 K 线
    Candle5m { inst_id: String },
    /// 逐笔成交
    Trades { inst_id: String },
    /// 深度数据
    Books { inst_id: String, depth: BookDepth },
    /// 5 档深度
    Books5 { inst_id: String },
    /// 50 档深度（Tick-by-Tick）
    Books50L2Tbt { inst_id: String },
}

/// 深度订阅级别
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BookDepth {
    Level5,
    Level50,
    Level200,
    /// Level 2 Top of Book Tick-by-Tick
    L2Tbt,
    /// Level 3 Tick-by-Tick
    L3Tbt,
}

// ──────────────────────────────────────────────
//  OKX REST API 响应类型（§3）
// ──────────────────────────────────────────────

/// OKX 通用 REST 响应包装
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

/// 交易对信息
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    pub inst_id: String,
    pub inst_type: String,
    pub base_ccy: String,
    pub quote_ccy: String,
    pub state: String,
}

// ──────────────────────────────────────────────
//  连接管理（§4）
// ──────────────────────────────────────────────

/// OKX 适配器配置
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OkxConfig {
    pub rest_base_url: String,
    pub ws_public_url: String,
    pub ws_private_url: String,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub passphrase: Option<String>,
    pub reconnect_max_attempts: u32,
    pub reconnect_base_delay_ms: u64,
    pub ping_interval_secs: u64,
}

impl Default for OkxConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://www.okx.com".into(),
            ws_public_url: "wss://ws.okx.com:8443/ws/v5/public".into(),
            ws_private_url: "wss://ws.okx.com:8443/ws/v5/private".into(),
            api_key: None,
            secret_key: None,
            passphrase: None,
            reconnect_max_attempts: 5,
            reconnect_base_delay_ms: 1000,
            ping_interval_secs: 20,
        }
    }
}

/// OKX 连接状态
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OkxConnection {
    pub subscriptions: Vec<OkxStream>,
    pub connected_at: Option<domainx::Timestamp>,
}

// ──────────────────────────────────────────────
//  VenueAdapter 实现
// ──────────────────────────────────────────────

/// OKX 交易所适配器
#[derive(Debug)]
pub struct OkxAdapter {
    base_url: String,
    ws_url: String,
    api_key: Option<String>,
}

impl OkxAdapter {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            base_url: "https://www.okx.com".into(),
            ws_url: "wss://ws.okx.com:8443/ws/v5/public".into(),
            api_key,
        }
    }
}

#[async_trait]
impl VenueAdapter for OkxAdapter {
    async fn connect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::connect: not implemented (skeleton)".into()))
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::disconnect: not implemented (skeleton)".into()))
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::subscribe_ticker: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::subscribe_order_book: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::subscribe_trades: not implemented (skeleton)".into(),
        ))
    }

    async fn place_order(&self, _order: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::place_order: not implemented (skeleton)".into()))
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::cancel_order: not implemented (skeleton)".into()))
    }

    async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::amend_order: not implemented (skeleton)".into()))
    }

    async fn get_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::get_order: not implemented (skeleton)".into()))
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::get_open_orders: not implemented (skeleton)".into(),
        ))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::get_account_info: not implemented (skeleton)".into(),
        ))
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Err(AdapterError::Internal(
            "OkxAdapter::get_instruments: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order_book(
        &self,
        _instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal("OkxAdapter::get_order_book: not implemented (skeleton)".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okx_adapter_new() {
        let adapter = OkxAdapter::new(None);
        assert_eq!(adapter.base_url, "https://www.okx.com");
    }
}
