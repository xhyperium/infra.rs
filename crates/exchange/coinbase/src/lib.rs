#![forbid(unsafe_code)]
#![allow(dead_code)]

//! # exchange_coinbase — Coinbase Exchange Adapter
//!
//! Coinbase 行情行情— Spot 市场行情优化ݡ★
//!
//! — SSOT: `.agents/ssot/market_data/coinbase/spec/spec.md`

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, ExecutionReport, InstrumentMeta, OrderAmend, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Order, OrderId};
use serde::{Deserialize, Serialize};

// Coinbase WebSocket channel types (SSOT §2.1)

/// Coinbase WebSocket 优化Ƶ碉斤
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoinbaseChannel {
    /// 行情Ƶ碉斤
    Heartbeats,
    /// K 优化—Ƶ行情优化栵֣ｏ
    Candles { granularity: CandleGranularity },
    /// ʵʱ Ticker行情—斤ۣｏ
    Ticker,
    /// Level 2 优化—斤
    Level2,
}

/// K 优化—
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CandleGranularity {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    SixHours,
    OneDay,
}

/// Coinbase WebSocket 行情
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CoinbaseSubscription {
    pub channel: CoinbaseChannel,
    pub product_ids: Vec<String>,
}

/// Coinbase 认证 — API Key + Secret 签名
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoinbaseAuthenticatedChannel {
    /// 用户认证与状态管理
    User,
}

// Coinbase REST API response types (SSOT §3)

/// 资产优化
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub product_id: String,
    pub base_currency_id: String,
    pub quote_currency_id: String,
    pub base_min_size: String,
    pub base_max_size: String,
    pub quote_increment: String,
    pub price_increment: String,
    pub status: String,
}

// Connection management (SSOT §4)

/// Coinbase 行情—
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CoinbaseConfig {
    pub rest_base_url: String,
    pub ws_base_url: String,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub reconnect_max_attempts: u32,
    pub reconnect_base_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
}

impl Default for CoinbaseConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://api.coinbase.com".into(),
            ws_base_url: "wss://advanced-trade-ws.coinbase.com".into(),
            api_key: None,
            secret_key: None,
            reconnect_max_attempts: 5,
            reconnect_base_delay_ms: 1000,
            reconnect_max_delay_ms: 60000,
        }
    }
}

/// Coinbase 行情—״̬
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CoinbaseConnection {
    pub subscriptions: Vec<CoinbaseChannel>,
    pub connected_at: Option<domainx::Timestamp>,
}

// VenueAdapter implementation

/// Coinbase 行情优化
#[derive(Debug)]
pub struct CoinbaseAdapter {
    base_url: String,
    ws_url: String,
    api_key: Option<String>,
}

impl CoinbaseAdapter {
    /// 使用 [`CoinbaseConfig::default`] 的 Advanced Trade 端点（CB-URL-001）。
    pub fn new(api_key: Option<String>) -> Self {
        Self::from_config(CoinbaseConfig { api_key, ..CoinbaseConfig::default() })
    }

    /// 从配置构造；禁止在 adapter 内硬编码第二套 endpoint。
    pub fn from_config(config: CoinbaseConfig) -> Self {
        Self { base_url: config.rest_base_url, ws_url: config.ws_base_url, api_key: config.api_key }
    }

    pub fn rest_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn ws_base_url(&self) -> &str {
        &self.ws_url
    }
}

#[async_trait]
impl VenueAdapter for CoinbaseAdapter {
    async fn connect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("CoinbaseAdapter::connect: not implemented (skeleton)".into()))
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::disconnect: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::subscribe_ticker: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::subscribe_order_book: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::subscribe_trades: not implemented (skeleton)".into(),
        ))
    }

    async fn place_order(&self, _order: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::place_order: not implemented (skeleton)".into(),
        ))
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::cancel_order: not implemented (skeleton)".into(),
        ))
    }

    async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::amend_order: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        Err(AdapterError::Internal("CoinbaseAdapter::get_order: not implemented (skeleton)".into()))
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::get_open_orders: not implemented (skeleton)".into(),
        ))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::get_account_info: not implemented (skeleton)".into(),
        ))
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::get_instruments: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order_book(
        &self,
        _instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal(
            "CoinbaseAdapter::get_order_book: not implemented (skeleton)".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_adapter_new() {
        let adapter = CoinbaseAdapter::new(None);
        assert_eq!(adapter.rest_base_url(), "https://api.coinbase.com");
    }

    #[test]
    fn cb_url_001_config_and_adapter_share_advanced_trade_endpoints() {
        let cfg = CoinbaseConfig::default();
        assert_eq!(cfg.ws_base_url, "wss://advanced-trade-ws.coinbase.com");
        assert_eq!(cfg.rest_base_url, "https://api.coinbase.com");

        let via_new = CoinbaseAdapter::new(Some("k".into()));
        assert_eq!(via_new.rest_base_url(), cfg.rest_base_url);
        assert_eq!(via_new.ws_base_url(), cfg.ws_base_url);
        // 不得再回落到旧 Exchange feed
        assert_ne!(via_new.ws_base_url(), "wss://ws-feed.exchange.coinbase.com");

        let via_cfg = CoinbaseAdapter::from_config(cfg.clone());
        assert_eq!(via_cfg.rest_base_url(), via_new.rest_base_url());
        assert_eq!(via_cfg.ws_base_url(), via_new.ws_base_url());
    }
}
