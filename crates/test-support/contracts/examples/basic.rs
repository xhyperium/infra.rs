//! 最小消费者路径：Fake KV + Tx 编排 + suite 断言。
//!
//! ```bash
//! cargo run -p contract-testkit --example basic
//! ```
//!
//! # 生产红线
//! - 本示例仅进程内 Fake，**不是** 真实后端 / L3 live 入口。

use contract_testkit::{
    FakeKeyValueStore, FakeTxRunner, RecordingTxRunner, assert_key_value_store, assert_tx_runner,
};
use contracts::{KeyValueStore, run_tx_lifecycle};

#[tokio::main]
async fn main() {
    let kv = FakeKeyValueStore::new();
    kv.set("k", b"v".to_vec(), None).await.expect("写入应成功");
    assert_eq!(kv.get("k").await.expect("读取应成功").as_deref(), Some(b"v".as_slice()));
    assert_key_value_store(&kv).await.expect("KV suite 应通过");
    println!("contract_testkit_example: kv_ok");

    assert_tx_runner(&FakeTxRunner).await.expect("事务 suite 应通过");
    let rec = RecordingTxRunner::new();
    let n = run_tx_lifecycle(&rec, || async move { Ok::<_, kernel::XError>(7u8) })
        .await
        .expect("提交应成功");
    assert_eq!(n, 7);
    assert!(*rec.committed.lock().expect("提交锁应可用"));
    println!("contract_testkit_example: tx_ok committed=true");
}
