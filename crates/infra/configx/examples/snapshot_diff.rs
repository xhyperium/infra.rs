//! 快照与 diff 公共路径。
use configx::{ConfigSnapshot, diff_snapshots, store_from_pairs};

fn main() {
    let a = store_from_pairs([("host", "a"), ("port", "1")]).expect("a");
    let b = store_from_pairs([("host", "b"), ("env", "dev")]).expect("b");
    let d = diff_snapshots(&ConfigSnapshot::capture(&a), &ConfigSnapshot::capture(&b));
    println!("changes={}", d.total_changes());
    assert!(d.total_changes() >= 2);
}
