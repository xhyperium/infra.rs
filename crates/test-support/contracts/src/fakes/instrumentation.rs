//! Instrumentation Recording。

use contracts::Instrumentation;
use kernel::XResult;
use std::sync::{Arc, Mutex};

/// 单条可观测记录（Recording）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstrEvent {
    /// 重试。
    Retry {
        /// 操作名。
        op: String,
        /// 尝试序号。
        attempt: u32,
    },
    /// 熔断打开。
    CircuitOpen {
        /// 操作名。
        op: String,
    },
    /// 熔断关闭。
    CircuitClose {
        /// 操作名。
        op: String,
    },
}

/// 可观察的 [`Instrumentation`]：将调用写入内存向量。
#[derive(Debug, Default, Clone)]
pub struct RecordingInstrumentation {
    events: Arc<Mutex<Vec<InstrEvent>>>,
}

impl RecordingInstrumentation {
    /// 新建空记录器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 已记录事件的快照。
    pub fn snapshot(&self) -> XResult<Vec<InstrEvent>> {
        let g = self.events.lock().map_err(|_| kernel::XError::internal("instr lock 中毒"))?;
        Ok(g.clone())
    }

    /// 清空记录。
    pub fn clear(&self) -> XResult<()> {
        let mut g = self.events.lock().map_err(|_| kernel::XError::internal("instr lock 中毒"))?;
        g.clear();
        Ok(())
    }
}

impl Instrumentation for RecordingInstrumentation {
    fn record_retry(&self, op: &str, attempt: u32) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::Retry { op: op.to_string(), attempt });
        }
    }

    fn record_circuit_open(&self, op: &str) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::CircuitOpen { op: op.to_string() });
        }
    }

    fn record_circuit_close(&self, op: &str) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::CircuitClose { op: op.to_string() });
        }
    }
}
