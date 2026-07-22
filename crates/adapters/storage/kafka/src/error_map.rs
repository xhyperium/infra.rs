//! 错误映射（驱动文本仅用于分类；公开上下文不回显驱动原文）。

use std::error::Error;

use kernel::{ErrorKind, XError};

/// 将驱动错误映射为 [`XError`]，并保留真实 source。
#[must_use]
pub fn map_kafka_err<E>(context: &str, err: E) -> XError
where
    E: Error + Send + Sync + 'static,
{
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
    match kind {
        ErrorKind::DeadlineExceeded => {
            XError::deadline_exceeded(format!("{context}: {summary}")).with_source(err)
        }
        ErrorKind::Cancelled => XError::cancelled(format!("{context}: {summary}")).with_source(err),
        ErrorKind::Invalid => XError::invalid(format!("{context}: {summary}")).with_source(err),
        ErrorKind::Transient => XError::transient(format!("{context}: {summary}")).with_source(err),
        _ => XError::unavailable(format!("{context}: {summary}")).with_source(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_timeout_to_deadline() {
        let e = map_kafka_err("prod", std::io::Error::other("request timed out"));
        assert_eq!(e.kind(), ErrorKind::DeadlineExceeded);
    }

    #[test]
    fn maps_auth_to_unavailable() {
        let e = map_kafka_err("conn", std::io::Error::other("SASL authentication failed"));
        assert_eq!(e.kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn maps_unknown_topic_to_invalid() {
        let e = map_kafka_err("pub", std::io::Error::other("Unknown topic or partition"));
        assert_eq!(e.kind(), ErrorKind::Invalid);
    }

    #[test]
    fn public_context_is_sanitized_and_real_source_is_preserved() {
        let error =
            map_kafka_err("kafkax connect", std::io::Error::other("SASL authentication failed"));
        assert_eq!(error.kind(), ErrorKind::Unavailable);
        assert!(!error.context().contains("authentication failed"));
        let source = std::error::Error::source(&error).expect("保留真实 source");
        assert!(source.downcast_ref::<std::io::Error>().is_some());
    }
}
