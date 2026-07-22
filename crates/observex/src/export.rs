//! OTEL 兼容导出面（进程内缓冲；非完整 OpenTelemetry SDK）。

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use contracts::Instrumentation;

/// 导出错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportError {
    /// 导出器已关闭。
    Shutdown,
    /// 内部不可用。
    Unavailable,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shutdown => write!(f, "telemetry exporter shutdown"),
            Self::Unavailable => write!(f, "telemetry exporter unavailable"),
        }
    }
}
impl std::error::Error for ExportError {}

/// 简化 span 事件（OTEL-compatible 字段子集）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanEvent {
    /// 操作名。
    pub name: String,
    /// 开始时间（unix ms，可选近似）。
    pub start_unix_ms: u64,
    /// 属性（扁平）。
    pub attributes: Vec<(String, String)>,
}

/// 简化 metric 事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricEvent {
    /// 指标名。
    pub name: String,
    /// 数值。
    pub value: i64,
    /// 属性。
    pub attributes: Vec<(String, String)>,
}

/// 遥测导出器（OTEL-compatible 表面，非完整协议）。
pub trait TelemetryExporter: Send + Sync {
    /// 导出 spans。
    fn export_spans(&self, spans: &[SpanEvent]) -> Result<(), ExportError>;
    /// 导出 metrics。
    fn export_metrics(&self, metrics: &[MetricEvent]) -> Result<(), ExportError>;
    /// 刷新缓冲（实现定义：清空或持久化）。
    fn flush(&self) -> Result<(), ExportError>;
    /// 关闭；后续 export 应失败；幂等。
    fn shutdown(&self) -> Result<(), ExportError>;
}

#[derive(Debug, Default)]
struct MemExportState {
    spans: Vec<SpanEvent>,
    metrics: Vec<MetricEvent>,
    /// flush 后累计的 span 数（持久化计数，便于测试）。
    flushed_spans: usize,
    flushed_metrics: usize,
    shutdown: bool,
}

/// 进程内导出器：export 写入缓冲；flush 清空并累计；shutdown 幂等。
#[derive(Debug, Default)]
pub struct InMemoryExporter {
    inner: Mutex<MemExportState>,
}

impl InMemoryExporter {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前缓冲 spans。
    #[must_use]
    pub fn buffered_spans(&self) -> Vec<SpanEvent> {
        self.inner.lock().map(|g| g.spans.clone()).unwrap_or_default()
    }

    /// 当前缓冲 metrics。
    #[must_use]
    pub fn buffered_metrics(&self) -> Vec<MetricEvent> {
        self.inner.lock().map(|g| g.metrics.clone()).unwrap_or_default()
    }

    /// 已 flush 的 span 累计数。
    #[must_use]
    pub fn flushed_span_count(&self) -> usize {
        self.inner.lock().map(|g| g.flushed_spans).unwrap_or(0)
    }

    /// 已 flush 的 metric 累计数。
    #[must_use]
    pub fn flushed_metric_count(&self) -> usize {
        self.inner.lock().map(|g| g.flushed_metrics).unwrap_or(0)
    }

    /// 是否已 shutdown。
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.inner.lock().map(|g| g.shutdown).unwrap_or(true)
    }
}

impl TelemetryExporter for InMemoryExporter {
    fn export_spans(&self, spans: &[SpanEvent]) -> Result<(), ExportError> {
        let mut g = self.inner.lock().map_err(|_| ExportError::Unavailable)?;
        if g.shutdown {
            return Err(ExportError::Shutdown);
        }
        g.spans.extend(spans.iter().cloned());
        Ok(())
    }

    fn export_metrics(&self, metrics: &[MetricEvent]) -> Result<(), ExportError> {
        let mut g = self.inner.lock().map_err(|_| ExportError::Unavailable)?;
        if g.shutdown {
            return Err(ExportError::Shutdown);
        }
        g.metrics.extend(metrics.iter().cloned());
        Ok(())
    }

