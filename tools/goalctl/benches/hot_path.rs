//! goalctl 热路径：compile 稳定 digest（无网络）。
use goalctl::compile_goal_str;
use std::time::Instant;

fn sample_goal() -> &'static str {
    r#"
id: GOAL-BENCH-001
outcome: cargo test -p goalctl passes
risk: R1
acceptance:
  - id: AC-1
    statement: doctor exits 0
invariants:
  - offline unit tests
forbidden:
  - embed production secrets
not_in_scope:
  - full authority plane
touches:
  - tools/goalctl
"#
}

fn main() {
    let raw = sample_goal();
    let _ = compile_goal_str(raw, Some("yaml")).expect("compile");
    let iters = 500usize;
    let t0 = Instant::now();
    let mut last = String::new();
    for _ in 0..iters {
        let out = compile_goal_str(raw, Some("yaml")).expect("compile");
        last = out.contract.digest.clone();
    }
    let elapsed = t0.elapsed();
    println!(
        "bench_goalctl_compile: iters={iters} total={elapsed:?} last_digest_prefix={}",
        &last[..12.min(last.len())]
    );
}
