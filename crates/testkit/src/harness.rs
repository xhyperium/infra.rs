//! 基于 [`crate::ManualClock`] 的确定性多步场景 runner。
//!
//! 本模块只编排进程内测试步骤；不负责网络、外部进程、真实服务或凭据。

use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use kernel::{BoxError, Timestamp};

use crate::{ManualClock, ManualClockFault, ManualClockSnapshot};

/// 单步的终态。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// 步骤成功，且 runner 成功取得终态快照。
    Passed,
    /// 步骤返回错误。
    Failed,
    /// 步骤发生 panic；runner 已捕获并终止后续步骤。
    Panicked,
    /// runner 无法取得终态快照。
    ObservationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HarnessFailureKind {
    Failed,
    Panicked,
    ObservationFailed,
}

impl HarnessFailureKind {
    const fn outcome(self) -> StepOutcome {
        match self {
            Self::Failed => StepOutcome::Failed,
            Self::Panicked => StepOutcome::Panicked,
            Self::ObservationFailed => StepOutcome::ObservationFailed,
        }
    }

    fn display_zh(self) -> &'static str {
        match self {
            Self::Failed => "失败",
            Self::Panicked => "发生 panic",
            Self::ObservationFailed => "终态观测失败",
        }
    }
}

/// 单步执行记录。
///
/// 字段保持私有，消费者必须通过只读 getter 观测记录：
///
/// ```compile_fail
/// use kernel::Timestamp;
/// use testkit::IntegrationHarness;
///
/// let report = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
///     .step("完成", |_| Ok::<(), std::io::Error>(()))
///     .run()
///     .expect("步骤成功");
/// let _ = &report.records()[0].name;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    name: String,
    outcome: StepOutcome,
    detail: Option<String>,
    before_snapshot: Option<ManualClockSnapshot>,
    after_snapshot: Option<ManualClockSnapshot>,
}

impl StepRecord {
    /// 步骤名。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 步骤终态。
    pub const fn outcome(&self) -> StepOutcome {
        self.outcome
    }

    /// 失败详情；成功时为 `None`。
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// 步骤执行前的可用快照；观测失败且无快照时为 `None`。
    pub const fn before_snapshot(&self) -> Option<ManualClockSnapshot> {
        self.before_snapshot
    }

    /// 步骤执行后的可用快照；step 前失败或观测无快照时为 `None`。
    pub const fn after_snapshot(&self) -> Option<ManualClockSnapshot> {
        self.after_snapshot
    }

    /// 步骤结束后保存的墙钟；快照失败时为 `None`，绝不使用 epoch 0 哨兵。
    pub const fn wall_after_ns(&self) -> Option<i64> {
        match self.after_snapshot {
            Some(snapshot) => Some(snapshot.wall().as_unix_nanos()),
            None => None,
        }
    }

    /// 步骤结束后的墙钟 fault 注入状态。
    pub const fn wall_fault_after(&self) -> Option<ManualClockFault> {
        match self.after_snapshot {
            Some(snapshot) => snapshot.wall_fault(),
            None => None,
        }
    }
}

/// 场景执行成功后的终态报告。
#[derive(Debug)]
pub struct HarnessReport {
    clock: ManualClock,
    records: Vec<StepRecord>,
}

impl HarnessReport {
    /// 最终手动时钟。
    pub const fn clock(&self) -> &ManualClock {
        &self.clock
    }

    /// 按执行顺序排列的步骤记录。
    pub fn records(&self) -> &[StepRecord] {
        &self.records
    }

    /// 断言全部步骤成功且数量符合预期。
    ///
    /// # Panics
    ///
    /// 步骤数量不等于 `expected_count`，或任一步骤不是成功终态时 panic。
    pub fn assert_all_ok(&self, expected_count: usize) {
        assert_eq!(self.records.len(), expected_count, "步骤数量不符: {:?}", self.records);
        for record in &self.records {
            assert_eq!(
                record.outcome,
                StepOutcome::Passed,
                "步骤 {} 未成功: {:?}",
                record.name,
                record.detail
            );
        }
    }

