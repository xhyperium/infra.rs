//! 错误映射（字符串仅用于分类；公开上下文不回显驱动原文）。

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
    let summary = match kind {
        ErrorKind::DeadlineExceeded => "驱动请求超时",
        ErrorKind::Cancelled => "驱动请求已取消",
        ErrorKind::Invalid => "驱动拒绝请求",
        ErrorKind::Transient => "驱动报告可重试故障",
        _ => "驱动不可用",
    };
    let source = std::io::Error::other(summary);
    match kind {
        ErrorKind::DeadlineExceeded => {
            XError::deadline_exceeded(format!("{context}: {summary}")).with_source(source)
        }
        ErrorKind::Cancelled => {
            XError::cancelled(format!("{context}: {summary}")).with_source(source)
        }
        ErrorKind::Invalid => XError::invalid(format!("{context}: {summary}")).with_source(source),
        ErrorKind::Transient => {
            XError::transient(format!("{context}: {summary}")).with_source(source)
        }
        _ => XError::unavailable(format!("{context}: {summary}")).with_source(source),
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

    #[test]
    fn public_error_does_not_echo_driver_text() {
        let secret = "SASL password=do-not-leak authentication failed";
        let error = map_kafka_err("kafkax connect", secret);
        assert_eq!(error.kind(), ErrorKind::Unavailable);
        assert!(!error.context().contains("do-not-leak"));
        let source = std::error::Error::source(&error).expect("保留脱敏 source").to_string();
        assert!(!source.contains("do-not-leak"));
    }
}
