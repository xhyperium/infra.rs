//! 关停并发集成测试。
//!
//! 验证 `ShutdownSignal` 在并发场景下的正确性：
//! - trigger-before-wait
//! - wait-before-trigger
//! - 多 observer
//! - 高并发无死锁

use kernel::ShutdownSignal;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_concurrent_waiters_all_wake_on_trigger() {
    let (guard, signal) = ShutdownSignal::new();
    let n = 20;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();

    for _ in 0..n {
        let sig = signal.clone();
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            sig.wait();
        }));
    }

    guard.trigger();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_many_signals_one_trigger() {
    let (guard, signal) = ShutdownSignal::new();
    let n = 10;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();

    for _ in 0..n {
        let sig = signal.clone();
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            sig.wait();
            assert!(sig.is_triggered());
        }));
    }

    guard.trigger();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_trigger_before_wait_all_signals() {
    let (guard, signal) = ShutdownSignal::new();
    let s2 = signal.clone();
    let s3 = signal.clone();

    guard.trigger();

    assert!(signal.is_triggered());
    assert!(s2.is_triggered());
    assert!(s3.is_triggered());

    signal.wait();
    s2.wait();
    s3.wait();
}

#[test]
fn test_stress_concurrent() {
    for _ in 0..1000 {
        let (guard, signal) = ShutdownSignal::new();
        let s = signal.clone();

        let h = thread::spawn(move || {
            s.wait();
        });

        thread::yield_now();
        guard.trigger();
        h.join().unwrap();
    }
}
