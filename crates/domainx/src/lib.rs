//! Domain shared value objects: Order, Position, Trade, Portfolio, and shared enums.
//!
//! This crate provides the L0 shared type layer used across all domain crates.

/// Re-export `rust_decimal::Decimal` for convenience.
pub use rust_decimal::Decimal;

mod validate;
pub use validate::{
    ValidationError, validate_created_before_updated, validate_gtd_deadline,
    validate_non_negative_quantities, validate_order, validate_order_prices,
    validate_quantity_balance,
};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Unique order identifier (exchange-assigned).
pub type OrderId = String;
/// Unique trade identifier (exchange-assigned).
pub type TradeId = String;
/// Unique execution report identifier.
pub type ReportId = String;
/// Unique position identifier.
pub type PositionId = String;
/// Unique portfolio identifier.
pub type PortfolioId = String;
/// Unix timestamp in milliseconds since 1970-01-01 UTC.
pub type Timestamp = i64;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Side of an order or trade.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Type of an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
}

/// Status of an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    PendingNew,
    PendingCancel,
    PendingReplace,
}

/// Time-in-force policy for an order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimeInForce {
    /// Good Till Cancelled.
    Gtc,
    /// Immediate Or Cancel.
    Ioc,
    /// Fill Or Kill.
    Fok,
    /// Good Till Date：截止时间戳为 UTC Unix 毫秒，不得早于订单创建时间。
    Gtd(Timestamp),
}

/// Direction of a position.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionDirection {
    Long,
    Short,
    Flat,
}

/// Status of a position.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionStatus {
    Open,
    Closed,
    Liquidated,
}

/// Execution type for an execution report.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecType {
    New,
    Canceled,
    Replaced,
    Rejected,
    Trade,
    Expired,
    TradeCancel,
    Status,
}

// ---------------------------------------------------------------------------
// Commission
// --------------------------------------------------------------------------

/// Commission details for a trade.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commission {
    /// Commission amount.
    pub amount: Decimal,
    /// Asset in which the commission is denominated.
    pub asset: String,
}

// ---------------------------------------------------------------------------
// Order
// ---------------------------------------------------------------------------

/// A trading order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Exchange-assigned order identifier.
    pub order_id: OrderId,
    /// Instrument 标识（String 兼容占位；canonical 迁移见 DX-CAN-001）。
    pub instrument: String,
    /// Order side (buy / sell).
    pub side: OrderSide,
    /// Order type (market, limit, stop, etc.).
    pub order_type: OrderType,
    /// Current order status.
    pub status: OrderStatus,
    /// Limit price (optional, `None` for market orders).
    pub price: Option<Decimal>,
    /// Stop / trigger price for StopMarket / StopLimit orders.
    pub stop_price: Option<Decimal>,
    /// Ordered quantity.
    pub quantity: Decimal,
    /// Quantity that has been filled.
    pub filled_quantity: Decimal,
    /// Quantity remaining to fill.
    pub remaining_quantity: Decimal,
    /// Average fill price (available after partial / full fill).
    pub avg_fill_price: Option<Decimal>,
    /// Time-in-force policy.
    pub time_in_force: TimeInForce,
    /// Order creation timestamp (Unix ms).
    pub created_at: Timestamp,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
    /// Client-supplied order identifier (optional).
    pub client_order_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

/// A trading position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Position identifier.
    pub position_id: PositionId,
    /// Instrument 标识（String 兼容占位；canonical 迁移见 DX-CAN-001）。
    pub instrument: String,
    /// Position direction (long / short / flat).
    pub direction: PositionDirection,
    /// 持仓状态（可选；历史 fixture 可缺省，DX-COMP-001）。
    #[serde(default)]
    pub status: Option<PositionStatus>,
    /// Position quantity.
    pub quantity: Decimal,
    /// Average entry price.
    pub entry_price: Decimal,
    /// Current market price.
    pub current_price: Decimal,
    /// Unrealised P&L.
    pub unrealized_pnl: Decimal,
    /// Realised P&L.
    pub realized_pnl: Decimal,
    /// Position creation timestamp (Unix ms).
    pub created_at: Timestamp,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Trade
// ---------------------------------------------------------------------------

