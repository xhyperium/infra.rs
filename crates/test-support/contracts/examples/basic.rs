//! 最小消费者路径：Fake KV + Tx 编排 + suite 断言。
//!
//! ```bash
//! cargo run -p contract-testkit --example basic
//! ```
//!
//! # 生产红线
//! - 本示例仅进程内 Fake，**不是** 真实后端 / L3 live 入口。

use contract_testkit::{
    FakeKeyValueStore, FakeTxRunner, FixtureNamespace, RecordingTxRunner, assert_key_value_store,
    assert_tx_runner,
};
use contracts::{KeyValueStore, run_tx_lifecycle};

#[tokio::main]
async fn main() {
    let kv = FakeKeyValueStore::new();
    kv.set("k", b"v".to_vec(), None).await.expect("set");
    assert_eq!(kv.get("k").await.expect("get").as_deref(), Some(b"v".as_slice()));
    let fixture = FixtureNamespace::new("ctk_example_key_value").expect("valid fixture");
    assert_key_value_store(&kv, &fixture).await.expect("kv suite");
    println!("contract_testkit_example: kv_ok");

    assert_tx_runner(&FakeTxRunner).await.expect("tx suite");
    let rec = RecordingTxRunner::new();
    let n = run_tx_lifecycle(&rec, || async move { Ok::<_, kernel::XError>(7u8) })
        .await
        .expect("commit");
    assert_eq!(n, 7);
    assert!(*rec.committed.lock().expect("lock"));
    println!("contract_testkit_example: tx_ok committed=true");
}
