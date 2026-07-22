//! 多步确定性集成测试 harness。
//!
//! 围绕 [`ManualClock`] 提供命名 step 记录、顺序执行与断言辅助。
//! **不是** 生产 runtime，仅供测试使用。

use std::fmt;
use std::time::Duration;

use kernel::{Clock, Timestamp};

use crate::ManualClock;

/// 单步执行结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRecord {
    /// step 名称。
    pub name: String,
    /// 是否成功。
    pub ok: bool,
    /// 失败时的说明（成功时为空）。
    pub detail: String,
    /// step 结束后墙钟（Unix ns）；失败时为执行前快照。
    pub wall_after_ns: i64,
}

/// 多步集成 harness：持有 [`ManualClock`]，顺序执行闭包并记录结果。
///
/// # 用法
///
/// ```rust
/// use std::time::Duration;
/// use kernel::{Clock, Timestamp};
/// use testkit::{IntegrationHarness, ManualClock};
///
/// let clock = ManualClock::new(Timestamp::from_unix_nanos(0));
/// let mut h = IntegrationHarness::new(clock);
/// h.step("advance", |c| {
///     c.advance_wall(Duration::from_nanos(10))?;
///     Ok(())
/// });
/// h.run().expect("all steps ok");
/// assert_eq!(h.clock().now().unwrap().as_unix_nanos(), 10);
/// ```
///
/// 需要 `dyn Clock` 时请用 [`Self::clock`] 取出内部 [`ManualClock`]。
#[derive(Debug)]
pub struct IntegrationHarness {
    clock: ManualClock,
    steps: Vec<PendingStep>,
    records: Vec<StepRecord>,
    ran: bool,
}

struct PendingStep {
    name: String,
    /// 返回 `Ok(())` 或错误说明字符串。
    run: Box<dyn FnOnce(&ManualClock) -> Result<(), String> + Send>,
}

impl fmt::Debug for PendingStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingStep").field("name", &self.name).finish()
    }
}

impl IntegrationHarness {
    /// 用已有 [`ManualClock`] 构造 harness。
    pub fn new(clock: ManualClock) -> Self {
        Self { clock, steps: Vec::new(), records: Vec::new(), ran: false }
    }

    /// 以初始墙钟构造（等价于 `new(ManualClock::new(wall))`）。
    pub fn with_wall(wall: Timestamp) -> Self {
        Self::new(ManualClock::new(wall))
    }

    /// 访问内部时钟（可变控制）。
    pub fn clock(&self) -> &ManualClock {
        &self.clock
    }

    /// 登记命名 step。闭包接收 `&ManualClock`；`Err(String)` 记为失败。
    pub fn step<F>(&mut self, name: impl Into<String>, f: F) -> &mut Self
    where
        F: FnOnce(&ManualClock) -> Result<(), String> + Send + 'static,
    {
        self.steps.push(PendingStep { name: name.into(), run: Box::new(f) });
        self
    }

    /// 登记：将墙钟前进 `delta`。
    pub fn step_advance_wall(&mut self, name: impl Into<String>, delta: Duration) -> &mut Self {
        self.step(name, move |c| c.advance_wall(delta).map(|_| ()).map_err(|e| e.to_string()))
    }

    /// 登记：将单调钟前进 `delta`。
    pub fn step_advance_monotonic(
        &mut self,
        name: impl Into<String>,
        delta: Duration,
    ) -> &mut Self {
        self.step(name, move |c| c.advance_monotonic(delta).map(|_| ()).map_err(|e| e.to_string()))
    }

