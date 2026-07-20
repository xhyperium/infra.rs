//! ManualClock 并发合同（SPEC-TESTKIT-002 §13.4）。
//!
//! 线性化点：成功获取 state mutex 后的状态读写。
//! 控制路径的多次 API 调用是独立临界区，故 wall 与 mono 可暂时不一致；
//! 单次 `snapshot` 保证三字段在同一临界区读取（无字段级撕裂）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use kernel::{Clock, Timestamp};
use testkit::{ManualClock, ManualClockFault};

#[test]
fn multi_thread_read_and_control_no_data_race() {
    let clock = Arc::new(ManualClock::new(Timestamp::from_unix_nanos(0)));
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let clock = Arc::clone(&clock);
        let stop = Arc::clone(&stop);
        handles.push(thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let snap = clock.snapshot().expect("snapshot");
                // 字段类型完整可读；fault 与 wall 值组合合法
                let _ = snap.wall().as_unix_nanos();
                let _ = snap.monotonic_elapsed().as_nanos();
                let _ = snap.wall_fault();
                let _ = clock.now();
                let _ = clock.monotonic();
            }
        }));
    }

    for i in 0..500 {
        clock.advance_wall(Duration::from_nanos(1)).expect("advance wall");
        clock.advance_monotonic(Duration::from_nanos(1)).expect("advance mono");
        if i % 25 == 0 {
            clock.set_wall_fault(ManualClockFault::Overflow).expect("fault");
            let s = clock.snapshot().expect("mid");
            assert_eq!(s.wall_fault(), Some(ManualClockFault::Overflow));
            // fault 不改 wall
            assert_eq!(s.wall().as_unix_nanos(), (i + 1) as i64);
            clock.clear_wall_fault().expect("clear");
        }
    }

    stop.store(true, Ordering::Relaxed);
    for h in handles {
        h.join().expect("join");
    }

    let s = clock.snapshot().expect("final");
    assert_eq!(s.wall().as_unix_nanos(), 500);
    assert_eq!(s.monotonic_elapsed(), Duration::from_nanos(500));
    assert!(s.wall_fault().is_none());
}

#[test]
fn arc_shared_timeline() {
    let clock = Arc::new(ManualClock::new(Timestamp::from_unix_nanos(1)));
    let c2 = Arc::clone(&clock);
    c2.advance_wall(Duration::from_nanos(9)).unwrap();
    assert_eq!(clock.now().unwrap().as_unix_nanos(), 10);
}
