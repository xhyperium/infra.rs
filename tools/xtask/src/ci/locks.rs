//! Toolchain / tools lock 校验（PHASE-2-03 / PR-14 / AC-09）。
//!
//! fail-closed：缺 msrv/primary pin、tools 与 install-cargo-tool 版本漂移 → FAIL。

use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct LockCheckReport {
    pub ok: bool,
    pub mode: &'static str,
    pub msrv: String,
    pub primary: String,
    pub issues: Vec<String>,
    pub note: String,
}

/// 与 `.github/actions/install-cargo-tool/action.yml` 中硬编码版本对齐（agent-safe 合同）。
const INSTALL_ACTION_EXPECTED: &[(&str, &str)] = &[
    ("nextest", "0.9.140"),
    ("cargo-deny", "0.20.2"),
    ("cargo-machete", "0.9.2"),
    ("cargo-llvm-cov", "0.8.7"),
];

pub fn check_locks(root: &Path) -> Result<LockCheckReport> {
    let mut issues = Vec::new();
    let tc = root.join(".github/ci/toolchains.lock.toml");
    let tools = root.join(".github/ci/tools.lock.toml");
    if !tc.is_file() {
        issues.push("missing toolchains.lock.toml".into());
    }
    if !tools.is_file() {
        issues.push("missing tools.lock.toml".into());
    }
    let raw = fs::read_to_string(&tc).unwrap_or_default();
    let msrv = key_top(&raw, "msrv").unwrap_or_default();
    let primary = key_top(&raw, "primary").unwrap_or_default();
    // 空 pin 必须 FAIL（不得用 is_empty 短路跳过）
    if msrv.is_empty() {
        issues.push("missing_msrv_pin".into());
    }
    if primary.is_empty() {
        issues.push("missing_primary_pin".into());
    }
    if !msrv.is_empty() && msrv != "1.94.1" {
        issues.push(format!("msrv_mismatch:{msrv}"));
    }
    if !primary.is_empty() && !msrv.is_empty() && primary != msrv {
        issues.push(format!("primary_ne_msrv:{primary}!={msrv}"));
    }
    // align with rust-toolchain / workspace if present
    let baseline = fs::read_to_string(root.join(".github/ci/baseline.toml")).unwrap_or_default();
    if baseline.contains("1.94.1") && !msrv.is_empty() && msrv != "1.94.1" {
        issues.push("baseline_msrv_drift".into());
    }

    // tools.lock ↔ install-cargo-tool version alignment
    let tools_raw = fs::read_to_string(&tools).unwrap_or_default();
    for (tool, expected_ver) in INSTALL_ACTION_EXPECTED {
        match tool_version_from_lock(&tools_raw, tool) {
            None => issues.push(format!("tools_lock_missing_version:{tool}")),
            Some(v) if v != *expected_ver => {
                issues.push(format!(
                    "tools_lock_install_mismatch:{tool}:lock={v}:install={expected_ver}"
                ));
            }
            Some(_) => {}
        }
    }

    // Cross-check install-cargo-tool action.yml embeds the same versions (if present)
    let action = root.join(".github/actions/install-cargo-tool/action.yml");
    if action.is_file() {
        let action_raw = fs::read_to_string(&action).unwrap_or_default();
        for (tool, expected_ver) in INSTALL_ACTION_EXPECTED {
            // crude: VERSION=<ver> near tool case block — require version string appears
            if !action_raw.contains(&format!("VERSION={expected_ver}"))
                && !action_raw.contains(&format!("VERSION = {expected_ver}"))
            {
                // nextest uses cargo-nextest-${VERSION}; still has VERSION=0.9.140
                if !action_raw.contains(*expected_ver) {
                    issues.push(format!(
                        "install_action_missing_version_pin:{tool}:{expected_ver}"
                    ));
                }
            }
        }
    }

    Ok(LockCheckReport {
        ok: issues.is_empty(),
        mode: "shadow",
        msrv,
        primary,
        issues,
        note:
            "lock files are contract pins; empty pin FAIL; tools.lock must match install-cargo-tool"
                .into(),
    })
}

pub fn check_locks_or_bail(root: &Path) -> Result<LockCheckReport> {
    let r = check_locks(root)?;
    if !r.ok {
        bail!("ci locks: {:?}", r.issues);
    }
    Ok(r)
}