/// A matched trade (fill).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade identifier.
    pub trade_id: TradeId,
    /// Parent order identifier.
    pub order_id: OrderId,
    /// Instrument key (string placeholder).
    pub instrument: String,
    /// Trade side.
    pub side: OrderSide,
    /// Execution price.
    pub price: Decimal,
    /// Filled quantity.
    pub quantity: Decimal,
    /// Commission charged (optional).
    pub commission: Option<Commission>,
    /// Execution timestamp (Unix ms).
    pub executed_at: Timestamp,
    /// Whether the trade was a maker (`true`), taker (`false`), or unknown (`None`).
    pub is_maker: Option<bool>,
}

// ---------------------------------------------------------------------------
// ExecutionReport
// ---------------------------------------------------------------------------

/// An execution report sent by the exchange in response to order lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReport {
    /// Report identifier.
    pub report_id: ReportId,
    /// Related order identifier.
    pub order_id: OrderId,
    /// Execution type describing the event.
    pub exec_type: ExecType,
    /// New order status after this event.
    pub order_status: OrderStatus,
    /// Instrument key (string placeholder).
    pub instrument: String,
    /// Order side.
    pub side: OrderSide,
    /// Order type.
    pub order_type: OrderType,
    /// Limit price (optional).
    pub price: Option<Decimal>,
    /// Order quantity.
    pub quantity: Decimal,
    /// Price of the last fill (optional).
    pub last_filled_price: Option<Decimal>,
    /// Quantity of the last fill (optional).
    pub last_filled_quantity: Option<Decimal>,
    /// Cumulative filled quantity.
    pub cumulative_filled_quantity: Decimal,
    /// Remaining quantity.
    pub remaining_quantity: Decimal,
    /// Commission charged (optional).
    pub commission: Option<Commission>,
    /// Trade identifier of the last fill (optional).
    pub trade_id: Option<TradeId>,
    /// Reason for rejection (optional).
    pub reject_reason: Option<String>,
    /// Timestamp of this event (Unix ms).
    pub occurred_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

/// A portfolio (collection of positions with aggregate P&L).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Portfolio {
    /// Portfolio identifier.
    pub portfolio_id: PortfolioId,
    /// Account identifier that owns this portfolio.
    pub account_id: String,
    /// Positions held in the portfolio.
    pub positions: Vec<Position>,
    /// Total unrealised P&L across all positions.
    pub total_unrealized_pnl: Decimal,
    /// Total realised P&L across all positions.
    pub total_realized_pnl: Decimal,
    /// Total commission accrued.
    ///
    /// 单资产汇总或调用方约定的主资产合计；多资产明细见 `commissions`（DX-COMP-001）。
    pub total_commission: Decimal,
    /// 按资产拆分的手续费明细（可空；与 `total_commission` 并存）。
    #[serde(default)]
    pub commissions: Vec<Commission>,
    /// Total number of trades executed.
    pub total_trades: u64,
    /// Last update timestamp (Unix ms).
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// DX-COMP-001 helpers
// ---------------------------------------------------------------------------

