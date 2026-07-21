//! 内存证据追加：开发/测试默认路径。
//!
//! ```bash
//! cargo run -p evidence --example append_memory
//! ```
//!
//! # 生产红线
//! - `InMemoryEvidenceAppender` **进程退出即失**，不可作合规审计落盘。
//! - 远程/签名 wire 仍为 SSOT DEFER。

use evidence::{EvidenceAppender, InMemoryEvidenceAppender};

fn main() {
    let app = InMemoryEvidenceAppender::new();
    let r1 = app.append_named("boot.start").expect("append");
    let r2 = app.append_named("boot.ready").expect("append");
    assert_eq!(r1.seq, 1);
    assert_eq!(r2.seq, 2);
    assert_eq!(r1.name, "boot.start");
    println!("evidence_memory_ok seq1={} seq2={} (in-memory only)", r1.seq, r2.seq);
}
