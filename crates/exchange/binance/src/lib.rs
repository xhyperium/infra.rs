#![forbid(unsafe_code)]
#![allow(dead_code)]

//! # exchange_binance — Binance Exchange Adapter
//!
//! Binance 交易所适配器，覆盖 Spot、USDⓈ-M Futures、COIN-M Futures、Options 四大产品线。
//!
//! 规格 SSOT：`.agents/ssot/market_data/binance/spec/spec.md`

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, ExecutionReport, InstrumentMeta, OrderAmend, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Order, OrderId};
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
//  Binance WebSocket Stream 类型（§2.1）
// ──────────────────────────────────────────────

/// Binance WebSocket 可订阅的数据流
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinanceStream {
    /// 逐笔成交
    Trade { symbol: String },
    /// 最优买卖价
    BookTicker { symbol: String },
    /// 深度数据
    Depth { symbol: String, levels: DepthLevel, speed: DepthSpeed },
    /// 深度增量
    DiffDepth { symbol: String, speed: DepthSpeed },
    /// K 线
    Kline { symbol: String, interval: KlineInterval },
    /// 24hr 统计
    Ticker24hr { symbol: String },
    /// 逐笔买卖压（所有交易对）
    BookTickerAll,
    /// 部分深度（5/10/20 档）
    PartialDepth { symbol: String, levels: DepthLevel },
}

/// 深度档位级别
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthLevel {
    Level5,
    Level10,
    Level20,
}

/// 深度推送速度
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthSpeed {
    Ms100,
    Ms500,
    Ms1000,
}

/// K 线时间间隔
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KlineInterval {
    M1,
    M3,
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
    Mo1,
}

// ──────────────────────────────────────────────
//  Binance REST API 响应类型（§3.1）
// ──────────────────────────────────────────────

/// 交易所信息响应
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}

/// 交易对信息
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: u32,
    pub quote_precision: u32,
}

/// K 线原始嬶斤应（Binance 返回的二维数组）
pub type KlineRawResponse = Vec<serde_json::Value>;

// ──────────────────────────────────────────────
//  连接管理（§4）
// ──────────────────────────────────────────────

/// Binance 适配器配置
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BinanceConfig {
    pub product_line: domain_market::ProductLine,
    pub rest_base_url: String,
    pub ws_base_url: String,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub reconnect_max_attempts: u32,
    pub reconnect_base_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
    pub ping_interval_secs: u64,
}

impl BinanceConfig {
    /// 按 [`domain_market::ProductLine`] 派生官方 REST/WS 基址（BN-ROUTE-001）。
    ///
    /// 路由表以 SSOT `.agents/ssot/binance/spec/spec.md` §2 与 design 端点主机为准。
    pub fn for_product_line(
        product_line: domain_market::ProductLine,
    ) -> Result<Self, AdapterError> {
        let (rest_base_url, ws_base_url) = endpoints_for_product_line(&product_line)?;
        Ok(Self {
            product_line,
            rest_base_url: rest_base_url.into(),
            ws_base_url: ws_base_url.into(),
            api_key: None,
            secret_key: None,
            reconnect_max_attempts: 5,
            reconnect_base_delay_ms: 1000,
            reconnect_max_delay_ms: 60000,
            ping_interval_secs: 180,
        })
    }
}

impl Default for BinanceConfig {
    fn default() -> Self {
        // PANIC: Spot 是规格内已知产品线，派生失败只可能是 enum 扩展未同步
        Self::for_product_line(domain_market::ProductLine::Spot).expect("Spot 产品线路由必须可派生")
    }
}

/// 产品线 → (REST base, WS base)。未知变体拒绝猜测。
fn endpoints_for_product_line(
    product_line: &domain_market::ProductLine,
) -> Result<(&'static str, &'static str), AdapterError> {
    use domain_market::ProductLine;
    match product_line {
        ProductLine::Spot => Ok(("https://api.binance.com", "wss://stream.binance.com:9443/ws")),
        // USDⓈ-M Futures
        ProductLine::Future => Ok(("https://fapi.binance.com", "wss://fstream.binance.com/ws")),
        // COIN-M 兼容映射（SSOT 标注需后续裁决 settlement 维度）
        ProductLine::Perpetual => Ok(("https://dapi.binance.com", "wss://dstream.binance.com/ws")),
        ProductLine::Option => {
            Ok(("https://eapi.binance.com", "wss://nbstream.binance.com/eoptions/ws"))
        }
        other => {
            Err(AdapterError::InvalidRequest(format!("Binance 尚不支持产品线路由: {other:?}")))
        }
    }
}

/// Binance 连接状态
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BinanceConnection {
    pub product_line: domain_market::ProductLine,
    pub listen_key: Option<String>,
    pub connected_at: Option<domainx::Timestamp>,
    pub subscriptions: Vec<BinanceStream>,
}

// ──────────────────────────────────────────────
//  VenueAdapter 实现
// ──────────────────────────────────────────────

