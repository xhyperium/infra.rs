//! loom 模型：Shutdown Mutex+Condvar 协议（SPEC §7.6 / §11.2）。
//!
//! 运行：
//! ```text
//! RUSTFLAGS="--cfg loom" cargo test -p kernel --test lifecycle_concurrency_loom --release
//! ```
//!
//! 仅在 `--cfg loom` 下编译，避免默认 `cargo test` 拉入 loom 模型成本。

#![cfg(loom)]

use kernel::ShutdownSignal;
use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::thread;

#[test]
fn loom_trigger_wakes_waiter() {
    loom::model(|| {
        let (guard, signal) = ShutdownSignal::new();
        let s = signal.clone();
        let t = thread::spawn(move || {
            s.wait();
        });
        guard.trigger();
        t.join().unwrap();
        assert!(signal.is_triggered());
    });
}

#[test]
fn loom_two_waiters() {
    loom::model(|| {
        let (guard, signal) = ShutdownSignal::new();
        let seen = Arc::new(AtomicUsize::new(0));
        let mut hs = Vec::new();
        for _ in 0..2 {
            let s = signal.clone();
            let c = Arc::clone(&seen);
            hs.push(thread::spawn(move || {
                s.wait();
                c.fetch_add(1, Ordering::SeqCst);
            }));
        }
        guard.trigger();
        for h in hs {
            h.join().unwrap();
        }
        assert_eq!(seen.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn loom_trigger_before_wait_observer_sees_flag() {
    loom::model(|| {
        let (guard, signal) = ShutdownSignal::new();
        guard.trigger();
        assert!(signal.is_triggered());
        signal.wait();
        let late = signal.clone();
        assert!(late.is_triggered());
        late.wait();
    });
}
