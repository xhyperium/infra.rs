//! testkit —— T0 确定性测试支持（SPEC-TESTKIT-002）。
//!
//! 稳定公开面：[`ManualClock`] 族 + [`IntegrationHarness`] 多步集成 harness。
//!
//! 已删除：`xlib_test!` / `mock!` / `FixtureBuilder` /
//! `provider_capability_contract_tests!`（迁至 `contract-testkit`）。

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

mod clock;
mod harness;

pub use clock::{ManualClock, ManualClockError, ManualClockFault, ManualClockSnapshot};
pub use harness::{HarnessReport, HarnessRunError, IntegrationHarness, StepOutcome, StepRecord};
