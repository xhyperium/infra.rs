//! bootstrap —— L1 启动期依赖组装（R3.1 豁免，spec §4.4，ADR-005 / **ADR-016**）。
//!
//! 组合根（PLAN-GATE-RETIRE-001 Implemented）：
//! - [`PlatformContext`]：横切只读依赖（instrumentation、shutdown_signal、可选 evidence）
//! - [`AppContext`]：已组装只读上下文
//! - [`BootstrappedApp`] + [`ShutdownController`]：build 产物与关停所有权
//! - [`MarketDataContext`] / [`ExecutionContext`]：有界服务上下文（§4）
//! - [`BootstrapError`]：组装错误 → kernel 反应分类
//!
//! **不再**依赖 runtime `gate` crate。禁止 TypeId/Any/字符串 Service Locator。
//!
//! # ADR-005 注入链
//!
//! - trait：[`contracts::Instrumentation`]（经本 crate re-export 为 [`Instrumentation`]）
//! - 默认实现：[`observex::TracingInstrumentation`]（[`Bootstrap::new`]）
//! - 静默替面：[`NoopInstrumentation`]（`with_instrumentation` 可选）
//! - 消费方（如 resiliencx）只依赖 `contracts`，**禁止**依赖 observex
//!
//! evidence：权威在 `xhyper-evidence`（re-export trait + `InMemoryEvidenceAppender`）。
//! 适配器接线：[`StoreSet`]；关停排空：[`AsyncDrain`]（组合根 drain 所有权）。
//! 完整 venue 业务协议仍由 adapters 承载；本 crate 提供有界对象安全接线面。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bounded;
mod drain;
mod error;
mod store_set;
pub mod traits;

pub use bounded::{ExecutionContext, MarketDataContext};
pub use drain::{AsyncDrain, DrainStepResult};
pub use error::{BootstrapError, into_xresult};
pub use observex::TracingInstrumentation;
pub use store_set::StoreSet;
pub use traits::{
    AppendReceipt, BoundedAccountSource, BoundedExecutionVenue, BoundedInstrumentCatalog,
    BoundedKeyValueStore, BoundedMarketDataSource, BoundedVenueTimeSource, EvidenceAppender,
    EvidenceError, InMemoryEvidenceAppender, Instrumentation, NoopInstrumentation,
};

use kernel::{ShutdownGuard, ShutdownSignal};
use observex::TracingInstrumentation as TracingInstr;
use std::sync::Arc;
use traits::{EvidenceAppender as EvidenceAppenderTrait, Instrumentation as InstrumentationTrait};

// ── Typed composition (ADR-016) ────────────────────────────────────────────

/// 平台横切能力（只读）。
///
/// 字段均为确定类型；**禁止** HashMap/Any/TypeId 注册表。
/// `evidence` 为可选：未 `with_evidence` 时为 `None`（开发默认）。
#[derive(Clone)]
pub struct PlatformContext {
    instrumentation: Arc<dyn InstrumentationTrait>,
    shutdown_signal: ShutdownSignal,
    evidence: Option<Arc<dyn EvidenceAppenderTrait>>,
    store_set: StoreSet,
}

impl PlatformContext {
    /// 访问 instrumentation。
    pub fn instrumentation(&self) -> &dyn InstrumentationTrait {
        self.instrumentation.as_ref()
    }

    /// 关停观察端（可 clone）。
    pub fn shutdown_signal(&self) -> &ShutdownSignal {
        &self.shutdown_signal
    }

    /// 可选审计追加器；未注入时 `None`。
    pub fn evidence(&self) -> Option<&dyn EvidenceAppenderTrait> {
        self.evidence.as_deref()
    }

    /// 已接线的适配器集合。
    pub fn store_set(&self) -> &StoreSet {
        &self.store_set
    }
}

/// 关停触发所有权（不可复制；`trigger` 消费 self）。
pub struct ShutdownController {
    guard: Option<ShutdownGuard>,
}

impl ShutdownController {
    /// 取出并触发关停信号。
    pub fn trigger(mut self) {
        if let Some(g) = self.guard.take() {
            g.trigger();
        }
    }

    /// 是否仍持有未触发的 guard（测试用）。
    pub fn has_guard(&self) -> bool {
        self.guard.is_some()
    }
}

/// build 后的应用：上下文 + 关停所有权。
pub struct BootstrappedApp {
    context: AppContext,
    shutdown: ShutdownController,
}

