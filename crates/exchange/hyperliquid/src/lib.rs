#![forbid(unsafe_code)]
#![allow(dead_code)]

//! # exchange_hyperliquid — Hyperliquid Exchange Adapter
//!
//! Hyperliquid 交易所适配器，覆盖永续合约（Perpetual Futures）市场数据。
//!
//! 规格 SSOT：`.agents/ssot/market_data/hyperliquid/spec/spec.md`

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, ExecutionReport, InstrumentMeta, OrderAmend, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Order, OrderId};
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
//  Hyperliquid WebSocket 频道类型（§2.1）
// ──────────────────────────────────────────────

/// Hyperliquid WebSocket 订阅频道
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HyperliquidStream {
    /// 所有夛斤斤种的中间价（→ Quote）
    AllMids,
    /// L2 深度快照 + 增量（→ OrderBook）
    L2Book { coin: String },
    /// 近期逐笔成交（→ Tick）
    Trades { coin: String },
    /// WebSocket L2 深度快照 + 更新（→ OrderBook，webbook2 格式）
    WebBook2 { coin: String },
}

// ──────────────────────────────────────────────
//  Hyperliquid REST API 类型（§3）
// ──────────────────────────────────────────────

/// REST Info 请求类型
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoRequestBody {
    #[serde(rename = "type")]
    pub request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req: Option<CandleSnapshotReq>,
}

/// K 线快照请求参数
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleSnapshotReq {
    pub coin: String,
    pub interval: String,
    pub start_time: u64,
    pub end_time: u64,
}

/// REST Info 请求类型枚举
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum InfoRequest {
    AllMids,
    L2Book { coin: String },
    CandleSnapshot { coin: String, interval: CandleInterval, start_time: u64, end_time: u64 },
    RecentTrades { coin: String },
    Meta,
}

/// K 线时间间隔
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CandleInterval {
    M1,
    M5,
    M15,
    M30,
    H1,
    H2,
    H4,
    H6,
    H8,
    H12,
    D1,
    W1,
}

/// 交易对元数据响应
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseInfo {
    pub universe: Vec<CoinMeta>,
}

/// 币种元数据
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinMeta {
    pub name: String,
    pub sz_decimals: u32,
}

// ──────────────────────────────────────────────
//  连接管理（§4）
// ──────────────────────────────────────────────

/// Hyperliquid 适配器配置
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HyperliquidConfig {
    pub rest_base_url: String,
    pub ws_url: String,
    pub reconnect_max_attempts: u32,
    pub reconnect_base_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
}

impl Default for HyperliquidConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://api.hyperliquid.xyz".into(),
            ws_url: "wss://api.hyperliquid.xyz/ws".into(),
            reconnect_max_attempts: 5,
            reconnect_base_delay_ms: 1000,
            reconnect_max_delay_ms: 60000,
        }
    }
}

/// Hyperliquid 连接状态
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HyperliquidConnection {
    pub subscriptions: Vec<HyperliquidStream>,
    pub connected_at: Option<domainx::Timestamp>,
}

// ──────────────────────────────────────────────
//  VenueAdapter 实现
// ──────────────────────────────────────────────

/// Hyperliquid 交易所适配器
#[derive(Debug)]
pub struct HyperliquidAdapter {
    base_url: String,
    ws_url: String,
}

impl HyperliquidAdapter {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.hyperliquid.xyz".into(),
            ws_url: "wss://api.hyperliquid.xyz/ws".into(),
        }
    }
}

impl Default for HyperliquidAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VenueAdapter for HyperliquidAdapter {
    async fn connect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::connect: not implemented (skeleton)".into(),
        ))
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::disconnect: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::subscribe_ticker: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::subscribe_order_book: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::subscribe_trades: not implemented (skeleton)".into(),
        ))
    }

    async fn place_order(&self, _order: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::place_order: not implemented (skeleton)".into(),
        ))
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::cancel_order: not implemented (skeleton)".into(),
        ))
    }

    async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::amend_order: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::get_order: not implemented (skeleton)".into(),
        ))
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::get_open_orders: not implemented (skeleton)".into(),
        ))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::get_account_info: not implemented (skeleton)".into(),
        ))
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::get_instruments: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order_book(
        &self,
        _instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal(
            "HyperliquidAdapter::get_order_book: not implemented (skeleton)".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperliquid_adapter_new() {
        let adapter = HyperliquidAdapter::new();
        assert_eq!(adapter.base_url, "https://api.hyperliquid.xyz");
    }
}
