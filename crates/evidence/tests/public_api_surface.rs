//! evidence 公开面：错误 Display/Error、len/is_empty、Default。

use evidence::{AppendReceipt, EvidenceAppender, EvidenceError, InMemoryEvidenceAppender};
use std::error::Error;

#[test]
fn full_appender_surface() {
    let a = InMemoryEvidenceAppender::default();
    assert!(a.is_empty());
    assert_eq!(a.len(), 0);
    let r = a.append_named("boot").unwrap();
    assert_eq!(r, AppendReceipt { name: "boot".into(), seq: 1 });
    assert!(!a.is_empty());
    assert_eq!(a.len(), 1);
    assert_eq!(a.names(), vec!["boot".to_string()]);

    a.fail_next();
    assert_eq!(a.append_named("x"), Err(EvidenceError::DurabilityFailure));
    a.close();
    assert_eq!(a.append_named("y"), Err(EvidenceError::Unavailable));

    for e in [EvidenceError::DurabilityFailure, EvidenceError::Unavailable] {
        assert!(!e.to_string().is_empty());
        let _dyn: &dyn Error = &e;
    }
}
