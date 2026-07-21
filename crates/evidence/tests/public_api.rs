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

#[test]
fn public_file_helpers_batch_parse() {
    use evidence::{
        FileEvidenceAppender, InMemoryEvidenceAppender, append_batch, append_checked, format_line,
        max_seq, parse_evidence_log, parse_line, render_log,
    };
    let mem = InMemoryEvidenceAppender::new();
    let rs = append_batch(&mem, &["a", "b"]).unwrap();
    assert_eq!(rs[0].seq, 1);
    let line = format_line(1, "x").unwrap();
    assert_eq!(parse_line(&line).unwrap().1, "x");
    let body = render_log(&[(1, "a".into())]).unwrap();
    assert_eq!(max_seq(&body), 1);
    assert!(!parse_evidence_log(&body).is_empty());
    let dir = std::env::temp_dir().join(format!("ev-it-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("t.log");
    let f = FileEvidenceAppender::open(&path).unwrap();
    append_checked(&f, "one").unwrap();
    assert_eq!(f.read_entries().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