impl BootstrappedApp {
    /// 只读应用上下文。
    pub fn context(&self) -> &AppContext {
        &self.context
    }

    /// 拆成上下文与关停控制器。
    pub fn into_parts(self) -> (AppContext, ShutdownController) {
        (self.context, self.shutdown)
    }

    /// 消费并触发关停。
    pub fn trigger_shutdown(self) {
        self.shutdown.trigger();
    }
}

// ── Builder ────────────────────────────────────────────────────────────────

/// 启动期 builder（R3.1 豁免，spec §4.4）。
pub struct Bootstrap {
    instrumentation: Arc<dyn InstrumentationTrait>,
    evidence: Option<Arc<dyn EvidenceAppenderTrait>>,
    require_evidence: bool,
    shutdown_guard: Option<ShutdownGuard>,
    shutdown_signal: ShutdownSignal,
    store_set: StoreSet,
    drain: AsyncDrain,
}

impl Bootstrap {
    /// 默认 [`TracingInstrumentation`] + 关停信号对；无 evidence。
    ///
    /// 需要静默观测时：`with_instrumentation(NoopInstrumentation::new())`。
    pub fn new() -> Self {
        let (guard, signal) = ShutdownSignal::new();
        Self {
            instrumentation: Arc::new(TracingInstr::new()),
            evidence: None,
            require_evidence: false,
            shutdown_guard: Some(guard),
            shutdown_signal: signal,
            store_set: StoreSet::new(),
            drain: AsyncDrain::new(),
        }
    }

    /// 替换 instrumentation 实现。
    pub fn with_instrumentation<I: InstrumentationTrait + 'static>(mut self, instr: I) -> Self {
        self.instrumentation = Arc::new(instr);
        self
    }

    /// 注入 typed [`EvidenceAppender`]。
    pub fn with_evidence(mut self, appender: Arc<dyn EvidenceAppenderTrait>) -> Self {
        self.evidence = Some(appender);
        self
    }

    /// 要求 build 时必须已注入 evidence（否则 [`BootstrapError::MissingDependency`]）。
    pub fn require_evidence(mut self) -> Self {
        self.require_evidence = true;
        self
    }

    /// 注入完整 [`StoreSet`]（替换既有接线）。
    pub fn with_store_set(mut self, store_set: StoreSet) -> Self {
        self.store_set = store_set;
        self
    }

    /// 注册关停 drain hook（LIFO；见 [`AsyncDrain`]）。
    pub fn register_drain<F>(self, name: impl Into<String>, hook: F) -> Result<Self, BootstrapError>
    where
        F: FnOnce() -> kernel::XResult<()> + Send + 'static,
    {
        self.drain
            .register(name, hook)
            .map_err(|_e| BootstrapError::InvalidConfiguration { name: "drain" })?;
        Ok(self)
    }

    /// 取出关停触发句柄（旧路径；推荐 [`build_app`](Self::build_app)）。
    pub fn take_shutdown_guard(&mut self) -> Option<ShutdownGuard> {
        self.shutdown_guard.take()
    }

    /// 关停观察端。
    pub fn shutdown_signal(&self) -> &ShutdownSignal {
        &self.shutdown_signal
    }

    fn validate(&self) -> Result<(), BootstrapError> {
        if self.require_evidence && self.evidence.is_none() {
            return Err(BootstrapError::MissingDependency { name: "evidence" });
        }
        Ok(())
    }

    fn into_app_context(self) -> AppContext {
        AppContext {
            platform: PlatformContext {
                instrumentation: Arc::clone(&self.instrumentation),
                shutdown_signal: self.shutdown_signal.clone(),
                evidence: self.evidence.clone(),
                store_set: self.store_set.clone(),
            },
            instrumentation: self.instrumentation,
            shutdown_signal: self.shutdown_signal,
            drain: self.drain,
        }
    }

    /// 消费 builder，返回 [`AppContext`]（不绑定 ShutdownController）。
    ///
    /// 未 `require_evidence` 时总是成功。
    /// 若 `require_evidence` 且未注入：**release/debug 均 panic**（fail-closed，infra-s9t.4）。
    /// 可恢复路径请用 [`try_build`](Self::try_build)。
    pub fn build(self) -> AppContext {
        if let Err(e) = self.validate() {
            // PANIC: require_evidence 未满足时禁止静默成功（含 release）
            panic!("Bootstrap::build 失败: {e}；请注入 evidence 或使用 try_build");
        }
        self.into_app_context()
    }

    /// 校验后 build；缺必需依赖返回 [`BootstrapError`]。
    pub fn try_build(self) -> Result<AppContext, BootstrapError> {
        self.validate()?;
        Ok(self.into_app_context())
    }

    /// 消费 builder，返回 [`BootstrappedApp`]。
    pub fn build_app(mut self) -> BootstrappedApp {
        let guard = self.shutdown_guard.take();
        let context = self.build();
        BootstrappedApp { context, shutdown: ShutdownController { guard } }
    }

    /// 校验后返回 [`BootstrappedApp`]。
    pub fn try_build_app(mut self) -> Result<BootstrappedApp, BootstrapError> {
        self.validate()?;
        let guard = self.shutdown_guard.take();
        let context = self.into_app_context();
        Ok(BootstrappedApp { context, shutdown: ShutdownController { guard } })
    }
}

