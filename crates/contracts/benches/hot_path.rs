//! contracts 热路径：RecordingInstrumentation + FakeKeyValueStore。
use std::hint::black_box;
use std::time::Instant;

use contracts::{Instrumentation, RecordingInstrumentation};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 1_000 } else { 50_000 }
}

fn main() {
    let n = iters();
    let rec = RecordingInstrumentation::new();
    for _ in 0..n.min(20) {
        rec.record_retry("w", 1);
    }
    let start = Instant::now();
    for i in 0..n {
        rec.record_retry("bench", i);
        if i % 4 == 0 {
            rec.record_circuit_open("bench");
        }
        if i % 8 == 0 {
            rec.record_circuit_close("bench");
        }
        black_box(i);
    }
    let snap = rec.snapshot().expect("snap");
    let elapsed = start.elapsed();
    println!(
        "bench_contracts_instr: iters={n} total={elapsed:?} per_iter={:?} events={}",
        elapsed / n,
        black_box(snap.len())
    );
}
