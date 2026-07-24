//! 最小 consumer：组装 `BootstrappedApp` 并执行优雅关停。

use bootstrap::Bootstrap;

fn main() {
    let app = Bootstrap::new().build_app();
    assert!(!app.context().shutdown_signal().is_triggered(), "新建应用不应处于关停状态");
    app.context().instrumentation().record_retry("example", 1);
    let signal = app.context().shutdown_signal().clone();
    let results = app.graceful_shutdown().expect("应用持有 shutdown guard");
    assert!(signal.is_triggered());
    assert!(results.is_empty());
    println!("bootstrap 示例：关停已触发");
}
