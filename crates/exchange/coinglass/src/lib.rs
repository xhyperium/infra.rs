#![forbid(unsafe_code)]
#![allow(dead_code)]

//! # exchange_coinglass — Coinglass Exchange Adapter
//!
//! Coinglass 数据聚合平台适配器，提供跨交易所的未平仓合约、资金费率、爆仓数据、多空比等聚合指标。
//!
//! 规格 SSOT：`.agents/ssot/market_data/coinglass/spec/spec.md`
//!
//! **能力边界（DE-REST-001）**：仅公开 REST 聚合数据；不支持 WebSocket 行情、账户与交易。
//! 不适用方法必须返回 [`AdapterError::Unsupported`]，不得伪装为 `Network` / `Internal`。

use async_trait::async_trait;
use domain_exchange::{
    AccountInfo, AdapterError, ExecutionReport, InstrumentMeta, OrderAmend, VenueAdapter,
};
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Order, OrderId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ──────────────────────────────────────────────
//  Coinglass REST API 类型（§2、§3）
// ──────────────────────────────────────────────

/// Coinglass 通用 REST 响应包装
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinglassResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

// ──────────────────────────────────────────────
//  连接管理（§5）
// ──────────────────────────────────────────────

/// 限频配置
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self { max_requests_per_minute: 30, burst_size: 5 }
    }
}

/// Coinglass 适配器配置
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CoinglassConfig {
    pub rest_base_url: String,
    pub api_key: Option<String>,
    pub rate_limit: RateLimitConfig,
    pub request_timeout: Duration,
}

impl Default for CoinglassConfig {
    fn default() -> Self {
        Self {
            rest_base_url: "https://open-api-v4.coinglass.com".into(),
            api_key: None,
            rate_limit: RateLimitConfig::default(),
            request_timeout: Duration::from_secs(10),
        }
    }
}

// ──────────────────────────────────────────────
//  VenueAdapter 实现
// ──────────────────────────────────────────────

/// Coinglass 适配器（仅 REST API，无 WebSocket / 交易支持）
#[derive(Debug)]
pub struct CoinglassAdapter {
    base_url: String,
    api_key: Option<String>,
}

impl CoinglassAdapter {
    /// 默认 V4 base URL（CG-URL-001 / SSOT §1）。
    pub fn new(api_key: Option<String>) -> Self {
        Self::from_config(CoinglassConfig { api_key, ..CoinglassConfig::default() })
    }

    /// 从配置构造；禁止在 adapter 内硬编码第二套 base URL。
    pub fn from_config(config: CoinglassConfig) -> Self {
        Self { base_url: config.rest_base_url, api_key: config.api_key }
    }

    pub fn rest_base_url(&self) -> &str {
        &self.base_url
    }

    fn unsupported(op: &str) -> AdapterError {
        AdapterError::Unsupported(format!("REST-only Coinglass 不支持 {op}（DE-REST-001）"))
    }
}

#[async_trait]
impl VenueAdapter for CoinglassAdapter {
    fn exchange_id(&self) -> &str {
        "coinglass"
    }

    fn capabilities(&self) -> domain_exchange::VenueCapabilities {
        domain_exchange::VenueCapabilities::rest_only_public()
    }

    /// REST 无会话：connect 幂等成功（骨架阶段不发起真实 HTTP）。
    async fn connect(&self) -> Result<(), AdapterError> {
        Ok(())
    }

    /// REST 无会话：disconnect 幂等成功。
    async fn disconnect(&self) -> Result<(), AdapterError> {
        Ok(())
    }

