//! 证据回放检查（本地）。

use crate::{EvidenceError, parse_evidence_log};

/// 检查日志是否严格递增 seq。
pub fn seq_is_monotonic(text: &str) -> bool {
    let mut last = 0u64;
    for (seq, _) in parse_evidence_log(text) {
        if seq <= last {
            return false;
        }
        last = seq;
    }
    true
}

/// 统计事件条数。
#[must_use]
pub fn event_count(text: &str) -> usize {
    parse_evidence_log(text).len()
}

/// 校验日志可读且单调。
pub fn validate_log_text(text: &str) -> Result<usize, EvidenceError> {
    if !seq_is_monotonic(text) {
        return Err(EvidenceError::DurabilityFailure);
    }
    Ok(event_count(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_and_count() {
        assert!(seq_is_monotonic("1\ta\n2\tb\n"));
        assert!(!seq_is_monotonic("2\ta\n1\tb\n"));
        assert_eq!(event_count("1\ta\n2\tb\n"), 2);
        assert_eq!(validate_log_text("1\ta\n2\tb\n").unwrap(), 2);
        assert!(validate_log_text("2\ta\n1\tb\n").is_err());
        for i in 1..40 {
            let mut body = String::new();
            for j in 1..=i {
                body.push_str(&format!("{j}\te{j}\n"));
            }
            assert_eq!(event_count(&body), i as usize);
            assert!(seq_is_monotonic(&body));
        }
    }
}
