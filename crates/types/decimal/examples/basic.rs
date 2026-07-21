//! 最小消费者路径：checked 四则 + Money/Currency。
//!
//! ```bash
//! cargo run -p decimalx --example basic
//! ```

use decimalx::{Currency, Decimal, Money, RoundingStrategy};
use std::str::FromStr;

fn main() {
    let a = Decimal::try_new(10, 0).expect("try_new");
    let b = Decimal::from_str("3").expect("parse");
    let sum = a.checked_add(b).expect("add");
    assert_eq!(sum.mantissa(), 13);

    let q = a.checked_div(b, RoundingStrategy::HalfEven).expect("div");
    assert!(q.mantissa() != 0);

    let ccy = Currency::try_new(*b"USD").expect("currency");
    let money = Money::try_new(a, ccy).expect("money");
    assert_eq!(money.currency().as_str(), "USD");
    assert_eq!(money.amount().mantissa(), 10);

    let json = serde_json::to_string(&money).expect("serialize");
    let back: Money = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.amount(), money.amount());

    println!(
        "decimalx-consumer: ok sum={} quot_mantissa={} money_usd={}",
        sum,
        q.mantissa(),
        money.amount()
    );
}
