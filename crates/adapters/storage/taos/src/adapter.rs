//! TDengine 内存 scaffold：`TimeSeriesStore`。

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use canonical::Tick;
use contracts::TimeSeriesStore;
use kernel::{XError, XResult};

pub struct TaosAdapter {
    name: String,
    endpoint: String,
    series: Mutex<HashMap<String, Vec<Tick>>>,
}

impl TaosAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), series: Mutex::new(HashMap::new()) }
    }

    pub fn local() -> Self {
        Self::new("taos-local", "taos://127.0.0.1:6030")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<Tick>>>> {
        self.series.lock().map_err(|e| XError::internal(format!("series lock poisoned: {e}")))
    }
}

#[async_trait]
impl TimeSeriesStore for TaosAdapter {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()> {
        self.lock()?.entry(table.to_string()).or_default().extend(points);
        Ok(())
    }

    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        let guard = self.lock()?;
        let Some(rows) = guard.get(table) else {
            return Ok(Vec::new());
        };
        Ok(rows.iter().filter(|t| t.ts >= start && t.ts <= end).cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimalx::{Decimal, Price};

    fn tick(ts: i64) -> Tick {
        Tick {
            symbol: "BTC".into(),
            bid: Price::new(Decimal::try_new(1, 0).expect("d")),
            ask: Price::new(Decimal::try_new(2, 0).expect("d")),
            ts,
        }
    }

    #[tokio::test]
    async fn write_query_range() {
        let a = TaosAdapter::local();
        a.write_series("t", vec![tick(10), tick(20), tick(30)]).await.expect("write");
        let got = a.query_series("t", 15, 25).await.expect("query");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].ts, 20);
    }
}
