//! 可移植契约替面与 ADR-005 注入面。
//!
//! - [`Instrumentation`]：权威定义在 [`contracts::Instrumentation`]；本模块 re-export。
//! - [`NoopInstrumentation`]：静默空实现（测试/显式关闭观测时用）。
//! - Evidence / 有界 venue 能力：完整 monorepo 平面仍 DEFER；保留最小对象安全形状。

// ── observability（ADR-005）────────────────────────────────────────────────

/// 可观测性注入点（ADR-005）——权威定义在 `xhyper-contracts`。
pub use contracts::Instrumentation;

/// 静默 no-op instrumentation。
///
/// 默认生产路径使用 [`observex::TracingInstrumentation`]（见 [`crate::Bootstrap::new`]）。
/// 需要零观测副作用时，通过 `with_instrumentation(NoopInstrumentation::new())` 注入。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopInstrumentation;

impl NoopInstrumentation {
    /// 构造默认实例。
    pub const fn new() -> Self {
        Self
    }
}

impl Instrumentation for NoopInstrumentation {
    fn record_retry(&self, _op: &str, _attempt: u32) {}
    fn record_circuit_open(&self, _op: &str) {}
    fn record_circuit_close(&self, _op: &str) {}
}

// ── evidence ───────────────────────────────────────────────────────────────

/// 证据追加错误（可移植最小面；非 monorepo `xhyper-evidence` 全量枚举）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// 持久化失败。
    DurabilityFailure,
    /// 存储/后端不可用。
    Unavailable,
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurabilityFailure => write!(f, "evidence durability failure"),
            Self::Unavailable => write!(f, "evidence backend unavailable"),
        }
    }
}

impl std::error::Error for EvidenceError {}

/// 审计证据追加器（对象安全；完整 AppendRequest/Receipt API DEFER 至 evidence crate）。
pub trait EvidenceAppender: Send + Sync {
    /// 按逻辑名追加一条审计事件（可移植探针；非 wire 兼容 monorepo）。
    fn append_named(&self, name: &str) -> Result<(), EvidenceError>;
}

// ── venue / storage（有界上下文字段用最小对象安全面）──────────────────────

/// 行情源能力（完整 async stream API DEFER）。
pub trait MarketDataSource: Send + Sync {
    /// 逻辑标识（测试/诊断）。
    fn label(&self) -> &str;
}

/// 标的目录能力。
pub trait InstrumentCatalog: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 键值存储能力。
pub trait KeyValueStore: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 执行场所能力。
pub trait ExecutionVenue: Send + Sync {
    /// 场所 id。
    fn venue_id(&self) -> &str;
}

/// 账户源能力。
pub trait AccountSource: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 场所时间源能力。
pub trait VenueTimeSource: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct StubVenue;
    impl MarketDataSource for StubVenue {
        fn label(&self) -> &str {
            "md"
        }
    }
    impl InstrumentCatalog for StubVenue {
        fn label(&self) -> &str {
            "cat"
        }
    }
    impl KeyValueStore for StubVenue {
        fn label(&self) -> &str {
            "kv"
        }
    }
    impl ExecutionVenue for StubVenue {
        fn venue_id(&self) -> &str {
            "stub"
        }
    }
    impl AccountSource for StubVenue {
        fn label(&self) -> &str {
            "acct"
        }
    }
    impl VenueTimeSource for StubVenue {
        fn label(&self) -> &str {
            "time"
        }
    }

    struct FailAppender;
    impl EvidenceAppender for FailAppender {
        fn append_named(&self, _name: &str) -> Result<(), EvidenceError> {
            Err(EvidenceError::DurabilityFailure)
        }
    }

    #[test]
    fn noop_instrumentation_is_object_safe() {
        let n = NoopInstrumentation::new();
        let d: &dyn Instrumentation = &n;
        d.record_retry("op", 1);
        d.record_circuit_open("op");
        d.record_circuit_close("op");
        let _: NoopInstrumentation = NoopInstrumentation;
    }

    #[test]
    fn contracts_instrumentation_is_same_trait() {
        // bootstrap::Instrumentation ≡ contracts::Instrumentation（类型别名 re-export）
        fn accept(_: &dyn Instrumentation) {}
        fn accept_contracts(_: &dyn contracts::Instrumentation) {}
        let n = NoopInstrumentation::new();
        accept(&n);
        accept_contracts(&n);
    }

    #[test]
    fn evidence_error_display_and_trait_object() {
        assert_eq!(EvidenceError::DurabilityFailure.to_string(), "evidence durability failure");
        assert_eq!(EvidenceError::Unavailable.to_string(), "evidence backend unavailable");
        let a: Arc<dyn EvidenceAppender> = Arc::new(FailAppender);
        assert_eq!(a.append_named("x"), Err(EvidenceError::DurabilityFailure));
    }

    #[test]
    fn bounded_capability_stubs_object_safe() {
        let s = StubVenue;
        let _: &dyn MarketDataSource = &s;
        let _: &dyn InstrumentCatalog = &s;
        let _: &dyn KeyValueStore = &s;
        let _: &dyn ExecutionVenue = &s;
        let _: &dyn AccountSource = &s;
        let _: &dyn VenueTimeSource = &s;
        assert_eq!(s.venue_id(), "stub");
        assert_eq!(MarketDataSource::label(&s), "md");
        assert_eq!(InstrumentCatalog::label(&s), "cat");
        assert_eq!(KeyValueStore::label(&s), "kv");
        assert_eq!(AccountSource::label(&s), "acct");
        assert_eq!(VenueTimeSource::label(&s), "time");
    }
}
