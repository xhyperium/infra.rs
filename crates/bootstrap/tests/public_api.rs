//! 公开 API 集成测试：从 crate 外部驱动 shipped 路径。

use bootstrap::{
    Bootstrap, BootstrapError, EvidenceAppender, EvidenceError, ExecutionContext, Instrumentation,
    MarketDataContext, NoopInstrumentation, TracingInstrumentation, into_xresult,
};
use kernel::ErrorKind;
use std::sync::{Arc, Mutex};

struct CountingInstr {
    n: Arc<Mutex<u32>>,
}

impl Instrumentation for CountingInstr {
    fn record_retry(&self, _op: &str, _attempt: u32) {
        *self.n.lock().expect("lock") += 1;
    }
    fn record_circuit_open(&self, _op: &str) {}
    fn record_circuit_close(&self, _op: &str) {}
}

struct ProbeAppender;

impl EvidenceAppender for ProbeAppender {
    fn append_named(&self, _name: &str) -> Result<(), EvidenceError> {
        Ok(())
    }
}

struct Cap(&'static str);

impl bootstrap::MarketDataSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::InstrumentCatalog for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::KeyValueStore for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::ExecutionVenue for Cap {
    fn venue_id(&self) -> &str {
        self.0
    }
}
impl bootstrap::AccountSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::VenueTimeSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}

#[test]
fn four_build_paths_and_shutdown_ownership() {
    let ctx = Bootstrap::new().build();
    assert!(!ctx.shutdown_signal().is_triggered());
    assert!(ctx.platform().evidence().is_none());

    let ctx2 = Bootstrap::new().try_build().expect("try_build");
    assert!(ctx2.platform().evidence().is_none());

    let app = Bootstrap::new().build_app();
    assert!(!app.context().shutdown_signal().is_triggered());
    let (ctx3, sc) = app.into_parts();
    sc.trigger();
    assert!(ctx3.shutdown_signal().is_triggered());

    let app2 = Bootstrap::new().try_build_app().expect("try_build_app");
    let signal = app2.context().shutdown_signal().clone();
    app2.trigger_shutdown();
    assert!(signal.is_triggered());
}

#[test]
fn custom_instrumentation_and_evidence_modes() {
    let n = Arc::new(Mutex::new(0u32));
    let ctx = Bootstrap::new().with_instrumentation(CountingInstr { n: Arc::clone(&n) }).build();
    ctx.instrumentation().record_retry("op", 1);
    assert_eq!(*n.lock().expect("lock"), 1);

    let err = Bootstrap::new().require_evidence().try_build().err().expect("must miss evidence");
    assert_eq!(err.kind(), ErrorKind::Missing);

    let appender: Arc<dyn EvidenceAppender> = Arc::new(ProbeAppender);
    let ctx = Bootstrap::new().with_evidence(appender).require_evidence().try_build().expect("ok");
    assert!(ctx.platform().evidence().is_some());
    ctx.platform().evidence().expect("e").append_named("t").expect("append");
}

#[test]
fn error_mapping_and_into_xresult() {
    for (e, kind) in [
        (BootstrapError::MissingDependency { name: "evidence" }, ErrorKind::Missing),
        (BootstrapError::InvalidConfiguration { name: "x" }, ErrorKind::Invalid),
        (
            BootstrapError::DependencyUnavailable {
                name: "redis",
                source: Box::new(std::io::Error::other("down")),
            },
            ErrorKind::Unavailable,
        ),
    ] {
        assert_eq!(e.kind(), kind);
        let x: kernel::XError = e.into();
        assert_eq!(x.kind(), kind);
    }

    assert_eq!(into_xresult(Ok(1u8)).unwrap(), 1);
    let _ = NoopInstrumentation::new();
    let _ = TracingInstrumentation::new();
}

#[test]
fn default_is_tracing_instrumentation_contracts_trait() {
    // ADR-005：Bootstrap::new 默认 observex 实现；trait 权威在 contracts
    let ctx = Bootstrap::new().build();
    let instr: &dyn Instrumentation = ctx.instrumentation();
    instr.record_retry("public_api_boot", 1);
    instr.record_circuit_open("public_api_boot");
    instr.record_circuit_close("public_api_boot");

    let silent = Bootstrap::new().with_instrumentation(NoopInstrumentation::new()).build();
    silent.instrumentation().record_retry("silent", 1);

    // 显式注入 TracingInstrumentation 与默认路径同型
    let explicit = Bootstrap::new().with_instrumentation(TracingInstrumentation::new()).build();
    explicit.instrumentation().record_retry("explicit", 1);
}

#[test]
fn bounded_contexts_from_platform_clone() {
    let platform = Bootstrap::new().build().platform_cloned();
    let c = Arc::new(Cap("cap"));
    let mdx = MarketDataContext::new(
        Arc::clone(&c) as Arc<dyn bootstrap::MarketDataSource>,
        Arc::clone(&c) as Arc<dyn bootstrap::InstrumentCatalog>,
        Arc::clone(&c) as Arc<dyn bootstrap::KeyValueStore>,
        platform.clone(),
    );
    assert_eq!(mdx.source().label(), "cap");
    let ex = ExecutionContext::new(
        Arc::clone(&c) as Arc<dyn bootstrap::ExecutionVenue>,
        Arc::clone(&c) as Arc<dyn bootstrap::AccountSource>,
        Arc::clone(&c) as Arc<dyn bootstrap::VenueTimeSource>,
        platform,
    );
    assert_eq!(ex.venue().venue_id(), "cap");
}

#[test]
fn no_service_locator_surface() {
    // 编译期存在性：公开 API 仅 typed composition；本测试通过构造证明无 gate 参与路径。
    let app = Bootstrap::default().build_app();
    let _ = app.context().platform().instrumentation();
    let _ = app.context().platform().evidence();
    assert!(app.into_parts().1.has_guard());
}
