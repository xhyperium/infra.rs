//! Kafka 消息与 `BusMessage.id` 编码。

use bytes::Bytes;
use chrono::{DateTime, Utc};

/// 编码 `BusMessage.id`：`topic/partition/offset`。
#[must_use]
pub fn encode_bus_id(topic: &str, partition: i32, offset: i64) -> String {
    format!("{topic}/{partition}/{offset}")
}

/// 解析 `topic/partition/offset`；失败返回 `None`。
#[must_use]
pub fn parse_bus_id(id: &str) -> Option<(&str, i32, i64)> {
    let mut parts = id.rsplitn(3, '/');
    let offset = parts.next()?.parse().ok()?;
    let partition = parts.next()?.parse().ok()?;
    let topic = parts.next()?;
    if topic.is_empty() {
        return None;
    }
    // rsplitn 反转了顺序：offset, partition, topic
    Some((topic, partition, offset))
}

/// 生产侧交付回执。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delivery {
    /// 分区。
    pub partition: i32,
    /// offset。
    pub offset: i64,
}

/// 消费到的消息（Kafka 专属面；带 partition/offset）。
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    /// topic。
    pub topic: String,
    /// 分区。
    pub partition: i32,
    /// offset。
    pub offset: i64,
    /// 载荷。
    pub payload: Bytes,
    /// 可选 key。
    pub key: Option<Bytes>,
    /// 记录时间戳（broker/record 侧；无 headers 公共面）。
    pub timestamp: Option<DateTime<Utc>>,
}

impl KafkaMessage {
    /// 编码为合同 `BusMessage.id`。
    #[must_use]
    pub fn bus_id(&self) -> String {
        encode_bus_id(&self.topic, self.partition, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_bus_id() {
        let id = encode_bus_id("orders", 3, 42);
        assert_eq!(id, "orders/3/42");
        let (t, p, o) = parse_bus_id(&id).expect("parse");
        assert_eq!(t, "orders");
        assert_eq!(p, 3);
        assert_eq!(o, 42);
    }

    #[test]
    fn topic_with_slash() {
        let id = encode_bus_id("a/b", 0, 1);
        let (t, p, o) = parse_bus_id(&id).expect("parse");
        assert_eq!(t, "a/b");
        assert_eq!(p, 0);
        assert_eq!(o, 1);
    }

    #[test]
    fn parse_bus_id_rejects_malformed() {
        assert!(parse_bus_id("").is_none());
        assert!(parse_bus_id("onlytopic").is_none());
        assert!(parse_bus_id("/0/1").is_none());
        assert!(parse_bus_id("t/x/1").is_none());
    }

    #[test]
    fn bus_id_method_matches_encoder() {
        let msg = KafkaMessage {
            topic: "t".into(),
            partition: 2,
            offset: 9,
            payload: Bytes::from_static(b"p"),
            key: Some(Bytes::from_static(b"k")),
            timestamp: None,
        };
        assert_eq!(msg.bus_id(), encode_bus_id("t", 2, 9));
        assert_eq!(msg.key.as_ref().map(|k| k.as_ref()), Some(&b"k"[..]));
    }
}
