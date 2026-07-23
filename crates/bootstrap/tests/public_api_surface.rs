//! bootstrap 公开面：ShutdownController、AppContext 访问器、into_xerror。

use bootstrap::{
    Bootstrap, BootstrapError, InMemoryEvidenceAppender, NoopInstrumentation, into_xresult,
};
use kernel::ErrorKind;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn app_context_accessors_and_shutdown_controller() {
    let app = Bootstrap::new().build_app();
    assert!(!app.context().platform().shutdown_signal().is_triggered());
    assert!(app.context().platform().evidence().is_none());
    assert!(app.context().platform_cloned().evidence().is_none());
    app.context().instrumentation().record_retry("s", 1);

    let (ctx, sc) = app.into_parts();
    assert!(sc.has_guard());
    sc.trigger();
    assert!(ctx.shutdown_signal().is_triggered());
}

#[test]
fn take_shutdown_guard_and_error_into_xerror() {
    let mut b = Bootstrap::new();
    let guard = b.take_shutdown_guard().expect("guard once");
    assert!(b.take_shutdown_guard().is_none());
    assert!(!b.shutdown_signal().is_triggered());
    guard.trigger();
    assert!(b.shutdown_signal().is_triggered());

    let e = BootstrapError::MissingDependency { name: "x" };
    assert_eq!(e.into_xerror().kind(), ErrorKind::Missing);
    assert_eq!(
        into_xresult::<u8>(Err(BootstrapError::InvalidConfiguration { name: "y" }))
            .unwrap_err()
            .kind(),
        ErrorKind::Invalid
    );

    let mem = Arc::new(InMemoryEvidenceAppender::new());
    let app = Bootstrap::new()
        .with_instrumentation(NoopInstrumentation::new())
        .with_evidence(mem)
        .build_app();
    assert!(app.context().platform().evidence().is_some());
}

#[test]
fn extracted_guard_is_not_recreated_and_ownerless_graceful_fails_closed() {
    let ran = Arc::new(AtomicUsize::new(0));
    let hook_ran = Arc::clone(&ran);
    let mut builder = Bootstrap::new();
    let signal = builder.shutdown_signal().clone();
    let guard = builder.take_shutdown_guard().expect("唯一 guard");
    let ctx = builder
        .register_drain("without-owner", move || {
            hook_ran.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("register drain")
        .build();

    let error = ctx.graceful_shutdown().expect_err("ownerless graceful 必须失败");
    assert!(
        matches!(&error, BootstrapError::MissingDependency { name: "shutdown_guard" }),
        "必须返回精确的 shutdown_guard Missing"
    );
    assert_eq!(error.to_string(), "缺少必需依赖：shutdown_guard");
    assert!(!signal.is_triggered(), "build 不得凭空重建已移出的 guard");
    assert_eq!(ran.load(Ordering::SeqCst), 0, "失败路径不得执行 hook");
    guard.trigger();
    assert!(signal.is_triggered());
}

#[test]
fn externally_triggered_ownerless_context_can_drain() {
    let ran = Arc::new(AtomicUsize::new(0));
    let hook_ran = Arc::clone(&ran);
    let mut builder = Bootstrap::new();
    let signal = builder.shutdown_signal().clone();
    let guard = builder.take_shutdown_guard().expect("唯一 guard");
    let ctx = builder
        .register_drain("external-owner", move || {
            hook_ran.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("register drain")
        .build();

    guard.trigger();
    let results = ctx.graceful_shutdown().expect("外部 guard 已触发，应允许 drain");
    assert!(signal.is_triggered());
    assert_eq!(ran.load(Ordering::SeqCst), 1);
    assert_eq!(
        results.iter().map(|step| step.name.as_str()).collect::<Vec<_>>(),
        ["external-owner"]
    );
}
