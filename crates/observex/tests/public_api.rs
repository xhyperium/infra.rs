//! 消费者侧导入面：crate 外使用 `observex::` 与 `contracts::Instrumentation`。

use contracts::Instrumentation;
use observex::{ObservexInstrumentation, TracingInstrumentation};

#[test]
fn consumer_can_construct_and_call_via_trait() {
    let instr = TracingInstrumentation::new();
    let obj: &dyn Instrumentation = &instr;
    obj.record_retry("consumer_op", 1);
    obj.record_circuit_open("consumer_op");
    obj.record_circuit_close("consumer_op");
}

#[test]
fn consumer_can_use_adr_alias() {
    let instr = ObservexInstrumentation::default();
    instr.record_retry("alias_op", 9);
}

#[test]
fn consumer_arc_dyn_works() {
    use std::sync::Arc;
    let instr: Arc<dyn Instrumentation> = Arc::new(TracingInstrumentation::new());
    instr.record_retry("arc", 1);
}

#[test]
fn public_counting_prefix_normalize() {
    use contracts::Instrumentation;
    use observex::{
        CountingInstrumentation, PrefixedInstrumentation, join_op_segments, normalize_op, op_depth,
        op_leaf, record_retry_normalized, sanitize_op, truncate_op,
    };
    let c = CountingInstrumentation::new();
    let p = PrefixedInstrumentation::new("mod", &c);
    p.record_retry("op", 2);
    assert_eq!(c.retry_count(), 1);
    assert_eq!(c.last_attempt(), 2);
    record_retry_normalized(&c, "", 1);
    assert_eq!(normalize_op(""), "_");
    assert_eq!(join_op_segments(&["a", "b"]), "a.b");
    assert!(truncate_op("abcdef", 3).len() <= 3);
    assert_eq!(op_depth("a.b"), 2);
    assert_eq!(sanitize_op("x\ny"), "xy");
    assert_eq!(op_leaf("a.b.c"), "c");
    // 公开面：多字节 op 截断不得 panic，且结果合法 UTF-8
    let zh = "配置服务";
    for max in [2usize, 3, 5, 6, 8, 9] {
        let t = truncate_op(zh, max);
        assert!(t.is_char_boundary(t.len()), "max={max} {t:?}");
        assert!(t.ends_with('~') || t == zh);
        if max < zh.len() && max > 1 {
            assert!(t.len() <= max, "max={max} len={}", t.len());
        }
    }
    assert_eq!(truncate_op(zh, 4), "配~");
}
