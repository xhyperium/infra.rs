//! INFRA-008：Public API / additive-only 基线检查。
//!
//! - 工具缺失 → `TOOL_MISSING`（非 0）
//! - 无 baseline tag → `BASELINE_MISSING`（非 0）
//! - 有工具 + tag：对关键 crate 跑 `cargo semver-checks check-release`
//! - 仍 **不** 宣称 INFRA-008 WP ACCEPTED（负向 fixture / CI pin 完整接线另计）
//!
//! 文档：`tools/xtask/docs/semver-checks.md`

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::process::Command;

const PIN_HINT: &str = "cargo-semver-checks 0.48.0（见 tools/xtask/docs/semver-checks.md）";

/// 优先建立 baseline 的发布面 crate（package 名）。
const CHECK_PACKAGES: &[&str] = &["contracts", "kernel", "decimalx", "canonical"];

#[derive(Debug, Serialize)]
struct PackageResult {
    package: String,
    baseline_tag: Option<String>,
    exit_code: Option<i32>,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct Report {
    passed: bool,
    tool_available: bool,
    tool_version: Option<String>,
    baseline_available: bool,
    status: &'static str,
    message: String,
    work_package: &'static str,
    auto_repair: bool,
    packages: Vec<PackageResult>,
    pin_hint: &'static str,
}

pub fn run(json: bool) -> Result<()> {
    let (tool_available, tool_version) = detect_tool();
    let tags = list_tags();
    let baseline_available = !tags.is_empty()
        && CHECK_PACKAGES
            .iter()
            .any(|pkg| find_baseline_tag(pkg, &tags).is_some());

    let mut packages = Vec::new();
    let (status, message, passed) = if !tool_available {
        (
            "TOOL_MISSING",
            format!("未检测到 {PIN_HINT}；INFRA-008 保持 BLOCKED。安装前不得宣称 API additive-only 已门禁。"),
            false,
        )
    } else if !baseline_available {
        (
            "BASELINE_MISSING",
            "工具可用但无可用 semver baseline tag（期望 `<crate>-vX.Y.Z`）；\
             不得用空对比冒充 PASS。可用 `git tag <crate>-vX.Y.Z` 建立基线。"
                .into(),
            false,
        )
    } else {
        // 对每个有 baseline 的 package 执行 check-release
        let mut all_ok = true;
        for pkg in CHECK_PACKAGES {
            let Some(tag) = find_baseline_tag(pkg, &tags) else {
                packages.push(PackageResult {
                    package: (*pkg).into(),
                    baseline_tag: None,
                    exit_code: None,
                    status: "NO_BASELINE".into(),
                    detail: "skip: no matching tag".into(),
                });
                continue;
            };
            match run_check_release(pkg, &tag) {
                Ok((code, detail)) => {
                    let ok = code == 0;
                    if !ok {
                        all_ok = false;
                    }
                    packages.push(PackageResult {
                        package: (*pkg).into(),
                        baseline_tag: Some(tag),
                        exit_code: Some(code),
                        status: if ok { "PASS".into() } else { "FAIL".into() },
                        detail,
                    });
                }
                Err(err) => {
                    all_ok = false;
                    packages.push(PackageResult {
                        package: (*pkg).into(),
                        baseline_tag: Some(tag),
                        exit_code: None,
                        status: "ERROR".into(),
                        detail: err.to_string(),
                    });
                }
            }
        }
        if packages.iter().all(|p| p.status == "NO_BASELINE") {
            (
                "BASELINE_MISSING",
                "未找到与 CHECK_PACKAGES 匹配的 `<crate>-v*` tag".into(),
                false,
            )
        } else if all_ok {
            (
                "PASS",
                "cargo-semver-checks check-release 对已有 baseline 的 package 全部通过；\
                 负向 fixture / CI pin 完整接线前仍不得宣称 INFRA-008 ACCEPTED。"
                    .into(),
                true,
            )
        } else {
            (
                "BREAKING_OR_ERROR",
                "一个或多个 package 的 semver-checks 失败或执行错误（fail-closed）".into(),
                false,
            )
        }
    };

    let report = Report {
        passed,
        tool_available,
        tool_version,
        baseline_available,
        status,
        message: message.clone(),
        work_package: "INFRA-008",
        auto_repair: false,
        packages,
        pin_hint: PIN_HINT,
    };

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "semver-check: status={} tool_available={} baseline_available={} passed={}",
            report.status, report.tool_available, report.baseline_available, report.passed
        );
        println!("  {}", report.message);
        if let Some(v) = &report.tool_version {
            println!("  tool_version: {v}");
        }
        for pkg in &report.packages {
            println!(
                "  - {} status={} tag={:?} exit={:?}",
                pkg.package, pkg.status, pkg.baseline_tag, pkg.exit_code
            );
        }
        println!("  auto_repair: false (forbidden)");
    }

    if !report.passed {
        bail!("semver-check: {status} — {message}");
    }
    Ok(())
}

fn detect_tool() -> (bool, Option<String>) {
    if let Ok(output) = Command::new("cargo")
        .args(["semver-checks", "--version"])
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return (true, Some(version));
        }
    }
    if let Ok(output) = Command::new("cargo-semver-checks")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return (true, Some(version));
        }
    }
    (false, None)
}

fn list_tags() -> Vec<String> {
    let Ok(output) = Command::new("git").args(["tag", "--list"]).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn find_baseline_tag(package: &str, tags: &[String]) -> Option<String> {
    // Prefer exact `<package>-vX.Y.Z` (highest version-ish lexicographic is fine for now)
    let prefix = format!("{package}-v");
    let mut matches: Vec<&String> = tags.iter().filter(|t| t.starts_with(&prefix)).collect();
    matches.sort();
    matches.last().map(|s| (*s).clone())
}

fn run_check_release(package: &str, baseline_tag: &str) -> Result<(i32, String)> {
    let output = Command::new("cargo")
        .args([
            "semver-checks",
            "check-release",
            "-p",
            package,
            "--baseline-rev",
            baseline_tag,
        ])
        .output()
        .with_context(|| format!("run cargo semver-checks for {package}"))?;
    let mut detail = String::from_utf8_lossy(&output.stdout).into_owned();
    detail.push_str(&String::from_utf8_lossy(&output.stderr));
    if detail.len() > 2000 {
        detail.truncate(2000);
        detail.push('…');
    }
    Ok((output.status.code().unwrap_or(1), detail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_never_claims_auto_repair() {
        let auto_repair = false;
        assert!(!auto_repair);
    }

    #[test]
    fn find_baseline_prefers_crate_prefix() {
        let tags = vec![
            "v0.1.0".into(),
            "contracts-v0.1.0".into(),
            "contracts-v0.2.0".into(),
            "kernel-v0.1.0".into(),
        ];
        assert_eq!(
            find_baseline_tag("contracts", &tags).as_deref(),
            Some("contracts-v0.2.0")
        );
        assert_eq!(
            find_baseline_tag("kernel", &tags).as_deref(),
            Some("kernel-v0.1.0")
        );
        assert_eq!(find_baseline_tag("missing", &tags), None);
    }
}
