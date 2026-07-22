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

impl StepOutcome {
    fn display_zh(self) -> &'static str {
        match self {
            Self::Passed => "成功",
            Self::Failed => "失败",
            Self::Panicked => "发生 panic",
            Self::ObservationFailed => "终态观测失败",
        }
    }
}

/// 单步执行记录。
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
pub struct HarnessRunError {
    step: String,
    kind: StepOutcome,
    detail: String,
    report: Box<HarnessReport>,
    source: Option<BoxError>,
}

impl HarnessRunError {
    fn new(
        step: String,
        kind: StepOutcome,
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
        self.kind
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
            .field("kind", &self.kind)
            .field("detail", &self.detail)
            .field("report", &self.report)
            .field("source", &self.source.as_ref().map(|_| "..."))
            .finish()
    }
}

impl fmt::Display for HarnessRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "测试步骤 {} {}: {}", self.step, self.kind.display_zh(), self.detail)
    }
}

impl std::error::Error for HarnessRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_ref() {
            Some(source) => Some(source.as_ref()),
            None => None,
        }
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
                        StepOutcome::ObservationFailed,
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
                            StepOutcome::ObservationFailed,
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
                    let (kind, after_snapshot) = match after {
                        Ok(snapshot) => (StepOutcome::Failed, Some(snapshot)),
                        Err((snapshot, observation)) => {
                            detail.push_str("；终态观测失败: ");
                            detail.push_str(&observation.to_string());
                            (StepOutcome::ObservationFailed, snapshot)
                        }
                    };
                    records.push(StepRecord {
                        name: name.clone(),
                        outcome: kind,
                        detail: Some(detail.clone()),
                        before_snapshot: Some(before),
                        after_snapshot,
                    });
                    let report = HarnessReport { clock: self.clock, records };
                    return Err(HarnessRunError::new(name, kind, detail, report, Some(source)));
                }
                Err(payload) => {
                    let mut detail = panic_detail(payload.as_ref());
                    let (kind, after_snapshot, source) = match after {
                        Ok(snapshot) => (StepOutcome::Panicked, Some(snapshot), None),
                        Err((snapshot, observation)) => {
                            detail.push_str("；终态观测失败: ");
                            detail.push_str(&observation.to_string());
                            (StepOutcome::ObservationFailed, snapshot, Some(observation))
                        }
                    };
                    records.push(StepRecord {
                        name: name.clone(),
                        outcome: kind,
                        detail: Some(detail.clone()),
                        before_snapshot: Some(before),
                        after_snapshot,
                    });
                    let report = HarnessReport { clock: self.clock, records };
                    return Err(HarnessRunError::new(name, kind, detail, report, source));
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

#[derive(Debug)]
struct WallFaultObservation(ManualClockFault);

impl fmt::Display for WallFaultObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "手动时钟墙钟存在故障注入: {:?}", self.0)
    }
}

impl std::error::Error for WallFaultObservation {}

fn panic_detail(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "步骤发生非字符串 panic".to_owned()
    }
}
