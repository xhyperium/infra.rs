//! contracts 公开面：FakeTx / RecordingTx / 默认 VenueAdapter 错误助手。

use contracts::{
    FakeTxContext, FakeTxRunner, RecordingTxRunner, TxContext, VENUE_CANCEL_REQUEST_DEFAULT_MSG,
    VENUE_QUERY_REQUEST_DEFAULT_MSG, is_default_cancel_order_request_error,
    is_default_query_order_request_error, run_tx_commit_on_ok,
};
use kernel::{ErrorKind, XError};

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
    let n = run_tx_commit_on_ok(&rec, |_ctx| async move { Ok::<_, XError>(3u8) })
        .await
        .expect("rec tx");
    assert_eq!(n, 3);

    let runner = FakeTxRunner;
    let m = run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, XError>(9u8) })
        .await
        .expect("fake tx");
    assert_eq!(m, 9);

    assert!(!VENUE_CANCEL_REQUEST_DEFAULT_MSG.is_empty());
    assert!(!VENUE_QUERY_REQUEST_DEFAULT_MSG.is_empty());
    let cancel_err = XError::invalid(VENUE_CANCEL_REQUEST_DEFAULT_MSG);
    assert!(is_default_cancel_order_request_error(&cancel_err));
    let query_err = XError::invalid(VENUE_QUERY_REQUEST_DEFAULT_MSG);
    assert!(is_default_query_order_request_error(&query_err));
    assert!(!is_default_cancel_order_request_error(&XError::invalid("other")));
}
