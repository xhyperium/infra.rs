//! Kafka 消息与 `BusMessage.id` 编码。

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

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

/// 应用层稳定分区：相同 key 在固定 `partitions` 下映射到同一分区。
///
/// 这是 **本库** 的显式路由辅助，不是 broker sticky partitioner / murmur2 协议保证。
/// `partitions <= 0` 时返回 `0`。
#[must_use]
pub fn partition_for_key(key: &[u8], partitions: i32) -> i32 {
    if partitions <= 1 {
        return 0;
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut h);
    (h.finish() % partitions as u64) as i32
}

/// 生产侧交付回执。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delivery {
    /// 分区。
    pub partition: i32,
    /// offset。
    pub offset: i64,
}

/// 生产侧记录（key / headers / payload / 分区）。
///
/// 经 [`crate::KafkaProducer::publish_record`] 发往 broker。
#[derive(Debug, Clone)]
pub struct PublishRecord {
    /// topic。
    pub topic: String,
    /// 目标分区（调用方负责；可用 [`partition_for_key`]）。
    pub partition: i32,
    /// 载荷。
    pub payload: Bytes,
    /// 可选 key。
    pub key: Option<Bytes>,
    /// 可选 headers（顺序无关；wire 为 map）。
    pub headers: BTreeMap<String, Bytes>,
}

impl PublishRecord {
    /// 仅 payload 到指定分区。
    #[must_use]
    pub fn payload(topic: impl Into<String>, partition: i32, payload: Bytes) -> Self {
        Self { topic: topic.into(), partition, payload, key: None, headers: BTreeMap::new() }
    }

    /// 带 key。
    #[must_use]
    pub fn with_key(mut self, key: Bytes) -> Self {
        self.key = Some(key);
        self
    }

    /// 插入单个 header。
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: Bytes) -> Self {
        self.headers.insert(name.into(), value);
        self
    }
}

/// 消费到的消息（Kafka 专属面；带 partition/offset/key/headers）。
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
    /// 记录 headers。
    pub headers: BTreeMap<String, Bytes>,
    /// 记录时间戳（broker/record 侧）。
    pub timestamp: Option<DateTime<Utc>>,
}

impl KafkaMessage {
    /// 编码为合同 `BusMessage.id`。
    #[must_use]
    pub fn bus_id(&self) -> String {
        encode_bus_id(&self.topic, self.partition, self.offset)
    }

    /// 读取 header；不存在返回 `None`。
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&Bytes> {
        self.headers.get(name)
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
            headers: BTreeMap::from([("h".into(), Bytes::from_static(b"v"))]),
            timestamp: None,
        };
        assert_eq!(msg.bus_id(), encode_bus_id("t", 2, 9));
        assert_eq!(msg.key.as_ref().map(|k| k.as_ref()), Some(&b"k"[..]));
        assert_eq!(msg.header("h").map(|b| b.as_ref()), Some(&b"v"[..]));
    }

    #[test]
    fn partition_for_key_is_stable() {
        assert_eq!(partition_for_key(b"same", 3), partition_for_key(b"same", 3));
        assert_eq!(partition_for_key(b"x", 1), 0);
        assert_eq!(partition_for_key(b"x", 0), 0);
    }

    #[test]
    fn publish_record_builder_sets_key_and_header() {
        let r = PublishRecord::payload("t", 1, Bytes::from_static(b"p"))
            .with_key(Bytes::from_static(b"k"))
            .header("x-selfcheck", Bytes::from_static(b"1"));
        assert_eq!(r.partition, 1);
        assert_eq!(r.key.as_ref().map(|k| k.as_ref()), Some(&b"k"[..]));
        assert_eq!(r.headers.get("x-selfcheck").map(|v| v.as_ref()), Some(&b"1"[..]));
    }
}
