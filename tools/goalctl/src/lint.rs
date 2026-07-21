//! 主观词 lint（fail-closed 提示，可配置为 error）。

/// 常见主观 / 不可测措辞（小写匹配）。
pub const SUBJECTIVE_WORDS: &[&str] = &[
    "更好",
    "更快",
    "尽量",
    "适当",
    "优雅",
    "basically",
    "hopefully",
    "nicer",
    "better",
    "faster",
    "somehow",
    "probably",
    "seems",
    "maybe",
];

/// 扫描文本中的主观词；返回命中列表（原文片段小写）。
#[must_use]
pub fn lint_subjective(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut hits = Vec::new();
    for w in SUBJECTIVE_WORDS {
        let needle = w.to_lowercase();
        if lower.contains(&needle) {
            hits.push((*w).to_string());
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english_subjective() {
        let h = lint_subjective("Make it better somehow");
        assert!(h.iter().any(|x| x == "better"));
        assert!(h.iter().any(|x| x == "somehow"));
    }

    #[test]
    fn clean_text() {
        assert!(lint_subjective("cargo test --workspace must pass").is_empty());
    }
}
