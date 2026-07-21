//! 最小 consumer：组装 `BootstrappedApp` 并触发关停。

use bootstrap::Bootstrap;

fn main() {
    let app = Bootstrap::new().build_app();
    assert!(!app.context().shutdown_signal().is_triggered(), "fresh app must not be shut down");
    app.context().instrumentation().record_retry("example", 1);
    let (ctx, shutdown) = app.into_parts();
    assert!(shutdown.has_guard());
    shutdown.trigger();
    assert!(ctx.shutdown_signal().is_triggered());
    println!("bootstrap-consumer: ok shutdown_triggered=true");
}
