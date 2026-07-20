//! Crate-外消费者路径：通过 lib 入口加载已交付类型并断言 checked 运算返回值。
//!
//! 非 mock、非 re-implementation：直接调用 `decimalx` 导出 API。

use decimalx::{Currency, Decimal, MAX_SCALE, Money, RoundingStrategy};

#[test]
fn entry_checked_add_one_plus_half() {
    // 1.00 + 0.50 = 1.50
    let a = Decimal::try_new(100, 2).expect("try_new 1.00");
    let b = Decimal::try_new(50, 2).expect("try_new 0.50");
    let sum = a.checked_add(b).expect("checked_add");
    assert_eq!(sum.mantissa, 150);
    assert_eq!(sum.scale, 2);
    assert_eq!(sum.to_string(), "1.5");
    assert!(sum.eq_value(Decimal::try_new(15, 1).unwrap()));
}

#[test]
fn entry_checked_div_with_explicit_rounding() {
    // 1.00 / 0.50 = 2.00（HalfUp）
    let a = Decimal::try_new(100, 2).unwrap();
    let b = Decimal::try_new(50, 2).unwrap();
    let q = a.checked_div(b, RoundingStrategy::HalfUp).expect("checked_div");
    assert_eq!(q.mantissa, 200);
    assert_eq!(q.scale, 2);
    assert_eq!(q.to_string(), "2");
}

#[test]
fn entry_div_zero_and_overflow_err() {
    let a = Decimal::try_new(1, 0).unwrap();
    let err = a.checked_div(Decimal::ZERO, RoundingStrategy::Floor).unwrap_err();
    assert!(err.to_string().contains("division by zero"));

    let overflow = Decimal::new(i128::MAX, 0).checked_add(Decimal::new(1, 0)).unwrap_err();
    assert!(overflow.to_string().contains("overflow"));

    // i128 除法溢出：MIN / -1 必须 Err，不得 panic
    let div_overflow = Decimal::new(i128::MIN, 0)
        .checked_div(Decimal::new(-1, 0), RoundingStrategy::HalfUp)
        .expect_err("i128 MIN/-1 must return Err");
    assert!(div_overflow.to_string().contains("overflow"));
}

#[test]
fn entry_money_currency_parse() {
    let c: Currency = "USD".parse().expect("currency parse");
    let m = Money::try_new(Decimal::try_new(999, 2).unwrap(), c).unwrap();
    assert_eq!(m.currency.as_str(), "USD");
    assert_eq!(m.amount.mantissa, 999);
    // 生产路径接受 MAX_SCALE 并拒绝 +1（驱动真实 exported 常量）
    assert!(Decimal::try_new(1, MAX_SCALE).is_ok());
    assert!(Decimal::try_new(1, MAX_SCALE.saturating_add(1)).is_err());
}