    fn flush(&self) -> Result<(), ExportError> {
        let mut g = self.inner.lock().map_err(|_| ExportError::Unavailable)?;
        if g.shutdown {
            return Err(ExportError::Shutdown);
        }
        g.flushed_spans = g.flushed_spans.saturating_add(g.spans.len());
        g.flushed_metrics = g.flushed_metrics.saturating_add(g.metrics.len());
        g.spans.clear();
        g.metrics.clear();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ExportError> {
        let mut g = self.inner.lock().map_err(|_| ExportError::Unavailable)?;
        g.shutdown = true;
        g.spans.clear();
        g.metrics.clear();
        Ok(())
    }
}

/// 包装内层 [`Instrumentation`]，将事件同时写入导出器缓冲。
pub struct ExportingInstrumentation<I, E> {
    inner: I,
    exporter: E,
}

impl<I, E> ExportingInstrumentation<I, E> {
    /// 构造。
    #[must_use]
    pub fn new(inner: I, exporter: E) -> Self {
        Self { inner, exporter }
    }

    /// 内层 instrumentation。
    #[must_use]
    pub fn inner(&self) -> &I {
        &self.inner
    }

    /// 导出器。
    #[must_use]
    pub fn exporter(&self) -> &E {
        &self.exporter
    }
}

impl<I, E> ExportingInstrumentation<I, E>
where
    E: TelemetryExporter,
{
    /// 刷新导出器。
    pub fn flush(&self) -> Result<(), ExportError> {
        self.exporter.flush()
    }

    /// 关闭导出器（幂等）。
    pub fn shutdown(&self) -> Result<(), ExportError> {
        self.exporter.shutdown()
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

impl<I, E> Instrumentation for ExportingInstrumentation<I, E>
where
    I: Instrumentation,
    E: TelemetryExporter,
{
    fn record_retry(&self, op: &str, attempt: u32) {
        self.inner.record_retry(op, attempt);
        let span = SpanEvent {
            name: format!("retry:{op}"),
            start_unix_ms: now_unix_ms(),
            attributes: vec![("attempt".into(), attempt.to_string())],
        };
        let metric = MetricEvent {
            name: "retry".into(),
            value: i64::from(attempt),
            attributes: vec![("op".into(), op.to_string())],
        };
        let _ = self.exporter.export_spans(std::slice::from_ref(&span));
        let _ = self.exporter.export_metrics(std::slice::from_ref(&metric));
    }

    fn record_circuit_open(&self, op: &str) {
        self.inner.record_circuit_open(op);
        let span = SpanEvent {
            name: format!("circuit_open:{op}"),
            start_unix_ms: now_unix_ms(),
            attributes: Vec::new(),
        };
        let _ = self.exporter.export_spans(std::slice::from_ref(&span));
    }

    fn record_circuit_close(&self, op: &str) {
        self.inner.record_circuit_close(op);
        let span = SpanEvent {
            name: format!("circuit_close:{op}"),
            start_unix_ms: now_unix_ms(),
            attributes: Vec::new(),
        };
        let _ = self.exporter.export_spans(std::slice::from_ref(&span));
    }
}

impl TelemetryExporter for &InMemoryExporter {
    fn export_spans(&self, spans: &[SpanEvent]) -> Result<(), ExportError> {
        (*self).export_spans(spans)
    }
    fn export_metrics(&self, metrics: &[MetricEvent]) -> Result<(), ExportError> {
        (*self).export_metrics(metrics)
    }
    fn flush(&self) -> Result<(), ExportError> {
        (*self).flush()
    }
    fn shutdown(&self) -> Result<(), ExportError> {
        (*self).shutdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CountingInstrumentation;

    #[test]
    fn record_export_flush_shutdown() {
        let counting = CountingInstrumentation::new();
        let exporter = InMemoryExporter::new();
        let instr = ExportingInstrumentation::new(&counting, &exporter);
        instr.record_retry("op", 1);
        instr.record_circuit_open("op");
        instr.record_circuit_close("op");
        assert_eq!(counting.retry_count(), 1);
        assert!(!exporter.buffered_spans().is_empty());
        assert!(!exporter.buffered_metrics().is_empty());
        instr.flush().unwrap();
        assert!(exporter.buffered_spans().is_empty());
        assert!(exporter.flushed_span_count() >= 3);
        instr.shutdown().unwrap();
        instr.shutdown().unwrap(); // idempotent
        assert!(exporter.is_shutdown());
        assert_eq!(exporter.export_spans(&[]), Err(ExportError::Shutdown));
        assert!(format!("{}", ExportError::Shutdown).contains("shutdown"));
    }
}