    async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(Self::unsupported("WebSocket ticker 订阅"))
    }

    async fn subscribe_order_book(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(Self::unsupported("WebSocket order book 订阅"))
    }

    async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
        Err(Self::unsupported("WebSocket trades 订阅"))
    }

    async fn place_order(&self, _order: &Order) -> Result<ExecutionReport, AdapterError> {
        Err(Self::unsupported("下单"))
    }

    async fn cancel_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<(), AdapterError> {
        Err(Self::unsupported("撤单"))
    }

    async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
        Err(Self::unsupported("改单"))
    }

    async fn get_order(
        &self,
        _order_id: &OrderId,
        _instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError> {
        Err(Self::unsupported("订单查询"))
    }

    async fn get_open_orders(
        &self,
        _instrument: &InstrumentKey,
    ) -> Result<Vec<Order>, AdapterError> {
        Err(Self::unsupported("未结订单查询"))
    }

    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
        Err(Self::unsupported("账户信息"))
    }

    /// 公开 REST 元数据路径：能力上允许，实现仍为骨架。
    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
        Err(AdapterError::Internal(
            "CoinglassAdapter::get_instruments: REST 映射尚未实现（skeleton）".into(),
        ))
    }

    /// 公开 REST 深度：能力上允许，实现仍为骨架。
    async fn get_order_book(
        &self,
        _instrument: &InstrumentKey,
        _limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError> {
        Err(AdapterError::Internal(
            "CoinglassAdapter::get_order_book: REST 映射尚未实现（skeleton）".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domainx::{Decimal, OrderSide, OrderStatus, OrderType, TimeInForce};

    fn sample_instrument() -> InstrumentKey {
        InstrumentKey { exchange: "coinglass".into(), symbol: "BTC".into() }
    }

    fn sample_order() -> Order {
        Order {
            order_id: "o1".into(),
            instrument: "BTC".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            status: OrderStatus::New,
            price: Some(Decimal::new(1, 0)),
            stop_price: None,
            quantity: Decimal::new(1, 0),
            filled_quantity: Decimal::ZERO,
            remaining_quantity: Decimal::new(1, 0),
            avg_fill_price: None,
            time_in_force: TimeInForce::Gtc,
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
            client_order_id: None,
        }
    }

    #[test]
    fn test_coinglass_adapter_new_uses_v4_base_url() {
        let adapter = CoinglassAdapter::new(None);
        assert_eq!(adapter.rest_base_url(), "https://open-api-v4.coinglass.com");

        let cfg = CoinglassConfig::default();
        assert_eq!(cfg.rest_base_url, "https://open-api-v4.coinglass.com");
        assert_eq!(CoinglassAdapter::from_config(cfg).rest_base_url(), adapter.rest_base_url());
        // 不得回落到旧 V3 host
        assert_ne!(adapter.rest_base_url(), "https://open-api.coinglass.com");
    }

    #[tokio::test]
    async fn rest_only_connect_disconnect_ok() {
        let adapter = CoinglassAdapter::new(None);
        adapter.connect().await.expect("connect");
        adapter.disconnect().await.expect("disconnect");
    }

    #[tokio::test]
    async fn rest_only_ws_and_trading_return_unsupported() {
        let adapter = CoinglassAdapter::new(None);
        let ik = sample_instrument();
        adapter.connect().await.unwrap();

        let cases: Vec<(&str, Result<(), AdapterError>)> = vec![
            ("ticker", adapter.subscribe_ticker(&ik).await.map(|_| ())),
            ("book_ws", adapter.subscribe_order_book(&ik).await.map(|_| ())),
            ("trades", adapter.subscribe_trades(&ik).await.map(|_| ())),
            ("place", adapter.place_order(&sample_order()).await.map(|_| ())),
            ("cancel", adapter.cancel_order(&"o1".into(), &ik).await),
            (
                "amend",
                adapter
                    .amend_order(&OrderAmend {
                        order_id: "o1".into(),
                        price: Some(Decimal::new(1, 0)),
                        quantity: None,
                        stop_price: None,
                        new_client_order_id: None,
                    })
                    .await
                    .map(|_| ()),
            ),
            ("get_order", adapter.get_order(&"o1".into(), &ik).await.map(|_| ())),
            ("open_orders", adapter.get_open_orders(&ik).await.map(|_| ())),
            ("account", adapter.get_account_info().await.map(|_| ())),
        ];

        for (label, result) in cases {
            let err = result.expect_err(label);
            let s = err.to_string();
            assert!(s.starts_with("Unsupported:"), "{label}: expected Unsupported, got {s}");
            assert!(matches!(err, AdapterError::Unsupported(_)), "{label}: {err}");
            assert!(
                !matches!(
                    err,
                    AdapterError::Network(_)
                        | AdapterError::Internal(_)
                        | AdapterError::WebSocket(_)
                ),
                "{label}: must not masquerade"
            );
        }
    }

    #[tokio::test]
    async fn rest_capable_paths_remain_internal_skeleton() {
        let adapter = CoinglassAdapter::new(None);
        let ik = sample_instrument();
        let err = adapter.get_instruments().await.expect_err("instruments");
        assert!(matches!(err, AdapterError::Internal(_)), "{err}");
        let err = adapter.get_order_book(&ik, Some(5)).await.expect_err("book");
        assert!(matches!(err, AdapterError::Internal(_)), "{err}");
    }
}