    /// 按登记顺序执行全部 step；任一步失败时后续不再执行。
    ///
    /// 成功返回记录切片；失败返回已执行记录（含失败步）。
    pub fn run(&mut self) -> Result<&[StepRecord], &[StepRecord]> {
        if self.ran {
            return if self.records.iter().all(|r| r.ok) {
                Ok(&self.records)
            } else {
                Err(&self.records)
            };
        }
        self.ran = true;
        let steps = std::mem::take(&mut self.steps);
        for step in steps {
            let wall_before = self.clock.now().map(|t| t.as_unix_nanos()).unwrap_or(0);
            match (step.run)(&self.clock) {
                Ok(()) => {
                    let wall_after =
                        self.clock.now().map(|t| t.as_unix_nanos()).unwrap_or(wall_before);
                    self.records.push(StepRecord {
                        name: step.name,
                        ok: true,
                        detail: String::new(),
                        wall_after_ns: wall_after,
                    });
                }
                Err(detail) => {
                    self.records.push(StepRecord {
                        name: step.name,
                        ok: false,
                        detail,
                        wall_after_ns: wall_before,
                    });
                    break;
                }
            }
        }
        if self.records.iter().all(|r| r.ok) && !self.records.is_empty() {
            Ok(&self.records)
        } else if self.records.iter().all(|r| r.ok) {
            // 无 step 也视为成功
            Ok(&self.records)
        } else {
            Err(&self.records)
        }
    }

    /// 已执行记录（`run` 之前为空）。
    pub fn records(&self) -> &[StepRecord] {
        &self.records
    }

    /// 断言全部 step 成功且数量为 `expected_count`。
    pub fn assert_all_ok(&self, expected_count: usize) {
        assert_eq!(self.records.len(), expected_count, "step count: {:?}", self.records);
        for r in &self.records {
            assert!(r.ok, "step {} failed: {}", r.name, r.detail);
        }
    }

    /// 断言墙钟等于期望 Unix ns。
    pub fn assert_wall_ns(&self, expected: i64) {
        let actual = self.clock.now().expect("wall available").as_unix_nanos();
        assert_eq!(actual, expected, "wall ns mismatch");
    }

    /// 断言单调流逝等于期望。
    pub fn assert_monotonic_elapsed(&self, expected: Duration) {
        let snap = self.clock.snapshot().expect("snapshot");
        assert_eq!(snap.monotonic_elapsed(), expected, "mono elapsed mismatch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Clock, Timestamp};
    use std::time::Duration;

    #[test]
    fn multi_step_advance_wall_and_mono() {
        let mut h = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(100));
        h.step_advance_wall("wall+50", Duration::from_nanos(50))
            .step_advance_monotonic("mono+7", Duration::from_nanos(7))
            .step("check", |c| {
                let w = c.now().map_err(|e| e.to_string())?.as_unix_nanos();
                if w != 150 {
                    return Err(format!("expected wall 150, got {w}"));
                }
                Ok(())
            });
        let records = h.run().expect("all ok");
        assert_eq!(records.len(), 3);
        assert!(records.iter().all(|r| r.ok));
        h.assert_all_ok(3);
        h.assert_wall_ns(150);
        h.assert_monotonic_elapsed(Duration::from_nanos(7));
        // 通过 clock() 使用 Clock trait
        assert_eq!(h.clock().now().unwrap().as_unix_nanos(), 150);
    }

    #[test]
    fn run_stops_on_first_failure() {
        let mut h = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0));
        h.step("ok", |_| Ok(())).step("fail", |_| Err("boom".into())).step("skipped", |_| Ok(()));
        let err = h.run().expect_err("must fail");
        assert_eq!(err.len(), 2);
        assert!(err[0].ok);
        assert!(!err[1].ok);
        assert_eq!(err[1].detail, "boom");
        assert_eq!(err[1].name, "fail");
    }

    #[test]
    fn empty_run_ok() {
        let mut h = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(1));
        assert!(h.run().is_ok());
        h.assert_all_ok(0);
        h.assert_wall_ns(1);
    }

    #[test]
    fn step_records_wall_after() {
        let mut h = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(10));
        h.step_advance_wall("a", Duration::from_nanos(5));
        let rec = h.run().unwrap();
        assert_eq!(rec[0].wall_after_ns, 15);
        assert_eq!(rec[0].name, "a");
    }

    #[test]
    fn re_run_returns_cached_records() {
        let mut h = IntegrationHarness::with_wall(Timestamp::from_unix_nanos(0));
        h.step("once", |_| Ok(()));
        h.run().unwrap();
        // 第二次 run 不再执行新 step（已 ran）
        h.step("ignored", |_| Err("no".into()));
        let rec = h.run().unwrap();
        assert_eq!(rec.len(), 1);
    }
}