    /// 断言最终保存墙钟等于给定 Unix ns。
    ///
    /// # Panics
    ///
    /// 最终快照不可读，或墙钟不等于 `expected` 时 panic。
    pub fn assert_wall_ns(&self, expected: i64) {
        let snapshot = self.clock.snapshot().expect("测试报告中的时钟快照必须可读");
        assert_eq!(snapshot.wall().as_unix_nanos(), expected, "墙钟纳秒不符");
    }

    /// 断言最终单调流逝时间。
    ///
    /// # Panics
    ///
    /// 最终快照不可读，或单调流逝时间不等于 `expected` 时 panic。
    pub fn assert_monotonic_elapsed(&self, expected: Duration) {
        let snapshot = self.clock.snapshot().expect("测试报告中的时钟快照必须可读");
        assert_eq!(snapshot.monotonic_elapsed(), expected, "单调流逝时间不符");
    }
}

/// 场景执行的终止错误。
///
/// 错误始终保留 terminal [`HarnessReport`]；步骤或快照错误可通过
/// [`std::error::Error::source`] 继续访问。
#[derive(thiserror::Error)]
#[error("测试步骤 {step} {}: {detail}", .kind.display_zh())]
pub struct HarnessRunError {
    step: String,
    kind: HarnessFailureKind,
    detail: String,
    report: Box<HarnessReport>,
    #[source]
    source: Option<BoxError>,
}

impl HarnessRunError {
    fn new(
        step: String,
        kind: HarnessFailureKind,
        detail: String,
        report: HarnessReport,
        source: Option<BoxError>,
    ) -> Self {
        Self { step, kind, detail, report: Box::new(report), source }
    }

    /// 失败步骤名。
    pub fn step(&self) -> &str {
        &self.step
    }

    /// 失败终态。
    pub const fn kind(&self) -> StepOutcome {
        self.kind.outcome()
    }

    /// 失败详情。
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// 失败时的 terminal 报告。
    pub fn report(&self) -> &HarnessReport {
        self.report.as_ref()
    }

    /// 消耗错误并取出 terminal 报告。
    pub fn into_report(self) -> HarnessReport {
        *self.report
    }
}

impl fmt::Debug for HarnessRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HarnessRunError")
            .field("step", &self.step)
            .field("kind", &self.kind.outcome())
            .field("detail", &self.detail)
            .field("report", &self.report)
            .field("source", &self.source.as_ref().map(|_| "..."))
            .finish()
    }
}

struct PendingStep {
    name: String,
    run: Box<dyn FnOnce(&ManualClock) -> Result<(), BoxError> + Send>,
}

impl fmt::Debug for PendingStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingStep").field("name", &self.name).finish()
    }
}

/// 消费型确定性场景 builder。
///
/// [`Self::run`] 消耗 builder，因此运行后无法追加步骤或重跑；错误路径同样返回
/// terminal report，避免 panic/错误后误报成功。
///
/// ```compile_fail
/// use kernel::Timestamp;
/// use testkit::IntegrationHarness;
///
/// let harness = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0));
/// let _ = harness.run();
/// let _ = harness.step("运行后追加", |_| Ok::<(), std::io::Error>(()));
/// ```
///
/// ```compile_fail
/// use kernel::Timestamp;
/// use testkit::IntegrationHarness;
///
/// let harness = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
///     .step("失败", |_| Err(std::io::Error::other("失败")));
/// let _ = harness.run();
/// let _ = harness.run();
/// ```
///
/// ```compile_fail
/// use kernel::Timestamp;
/// use testkit::IntegrationHarness;
///
/// let harness = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
///     .step("成功", |_| Ok::<(), std::io::Error>(()));
/// let _ = harness.run();
/// let _ = harness.run();
/// ```
#[derive(Debug)]
pub struct IntegrationHarness {
    clock: ManualClock,
    steps: Vec<PendingStep>,
}

