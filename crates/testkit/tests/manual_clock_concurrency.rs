//! ManualClock 并发合同（SPEC-TESTKIT-002 §13.4）。
//!
//! 线性化点：成功获取 state mutex 后的状态读写。
//! 控制路径的多次 API 调用是独立临界区，故 wall 与 mono 可暂时不一致；
//! 单次 `snapshot` 保证三字段在同一临界区读取（无字段级撕裂）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use kernel::{Clock, Timestamp};
use testkit::{ManualClock, ManualClockFault};

#[test]
fn multi_thread_read_and_control_no_data_race() {
    let clock = Arc::new(ManualClock::new(Timestamp::from_unix_nanos(0)));
    let stop = Arc::new(AtomicBool::new(false));
    let start = Arc::new(Barrier::new(9));
    let first_read = Arc::new(Barrier::new(9));
    let first_control_observed = Arc::new(Barrier::new(9));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let clock = Arc::clone(&clock);
        let stop = Arc::clone(&stop);
        let start = Arc::clone(&start);
        let first_read = Arc::clone(&first_read);
        let first_control_observed = Arc::clone(&first_control_observed);
        handles.push(thread::spawn(move || {
            start.wait();
            let initial = clock.snapshot().expect("initial snapshot");
            assert_snapshot_relation(initial.wall().as_unix_nanos(), initial.monotonic_elapsed());
            first_read.wait();
            loop {
                let snap = clock.snapshot().expect("控制开始后的 snapshot");
                assert_snapshot_relation(snap.wall().as_unix_nanos(), snap.monotonic_elapsed());
                if snap.wall().as_unix_nanos() >= 1 {
                    break;
                }
            }
            first_control_observed.wait();
            while !stop.load(Ordering::Acquire) {
                let snap = clock.snapshot().expect("snapshot");
                assert_snapshot_relation(snap.wall().as_unix_nanos(), snap.monotonic_elapsed());
                let _ = snap.wall_fault();
                let _ = clock.now();
                let _ = clock.monotonic();
            }
        }));
    }

    start.wait();
    first_read.wait();

    clock.advance_wall(Duration::from_nanos(1)).expect("首次推进 wall");
    clock.advance_monotonic(Duration::from_nanos(1)).expect("首次推进 mono");
    first_control_observed.wait();

    for i in 1_i64..500 {
        clock.advance_wall(Duration::from_nanos(1)).expect("advance wall");
        clock.advance_monotonic(Duration::from_nanos(1)).expect("advance mono");
        if i % 25 == 0 {
            clock.set_wall_fault(ManualClockFault::Overflow).expect("fault");
            let s = clock.snapshot().expect("mid");
            assert_eq!(s.wall_fault(), Some(ManualClockFault::Overflow));
            // fault 不改 wall
            assert_eq!(s.wall().as_unix_nanos(), i + 1);
            clock.clear_wall_fault().expect("clear");
        }
    }

    stop.store(true, Ordering::Release);
    for h in handles {
        h.join().expect("join");
    }

    let s = clock.snapshot().expect("final");
    assert_eq!(s.wall().as_unix_nanos(), 500);
    assert_eq!(s.monotonic_elapsed(), Duration::from_nanos(500));
    assert!(s.wall_fault().is_none());
}

fn assert_snapshot_relation(wall_ns: i64, monotonic_elapsed: Duration) {
    let monotonic_ns = i64::try_from(monotonic_elapsed.as_nanos()).expect("测试域内可转为 i64");
    assert!(wall_ns >= monotonic_ns, "墙钟不得落后于本测试的单调推进: {wall_ns}/{monotonic_ns}");
    assert!(
        wall_ns - monotonic_ns <= 1,
        "单锁快照只能观察到成对推进之间的一个步长: {wall_ns}/{monotonic_ns}"
    );
}

#[test]
fn arc_shared_timeline() {
    let clock = Arc::new(ManualClock::new(Timestamp::from_unix_nanos(1)));
    let c2 = Arc::clone(&clock);
    c2.advance_wall(Duration::from_nanos(9)).unwrap();
    assert_eq!(clock.now().unwrap().as_unix_nanos(), 10);
}

#[test]
fn multiple_controllers_advance_wall_without_lost_updates() {
    const CONTROLLERS: usize = 8;
    const STEPS: usize = 500;

    let clock = Arc::new(ManualClock::new(Timestamp::from_unix_nanos(0)));
    let start = Arc::new(Barrier::new(CONTROLLERS + 1));
    let mut handles = Vec::with_capacity(CONTROLLERS);

    for _ in 0..CONTROLLERS {
        let clock = Arc::clone(&clock);
        let start = Arc::clone(&start);
        handles.push(thread::spawn(move || {
            start.wait();
            for _ in 0..STEPS {
                clock.advance_wall(Duration::from_nanos(1)).expect("并发推进不得丢失更新");
            }
        }));
    }

    start.wait();
    for handle in handles {
        handle.join().expect("控制线程不得 panic");
    }

    let expected = i64::try_from(CONTROLLERS * STEPS).expect("测试规模可表示为 i64");
    assert_eq!(clock.snapshot().expect("最终快照").wall().as_unix_nanos(), expected);
}
