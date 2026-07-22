//! `IntegrationHarness` 公开 seam 的 fail-closed 合同。

use std::error::Error;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kernel::Timestamp;
use testkit::{IntegrationHarness, ManualClockFault, StepOutcome};

#[test]
fn successful_run_returns_terminal_report() {
    let report = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(10))
        .step_advance_wall("推进墙钟", Duration::from_nanos(5))
        .step("检查", |clock| {
            assert_eq!(clock.snapshot()?.wall().as_unix_nanos(), 15);
            Ok::<(), testkit::ManualClockError>(())
        })
        .run()
        .expect("全部步骤成功");

    assert_eq!(report.records().len(), 2);
    assert!(report.records().iter().all(|record| record.outcome() == StepOutcome::Passed));
    assert_eq!(report.records()[1].wall_after_ns(), Some(15));
    assert_eq!(report.records()[1].before_snapshot().unwrap().wall().as_unix_nanos(), 15);
    assert_eq!(report.records()[1].after_snapshot().unwrap().wall().as_unix_nanos(), 15);
}

#[test]
fn preexisting_wall_fault_stops_before_step_execution() {
    let clock = testkit::ManualClock::new(Timestamp::from_unix_nanos(7));
    clock.set_wall_fault(ManualClockFault::Overflow).unwrap();
    let executed = Arc::new(AtomicBool::new(false));
    let marker = Arc::clone(&executed);

    let error = IntegrationHarness::new(clock)
        .step("不得执行", move |_| {
            marker.store(true, Ordering::SeqCst);
            Ok::<(), io::Error>(())
        })
        .run()
        .expect_err("step 前的 wall fault 必须终止");

    assert!(!executed.load(Ordering::SeqCst));
    assert_eq!(error.kind(), StepOutcome::ObservationFailed);
    let record = &error.report().records()[0];
    assert_eq!(record.before_snapshot().unwrap().wall_fault(), Some(ManualClockFault::Overflow));
    assert!(record.after_snapshot().is_none());
}

#[test]
fn step_error_preserves_source_and_terminal_report() {
    let later_executed = Arc::new(AtomicBool::new(false));
    let marker = Arc::clone(&later_executed);
    let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
        .step("失败步骤", |_| Err(io::Error::other("boom")))
        .step("不得执行", move |_| {
            marker.store(true, Ordering::SeqCst);
            Ok::<(), io::Error>(())
        })
        .run()
        .expect_err("步骤错误必须终止运行");

    assert!(!later_executed.load(Ordering::SeqCst));
    assert_eq!(error.kind(), StepOutcome::Failed);
    assert_eq!(error.step(), "失败步骤");
    assert_eq!(error.source().expect("保留 source").to_string(), "boom");
    assert_eq!(error.report().records().len(), 1);
    assert_eq!(error.report().records()[0].outcome(), StepOutcome::Failed);
}

#[test]
fn non_text_panic_payload_is_classified_without_secondary_panic() {
    let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
        .step("非文本 panic", |_| -> Result<(), io::Error> { std::panic::panic_any(7_u8) })
        .run()
        .expect_err("非文本 panic 必须成为 terminal error");

    assert_eq!(error.kind(), StepOutcome::Panicked);
    assert_eq!(error.detail(), "步骤发生非字符串 panic");
}

#[test]
fn panic_is_captured_as_terminal_failure() {
    let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
        .step("崩溃步骤", |_| -> Result<(), io::Error> { panic!("boom") })
        .run()
        .expect_err("panic 不得在后续重跑中变成成功");

    assert_eq!(error.kind(), StepOutcome::Panicked);
    assert_eq!(error.report().records()[0].outcome(), StepOutcome::Panicked);
    assert!(error.to_string().contains("发生 panic"));
    assert!(!error.to_string().contains("Panicked"));
}

#[test]
fn injected_wall_fault_is_explicit_in_record_without_epoch_sentinel() {
    let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(42))
        .step("注入故障", |clock| {
            clock.set_wall_fault(ManualClockFault::Unavailable)?;
            Ok::<(), testkit::ManualClockError>(())
        })
        .run()
        .expect_err("step 后存在 wall fault 必须 fail closed");

    assert_eq!(error.kind(), StepOutcome::ObservationFailed);
    let record = &error.report().records()[0];
    assert_eq!(record.wall_after_ns(), Some(42));
    assert_eq!(record.wall_fault_after(), Some(ManualClockFault::Unavailable));
    assert!(error.source().is_some());
}
