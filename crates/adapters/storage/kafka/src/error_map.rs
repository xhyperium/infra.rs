//! rdkafka 错误 → [`kernel::XError`]。

use kernel::XError;
use rdkafka::error::KafkaError;

/// 将 Kafka 驱动错误映射为语义化 `XError`（不回显凭据）。
pub fn map_kafka_error(op: &str, err: KafkaError) -> XError {
    let ctx = format!("kafkax {op}: {err}");
    // 粗粒度分类：具体 code 不作为稳定 API。
    let s = err.to_string().to_ascii_lowercase();
    if s.contains("timed out") || s.contains("timeout") {
        XError::deadline_exceeded(ctx).with_source(err)
    } else if s.contains("auth") || s.contains("sasl") || s.contains("ssl") {
        XError::unavailable(ctx).with_source(err)
    } else if s.contains("unknown topic") || s.contains("unknown_topic") {
        XError::missing(ctx).with_source(err)
    } else if s.contains("transport") || s.contains("disconnect") || s.contains("broker") {
        XError::transient(ctx).with_source(err)
    } else {
        XError::unavailable(ctx).with_source(err)
    }
}
