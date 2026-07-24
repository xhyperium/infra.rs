//! 订单与共享值对象的纯校验逻辑（DX-VAL-001..005）。
//!
//! 本模块不执行下单或网络请求；仅检查字段不变量，供 adapter/service 在写入前调用。

use crate::{Decimal, Order, OrderType, TimeInForce, Timestamp};
use thiserror::Error;

/// 值对象校验失败原因。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// 数量字段为负或关系不成立。
    #[error("数量不变量违反: {0}")]
    Quantity(String),
    /// 价格/止损价与订单类型不匹配。
    #[error("价格字段不变量违反: {0}")]
    Price(String),
    /// 时间戳顺序或 GTD 截止不合法。
    #[error("时间不变量违反: {0}")]
    Time(String),
}

/// DX-VAL-001：quantity / filled / remaining 均不得为负。
pub fn validate_non_negative_quantities(
    quantity: Decimal,
    filled_quantity: Decimal,
    remaining_quantity: Decimal,
) -> Result<(), ValidationError> {
    if quantity < Decimal::ZERO {
        return Err(ValidationError::Quantity(format!("quantity 不能为负: {quantity}")));
    }
    if filled_quantity < Decimal::ZERO {
        return Err(ValidationError::Quantity(format!(
            "filled_quantity 不能为负: {filled_quantity}"
        )));
    }
    if remaining_quantity < Decimal::ZERO {
        return Err(ValidationError::Quantity(format!(
            "remaining_quantity 不能为负: {remaining_quantity}"
        )));
    }
    Ok(())
}

/// DX-VAL-002：filled + remaining == quantity。
pub fn validate_quantity_balance(
    quantity: Decimal,
    filled_quantity: Decimal,
    remaining_quantity: Decimal,
) -> Result<(), ValidationError> {
    let sum = filled_quantity + remaining_quantity;
    if sum != quantity {
        return Err(ValidationError::Quantity(format!(
            "filled_quantity({filled_quantity}) + remaining_quantity({remaining_quantity}) \
             != quantity({quantity})"
        )));
    }
    Ok(())
}

/// DX-VAL-003：price / stop_price 仅在对应订单类型允许时出现。
///
/// | 类型 | price | stop_price |
/// |------|-------|------------|
/// | Market | 必须 None | 必须 None |
/// | Limit | 必须 Some | 必须 None |
/// | StopMarket | 必须 None | 必须 Some |
/// | StopLimit | 必须 Some | 必须 Some |
pub fn validate_order_prices(
    order_type: &OrderType,
    price: Option<Decimal>,
    stop_price: Option<Decimal>,
) -> Result<(), ValidationError> {
    match order_type {
        OrderType::Market => {
            if price.is_some() {
                return Err(ValidationError::Price("Market 订单不允许 price".into()));
            }
            if stop_price.is_some() {
                return Err(ValidationError::Price("Market 订单不允许 stop_price".into()));
            }
        }
        OrderType::Limit => {
            if price.is_none() {
                return Err(ValidationError::Price("Limit 订单必须提供 price".into()));
            }
            if stop_price.is_some() {
                return Err(ValidationError::Price("Limit 订单不允许 stop_price".into()));
            }
        }
        OrderType::StopMarket => {
            if price.is_some() {
                return Err(ValidationError::Price("StopMarket 订单不允许 price".into()));
            }
            if stop_price.is_none() {
                return Err(ValidationError::Price("StopMarket 订单必须提供 stop_price".into()));
            }
        }
        OrderType::StopLimit => {
            if price.is_none() {
                return Err(ValidationError::Price("StopLimit 订单必须提供 price".into()));
            }
            if stop_price.is_none() {
                return Err(ValidationError::Price("StopLimit 订单必须提供 stop_price".into()));
            }
        } // 定义 crate 内 non_exhaustive 仍穷尽；新增变体时编译器会要求补充分支。
    }
    Ok(())
}

/// DX-VAL-004：created_at <= updated_at。
pub fn validate_created_before_updated(
    created_at: Timestamp,
    updated_at: Timestamp,
) -> Result<(), ValidationError> {
    if created_at > updated_at {
        return Err(ValidationError::Time(format!(
            "created_at({created_at}) 晚于 updated_at({updated_at})"
        )));
    }
    Ok(())
}

/// DX-VAL-005：Gtd 截止时间不得早于创建时间。
pub fn validate_gtd_deadline(
    time_in_force: &TimeInForce,
    created_at: Timestamp,
) -> Result<(), ValidationError> {
    if let TimeInForce::Gtd(deadline) = time_in_force {
        if *deadline < created_at {
            return Err(ValidationError::Time(format!(
                "Gtd 截止({deadline}) 早于 created_at({created_at})"
            )));
        }
    }
    Ok(())
}

