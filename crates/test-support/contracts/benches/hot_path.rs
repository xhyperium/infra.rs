//! contract-testkit 热路径：RecordingInstrumentation + FakeKeyValueStore。
use std::hint::black_box;
use std::time::Instant;

use contract_testkit::{FakeKeyValueStore, RecordingInstrumentation};
use contracts::{Instrumentation, KeyValueStore};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 1_000 } else { 50_000 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let rec = RecordingInstrumentation::new();
    let kv = FakeKeyValueStore::new();

    for _ in 0..n.min(20) {
        rec.record_retry("w", 1);
        let _ = kv.set("k", b"v".to_vec(), None).await;
    }

    let start = Instant::now();
    for i in 0..n {
        rec.record_retry("bench", i);
        if i % 4 == 0 {
            rec.record_circuit_open("bench");
        }
        kv.set("k", b"v".to_vec(), None).await.expect("set");
        black_box(kv.get("k").await.expect("get"));
    }
    let snap = rec.snapshot().expect("snap");
    let elapsed = start.elapsed();
    println!(
        "bench_contract_testkit: iters={n} total={elapsed:?} per_iter={:?} events={}",
        elapsed / n,
        black_box(snap.len())
    );
}
