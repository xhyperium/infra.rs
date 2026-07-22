//! verifyctl 热路径：build_plan（无网络）。
use std::time::Instant;
use verifyctl::{PlanOptions, build_plan};

fn main() {
    let contract = r#"{"schema":"goal-contract/v1","digest":"abc","touches":["tools/verifyctl"]}"#;
    let changed = vec!["tools/verifyctl".to_string()];
    let opts = PlanOptions { dry: true };
    let _ = build_plan(contract, &changed, &opts).expect("plan");
    let iters = 200usize;
    let t0 = Instant::now();
    let mut last = String::new();
    for _ in 0..iters {
        let p = build_plan(contract, &changed, &opts).expect("plan");
        last = p.plan_digest;
    }
    let elapsed = t0.elapsed();
    println!(
        "bench_verifyctl_plan: iters={iters} total={elapsed:?} last_digest_prefix={}",
        &last[..12.min(last.len())]
    );
}
