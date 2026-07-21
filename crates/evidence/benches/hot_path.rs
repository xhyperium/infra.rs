//! evidence 热路径：InMemoryEvidenceAppender::append_named。
use std::hint::black_box;
use std::time::Instant;

use evidence::{EvidenceAppender, InMemoryEvidenceAppender};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 100_000 }
}

fn main() {
    let n = iters();
    let a = InMemoryEvidenceAppender::new();
    for _ in 0..n.min(20) {
        let _ = black_box(a.append_named("warmup"));
    }
    let start = Instant::now();
    for i in 0..n {
        black_box(a.append_named(&format!("evt-{i}")).expect("append"));
    }
    let elapsed = start.elapsed();
    println!(
        "bench_evidence_append: iters={n} total={elapsed:?} per_iter={:?} len={}",
        elapsed / n,
        black_box(a.len())
    );
}
