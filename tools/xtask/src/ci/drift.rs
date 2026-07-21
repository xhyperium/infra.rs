//! Generated projection drift detection（PHASE-1-11 / AC-02）— fail-closed。

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use super::{render_to, BASELINE_REL};

#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub ok: bool,
    pub mode: &'static str,
    pub status: String,
    pub drifts: Vec<String>,
    pub expected_digest: String,
    pub observed_digest: Option<String>,
    pub note: String,
}

/// 将 render 结果与 checked-in generated 比较；不一致 → DRIFT fail-closed。
pub fn check_generated_drift(root: &Path) -> Result<DriftReport> {
    let tmp = std::env::temp_dir().join(format!(
        "xhyper-ci-drift-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).context("create temp for drift")?;
    let out = tmp.join("generated");
    let rendered = render_to(root, &out)?;
    if !rendered.ok {
        let _ = fs::remove_dir_all(&tmp);
        return Ok(DriftReport {
            ok: false,
            mode: "shadow",
            status: "FAIL".into(),
            drifts: vec!["render_failed".into()],
            expected_digest: String::new(),
            observed_digest: None,
            note: "cannot render baseline for drift check".into(),
        });
    }

    let expected_contract = fs::read_to_string(out.join("workflow-contract.json"))?;
    let expected_digest = format!("{:x}", Sha256::digest(expected_contract.as_bytes()));

    let checked = root.join(".github/ci/generated/workflow-contract.json");
    if !checked.is_file() {
        let _ = fs::remove_dir_all(&tmp);
        return Ok(DriftReport {
            ok: false,
            mode: "shadow",
            status: "DRIFT".into(),
            drifts: vec!["missing_checked_in_workflow_contract".into()],
            expected_digest,
            observed_digest: None,
            note: "checked-in generated missing; run ci render and commit".into(),
        });
    }
    let observed = fs::read_to_string(&checked)?;
    let observed_digest = format!("{:x}", Sha256::digest(observed.as_bytes()));
    let mut drifts = Vec::new();
    if expected_digest != observed_digest {
        drifts.push("workflow-contract.json".into());
    }
    let exp_pol = out.join("policy-table.md");
    let obs_pol = root.join(".github/ci/generated/policy-table.md");
    // 缺 checked-in policy-table 亦 DRIFT（与 hand-edit 并列 fail-closed）
    if exp_pol.is_file() && !obs_pol.is_file() {
        drifts.push("missing_checked_in_policy_table".into());
    } else if exp_pol.is_file() && obs_pol.is_file() {
        let a = fs::read_to_string(&exp_pol)?;
        let b = fs::read_to_string(&obs_pol)?;
        if a != b {
            drifts.push("policy-table.md".into());
        }
    } else if !exp_pol.is_file() {
        // render 应产出 policy-table；缺失视为 render 合同破损
        drifts.push("render_missing_policy_table".into());
    }
    if !root.join(BASELINE_REL).is_file() {
        drifts.push("baseline_missing".into());
    }

    let status = if drifts.is_empty() {
        "MATCH".to_string()
    } else {
        "DRIFT".to_string()
    };
    let report = DriftReport {
        ok: drifts.is_empty(),
        mode: "shadow",
        status,
        drifts,
        expected_digest,
        observed_digest: Some(observed_digest),
        note: "generated must equal render(baseline); hand-edit forbidden (HC-08)".into(),
    };
    let _ = fs::remove_dir_all(&tmp);
    Ok(report)
}

pub fn drift_or_bail(root: &Path) -> Result<DriftReport> {
    let r = check_generated_drift(root)?;
    if !r.ok {
        bail!("ci drift: {} drifts={:?}", r.status, r.drifts);
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;
    use std::path::PathBuf;

    /// 隔离 fixture：拷贝真实 baseline 并 render 到 temp，避免 nextest 多进程改写仓库内 generated。
    /// in-process Mutex 对 nextest（每测一进程）无效；见 locks 负测同款隔离策略。
    fn fixture_root_with_generated() -> (tempfile::TempDir, PathBuf) {
        let real = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join(".github/ci/generated")).expect("mkdir generated");
        fs::copy(real.join(BASELINE_REL), root.join(BASELINE_REL)).expect("copy baseline");
        let rendered = render_to(&root, &root.join(".github/ci/generated")).expect("render");
        assert!(rendered.ok, "fixture render must succeed: {rendered:?}");
        (tmp, root)
    }

    #[test]
    fn drift_match_on_real_repo_after_render() {
        // 只读校验：仓库 checked-in generated 须与 render 一致（不写回 generated，避免 nextest 竞态）
        let root = repo_root_from_manifest();
        let r = check_generated_drift(&root).expect("drift runs");
        assert!(r.ok, "expected MATCH on real checked-in generated: {:?}", r);
    }

    #[test]
    fn drift_missing_policy_table_is_drift() {
        let (_tmp, root) = fixture_root_with_generated();
        let pol = root.join(".github/ci/generated/policy-table.md");
        assert!(
            pol.is_file(),
            "policy-table must exist in fixture after render"
        );
        fs::remove_file(&pol).expect("remove policy-table");
        let r = check_generated_drift(&root).expect("drift runs");
        assert!(!r.ok, "missing policy-table must DRIFT: {r:?}");
        assert!(
            r.drifts
                .iter()
                .any(|d| d.contains("policy_table") || d.contains("policy-table")),
            "drifts must mention policy-table: {:?}",
            r.drifts
        );
    }

    #[test]
    fn drift_match_on_isolated_render() {
        let (_tmp, root) = fixture_root_with_generated();
        let r = check_generated_drift(&root).expect("drift runs");
        assert!(r.ok, "expected MATCH after isolated render: {:?}", r);
    }
}