/// 将多资产手续费按 asset 汇总（同资产 amount 相加）。
pub fn aggregate_commissions_by_asset(items: &[Commission]) -> Vec<Commission> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<&str, Decimal> = BTreeMap::new();
    for c in items {
        *map.entry(c.asset.as_str()).or_insert(Decimal::ZERO) += c.amount;
    }
    map.into_iter().map(|(asset, amount)| Commission { amount, asset: asset.to_string() }).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_order_creation() {
        let order = Order {
            order_id: "abc123".into(),
            instrument: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            status: OrderStatus::New,
            price: Some(Decimal::new(50000, 0)),
            stop_price: None,
            quantity: Decimal::new(1, 0),
            filled_quantity: Decimal::ZERO,
            remaining_quantity: Decimal::new(1, 0),
            avg_fill_price: None,
            time_in_force: TimeInForce::Gtc,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_000_000,
            client_order_id: Some("my-order".into()),
        };
        assert_eq!(order.order_id, "abc123");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.status, OrderStatus::New);
    }

    #[test]
    fn test_order_serialization() {
        let order = Order {
            order_id: "abc".into(),
            instrument: "BTCUSDT".into(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            status: OrderStatus::Filled,
            price: None,
            stop_price: None,
            quantity: Decimal::new(2, 0),
            filled_quantity: Decimal::new(2, 0),
            remaining_quantity: Decimal::ZERO,
            avg_fill_price: Some(Decimal::new(49000, 0)),
            time_in_force: TimeInForce::Ioc,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_001_000,
            client_order_id: None,
        };
        let json = serde_json::to_string(&order).expect("serialize order");
        let deserialized: Order = serde_json::from_str(&json).expect("deserialize order");
        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_position_lifecycle() {
        let pos = Position {
            position_id: "pos1".into(),
            instrument: "ETHUSDT".into(),
            direction: PositionDirection::Long,
            status: None,
            quantity: Decimal::new(10, 0),
            entry_price: Decimal::new(3000, 0),
            current_price: Decimal::new(3100, 0),
            unrealized_pnl: Decimal::new(1000, 0),
            realized_pnl: Decimal::ZERO,
            created_at: 1_000_000_000_000,
            updated_at: 1_000_000_001_000,
        };
        assert_eq!(pos.direction, PositionDirection::Long);
        assert!(pos.unrealized_pnl > Decimal::ZERO);
    }

    #[test]
    fn test_trade_with_commission() {
        let trade = Trade {
            trade_id: "t1".into(),
            order_id: "o1".into(),
            instrument: "BTCUSDT".into(),
            side: OrderSide::Buy,
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            commission: Some(Commission { amount: Decimal::new(10, 0), asset: "BTC".into() }),
            executed_at: 1_000_000_000_000,
            is_maker: Some(true),
        };
        let json = serde_json::to_string(&trade).expect("serialize trade");
        let deserialized: Trade = serde_json::from_str(&json).expect("deserialize trade");
        assert_eq!(trade, deserialized);
    }

    #[test]
    fn test_time_in_force_gtd() {
        let tif = TimeInForce::Gtd(1_700_000_000_000);
        let json = serde_json::to_string(&tif).expect("serialize tif");
        let deserialized: TimeInForce = serde_json::from_str(&json).expect("deserialize tif");
        assert_eq!(tif, deserialized);
    }

    #[test]
    fn test_execution_report() {
        let report = ExecutionReport {
            report_id: "r1".into(),
            order_id: "o1".into(),
            exec_type: ExecType::Trade,
            order_status: OrderStatus::PartiallyFilled,
            instrument: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(Decimal::new(50000, 0)),
            quantity: Decimal::new(1, 0),
            last_filled_price: Some(Decimal::new(50000, 0)),
            last_filled_quantity: Some(Decimal::new(5, 1)),
            cumulative_filled_quantity: Decimal::new(5, 1),
            remaining_quantity: Decimal::new(5, 1),
            commission: Some(Commission { amount: Decimal::new(1, 2), asset: "BNB".into() }),
            trade_id: Some("t1".into()),
            reject_reason: None,
            occurred_at: 1_000_000_000_000,
        };
        let json = serde_json::to_string(&report).expect("serialize report");
        let deserialized: ExecutionReport =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(report, deserialized);
    }

    #[test]
    fn test_portfolio() {
        let portfolio = Portfolio {
            portfolio_id: "pf1".into(),
            account_id: "acc1".into(),
            positions: vec![Position {
                position_id: "pos1".into(),
                instrument: "BTCUSDT".into(),
                direction: PositionDirection::Long,
                status: None,
                quantity: Decimal::new(1, 0),
                entry_price: Decimal::new(50000, 0),
                current_price: Decimal::new(51000, 0),
                unrealized_pnl: Decimal::new(1000, 0),
                realized_pnl: Decimal::ZERO,
                created_at: 1_000_000_000_000,
                updated_at: 1_000_000_001_000,
            }],
            total_unrealized_pnl: Decimal::new(1000, 0),
            total_realized_pnl: Decimal::ZERO,
            total_commission: Decimal::new(50, 0),
            commissions: vec![Commission { amount: Decimal::new(50, 0), asset: "USDT".into() }],
            total_trades: 5,
            updated_at: 1_000_000_001_000,
        };
        assert_eq!(portfolio.positions.len(), 1);
    }

    #[test]
    fn test_enum_non_exhaustive_match() {
        // Verify that all variants can be matched (ensuring #[non_exhaustive] is compatible)
        let side = OrderSide::Buy;
        match side {
            OrderSide::Buy | OrderSide::Sell => {}
        }
        let status = OrderStatus::New;
        match status {
            OrderStatus::New
            | OrderStatus::PartiallyFilled
            | OrderStatus::Filled
            | OrderStatus::Canceled
            | OrderStatus::Rejected
            | OrderStatus::Expired
            | OrderStatus::PendingNew
            | OrderStatus::PendingCancel
            | OrderStatus::PendingReplace => {}
        }
    }
}
