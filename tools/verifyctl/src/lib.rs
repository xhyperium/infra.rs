//! `verifyctl` — 最小验证计划 / 执行 / 报告。
//!
//! - `plan`：Goal Contract + changed paths → VerificationPlan
//! - `execute`：运行计划内 shell 检查（有界、cwd=repo root）
//! - `report`：聚合 RunResult（PASS/FAIL）

#![forbid(unsafe_code)]

mod execute;
mod plan;
mod report;
mod types;

pub use execute::{ExecuteError, execute_plan};
pub use plan::{PlanError, PlanOptions, build_plan};
pub use report::{aggregate_report, write_report};
pub use types::{CheckKind, CheckResult, CheckSpec, RunResult, RunStatus, VerificationPlan};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 可选 evidence 钩子：按名追加事件。
#[cfg(feature = "with-evidence")]
pub fn append_evidence(path: &std::path::Path, name: &str) -> Result<(), String> {
    use evidence::{FileEvidenceAppender, append_checked};
    let app = FileEvidenceAppender::open(path).map_err(|e| format!("evidence open: {e:?}"))?;
    append_checked(&app, name).map_err(|e| format!("evidence append: {e}"))?;
    Ok(())
}