impl IntegrationHarness {
    /// 用已有手动时钟构造 runner。
    pub fn new(clock: ManualClock) -> Self {
        Self { clock, steps: Vec::new() }
    }

    /// 以初始墙钟构造 runner。
    pub fn with_wall(wall: Timestamp) -> Self {
        Self::new(ManualClock::new(wall))
    }

    /// 登记命名步骤并返回 builder。
    pub fn step<F, E>(mut self, name: impl Into<String>, run: F) -> Self
    where
        F: FnOnce(&ManualClock) -> Result<(), E> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.steps.push(PendingStep {
            name: name.into(),
            run: Box::new(move |clock| run(clock).map_err(|error| -> BoxError { Box::new(error) })),
        });
        self
    }

    /// 登记推进墙钟的步骤。
    pub fn step_advance_wall(self, name: impl Into<String>, delta: Duration) -> Self {
        self.step(name, move |clock| clock.advance_wall(delta).map(|_| ()))
    }

    /// 登记推进单调钟的步骤。
    pub fn step_advance_monotonic(self, name: impl Into<String>, delta: Duration) -> Self {
        self.step(name, move |clock| clock.advance_monotonic(delta).map(|_| ()))
    }

    /// 按登记顺序执行全部步骤；首个失败或 panic 立即终止。
    ///
    /// # Errors
    ///
    /// step 前后快照不可用、存在墙钟 fault、step 返回错误或发生 panic 时返回
    /// [`HarnessRunError`]；错误始终保留 terminal report，并在存在原始错误时保留 source chain。
    pub fn run(mut self) -> Result<HarnessReport, HarnessRunError> {
        let steps = std::mem::take(&mut self.steps);
        let mut records = Vec::with_capacity(steps.len());

        for pending in steps {
            let name = pending.name;
            let before = match observe_snapshot(&self.clock) {
                Ok(snapshot) => snapshot,
                Err((snapshot, source)) => {
                    let detail = source.to_string();
                    records.push(StepRecord {
                        name: name.clone(),
                        outcome: StepOutcome::ObservationFailed,
                        detail: Some(detail.clone()),
                        before_snapshot: snapshot,
                        after_snapshot: None,
                    });
                    let report = HarnessReport { clock: self.clock, records };
                    return Err(HarnessRunError::new(
                        name,
                        HarnessFailureKind::ObservationFailed,
                        detail,
                        report,
                        Some(source),
                    ));
                }
            };
            let result = catch_unwind(AssertUnwindSafe(|| (pending.run)(&self.clock)));
            let after = observe_snapshot(&self.clock);
            match result {
                Ok(Ok(())) => {
                    if let Err((snapshot, source)) = after {
                        let detail = source.to_string();
                        records.push(StepRecord {
                            name: name.clone(),
                            outcome: StepOutcome::ObservationFailed,
                            detail: Some(detail.clone()),
                            before_snapshot: Some(before),
                            after_snapshot: snapshot,
                        });
                        let report = HarnessReport { clock: self.clock, records };
                        return Err(HarnessRunError::new(
                            name,
                            HarnessFailureKind::ObservationFailed,
                            detail,
                            report,
                            Some(source),
                        ));
                    }
                    records.push(StepRecord {
                        name,
                        outcome: StepOutcome::Passed,
                        detail: None,
                        before_snapshot: Some(before),
                        after_snapshot: after.ok(),
                    });
                }
                Ok(Err(source)) => {
                    let mut detail = source.to_string();
                    let (failure, after_snapshot, source) = match after {
                        Ok(snapshot) => (HarnessFailureKind::Failed, Some(snapshot), source),
                        Err((snapshot, observation)) => {
                            detail.push_str("；终态观测失败: ");
                            detail.push_str(&observation.to_string());
                            let combined: BoxError = Box::new(CombinedObservationError {
                                observation,
                                preceding: source,
                            });
                            (HarnessFailureKind::ObservationFailed, snapshot, combined)
                        }
                    };
                    let kind = failure.outcome();
                    records.push(StepRecord {
                        name: name.clone(),
                        outcome: kind,
                        detail: Some(detail.clone()),
                        before_snapshot: Some(before),
                        after_snapshot,
                    });
                    let report = HarnessReport { clock: self.clock, records };
                    return Err(HarnessRunError::new(name, failure, detail, report, Some(source)));
                }
                Err(payload) => {
                    let mut detail = panic_detail(payload.as_ref());
                    let (failure, after_snapshot, source) = match after {
                        Ok(snapshot) => (HarnessFailureKind::Panicked, Some(snapshot), None),
                        Err((snapshot, observation)) => {
                            detail.push_str("；终态观测失败: ");
                            detail.push_str(&observation.to_string());
                            (HarnessFailureKind::ObservationFailed, snapshot, Some(observation))
                        }
                    };
                    let kind = failure.outcome();
                    records.push(StepRecord {
                        name: name.clone(),
                        outcome: kind,
                        detail: Some(detail.clone()),
                        before_snapshot: Some(before),
                        after_snapshot,
                    });
                    let report = HarnessReport { clock: self.clock, records };
                    return Err(HarnessRunError::new(name, failure, detail, report, source));
                }
            }
        }

        Ok(HarnessReport { clock: self.clock, records })
    }
}