fn key_top(raw: &str, key: &str) -> Option<String> {
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if let Some(rest) = t.strip_prefix(key) {
            let rest = rest.trim_start().strip_prefix('=')?.trim();
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// 从 tools.lock.toml 解析 `[tool.<name>] version = "..."`。
fn tool_version_from_lock(raw: &str, tool: &str) -> Option<String> {
    let header = format!("[tool.{tool}]");
    let mut in_section = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_section = t == header;
            continue;
        }
        if in_section && t.starts_with("version") {
            if let Some(rest) = t.strip_prefix("version") {
                let rest = rest.trim_start().strip_prefix('=')?.trim();
                return Some(rest.trim_matches('"').to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;
    use std::path::PathBuf;

    /// 与 `INSTALL_ACTION_EXPECTED` 对齐的最小 tools.lock，隔离 toolchains pin 负向。
    const VALID_TOOLS_LOCK: &str = r#"
schema_version = 1
[tool.nextest]
version = "0.9.140"
[tool.cargo-deny]
version = "0.20.2"
[tool.cargo-machete]
version = "0.9.2"
[tool.cargo-llvm-cov]
version = "0.8.7"
"#;

    /// 构造仅含 shipped `check_locks` 所需文件的 temp root（**不**走 test-only 旁路）。
    fn fixture_root(toolchains: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join(".github/ci")).expect("mkdir ci");
        fs::write(root.join(".github/ci/toolchains.lock.toml"), toolchains).expect("tc");
        fs::write(root.join(".github/ci/tools.lock.toml"), VALID_TOOLS_LOCK).expect("tools");
        (tmp, root)
    }

    #[test]
    fn locks_present_and_msrv() {
        let root = repo_root_from_manifest();
        let r = check_locks(&root).unwrap();
        assert!(r.ok, "{r:?}");
        assert_eq!(r.msrv, "1.94.1");
        assert_eq!(r.primary, "1.94.1");
    }

    #[test]
    fn empty_msrv_pin_fails_via_shipped_check_locks() {
        let (_tmp, root) = fixture_root(
            r#"
schema_version = 1
primary = "1.94.1"
"#,
        );
        let r = check_locks(&root).expect("check_locks runs on fixture");
        assert!(!r.ok, "empty msrv must FAIL via check_locks: {r:?}");
        assert!(r.issues.iter().any(|i| i == "missing_msrv_pin"), "{r:?}");
        let err = check_locks_or_bail(&root);
        assert!(err.is_err(), "check_locks_or_bail must bail: {err:?}");
        let msg = format!("{:#}", err.unwrap_err());
        assert!(
            msg.contains("missing_msrv_pin") || msg.contains("ci locks"),
            "{msg}"
        );
    }

    #[test]
    fn empty_primary_pin_fails_via_shipped_check_locks() {
        let (_tmp, root) = fixture_root(
            r#"
schema_version = 1
msrv = "1.94.1"
"#,
        );
        let r = check_locks(&root).expect("check_locks runs on fixture");
        assert!(!r.ok, "empty primary must FAIL via check_locks: {r:?}");
        assert!(r.issues.iter().any(|i| i == "missing_primary_pin"), "{r:?}");
        assert!(check_locks_or_bail(&root).is_err());
    }

    #[test]
    fn both_empty_pins_fail_via_shipped_check_locks() {
        let (_tmp, root) = fixture_root("schema_version = 1\n");
        let r = check_locks(&root).expect("check_locks");
        assert!(!r.ok, "{r:?}");
        assert!(r.issues.iter().any(|i| i == "missing_msrv_pin"), "{r:?}");
        assert!(r.issues.iter().any(|i| i == "missing_primary_pin"), "{r:?}");
        assert!(check_locks_or_bail(&root).is_err());
    }

    #[test]
    fn missing_toolchains_lock_file_fails_via_check_locks() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        fs::create_dir_all(root.join(".github/ci")).unwrap();
        fs::write(root.join(".github/ci/tools.lock.toml"), VALID_TOOLS_LOCK).unwrap();
        // no toolchains.lock.toml
        let r = check_locks(root).expect("runs");
        assert!(!r.ok, "{r:?}");
        assert!(
            r.issues
                .iter()
                .any(|i| i.contains("missing toolchains") || i == "missing_msrv_pin"),
            "{r:?}"
        );
        assert!(check_locks_or_bail(root).is_err());
    }
}
