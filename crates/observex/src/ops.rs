//! op 名处理辅助。

use super::normalize_op;

/// 将层级路径片段连接为 op 名（跳过空片段）。
#[must_use]
pub fn join_op_segments(segments: &[&str]) -> String {
    segments.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(".")
}

/// 限制 op 显示长度（按字节截断）。
#[must_use]
pub fn truncate_op(op: &str, max_bytes: usize) -> String {
    let op = normalize_op(op);
    if op.len() <= max_bytes {
        return op.to_string();
    }
    if max_bytes <= 1 {
        return op.chars().take(1).collect();
    }
    format!("{}~", &op[..max_bytes.saturating_sub(1)])
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
        assert_eq!(truncate_op("abcdefghij", 4).len(), 4);
        assert_eq!(truncate_op("", 3), "_");
        assert!(is_friendly_op("svc.fetch"));
        assert!(!is_friendly_op(""));
        assert!(!is_friendly_op("a\nb"));
        assert!(!is_friendly_op(&"x".repeat(200)));
    }

    #[test]
    fn join_many_segments() {
        let segs: Vec<&str> = (0..20).map(|_| "p").collect();
        let j = join_op_segments(&segs);
        assert!(j.starts_with("p.p"));
        assert_eq!(j.matches('.').count(), 19);
        for max in [1usize, 2, 8, 64] {
            let t = truncate_op(&j, max);
            assert!(t.len() <= max + 1);
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
