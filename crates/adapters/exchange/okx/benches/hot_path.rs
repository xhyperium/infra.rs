//! okxx 离线热路径：server_time JSON 解析 + mock adapter 构造。
//! `cargo test --all-targets` 会编译并运行本 bench（无网络）。
use std::time::Instant;

use okxx::{AdapterState, OkxAdapter, parse_okx_server_time};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 200 } else { 5_000 }
}

fn main() {
    let n = iters();
    let body = br#"{"data":[{"ts":"1710000000456"}]}"#;
    let start = Instant::now();
    let mut acc = 0i64;
    for _ in 0..n {
        acc = acc.wrapping_add(parse_okx_server_time(body).expect("parse"));
    }
    println!("bench_okxx_parse_server_time: iters={n} total={:?} acc={acc}", start.elapsed());

    let start = Instant::now();
    let mut flips = 0u32;
    for i in 0..n {
        let a = if i % 2 == 0 {
            OkxAdapter::demo()
        } else {
            OkxAdapter::new("okx-bench", "https://www.okx.com")
        };
        assert_eq!(a.state(), AdapterState::Disconnected);
        flips = flips.wrapping_add(1);
    }
    println!("bench_okxx_adapter_construct: iters={n} total={:?} flips={flips}", start.elapsed());
}
