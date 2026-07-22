//! Batch-2 Fake：ObjectStore / TimeSeriesStore / AnalyticsSink / PubSub。

use async_trait::async_trait;
use bytes::Bytes;
use canonical::Tick;
use contracts::{AnalyticsSink, BusMessage, ObjectStore, PubSub, TimeSeriesStore};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};

/// 内存 [`ObjectStore`]。
#[derive(Debug, Default)]
pub struct FakeObjectStore {
    inner: Mutex<HashMap<String, Bytes>>,
}

impl FakeObjectStore {
    /// 新建空对象存储。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 条目数。
    pub fn len(&self) -> XResult<usize> {
        Ok(self.inner.lock().map_err(|_| XError::internal("obj lock 中毒"))?.len())
    }

    /// 是否为空。
    pub fn is_empty(&self) -> XResult<bool> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl ObjectStore for FakeObjectStore {
    async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
        let mut g = self.inner.lock().map_err(|_| XError::internal("对象存储锁中毒"))?;
        g.insert(key.to_string(), data);
        Ok(())
    }

    async fn get_object(&self, key: &str) -> XResult<Bytes> {
        let g = self.inner.lock().map_err(|_| XError::internal("对象存储锁中毒"))?;
        g.get(key).cloned().ok_or_else(|| XError::missing(format!("对象不存在: {key}")))
    }
}

/// 内存 [`TimeSeriesStore`]。
#[derive(Debug, Default)]
pub struct FakeTimeSeriesStore {
    inner: Mutex<HashMap<String, Vec<Tick>>>,
}

impl FakeTimeSeriesStore {
    /// 新建。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TimeSeriesStore for FakeTimeSeriesStore {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()> {
        let mut g = self.inner.lock().map_err(|_| XError::internal("时序存储锁中毒"))?;
        g.entry(table.to_string()).or_default().extend(points);
        Ok(())
    }

    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        let g = self.inner.lock().map_err(|_| XError::internal("时序存储锁中毒"))?;
        let Some(rows) = g.get(table) else {
            return Ok(vec![]);
        };
        Ok(rows.iter().filter(|t| t.ts >= start && t.ts <= end).cloned().collect())
    }
}

/// 内存 [`AnalyticsSink`]。
#[derive(Debug, Default)]
pub struct FakeAnalyticsSink {
    events: Mutex<Vec<(String, Bytes)>>,
}

impl FakeAnalyticsSink {
    /// 新建。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 已写入事件。
    pub fn events(&self) -> XResult<Vec<(String, Bytes)>> {
        Ok(self.events.lock().map_err(|_| XError::internal("分析汇聚锁中毒"))?.clone())
    }
}

#[async_trait]
impl AnalyticsSink for FakeAnalyticsSink {
    async fn sink(&self, event: &str, payload: Bytes) -> XResult<()> {
        self.events
            .lock()
            .map_err(|_| XError::internal("分析汇聚锁中毒"))?
            .push((event.to_string(), payload));
        Ok(())
    }
}

/// 内存 [`PubSub`]（at-most-once；订阅返回已缓冲消息快照流）。
#[derive(Debug, Default)]
pub struct FakePubSub {
    channels: Mutex<HashMap<String, Vec<Bytes>>>,
}

impl FakePubSub {
    /// 新建。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

struct VecStream {
    items: Vec<BusMessage>,
    idx: usize,
}

impl futures_core::Stream for VecStream {
    type Item = BusMessage;
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.idx >= self.items.len() {
            return Poll::Ready(None);
        }
        let item = self.items[self.idx].clone();
        self.idx += 1;
        Poll::Ready(Some(item))
    }
}

#[async_trait]
impl PubSub for FakePubSub {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()> {
        self.channels
            .lock()
            .map_err(|_| XError::internal("发布订阅锁中毒"))?
            .entry(channel.to_string())
            .or_default()
            .push(msg);
        Ok(())
    }

    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = self
            .channels
            .lock()
            .map_err(|_| XError::internal("发布订阅锁中毒"))?
            .get(channel)
            .cloned()
            .unwrap_or_default();
        let items = msgs
            .into_iter()
            .enumerate()
            .map(|(i, payload)| BusMessage { id: format!("{channel}-{i}"), payload })
            .collect();
        Ok(Box::pin(VecStream { items, idx: 0 }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimalx::Decimal;
    use decimalx::Price;
    use futures_core::Stream;
    use std::task::Waker;

    #[tokio::test]
    async fn object_store_roundtrip() {
        let s = FakeObjectStore::new();
        s.put_object("k", Bytes::from_static(b"v")).await.unwrap();
        assert_eq!(s.get_object("k").await.unwrap().as_ref(), b"v");
        assert_eq!(s.len().unwrap(), 1);
    }

    #[tokio::test]
    async fn timeseries_query_range() {
        let s = FakeTimeSeriesStore::new();
        let tick = Tick {
            symbol: "BTC".into(),
            bid: Price::new(Decimal::new(1, 0)),
            ask: Price::new(Decimal::new(2, 0)),
            ts: 100,
        };
        s.write_series("t", vec![tick.clone()]).await.unwrap();
        let q = s.query_series("t", 50, 150).await.unwrap();
        assert_eq!(q.len(), 1);
        assert!(s.query_series("t", 200, 300).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn analytics_and_pubsub() {
        let a = FakeAnalyticsSink::new();
        a.sink("e", Bytes::from_static(b"1")).await.unwrap();
        assert_eq!(a.events().unwrap().len(), 1);

        let p = FakePubSub::new();
        p.pub_message("c", Bytes::from_static(b"m")).await.unwrap();
        let mut stream = p.sub_channel("c").await.unwrap();
        let waker = Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(Some(msg)) => assert_eq!(msg.payload.as_ref(), b"m"),
            _ => panic!("expected msg"),
        }
    }
}
