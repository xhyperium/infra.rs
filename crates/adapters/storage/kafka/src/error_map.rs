//! 错误映射（字符串启发式；rskafka 错误类型不统一暴露 SQLSTATE 类）。

use kernel::{ErrorKind, XError};

/// 将驱动错误字符串映射为 [`XError`]。
#[must_use]
pub fn map_kafka_err(context: &str, err: impl std::fmt::Display) -> XError {
    let s = err.to_string().to_ascii_lowercase();
    let kind = if s.contains("timeout") || s.contains("timed out") {
        ErrorKind::DeadlineExceeded
    } else if s.contains("cancel") {
        ErrorKind::Cancelled
    } else if s.contains("auth") || s.contains("sasl") || s.contains("ssl") {
        ErrorKind::Unavailable
    } else if s.contains("invalid") || s.contains("unknown topic") {
        ErrorKind::Invalid
    } else if s.contains("not leader") || s.contains("retriable") || s.contains("network") {
        ErrorKind::Transient
    } else {
        ErrorKind::Unavailable
    };
    match kind {
        ErrorKind::DeadlineExceeded => XError::deadline_exceeded(format!("{context}: {err}")),
        ErrorKind::Cancelled => XError::cancelled(format!("{context}: {err}")),
        ErrorKind::Invalid => XError::invalid(format!("{context}: {err}")),
        ErrorKind::Transient => XError::transient(format!("{context}: {err}")),
        _ => XError::unavailable(format!("{context}: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_timeout_to_deadline() {
        let e = map_kafka_err("prod", "request timed out");
        assert_eq!(e.kind(), ErrorKind::DeadlineExceeded);
    }

    #[test]
    fn maps_auth_to_unavailable() {
        let e = map_kafka_err("conn", "SASL authentication failed");
        assert_eq!(e.kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn maps_unknown_topic_to_invalid() {
        let e = map_kafka_err("pub", "Unknown topic or partition");
        assert_eq!(e.kind(), ErrorKind::Invalid);
    }
}
