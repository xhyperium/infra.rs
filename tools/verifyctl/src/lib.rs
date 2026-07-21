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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CheckKind, CheckResult, RunResult, RunStatus};

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn aggregate_and_write_report_roundtrip() {
        let run = RunResult {
            schema: RunResult::SCHEMA.into(),
            status: RunStatus::Pass,
            plan_digest: "abc".into(),
            commit: "deadbeef".into(),
            checks: vec![CheckResult {
                id: "fmt".into(),
                kind: CheckKind::Fmt,
                exit_code: 0,
                output_digest: "00".into(),
                duration_ms: 1,
            }],
        };
        let aggregated = aggregate_report(run.clone());
        assert_eq!(aggregated.status, RunStatus::Pass);
        assert_eq!(aggregated.plan_digest, "abc");
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("report.json");
        write_report(&path, &aggregated).expect("write");
        let raw = std::fs::read_to_string(&path).expect("read");
        assert!(raw.contains("verification-run/v1"));
        assert!(raw.contains("exit_code"));
    }
}
