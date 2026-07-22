//! 可移植契约替面与 ADR-005 注入面。
//!
//! - [`Instrumentation`]：权威定义在 [`contracts::Instrumentation`]；本模块 re-export。
//! - [`NoopInstrumentation`]：静默空实现（测试/显式关闭观测时用）。
//! - Evidence：权威在 [`evidence`]；本模块 re-export trait / error / 内存实现。
//! - 有界 venue 能力：完整 async 平面仍 DEFER；保留最小对象安全形状。

// ── observability（ADR-005）────────────────────────────────────────────────

/// 可观测性注入点（ADR-005）——权威定义在 package/lib `contracts`。
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

// ── evidence（权威在 package/lib evidence）───────────────────────────────

pub use evidence::{AppendReceipt, EvidenceAppender, EvidenceError, InMemoryEvidenceAppender};

// ── venue / storage（有界上下文字段用最小对象安全面）──────────────────────

/// 有界行情源能力（bootstrap 组合根字段用；**不是** [`contracts::MarketDataSource`]）。
///
/// 与 contracts 同名历史面已收敛：本类型加 `Bounded` 前缀以消除静默双平面冲突。
pub trait BoundedMarketDataSource: Send + Sync {
    /// 逻辑标识（测试/诊断）。
    fn label(&self) -> &str;
}

/// 标的目录能力。
pub trait BoundedInstrumentCatalog: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 键值存储能力。
pub trait BoundedKeyValueStore: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 执行场所能力。
pub trait BoundedExecutionVenue: Send + Sync {
    /// 场所 id。
    fn venue_id(&self) -> &str;
}

/// 账户源能力。
pub trait BoundedAccountSource: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

/// 场所时间源能力。
pub trait BoundedVenueTimeSource: Send + Sync {
    /// 逻辑标识。
    fn label(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct StubVenue;
    impl BoundedMarketDataSource for StubVenue {
        fn label(&self) -> &str {
            "md"
        }
    }
    impl BoundedInstrumentCatalog for StubVenue {
        fn label(&self) -> &str {
            "cat"
        }
    }
    impl BoundedKeyValueStore for StubVenue {
        fn label(&self) -> &str {
            "kv"
        }
    }
    impl BoundedExecutionVenue for StubVenue {
        fn venue_id(&self) -> &str {
            "stub"
        }
    }
    impl BoundedAccountSource for StubVenue {
        fn label(&self) -> &str {
            "acct"
        }
    }
    impl BoundedVenueTimeSource for StubVenue {
        fn label(&self) -> &str {
            "time"
        }
    }

    struct FailAppender;
    impl EvidenceAppender for FailAppender {
        fn append_named(&self, _name: &str) -> Result<AppendReceipt, EvidenceError> {
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
        let _: &dyn BoundedMarketDataSource = &s;
        let _: &dyn BoundedInstrumentCatalog = &s;
        let _: &dyn BoundedKeyValueStore = &s;
        let _: &dyn BoundedExecutionVenue = &s;
        let _: &dyn BoundedAccountSource = &s;
        let _: &dyn BoundedVenueTimeSource = &s;
        assert_eq!(s.venue_id(), "stub");
        assert_eq!(BoundedMarketDataSource::label(&s), "md");
        assert_eq!(BoundedInstrumentCatalog::label(&s), "cat");
        assert_eq!(BoundedKeyValueStore::label(&s), "kv");
        assert_eq!(BoundedAccountSource::label(&s), "acct");
        assert_eq!(BoundedVenueTimeSource::label(&s), "time");
    }
}
