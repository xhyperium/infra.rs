//! 最小消费者路径：Order 构造 + committed wire serde 往返。
//!
//! ```bash
//! cargo run -p canonical --example basic
//! ```

use canonical::{
    CancelOrderRequest, Order, OrderRef, OrderStatus, Side, WireCommitment, ns_from_unix_millis,
    wire_commitment,
};
use decimalx::{Decimal, Price, Qty};

fn main() {
    let order = Order {
        id: "o-1".into(),
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        price: Price::new(Decimal::new(50_000, 0)),
        qty: Qty::new(Decimal::new(1, 0)),
        status: OrderStatus::Open,
    };
    let json = serde_json::to_string(&order).expect("serialize Order");
    let back: Order = serde_json::from_str(&json).expect("deserialize Order");
    assert_eq!(back, order);
    assert_eq!(wire_commitment("Order"), WireCommitment::CommittedV1);

    let cancel = CancelOrderRequest {
        venue: "okx".into(),
        instrument: "BTC-USDT".into(),
        id: OrderRef::Exchange("987".into()),
    };
    let cjson = serde_json::to_string(&cancel).expect("serialize cancel");
    let cback: CancelOrderRequest = serde_json::from_str(&cjson).expect("deserialize cancel");
    assert_eq!(cback, cancel);
    assert_eq!(wire_commitment("CancelOrderRequest"), WireCommitment::CommittedV1);

    let ns = ns_from_unix_millis(1_700_000_000_000).expect("ms→ns");
    assert_eq!(ns, 1_700_000_000_000_000_000);

    println!(
        "canonical-consumer: ok order_id={} cancel_venue={} ts_ns={}",
        back.id, cback.venue, ns
    );
}
