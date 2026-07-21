//! decimalx 公开消费面：构造器、四则、newtypes、Money/Currency。

use decimalx::{
    Currency, Decimal, DecimalError, DecimalErrorKind, DecimalLimits, DecimalResult, MAX_SCALE,
    Money, Price, Qty, Ratio, RoundingStrategy, TECH_MAX_POW10_EXP,
};
use std::cmp::Ordering;

#[test]
fn constants_and_limits() {
    assert_eq!(MAX_SCALE, 18);
    assert_eq!(TECH_MAX_POW10_EXP, 38);
    assert_eq!(DecimalLimits::MAX_SCALE, MAX_SCALE);
    assert_eq!(DecimalLimits::TECH_MAX_POW10_EXP, TECH_MAX_POW10_EXP);
}

#[test]
fn decimal_construct_ops_normalize() {
    let z = Decimal::ZERO;
    assert_eq!(z.mantissa(), 0);
    let d = Decimal::new(1234, 2);
    assert_eq!(d.scale(), 2);
    assert!(d.is_within_limits());
    assert_eq!(d.validate().unwrap().mantissa(), 1234);
    assert!(Decimal::try_new(1, 99).is_err());

    let a = Decimal::new(10, 0);
    let b = Decimal::new(3, 0);
    assert_eq!(a.checked_add(b).unwrap().mantissa(), 13);
    assert_eq!(a.checked_sub(b).unwrap().mantissa(), 7);
    assert_eq!(a.checked_mul(b).unwrap().mantissa(), 30);
    let q = a.checked_div(b, RoundingStrategy::HalfEven).unwrap();
    assert_eq!(a.div(b, RoundingStrategy::HalfEven).unwrap(), q);
    assert_eq!(a.cmp_value(b), Ordering::Greater);
    assert!(a.eq_value(Decimal::new(100, 1)));
    let r = a.rescale(2, RoundingStrategy::HalfUp);
    assert_eq!(r.scale(), 2);
    assert!(a.checked_rescale(2, RoundingStrategy::HalfEven).is_ok());
    assert_eq!(Decimal::new(100, 2).normalize().mantissa(), 1);
}

#[test]
fn newtypes_money_currency_errors() {
    let d = Decimal::new(5, 0);
    assert_eq!(Price::new(d).as_decimal(), d);
    assert_eq!(Price::new(d).into_inner(), d);
    assert_eq!(Qty::new(d).as_decimal(), d);
    assert_eq!(Qty::new(d).into_inner(), d);
    assert_eq!(Ratio::new(d).as_decimal(), d);
    assert_eq!(Ratio::new(d).into_inner(), d);

    let c = Currency::try_new(*b"USD").unwrap();
    assert_eq!(c.as_str(), "USD");
    assert_eq!(c.as_bytes(), *b"USD");
    assert!(c.is_valid());
    assert!(c.validate().is_ok());
    assert!(Currency::try_new(*b"usd").is_err());

    let m = Money::try_new(d, c).unwrap();
    assert_eq!(m.amount(), d);
    assert_eq!(m.currency(), c);
    assert!(m.validate().is_ok());

    let err = Decimal::try_new(1, 99).unwrap_err();
    let _k: DecimalErrorKind = err.kind();
    let _r: DecimalResult<Decimal> = Err(err);
    let _ = format!("{:?}", DecimalError::ScaleOutOfRange { scale: 99, max: MAX_SCALE });
}
