//! ClickHouse 内存 scaffold：`AnalyticsSink`。

use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::AnalyticsSink;
use kernel::{XError, XResult};

pub struct ClickHouseAdapter {
    name: String,
    endpoint: String,
    events: Mutex<Vec<(String, Bytes)>>,
}

impl ClickHouseAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), events: Mutex::new(Vec::new()) }
    }

    pub fn local() -> Self {
        Self::new("clickhouse-local", "http://127.0.0.1:8123")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

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
}
