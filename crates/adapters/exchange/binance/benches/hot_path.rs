//! binancex 离线热路径：server_time JSON 解析 + mock adapter 状态机。
//! `cargo test --all-targets` 会编译并运行本 bench（无网络）。
use std::time::Instant;

use binancex::{AdapterState, BinanceAdapter, parse_binance_server_time};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 200 } else { 5_000 }
}

fn main() {
    let n = iters();
    let body = br#"{"serverTime":1710000000123}"#;
    let start = Instant::now();
    let mut acc = 0i64;
    for _ in 0..n {
        acc = acc.wrapping_add(parse_binance_server_time(body).expect("parse"));
    }
    println!("bench_binancex_parse_server_time: iters={n} total={:?} acc={acc}", start.elapsed());

    let start = Instant::now();
    let mut flips = 0u32;
    for i in 0..n {
        let a = if i % 2 == 0 { BinanceAdapter::mainnet() } else { BinanceAdapter::testnet() };
        assert_eq!(a.state(), AdapterState::Disconnected);
        flips = flips.wrapping_add(1);
    }
    println!(
        "bench_binancex_adapter_construct: iters={n} total={:?} flips={flips}",
        start.elapsed()
    );
}
