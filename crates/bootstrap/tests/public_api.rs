//! 公开 API 集成测试：从 crate 外部驱动 shipped 路径。

use async_trait::async_trait;
use bootstrap::{
    Bootstrap, BootstrapError, ContractStoreSet, EvidenceAppender, EvidenceError, ExecutionContext,
    InMemoryEvidenceAppender, Instrumentation, MarketDataContext, NoopInstrumentation,
    TracingInstrumentation, into_xresult,
};
use bytes::Bytes;
use contracts::{EventBus, KeyValueStore};
use futures_core::stream::BoxStream;
use kernel::ErrorKind;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

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
    fn append_named(&self, name: &str) -> Result<bootstrap::AppendReceipt, EvidenceError> {
        Ok(bootstrap::AppendReceipt { name: name.to_string(), seq: 1 })
    }
}

struct Cap(&'static str);

struct ContractProbe;

#[async_trait]
impl KeyValueStore for ContractProbe {
    async fn get(&self, _key: &str) -> kernel::XResult<Option<Vec<u8>>> {
        Ok(Some(b"contract".to_vec()))
    }

    async fn set(
        &self,
        _key: &str,
        _val: Vec<u8>,
        _ttl: Option<std::time::Duration>,
    ) -> kernel::XResult<()> {
        Ok(())
    }
}

#[async_trait]
impl EventBus for ContractProbe {
    async fn publish(&self, _topic: &str, _payload: Bytes) -> kernel::XResult<()> {
        Ok(())
    }

    async fn subscribe(
        &self,
        _topic: &str,
    ) -> kernel::XResult<BoxStream<'static, contracts::BusMessage>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

impl bootstrap::BoundedMarketDataSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::BoundedInstrumentCatalog for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::BoundedKeyValueStore for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::BoundedExecutionVenue for Cap {
    fn venue_id(&self) -> &str {
        self.0
    }
}
impl bootstrap::BoundedAccountSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}
impl bootstrap::BoundedVenueTimeSource for Cap {
    fn label(&self) -> &str {
        self.0
    }
}

#[test]
fn four_build_paths_and_shutdown_ownership() {
    let ctx = Bootstrap::new().build();
    let signal = ctx.shutdown_signal().clone();
    assert!(ctx.platform().evidence().is_none());
    assert!(ctx.graceful_shutdown().expect("build graceful").is_empty());
    assert!(signal.is_triggered());

    let ctx2 = Bootstrap::new().try_build().expect("try_build");
    let signal2 = ctx2.shutdown_signal().clone();
    assert!(ctx2.platform().evidence().is_none());
    assert!(ctx2.graceful_shutdown().expect("try_build graceful").is_empty());
    assert!(signal2.is_triggered());

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
fn app_context_graceful_shutdown_signals_then_drains_lifo_and_keeps_all_results() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let builder = Bootstrap::new();
    let signal = builder.shutdown_signal().clone();

    let order_a = Arc::clone(&order);
    let signal_a = signal.clone();
    let builder = builder
        .register_drain("a", move || {
            assert!(signal_a.is_triggered(), "drain 前必须先触发 signal");
            order_a.lock().expect("order lock").push("a");
            Ok(())
        })
        .expect("register a");
    let order_b = Arc::clone(&order);
    let signal_b = signal.clone();
    let builder = builder
        .register_drain("b", move || {
            assert!(signal_b.is_triggered(), "失败 hook 也必须观察到 signal");
            order_b.lock().expect("order lock").push("b");
            Err(kernel::XError::unavailable("b failed"))
        })
        .expect("register b");
    let order_c = Arc::clone(&order);
    let signal_c = signal.clone();
    let ctx = builder
        .register_drain("c", move || {
            assert!(signal_c.is_triggered(), "drain 前必须先触发 signal");
            order_c.lock().expect("order lock").push("c");
            Ok(())
        })
        .expect("register c")
        .build();

    let results = ctx.graceful_shutdown().expect("graceful shutdown");
    assert!(signal.is_triggered());
    assert_eq!(*order.lock().expect("order lock"), ["c", "b", "a"]);
    assert_eq!(results.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(), ["c", "b", "a"]);
    assert_eq!(results.iter().map(|step| step.ok).collect::<Vec<_>>(), [true, false, true]);
    assert!(results[1].error.as_deref().is_some_and(|error| error.contains("b failed")));
}

#[test]
fn bootstrapped_app_graceful_shutdown_delegates_and_trigger_only_stays_compatible() {
    let graceful_builder = Bootstrap::new();
    let graceful_signal = graceful_builder.shutdown_signal().clone();
    let observed = graceful_signal.clone();
    let app = graceful_builder
        .register_drain("app", move || {
            assert!(observed.is_triggered());
            Ok(())
        })
        .expect("register app")
        .try_build_app()
        .expect("try_build_app");
    let results = app.graceful_shutdown().expect("app graceful shutdown");
    assert!(graceful_signal.is_triggered());
    assert_eq!(results.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(), ["app"]);

    let ran = Arc::new(AtomicUsize::new(0));
    let ran_hook = Arc::clone(&ran);
    let trigger_builder = Bootstrap::new();
    let trigger_signal = trigger_builder.shutdown_signal().clone();
    let app = trigger_builder
        .register_drain("must-not-run", move || {
            ran_hook.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("register trigger-only hook")
        .build_app();
    app.trigger_shutdown();
    assert!(trigger_signal.is_triggered());
    assert_eq!(ran.load(Ordering::SeqCst), 0);

    let ownerless_ran = Arc::new(AtomicUsize::new(0));
    let ownerless_hook = Arc::clone(&ownerless_ran);
    let mut ownerless_builder = Bootstrap::new();
    let ownerless_signal = ownerless_builder.shutdown_signal().clone();
    let _external_guard = ownerless_builder.take_shutdown_guard().expect("取出 app guard");
    let ownerless_app = ownerless_builder
        .register_drain("ownerless-app", move || {
            ownerless_hook.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("register ownerless app hook")
        .build_app();
    let error = ownerless_app.graceful_shutdown().expect_err("ownerless app 必须 fail closed");
    assert!(matches!(&error, BootstrapError::MissingDependency { name: "shutdown_guard" }));
    assert_eq!(error.to_string(), "缺少必需依赖：shutdown_guard");
    assert!(!ownerless_signal.is_triggered());
    assert_eq!(ownerless_ran.load(Ordering::SeqCst), 0);
}

#[test]
fn into_parts_transfers_shutdown_owner_and_context_can_still_drain() {
    let blocked_ran = Arc::new(AtomicUsize::new(0));
    let blocked_hook = Arc::clone(&blocked_ran);
    let blocked_builder = Bootstrap::new();
    let blocked_signal = blocked_builder.shutdown_signal().clone();
    let blocked_app = blocked_builder
        .register_drain("blocked-split", move || {
            blocked_hook.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("register blocked split")
        .build_app();
    let (blocked_ctx, blocked_shutdown) = blocked_app.into_parts();
    let error = blocked_ctx.graceful_shutdown().expect_err("未触发 controller 必须拒绝 drain");
    assert!(matches!(&error, BootstrapError::MissingDependency { name: "shutdown_guard" }));
    assert_eq!(error.to_string(), "缺少必需依赖：shutdown_guard");
    assert_eq!(blocked_ran.load(Ordering::SeqCst), 0);
    assert!(!blocked_signal.is_triggered());
    blocked_shutdown.trigger();
    assert!(blocked_signal.is_triggered());

    let builder = Bootstrap::new();
    let signal = builder.shutdown_signal().clone();
    let app = builder.register_drain("split", || Ok(())).expect("register split").build_app();
    let (ctx, shutdown) = app.into_parts();
    assert!(shutdown.has_guard());
    shutdown.trigger();
    let results = ctx.graceful_shutdown().expect("外部 controller 已触发");
    assert!(signal.is_triggered());
    assert_eq!(results.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(), ["split"]);
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
    let receipt = ctx.platform().evidence().expect("e").append_named("t").expect("append");
    assert_eq!(receipt.seq, 1);

    let mem = Arc::new(InMemoryEvidenceAppender::new());
    let mem_appender: Arc<dyn EvidenceAppender> = mem.clone();
    let ctx = Bootstrap::new().with_evidence(mem_appender).build();
    ctx.platform().evidence().expect("m").append_named("boot").expect("ok");
    assert_eq!(mem.names(), vec!["boot".to_string()]);
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
    let market_source: Arc<dyn bootstrap::BoundedMarketDataSource> = c.clone();
    let catalog: Arc<dyn bootstrap::BoundedInstrumentCatalog> = c.clone();
    let kv: Arc<dyn bootstrap::BoundedKeyValueStore> = c.clone();
    let mdx = MarketDataContext::new(market_source, catalog, kv, platform.clone());
    assert_eq!(mdx.source().label(), "cap");
    let venue: Arc<dyn bootstrap::BoundedExecutionVenue> = c.clone();
    let account: Arc<dyn bootstrap::BoundedAccountSource> = c.clone();
    let venue_time: Arc<dyn bootstrap::BoundedVenueTimeSource> = c;
    let ex = ExecutionContext::new(venue, account, venue_time, platform);
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

#[tokio::test]
async fn formal_contract_store_set_is_callable_from_app_context() {
    let probe = Arc::new(ContractProbe);
    let kv: Arc<dyn KeyValueStore> = probe.clone();
    let event_bus: Arc<dyn EventBus> = probe;
    let contracts = ContractStoreSet::new().with_kv(kv).with_event_bus(event_bus);
    let ctx = Bootstrap::new().with_contract_store_set(contracts).try_build().expect("build");

    let stores = ctx.contract_store_set();
    stores.kv().expect("kv").set("key", b"contract".to_vec(), None).await.expect("set");
    assert_eq!(stores.kv().expect("kv").get("key").await.expect("get"), Some(b"contract".to_vec()));
    stores.event_bus().expect("bus").publish("topic", Bytes::new()).await.expect("publish");
    assert_eq!(stores.wired_count(), 2);
}
