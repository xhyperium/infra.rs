//! canonical 热路径：Order serde 往返。
use std::hint::black_box;
use std::time::Instant;

use canonical::{Order, OrderStatus, Side};
use decimalx::{Decimal, Price, Qty};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 500 } else { 20_000 }
}

fn main() {
    let n = iters();
    let order = Order {
        id: "o1".into(),
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        price: Price::new(Decimal::new(50_000, 0)),
        qty: Qty::new(Decimal::new(1, 0)),
        status: OrderStatus::Open,
    };
    for _ in 0..n.min(20) {
        let j = serde_json::to_string(&order).expect("ser");
        let _: Order = serde_json::from_str(&j).expect("de");
    }
    let start = Instant::now();
    let mut len = 0usize;
    for _ in 0..n {
        let j = serde_json::to_string(&order).expect("ser");
        let back: Order = serde_json::from_str(&j).expect("de");
        len = len.wrapping_add(back.id.len());
        black_box(back);
    }
    let elapsed = start.elapsed();
    println!(
        "bench_canonical_order_serde: iters={n} total={elapsed:?} per_iter={:?} len={}",
        elapsed / n,
        black_box(len)
    );
}