/// Binance 交易所适配器
#[derive(Debug)]
pub struct BinanceAdapter {
    product_line: domain_market::ProductLine,
    base_url: String,
    ws_url: String,
    api_key: Option<String>,
}

impl BinanceAdapter {
    /// 默认 Spot 产品线；地址必须与 [`BinanceConfig::for_product_line`] 同源派生。
    pub fn new(api_key: Option<String>) -> Self {
        Self::from_config(BinanceConfig { api_key, ..BinanceConfig::default() })
    }

    /// 从不可变配置构造；禁止在 adapter 内再硬编码第二套地址。
    pub fn from_config(config: BinanceConfig) -> Self {
        Self {
            product_line: config.product_line,
            base_url: config.rest_base_url,
            ws_url: config.ws_base_url,
            api_key: config.api_key,
        }
    }

    /// 当前产品线（用于路由审计与后续实现）。
    pub fn product_line(&self) -> &domain_market::ProductLine {
        &self.product_line
    }

    /// REST base URL（与 config 一致）。
    pub fn rest_base_url(&self) -> &str {
        &self.base_url
    }

    /// WebSocket base URL（与 config 一致）。
    pub fn ws_base_url(&self) -> &str {
        &self.ws_url
    }
}

#[async_trait]
impl VenueAdapter for BinanceAdapter {
    async fn connect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("BinanceAdapter::connect: not implemented (skeleton)".into()))
    }

    async fn disconnect(&self) -> Result<(), AdapterError> {
        Err(AdapterError::Internal("BinanceAdapter::disconnect: not implemented (skeleton)".into()))
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::subscribe_ticker: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::subscribe_order_book: not implemented (skeleton)".into(),
        ))
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::subscribe_trades: not implemented (skeleton)".into(),
        ))
    }

    async fn place_order(&self, _order: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::place_order: not implemented (skeleton)".into(),
        ))
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::cancel_order: not implemented (skeleton)".into(),
        ))
    }

    async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::amend_order: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        Err(AdapterError::Internal("BinanceAdapter::get_order: not implemented (skeleton)".into()))
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::get_open_orders: not implemented (skeleton)".into(),
        ))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::get_account_info: not implemented (skeleton)".into(),
        ))
    }

    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::get_instruments: not implemented (skeleton)".into(),
        ))
    }

    async fn get_order_book(
        &self,
        _instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal(
            "BinanceAdapter::get_order_book: not implemented (skeleton)".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_market::ProductLine;

    #[test]
    fn test_binance_adapter_new_uses_spot_config_endpoints() {
        let adapter = BinanceAdapter::new(None);
        let cfg = BinanceConfig::for_product_line(ProductLine::Spot).expect("spot");
        assert_eq!(adapter.product_line(), &ProductLine::Spot);
        assert_eq!(adapter.rest_base_url(), cfg.rest_base_url);
        assert_eq!(adapter.ws_base_url(), cfg.ws_base_url);
        assert_eq!(adapter.rest_base_url(), "https://api.binance.com");
        assert_eq!(adapter.ws_base_url(), "wss://stream.binance.com:9443/ws");
    }

    #[test]
    fn bn_route_001_product_line_endpoints_from_config_only() {
        let cases = [
            (ProductLine::Spot, "https://api.binance.com", "wss://stream.binance.com:9443/ws"),
            (ProductLine::Future, "https://fapi.binance.com", "wss://fstream.binance.com/ws"),
            (ProductLine::Perpetual, "https://dapi.binance.com", "wss://dstream.binance.com/ws"),
            (
                ProductLine::Option,
                "https://eapi.binance.com",
                "wss://nbstream.binance.com/eoptions/ws",
            ),
        ];

        for (line, rest, ws) in cases {
            let cfg = BinanceConfig::for_product_line(line.clone()).expect("route");
            assert_eq!(cfg.product_line, line);
            assert_eq!(cfg.rest_base_url, rest, "{line:?} rest");
            assert_eq!(cfg.ws_base_url, ws, "{line:?} ws");

            let adapter = BinanceAdapter::from_config(cfg.clone());
            assert_eq!(adapter.product_line(), &line);
            assert_eq!(adapter.rest_base_url(), rest);
            assert_eq!(adapter.ws_base_url(), ws);
            // new(api_key) 不得引入与 config 不同的第二套 Spot 地址
            if line == ProductLine::Spot {
                let via_new = BinanceAdapter::new(Some("k".into()));
                assert_eq!(via_new.rest_base_url(), adapter.rest_base_url());
                assert_eq!(via_new.ws_base_url(), adapter.ws_base_url());
            }
        }
    }

    #[test]
    fn default_config_matches_spot_route() {
        let d = BinanceConfig::default();
        let s = BinanceConfig::for_product_line(ProductLine::Spot).unwrap();
        assert_eq!(d.product_line, s.product_line);
        assert_eq!(d.rest_base_url, s.rest_base_url);
        assert_eq!(d.ws_base_url, s.ws_base_url);
    }
}
