//! 执行 VerificationPlan 中的 shell 检查。

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::types::{CheckResult, CheckSpec, RunResult, RunStatus, VerificationPlan};

/// 执行错误（计划级；单检查失败体现在 exit_code）。
#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("empty argv for check {0}")]
    EmptyArgv(String),
    #[error("spawn {id}: {source}")]
    Spawn {
        id: String,
        #[source]
        source: std::io::Error,
    },
}

/// 在 `cwd` 下执行计划；汇总为 [`RunResult`]。
pub fn execute_plan(plan: &VerificationPlan, cwd: &Path) -> Result<RunResult, ExecuteError> {
    let mut checks = Vec::with_capacity(plan.checks.len());
    for spec in &plan.checks {
        checks.push(run_one(spec, cwd)?);
    }
    let status =
        if checks.iter().all(|c| c.exit_code == 0) { RunStatus::Pass } else { RunStatus::Fail };
    Ok(RunResult {
        schema: RunResult::SCHEMA.into(),
        status,
        plan_digest: plan.plan_digest.clone(),
        commit: git_head(cwd),
        checks,
    })
}

fn run_one(spec: &CheckSpec, cwd: &Path) -> Result<CheckResult, ExecuteError> {
    let program = spec.argv.first().ok_or_else(|| ExecuteError::EmptyArgv(spec.id.clone()))?;
    let args = &spec.argv[1..];
    let start = Instant::now();
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|source| ExecuteError::Spawn { id: spec.id.clone(), source })?;
    let duration_ms = start.elapsed().as_millis() as u64;
    let mut blob = output.stdout.clone();
    blob.extend_from_slice(&output.stderr);
    let output_digest = {
        let mut h = Sha256::new();
        h.update(&blob);
        hex::encode(h.finalize())
    };
    Ok(CheckResult {
        id: spec.id.clone(),
        kind: spec.kind,
        exit_code: output.status.code().unwrap_or(1),
        output_digest,
        duration_ms,
    })
}

fn git_head(cwd: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CheckKind;

    #[test]
    fn execute_true_passes() {
        let plan = VerificationPlan {
            schema: VerificationPlan::SCHEMA.into(),
            contract_digest: String::new(),
            changed_paths: vec![],
            checks: vec![CheckSpec {
                id: "dry-true".into(),
                kind: CheckKind::Custom,
                argv: vec!["true".into()],
                timeout_secs: 10,
            }],
            plan_digest: "abc".into(),
        };
        let run = execute_plan(&plan, Path::new(".")).unwrap();
        assert_eq!(run.status, RunStatus::Pass);
        assert_eq!(run.checks[0].exit_code, 0);
    }
}
