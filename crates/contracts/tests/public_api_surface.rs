//! contracts 公开面：VenueAdapter 默认错误助手 + Tx 编排（Fake 来自 contract-testkit）。

use async_trait::async_trait;
use contract_testkit::{FakeTxContext, FakeTxRunner, RecordingTxRunner};
use contracts::{
    TxContext, VENUE_CANCEL_REQUEST_DEFAULT_MSG, VENUE_QUERY_REQUEST_DEFAULT_MSG,
    is_default_cancel_order_request_error, is_default_query_order_request_error, run_tx_lifecycle,
};
use kernel::{ErrorKind, XError};
use std::cell::Cell;

struct SendNotSyncTx(Cell<bool>);

#[async_trait]
impl TxContext for SendNotSyncTx {
    async fn commit(&mut self) -> kernel::XResult<()> {
        self.0.set(true);
        Ok(())
    }

    async fn rollback(&mut self) -> kernel::XResult<()> {
        self.0.set(false);
        Ok(())
    }
}

fn assert_send<T: Send>() {}

#[test]
fn tx_context_requires_send_but_accepts_non_sync_implementation() {
    assert_send::<SendNotSyncTx>();
    let _ctx: Box<dyn TxContext> = Box::new(SendNotSyncTx(Cell::new(false)));
}

#[tokio::test]
async fn fake_tx_context_commit_rollback_and_failure() {
    let mut ok = FakeTxContext::new();
    ok.commit().await.expect("commit");
    assert!(ok.committed);
    ok.rollback().await.expect("rb");
    assert!(ok.rolled_back);

    let mut bad = FakeTxContext::new().with_commit_failure();
    assert_eq!(bad.commit().await.unwrap_err().kind(), ErrorKind::Transient);
}

#[tokio::test]
async fn recording_and_fake_tx_runners() {
    let rec = RecordingTxRunner::new();
    let n =
        run_tx_lifecycle(&rec, || async move { Ok::<_, XError>(3u8) }).await.expect("记录事务成功");
    assert_eq!(n, 3);

    let runner = FakeTxRunner;
    let m = run_tx_lifecycle(&runner, || async move { Ok::<_, XError>(9u8) })
        .await
        .expect("Fake 事务成功");
    assert_eq!(m, 9);

    assert!(!VENUE_CANCEL_REQUEST_DEFAULT_MSG.is_empty());
    assert!(!VENUE_QUERY_REQUEST_DEFAULT_MSG.is_empty());
    let cancel_err = XError::invalid(VENUE_CANCEL_REQUEST_DEFAULT_MSG);
    assert!(is_default_cancel_order_request_error(&cancel_err));
    let query_err = XError::invalid(VENUE_QUERY_REQUEST_DEFAULT_MSG);
    assert!(is_default_query_order_request_error(&query_err));
    assert!(!is_default_cancel_order_request_error(&XError::invalid("other")));
}
