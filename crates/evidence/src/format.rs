//! 证据日志行格式。

use crate::{EvidenceError, validate_event_name};

/// 格式化为日志行（不含换行）。
pub fn format_line(seq: u64, name: &str) -> Result<String, EvidenceError> {
    validate_event_name(name)?;
    Ok(format!("{seq}\t{name}"))
}

/// 解析单行；坏行返回 None。
#[must_use]
pub fn parse_line(line: &str) -> Option<(u64, String)> {
    // 仅剥行尾 CR/LF，保留 TAB（否则 "5\t" 会被 trim 成 "5" 丢空 name 分支）
    let line = line.trim_end_matches(['\r', '\n']);
    if line.trim().is_empty() {
        return None;
    }
    let (seq_s, name) = line.split_once('\t')?;
    let seq = seq_s.trim().parse().ok()?;
    if name.is_empty() {
        return None;
    }
    Some((seq, name.to_string()))
}

/// 渲染多行日志正文。
pub fn render_log(entries: &[(u64, String)]) -> Result<String, EvidenceError> {
    let mut out = String::new();
    for (seq, name) in entries {
        let line = format_line(*seq, name)?;
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// 统计日志中最大 seq（空 → 0）。
#[must_use]
pub fn max_seq(text: &str) -> u64 {
    let mut m = 0u64;
    for line in text.lines() {
        if let Some((seq, _)) = parse_line(line) {
            m = m.max(seq);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_after_tab() {
        assert!(parse_line("5\t").is_none());
        assert!(parse_line("\tname").is_none());
    }

    #[test]
    fn format_parse_roundtrip() {
        let line = format_line(7, "evt").unwrap();
        assert_eq!(line, "7\tevt");
        assert_eq!(parse_line(&line), Some((7, "evt".into())));
        assert!(format_line(1, "").is_err());
        assert!(parse_line("nope").is_none());
        assert!(parse_line("").is_none());
        let body = render_log(&[(1, "a".into()), (2, "b".into())]).unwrap();
        assert!(body.contains("1\ta"));
        assert!(body.contains("2\tb"));
        assert_eq!(max_seq(&body), 2);
        assert_eq!(max_seq(""), 0);
        assert_eq!(max_seq("bad\n9\tz\n"), 9);
    }

    #[test]
    fn render_many_and_reject_bad_name() {
        let entries: Vec<(u64, String)> = (1..=30).map(|i| (i, format!("e{i}"))).collect();
        let body = render_log(&entries).unwrap();
        assert_eq!(body.lines().count(), 30);
        assert_eq!(max_seq(&body), 30);
        assert!(render_log(&[(1, "bad\nname".into())]).is_err());
        assert!(format_line(1, "x\ry").is_err());
        for i in 0..20 {
            let line = format_line(i, &format!("n{i}")).unwrap();
            let parsed = parse_line(&line).unwrap();
            assert_eq!(parsed.0, i);
        }
    }
}
