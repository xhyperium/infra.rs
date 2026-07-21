//! 最小 Fake 合同面：KV set/get + 事务 Ok→commit 编排。
//!
//! ```bash
//! cargo run -p contracts --example fake_surface
//! ```
//!
//! # 生产红线
//! - 本示例使用 **进程内 Fake**，不是非 scaffold 真实后端。
//! - L3 Contract Ready 仍要求至少一非 scaffold 验证入口（adapters W4 / `infra-s9t.2`）。
//! - 勿将 Fake 路径当作集成测完成或生产存储。

use contracts::{
    FakeKeyValueStore, FakeTxRunner, KeyValueStore, RecordingTxRunner, run_tx_commit_on_ok,
};

#[tokio::main]
async fn main() {
    // ── KV Fake ───────────────────────────────────────────
    let kv = FakeKeyValueStore::new();
    kv.set("k", b"v".to_vec(), None).await.expect("set");
    let got = kv.get("k").await.expect("get");
    assert_eq!(got.as_deref(), Some(b"v".as_slice()));
    println!("fake_kv_ok key=k value=v");

    // ── Tx: Ok → commit ───────────────────────────────────
    let runner = FakeTxRunner;
    let v = run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, kernel::XError>(42u32) })
        .await
        .expect("commit path");
    assert_eq!(v, 42);
    println!("fake_tx_ok value={v}");

    // ── Recording: 可观察 commit 标志 ─────────────────────
    let rec = RecordingTxRunner::new();
    run_tx_commit_on_ok(&rec, |_ctx| async move { Ok::<_, kernel::XError>(()) })
        .await
        .expect("record commit");
    let committed = *rec.committed.lock().expect("lock");
    assert!(committed);
    println!("recording_tx_ok committed={committed}");
}
