//! ClickHouse 内存 scaffold：`AnalyticsSink`（feature `scaffold`）。

use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::AnalyticsSink;
use kernel::{XError, XResult};

/// 进程内事件缓冲（**非**真实 ClickHouse I/O）。
pub struct ClickHouseAdapter {
    name: String,
    endpoint: String,
    events: Mutex<Vec<(String, Bytes)>>,
}

impl ClickHouseAdapter {
    /// 构造命名 scaffold。
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), events: Mutex::new(Vec::new()) }
    }

    /// 本地默认端点。
    pub fn local() -> Self {
        Self::new("clickhouse-local", "http://127.0.0.1:8123")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 端点。
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// scaffold 观测：已写入事件数。
    pub fn event_count(&self) -> XResult<usize> {
        Ok(self
            .events
            .lock()
            .map_err(|e| XError::internal(format!("events lock poisoned: {e}")))?
            .len())
    }
}

#[async_trait]
impl AnalyticsSink for ClickHouseAdapter {
    async fn sink(&self, event: &str, payload: Bytes) -> XResult<()> {
        self.events
            .lock()
            .map_err(|e| XError::internal(format!("events lock poisoned: {e}")))?
            .push((event.to_string(), payload));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sink_records() {
        let a = ClickHouseAdapter::local();
        a.sink("e", Bytes::from_static(b"p")).await.expect("sink");
        assert_eq!(a.event_count().expect("count"), 1);
    }

    // R1：多次 sink 必须累加而不是覆盖，且 name/endpoint 访问器与构造参数一致。
    #[tokio::test]
    async fn sink_accumulates_multiple_events_and_exposes_identity() {
        let a = ClickHouseAdapter::new("custom-name", "http://example.invalid:8123");
        assert_eq!(a.name(), "custom-name");
        assert_eq!(a.endpoint(), "http://example.invalid:8123");

        for i in 0..5 {
            a.sink(&format!("event-{i}"), Bytes::from(format!("payload-{i}"))).await.expect("sink");
        }
        assert_eq!(a.event_count().expect("count"), 5);
    }

    #[tokio::test]
    async fn local_defaults_to_expected_endpoint() {
        let a = ClickHouseAdapter::local();
        assert_eq!(a.name(), "clickhouse-local");
        assert_eq!(a.endpoint(), "http://127.0.0.1:8123");
        assert_eq!(a.event_count().expect("count"), 0);
    }
}