impl Default for Bootstrap {
    fn default() -> Self {
        Self::new()
    }
}

/// 装配完成的应用上下文（spec §4.4 + ADR-016）。
///
/// 只读；无 register/resolve/get 动态 API。
pub struct AppContext {
    platform: PlatformContext,
    instrumentation: Arc<dyn InstrumentationTrait>,
    shutdown_signal: ShutdownSignal,
    drain: AsyncDrain,
}

impl AppContext {
    /// 强类型平台上下文。
    pub fn platform(&self) -> &PlatformContext {
        &self.platform
    }

    /// 克隆平台上下文（供有界上下文组合）。
    pub fn platform_cloned(&self) -> PlatformContext {
        self.platform.clone()
    }

    /// 访问 instrumentation。
    pub fn instrumentation(&self) -> &dyn InstrumentationTrait {
        self.instrumentation.as_ref()
    }

    /// 关停观察端。
    pub fn shutdown_signal(&self) -> &ShutdownSignal {
        &self.shutdown_signal
    }

    /// 适配器 StoreSet。
    pub fn store_set(&self) -> &StoreSet {
        self.platform.store_set()
    }

    /// 关停排空编排器。
    pub fn drain(&self) -> &AsyncDrain {
        &self.drain
    }

    /// 执行组合根 drain（LIFO）。
    pub fn run_drain(&self) -> Vec<DrainStepResult> {
        self.drain.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;
    use std::sync::Mutex;

    #[test]
    fn new_creates_default_instrumentation() {
        let ctx = Bootstrap::new().build();
        // 默认 TracingInstrumentation：无 subscriber 时 tracing 为 no-op，不 panic
        ctx.instrumentation().record_retry("op", 1);
        ctx.instrumentation().record_circuit_open("op");
        ctx.instrumentation().record_circuit_close("op");
        assert!(ctx.platform().evidence().is_none());
    }

    #[test]
    fn default_instrumentation_is_tracing_and_overridable_to_noop() {
        // 构造路径与 Bootstrap::new 一致：TracingInstrumentation 可装箱为 dyn
        let tracing: Arc<dyn InstrumentationTrait> = Arc::new(TracingInstr::new());
        tracing.record_retry("prove_tracing", 1);

        let ctx = Bootstrap::new().with_instrumentation(NoopInstrumentation::new()).build();
        ctx.instrumentation().record_retry("silent", 1);
        ctx.instrumentation().record_circuit_open("silent");
        ctx.instrumentation().record_circuit_close("silent");
    }

    #[test]
    fn contracts_and_resiliencx_share_instrumentation_trait() {
        // 同一 trait 对象可同时满足 contracts / bootstrap / 计数 double
        let (instr, counter) = CountingInstr::paired();
        let boxed: Arc<dyn contracts::Instrumentation> = Arc::new(instr);
        boxed.record_retry("shared", 1);
        assert_eq!(*counter.lock().expect("lock"), 1);
        let ctx = Bootstrap::new()
            .with_instrumentation(CountingInstr { count: Arc::clone(&counter) })
            .build();
        ctx.instrumentation().record_retry("shared", 2);
        assert_eq!(*counter.lock().expect("lock"), 2);
    }

    #[test]
    fn default_equals_new() {
        let a = Bootstrap::new().build();
        let b = Bootstrap::default().build();
        a.instrumentation().record_retry("x", 1);
        b.instrumentation().record_retry("x", 1);
        assert!(!a.platform().shutdown_signal().is_triggered());
        assert!(!b.platform().shutdown_signal().is_triggered());
    }

    #[test]
    fn build_returns_app_context_with_typed_accessors() {
        let ctx = Bootstrap::new().build();
        ctx.instrumentation().record_retry("op", 1);
        ctx.platform().instrumentation().record_retry("op", 2);
        assert!(!ctx.platform().shutdown_signal().is_triggered());
        assert!(!ctx.shutdown_signal().is_triggered());
    }

    struct CountingInstr {
        count: Arc<Mutex<u32>>,
    }

    impl CountingInstr {
        fn paired() -> (Self, Arc<Mutex<u32>>) {
            let count = Arc::new(Mutex::new(0));
            let instr = Self { count: Arc::clone(&count) };
            (instr, count)
        }
    }

    impl InstrumentationTrait for CountingInstr {
        fn record_retry(&self, _op: &str, _attempt: u32) {
            *self.count.lock().expect("counter lock") += 1;
        }
        fn record_circuit_open(&self, _op: &str) {}
        fn record_circuit_close(&self, _op: &str) {}
    }

    struct CountingAppender {
        count: Arc<Mutex<u32>>,
    }

    impl EvidenceAppenderTrait for CountingAppender {
        fn append_named(&self, _name: &str) -> Result<AppendReceipt, EvidenceError> {
            *self.count.lock().expect("counter lock") += 1;
            Err(EvidenceError::DurabilityFailure)
        }
    }

    #[test]
    fn with_instrumentation_replaces_default() {
        let (instr, counter) = CountingInstr::paired();
        let ctx = Bootstrap::new().with_instrumentation(instr).build();
        let instr_ref: &dyn InstrumentationTrait = ctx.instrumentation();
        instr_ref.record_retry("fetch", 1);
        instr_ref.record_retry("fetch", 2);
        assert_eq!(*counter.lock().expect("lock"), 2);
        ctx.platform().instrumentation().record_retry("fetch", 3);
        assert_eq!(*counter.lock().expect("lock"), 3);
    }

    #[test]
    fn with_evidence_injects_typed_appender() {
        let count = Arc::new(Mutex::new(0u32));
        let appender: Arc<dyn EvidenceAppenderTrait> =
            Arc::new(CountingAppender { count: Arc::clone(&count) });
        let ctx = Bootstrap::new().with_evidence(appender).build();
        assert!(ctx.platform().evidence().is_some());
        assert!(Bootstrap::new().build().platform().evidence().is_none());
        assert_eq!(*count.lock().expect("lock"), 0);
        let e = ctx.platform().evidence().expect("injected");
        assert_eq!(e.append_named("probe"), Err(EvidenceError::DurabilityFailure));
        assert_eq!(*count.lock().expect("lock"), 1);
    }

    #[test]
    #[should_panic(expected = "try_build")]
    fn require_evidence_build_panics_without_injection() {
        let _ = Bootstrap::new().require_evidence().build();
    }

    #[test]
    fn require_evidence_fails_try_build_without_injection() {
        let err =
            Bootstrap::new().require_evidence().try_build().err().expect("must miss evidence");
        assert_eq!(err.kind(), ErrorKind::Missing);
        let x: kernel::XError = err.into();
        assert_eq!(x.kind(), ErrorKind::Missing);
    }

    #[test]
    fn require_evidence_ok_with_injection() {
        let appender: Arc<dyn EvidenceAppenderTrait> =
            Arc::new(CountingAppender { count: Arc::new(Mutex::new(0)) });
        let ctx = Bootstrap::new()
            .with_evidence(appender)
            .require_evidence()
            .try_build()
            .expect("evidence present");
        assert!(ctx.platform().evidence().is_some());
    }

    #[test]
    fn try_build_app_require_evidence_fail_and_ok() {
        let err =
            Bootstrap::new().require_evidence().try_build_app().err().expect("must miss evidence");
        assert_eq!(err.kind(), ErrorKind::Missing);

        let appender: Arc<dyn EvidenceAppenderTrait> =
            Arc::new(CountingAppender { count: Arc::new(Mutex::new(0)) });
        let app = Bootstrap::new()
            .with_evidence(appender)
            .require_evidence()
            .try_build_app()
            .expect("ok");
        assert!(app.context().platform().evidence().is_some());
        assert!(app.into_parts().1.has_guard());
    }

    #[test]
    fn instrumentation_injection_pattern_via_trait_object() {
        let (instr, counter) = CountingInstr::paired();
        let ctx = Bootstrap::new().with_instrumentation(instr).build();
        let instr_ref: &dyn InstrumentationTrait = ctx.instrumentation();
        instr_ref.record_retry("op", 1);
        instr_ref.record_circuit_open("op");
        instr_ref.record_circuit_close("op");
        assert_eq!(*counter.lock().expect("lock"), 1);
    }

    #[test]
    fn shutdown_guard_trigger_sets_app_context_signal() {
        let mut b = Bootstrap::new();
        let guard = b.take_shutdown_guard().expect("guard present after new");
        assert!(b.take_shutdown_guard().is_none());
        let ctx = b.build();
        assert!(!ctx.shutdown_signal().is_triggered());
        guard.trigger();
        assert!(ctx.shutdown_signal().is_triggered());
        assert!(ctx.shutdown_signal().clone().is_triggered());
    }

    #[test]
    fn build_app_returns_bootstrapped_with_shutdown_controller() {
        let app = Bootstrap::new().build_app();
        assert!(!app.context().platform().shutdown_signal().is_triggered());
        app.context().instrumentation().record_retry("boot", 1);
        app.context().platform().instrumentation().record_retry("boot", 2);
        let (_ctx, shutdown) = app.into_parts();
        assert!(shutdown.has_guard());
    }

    #[test]
    fn shutdown_controller_trigger_sets_signal() {
        let app = Bootstrap::new().build_app();
        let (ctx, shutdown) = app.into_parts();
        assert!(!ctx.shutdown_signal().is_triggered());
        shutdown.trigger();
        assert!(ctx.shutdown_signal().is_triggered());
        assert!(ctx.platform().shutdown_signal().is_triggered());
    }

    #[test]
    fn bootstrapped_app_trigger_shutdown() {
        let app = Bootstrap::new().build_app();
        // take signal clone before consuming app
        let signal = app.context().shutdown_signal().clone();
        assert!(!signal.is_triggered());
        app.trigger_shutdown();
        assert!(signal.is_triggered());
    }

    #[test]
    fn shutdown_controller_trigger_without_guard_is_noop() {
        // take guard out of builder so build_app gets None guard
        let mut b = Bootstrap::new();
        let _external = b.take_shutdown_guard();
        let app = b.build_app();
        let (ctx, shutdown) = app.into_parts();
        assert!(!shutdown.has_guard());
        shutdown.trigger(); // no-op path
        assert!(!ctx.shutdown_signal().is_triggered());
    }

    #[test]
    fn platform_context_typed_only() {
        let ctx = Bootstrap::new().build();
        let p = ctx.platform();
        let _ = p.instrumentation();
        let _ = p.shutdown_signal();
        let _ = p.evidence();
    }

    #[test]
    fn platform_clone_for_bounded_context() {
        let ctx = Bootstrap::new().build();
        let p = ctx.platform_cloned();
        assert!(!p.shutdown_signal().is_triggered());
        let _p2 = p.clone();
    }

    #[test]
    fn bootstrap_shutdown_signal_accessor() {
        let b = Bootstrap::new();
        assert!(!b.shutdown_signal().is_triggered());
    }

    #[test]
    fn drop_shutdown_controller_without_trigger_leaves_signal_idle() {
        let app = Bootstrap::new().build_app();
        let (ctx, shutdown) = app.into_parts();
        assert!(shutdown.has_guard());
        drop(shutdown);
        assert!(!ctx.shutdown_signal().is_triggered());
    }

    #[test]
    fn try_build_success_without_require() {
        let ctx = Bootstrap::new().try_build().expect("ok");
        assert!(ctx.platform().evidence().is_none());
    }

    #[test]
    fn store_set_wired_through_build() {
        struct Stub;
        impl traits::BoundedKeyValueStore for Stub {
            fn label(&self) -> &str {
                "redis-stub"
            }
        }
        let set = StoreSet::new().with_kv(Arc::new(Stub) as Arc<dyn traits::BoundedKeyValueStore>);
        let ctx = Bootstrap::new().with_store_set(set).build();
        assert_eq!(ctx.store_set().wired_count(), 1);
        assert_eq!(ctx.platform().store_set().kv().expect("kv").label(), "redis-stub");
    }

    #[test]
    fn drain_registered_and_run_on_context() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let b = Bootstrap::new()
            .register_drain("step", || {
                N.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("reg");
        let ctx = b.build();
        assert_eq!(ctx.drain().len(), 1);
        let results = ctx.run_drain();
        assert_eq!(results.len(), 1);
        assert!(results[0].ok);
        assert_eq!(N.load(Ordering::SeqCst), 1);
    }
}
