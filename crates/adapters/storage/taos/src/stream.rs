//! 有界查询流：`query_series` 结果按块 yield（慢 consumer 不无限堆积）。

use std::pin::Pin;
use std::task::{Context, Poll};

use canonical::Tick;
use futures_core::Stream;
use kernel::{XError, XResult};

use crate::client::TaosPool;

/// `TimeSeriesStore` 流式查询包装（内部仍受 `max_query_rows` 限制）。
pub struct TaosQueryStream {
    rows: std::vec::IntoIter<Tick>,
    done: bool,
}

impl TaosQueryStream {
    /// 从已物化的行构造流（测试/内部）。
    #[must_use]
    pub fn from_rows(rows: Vec<Tick>) -> Self {
        Self { rows: rows.into_iter(), done: false }
    }

    /// 剩余大致行数。
    #[must_use]
    pub fn remaining_hint(&self) -> usize {
        self.rows.len()
    }
}

impl Stream for TaosQueryStream {
    type Item = XResult<Tick>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        match self.rows.next() {
            Some(t) => Poll::Ready(Some(Ok(t))),
            None => {
                self.done = true;
                Poll::Ready(None)
            }
        }
    }
}

impl TaosPool {
    /// 流式查询：先有界 collect，再按行 yield（遵守 `max_query_rows`）。
    ///
    /// 取消：丢弃 stream 即停止消费；服务端查询在 collect 阶段已完成（REST 限制）。
    pub async fn query_series_stream(
        &self,
        table: &str,
        start: i64,
        end: i64,
    ) -> XResult<TaosQueryStream> {
        use contracts::TimeSeriesStore;
        let rows = self.query_series(table, start, end).await?;
        Ok(TaosQueryStream::from_rows(rows))
    }

    /// 带块大小提示的流（语义同 [`Self::query_series_stream`]；chunk 仅用于 API 完整）。
    pub async fn query_series_stream_chunked(
        &self,
        table: &str,
        start: i64,
        end: i64,
        chunk_hint: usize,
    ) -> XResult<TaosQueryStream> {
        if chunk_hint == 0 {
            return Err(XError::invalid("chunk_hint 必须 ≥ 1"));
        }
        let _ = chunk_hint;
        self.query_series_stream(table, start, end).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimalx::{Decimal, Price};
    use futures_util::StreamExt;

    #[tokio::test]
    async fn stream_yields_all() {
        let rows = vec![
            Tick {
                symbol: "A".into(),
                bid: Price::new(Decimal::try_new(1, 2).unwrap()),
                ask: Price::new(Decimal::try_new(2, 2).unwrap()),
                ts: 1,
            },
            Tick {
                symbol: "B".into(),
                bid: Price::new(Decimal::try_new(3, 2).unwrap()),
                ask: Price::new(Decimal::try_new(4, 2).unwrap()),
                ts: 2,
            },
        ];
        let mut s = TaosQueryStream::from_rows(rows);
        assert_eq!(s.remaining_hint(), 2);
        let a = s.next().await.unwrap().unwrap();
        assert_eq!(a.symbol, "A");
        let b = s.next().await.unwrap().unwrap();
        assert_eq!(b.symbol, "B");
        assert!(s.next().await.is_none());
    }
}
