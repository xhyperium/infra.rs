//! bootstrap 公开面：ShutdownController、AppContext 访问器、into_xerror。

use bootstrap::{
    Bootstrap, BootstrapError, InMemoryEvidenceAppender, NoopInstrumentation, into_xresult,
};
use kernel::ErrorKind;
use std::sync::Arc;

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