/// 对完整 `Order` 运行 DX-VAL-001..005。
pub fn validate_order(order: &Order) -> Result<(), ValidationError> {
    validate_non_negative_quantities(
        order.quantity,
        order.filled_quantity,
        order.remaining_quantity,
    )?;
    validate_quantity_balance(order.quantity, order.filled_quantity, order.remaining_quantity)?;
    validate_order_prices(&order.order_type, order.price, order.stop_price)?;
    validate_created_before_updated(order.created_at, order.updated_at)?;
    validate_gtd_deadline(&order.time_in_force, order.created_at)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OrderSide, OrderStatus, TimeInForce};

    fn base_order() -> Order {
        Order {
            order_id: "o1".into(),
            instrument: "BTCUSDT".into(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            status: OrderStatus::New,
            price: Some(Decimal::new(50000, 0)),
            stop_price: None,
            quantity: Decimal::new(2, 0),
            filled_quantity: Decimal::new(1, 0),
            remaining_quantity: Decimal::new(1, 0),
            avg_fill_price: None,
            time_in_force: TimeInForce::Gtc,
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_001_000,
            client_order_id: None,
        }
    }

    #[test]
    fn val001_rejects_negative_quantity() {
        let err =
            validate_non_negative_quantities(Decimal::new(-1, 0), Decimal::ZERO, Decimal::ZERO)
                .expect_err("negative quantity");
        assert!(matches!(err, ValidationError::Quantity(_)));

        let err =
            validate_non_negative_quantities(Decimal::ONE, Decimal::new(-1, 0), Decimal::ZERO)
                .expect_err("negative filled");
        assert!(matches!(err, ValidationError::Quantity(_)));

        let err =
            validate_non_negative_quantities(Decimal::ONE, Decimal::ZERO, Decimal::new(-1, 0))
                .expect_err("negative remaining");
        assert!(matches!(err, ValidationError::Quantity(_)));
    }

    #[test]
    fn val002_rejects_unbalanced_fill() {
        let err =
            validate_quantity_balance(Decimal::new(2, 0), Decimal::new(1, 0), Decimal::new(2, 0))
                .expect_err("unbalanced");
        assert!(matches!(err, ValidationError::Quantity(_)));
    }

    #[test]
    fn val002_accepts_balanced() {
        validate_quantity_balance(Decimal::new(2, 0), Decimal::new(1, 0), Decimal::new(1, 0))
            .expect("balanced");
    }

    #[test]
    fn val003_market_rejects_price() {
        let err = validate_order_prices(&OrderType::Market, Some(Decimal::new(1, 0)), None)
            .expect_err("market+price");
        assert!(matches!(err, ValidationError::Price(_)));
    }

    #[test]
    fn val003_limit_requires_price() {
        let err =
            validate_order_prices(&OrderType::Limit, None, None).expect_err("limit without price");
        assert!(matches!(err, ValidationError::Price(_)));
    }

    #[test]
    fn val003_stop_limit_requires_both() {
        validate_order_prices(
            &OrderType::StopLimit,
            Some(Decimal::new(100, 0)),
            Some(Decimal::new(99, 0)),
        )
        .expect("stop limit ok");
        let err = validate_order_prices(&OrderType::StopLimit, Some(Decimal::new(100, 0)), None)
            .expect_err("stop limit missing stop");
        assert!(matches!(err, ValidationError::Price(_)));
    }

    #[test]
    fn val003_stop_market_rules() {
        validate_order_prices(&OrderType::StopMarket, None, Some(Decimal::new(99, 0)))
            .expect("stop market ok");
        let err = validate_order_prices(
            &OrderType::StopMarket,
            Some(Decimal::new(100, 0)),
            Some(Decimal::new(99, 0)),
        )
        .expect_err("stop market with price");
        assert!(matches!(err, ValidationError::Price(_)));
    }

    #[test]
    fn val004_rejects_created_after_updated() {
        let err = validate_created_before_updated(200, 100).expect_err("time order");
        assert!(matches!(err, ValidationError::Time(_)));
    }

    #[test]
    fn val005_rejects_gtd_before_created() {
        let err = validate_gtd_deadline(&TimeInForce::Gtd(100), 200).expect_err("gtd");
        assert!(matches!(err, ValidationError::Time(_)));
        validate_gtd_deadline(&TimeInForce::Gtd(200), 200).expect("equal ok");
        validate_gtd_deadline(&TimeInForce::Gtc, 200).expect("gtc ok");
    }

    #[test]
    fn validate_order_accepts_valid_limit() {
        validate_order(&base_order()).expect("valid order");
    }

    #[test]
    fn validate_order_rejects_bad_gtd() {
        let mut order = base_order();
        order.time_in_force = TimeInForce::Gtd(order.created_at - 1);
        let err = validate_order(&order).expect_err("bad gtd");
        assert!(matches!(err, ValidationError::Time(_)));
    }
}
