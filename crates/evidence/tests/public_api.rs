//! 库外消费者：`evidence::` 公开面。

use evidence::{AppendReceipt, EvidenceAppender, EvidenceError, InMemoryEvidenceAppender};
use std::sync::Arc;

#[test]
fn consumer_in_memory_roundtrip() {
    let a = Arc::new(InMemoryEvidenceAppender::new());
    let obj: Arc<dyn EvidenceAppender> = a.clone();
    let r: AppendReceipt = obj.append_named("evt").expect("ok");
    assert_eq!(r.seq, 1);
    assert_eq!(a.names(), vec!["evt".to_string()]);
}

#[test]
fn consumer_error_variants() {
    let a = InMemoryEvidenceAppender::new();
    a.fail_next();
    assert_eq!(a.append_named("x"), Err(EvidenceError::DurabilityFailure));
    a.close();
    assert_eq!(a.append_named("y"), Err(EvidenceError::Unavailable));
}
