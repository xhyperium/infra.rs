//! op 名处理辅助。

use super::normalize_op;

/// 将层级路径片段连接为 op 名（跳过空片段）。
#[must_use]
pub fn join_op_segments(segments: &[&str]) -> String {
    segments.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(".")
}

/// 将索引下压到合法 UTF-8 字符边界（不含 `str::floor_char_boundary`，兼容 MSRV 1.85）。
#[must_use]
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// 限制 op 显示长度（按字节预算截断，落在 UTF-8 字符边界）。
///
/// - 未超长：原样返回。
/// - `max_bytes == 0`：空串。
/// - `max_bytes == 1`：取首字符（可能多字节，与「至少展示一字」一致）。
/// - 其余：在 `max_bytes - 1` 字节内 floor 到字符边界后追加 `~`。
#[must_use]
pub fn truncate_op(op: &str, max_bytes: usize) -> String {
    let op = normalize_op(op);
    if max_bytes == 0 {
        return String::new();
    }
    if op.len() <= max_bytes {
        return op.to_string();
    }
    if max_bytes == 1 {
        return op.chars().take(1).collect();
    }
    let budget = max_bytes.saturating_sub(1);
    let end = floor_char_boundary(op, budget);
    if end == 0 {
        return "~".to_string();
    }
    format!("{}~", &op[..end])
}

/// 判断 op 是否「可观测友好」（非空、无控制字符、长度受限）。
#[must_use]
pub fn is_friendly_op(op: &str) -> bool {
    let op = op.trim();
    !op.is_empty() && op.len() <= 128 && !op.chars().any(|c| c.is_control())
}

/// 统计 op 层级深度（以 `.` 分段；空/归一后 `_` 视为 1）。
#[must_use]
pub fn op_depth(op: &str) -> usize {
    let op = normalize_op(op.trim());
    if op.is_empty() || op == "_" {
        return 1;
    }
    op.split('.').filter(|s| !s.is_empty()).count().max(1)
}

/// 去掉控制字符并 trim；结果为空则回落 `"_"`。
#[must_use]
pub fn sanitize_op(op: &str) -> String {
    let cleaned: String =
        op.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string();
    if cleaned.is_empty() { "_".to_string() } else { cleaned }
}

/// 取 op 的叶子段（最后一段）；无点则整段。
#[must_use]
pub fn op_leaf(op: &str) -> String {
    let op = sanitize_op(op);
    op.rsplit('.').next().unwrap_or("_").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_and_truncate() {
        assert_eq!(join_op_segments(&["a", "", " b ", "c"]), "a.b.c");
        assert_eq!(join_op_segments(&[]), "");
        assert_eq!(truncate_op("short", 10), "short");
        assert_eq!(truncate_op("abcdefghij", 4), "abc~");
        assert_eq!(truncate_op("abcdefghij", 4).len(), 4);
        assert_eq!(truncate_op("", 3), "_");
        assert_eq!(truncate_op("x", 0), "");
        assert!(is_friendly_op("svc.fetch"));
        assert!(!is_friendly_op(""));
        assert!(!is_friendly_op("a\nb"));
        assert!(!is_friendly_op(&"x".repeat(200)));
    }

    #[test]
    fn floor_char_boundary_edges() {
        assert_eq!(floor_char_boundary("abc", 0), 0);
        assert_eq!(floor_char_boundary("abc", 3), 3);
        assert_eq!(floor_char_boundary("abc", 99), 3);
        // 多字节：索引落在第二字节 → 0
        assert_eq!(floor_char_boundary("配", 1), 0);
        assert_eq!(floor_char_boundary("配", 2), 0);
        assert_eq!(floor_char_boundary("配", 3), 3);
    }

    #[test]
    fn truncate_op_multibyte_no_panic() {
        // 每个汉字 3 字节；旧实现按裸字节切片会 panic。
        let zh = "配置服务";
        assert_eq!(zh.len(), 12);
        for max in [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
            let t = truncate_op(zh, max);
            assert!(t.is_char_boundary(t.len()), "result must be valid UTF-8 for max={max}: {t:?}");
            if max == 0 {
                assert!(t.is_empty());
            } else if max == 1 {
                // 至少一字（可能超过 1 字节）
                assert_eq!(t, "配");
            } else if max >= zh.len() {
                assert_eq!(t, zh);
            } else {
                assert!(t.ends_with('~'), "max={max} got {t:?}");
                assert!(t.len() <= max, "max={max} got len={} {t:?}", t.len());
            }
        }
        // 明确边界：max=4 → budget=3 → 一字 + ~
        assert_eq!(truncate_op(zh, 4), "配~");
        assert_eq!(truncate_op(zh, 7), "配置~");
        // 混合 ASCII + 多字节
        let mixed = "api.配置";
        let t = truncate_op(mixed, 6);
        assert!(t.is_char_boundary(t.len()));
        assert!(t.len() <= 6);
    }

    #[test]
    fn join_many_segments() {
        let segs: Vec<&str> = (0..20).map(|_| "p").collect();
        let j = join_op_segments(&segs);
        assert!(j.starts_with("p.p"));
        assert_eq!(j.matches('.').count(), 19);
        for max in [1usize, 2, 8, 64] {
            let t = truncate_op(&j, max);
            // max==1 时可能返回单字符（len 1）；否则带 ~ 时 len <= max
            if max == 1 {
                assert!(!t.is_empty());
            } else {
                assert!(t.len() <= max, "max={max} got {}", t.len());
            }
        }
    }

    #[test]
    fn depth_sanitize_leaf() {
        assert_eq!(op_depth(""), 1);
        assert_eq!(op_depth("a.b.c"), 3);
        assert_eq!(op_depth("  single  "), 1);
        assert_eq!(sanitize_op("a\nb"), "ab");
        assert_eq!(sanitize_op("   "), "_");
        assert_eq!(op_leaf("api.orders.create"), "create");
        assert_eq!(op_leaf(""), "_");
        for n in 1..=10 {
            let segs: Vec<String> = (0..n).map(|i| format!("s{i}")).collect();
            let refs: Vec<&str> = segs.iter().map(String::as_str).collect();
            let j = join_op_segments(&refs);
            assert_eq!(op_depth(&j), n);
            assert_eq!(op_leaf(&j), format!("s{}", n - 1));
        }
    }
}
