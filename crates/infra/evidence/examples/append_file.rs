//! FileEvidenceAppender 最小持久化路径。
use evidence::{FileEvidenceAppender, append_checked};
use std::env::temp_dir;

fn main() {
    let path = temp_dir().join(format!("evidence-ex-{}.log", std::process::id()));
    let a = FileEvidenceAppender::open(&path).expect("open");
    let r = append_checked(&a, "boot").expect("append");
    println!("seq={} path={}", r.seq, a.path().display());
    assert_eq!(r.seq, 1);
    let _ = std::fs::remove_file(&path);
}
