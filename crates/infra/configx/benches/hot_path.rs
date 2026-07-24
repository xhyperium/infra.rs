//! configx 热路径：set/get。
use std::hint::black_box;
use std::time::Instant;

use configx::ConfigStore;

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 100_000 }
}

fn main() {
    let n = iters();
    let store = ConfigStore::new();
    for i in 0..n.min(20) {
        store.set(format!("k{i}"), format!("v{i}")).expect("set");
        let _ = black_box(store.get(&format!("k{i}")));
    }
    let start = Instant::now();
    for i in 0..n {
        let k = format!("key{i}");
        store.set(&k, "val").expect("set");
        black_box(store.get(&k).expect("get"));
    }
    let elapsed = start.elapsed();
    println!("bench_configx_set_get: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n);
}