fn observe_snapshot(
    clock: &ManualClock,
) -> Result<ManualClockSnapshot, (Option<ManualClockSnapshot>, BoxError)> {
    let snapshot = clock
        .snapshot()
        .map_err(|error| (None, Box::<dyn std::error::Error + Send + Sync>::from(error)))?;
    if let Some(fault) = snapshot.wall_fault() {
        return Err((Some(snapshot), Box::new(WallFaultObservation(fault))));
    }
    Ok(snapshot)
}

#[derive(Debug, thiserror::Error)]
#[error("手动时钟墙钟存在故障注入: {0:?}")]
struct WallFaultObservation(ManualClockFault);

#[derive(Debug, thiserror::Error)]
#[error("终态观测失败: {observation}；此前步骤错误: {preceding}")]
struct CombinedObservationError {
    observation: BoxError,
    #[source]
    preceding: BoxError,
}

fn panic_detail(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "步骤发生非字符串 panic".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn marker_step(
        executed: Arc<AtomicBool>,
    ) -> impl FnOnce(&ManualClock) -> Result<(), io::Error> + Send {
        move |_| {
            executed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn snapshot_synchronization_before_step_is_terminal() {
        let clock = ManualClock::new(Timestamp::from_unix_nanos(0));
        clock.poison_state_for_test();
        let executed = Arc::new(AtomicBool::new(false));

        let error = IntegrationHarness::new(clock)
            .step("不得执行", marker_step(Arc::clone(&executed)))
            .run()
            .expect_err("step 前快照同步失败必须终止");

        assert!(!executed.load(Ordering::SeqCst));
        assert_eq!(error.kind(), StepOutcome::ObservationFailed);
        let record = &error.report().records()[0];
        assert!(record.before_snapshot().is_none());
        assert_eq!(record.wall_after_ns(), None);
        assert_eq!(record.wall_fault_after(), None);
        assert!(error.to_string().contains("终态观测失败"));
        assert!(error.source().is_some());
    }

    #[test]
    fn snapshot_synchronization_after_step_is_terminal() {
        let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
            .step("毒化状态锁", |clock| {
                clock.poison_state_for_test();
                Ok::<(), io::Error>(())
            })
            .run()
            .expect_err("step 后快照同步失败必须终止");

        assert_eq!(error.kind(), StepOutcome::ObservationFailed);
        let record = &error.report().records()[0];
        assert!(record.before_snapshot().is_some());
        assert!(record.after_snapshot().is_none());
        assert!(error.source().is_some());
    }

    #[test]
    fn public_record_and_error_getters_preserve_terminal_state() {
        let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(3))
            .step("失败步骤", |_| Err(io::Error::other("单元错误")))
            .run()
            .expect_err("步骤必须失败");

        assert_eq!(error.step(), "失败步骤");
        assert_eq!(error.kind(), StepOutcome::Failed);
        assert_eq!(error.detail(), "单元错误");
        assert!(error.to_string().contains("失败"));
        let debug = format!("{error:?}");
        assert!(debug.contains("HarnessRunError"));
        assert!(debug.contains("source: Some(\"...\")"));
        let record = &error.report().records()[0];
        assert_eq!(record.name(), "失败步骤");
        assert_eq!(record.detail(), Some("单元错误"));
        assert_eq!(record.outcome(), StepOutcome::Failed);
        assert!(record.before_snapshot().is_some());
        assert!(record.after_snapshot().is_some());
        assert_eq!(record.wall_after_ns(), Some(3));
        assert_eq!(record.wall_fault_after(), None);

        let report = error.into_report();
        assert_eq!(report.clock().snapshot().expect("报告时钟").wall().as_unix_nanos(), 3);
        assert_eq!(report.records().len(), 1);
    }

    #[test]
    fn public_success_helpers_cover_complete_harness_surface() {
        let executed = Arc::new(AtomicBool::new(false));
        let harness = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(5))
            .step_advance_wall("推进墙钟", Duration::from_nanos(2))
            .step_advance_monotonic("推进单调钟", Duration::from_nanos(3))
            .step("标记执行", marker_step(Arc::clone(&executed)));
        let debug = format!("{harness:?}");
        assert!(debug.contains("PendingStep"));
        assert!(debug.contains("推进墙钟"));

        let report = harness.run().expect("公开成功路径必须完成");

        report.assert_all_ok(3);
        report.assert_wall_ns(7);
        report.assert_monotonic_elapsed(Duration::from_nanos(3));
        assert_eq!(report.records()[0].name(), "推进墙钟");
        assert_eq!(report.records()[1].name(), "推进单调钟");
        assert_eq!(report.records()[2].name(), "标记执行");
        assert!(executed.load(Ordering::SeqCst));
    }

    #[test]
    fn owned_string_panic_preserves_message() {
        let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0))
            .step("拥有型消息 panic", |_| -> Result<(), io::Error> {
                std::panic::panic_any(String::from("拥有型消息"));
            })
            .run()
            .expect_err("String panic 必须成为 terminal error");

        assert_eq!(error.kind(), StepOutcome::Panicked);
        assert_eq!(error.detail(), "拥有型消息");
    }

    #[test]
    fn panic_plus_wall_fault_is_observation_failure() {
        let error = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(9))
            .step("panic 并注入故障", |clock| -> Result<(), io::Error> {
                clock.set_wall_fault(ManualClockFault::Unavailable).expect("故障注入应成功");
                panic!("业务 panic");
            })
            .run()
            .expect_err("panic 后观测失败必须优先暴露为观测错误");

        assert_eq!(error.kind(), StepOutcome::ObservationFailed);
        assert!(error.detail().contains("业务 panic"));
        assert!(error.detail().contains("终态观测失败"));
        let source = error.source().expect("必须保留终态观测错误");
        let observation = source
            .downcast_ref::<WallFaultObservation>()
            .expect("source 必须是 WallFaultObservation");
        assert_eq!(observation.0, ManualClockFault::Unavailable);
        let record = &error.report().records()[0];
        assert_eq!(record.wall_after_ns(), Some(9));
        assert_eq!(record.wall_fault_after(), Some(ManualClockFault::Unavailable));
    }
}
