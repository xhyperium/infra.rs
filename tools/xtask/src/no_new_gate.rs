//! `no-new-gate`：禁止 runtime `xhyper-gate` / `gate` **回流**（PLAN-GATE-RETIRE-001 Phase 5）。
//!
//! 退役完成后规则：
//! - workspace **不得**含 package `xhyper-gate`（或 lib 路径 `crates/infra/gate`）
//! - **禁止**任何 package 依赖 `xhyper-gate` / path `infra/gate`
//! - 源码禁止 `use gate::` / `gate::Gate` / `gate::Capability` / `register_capability`
//!
//! **不**匹配：`VenueSafetyGate`、`archgate`、`.agent/gates`、裸词 `gate`。

use anyhow::{bail, Result};
use cargo_metadata::{MetadataCommand, Package};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// 本检查器自身含模式字符串/测例，不得自命中。
const META_ALLOWLIST_FILES: &[&str] = &["tools/xtask/src/no_new_gate.rs"];

#[derive(Debug, Clone, Serialize)]
struct Finding {
    rule: &'static str,
    path: String,
    detail: String,
}

#[derive(Serialize)]
struct Report {
    ok: bool,
    findings: Vec<Finding>,
}

pub fn run(json: bool) -> Result<()> {
    let root = workspace_root()?;
    let mut findings = Vec::new();
    findings.extend(scan_packages_and_deps()?);
    findings.extend(scan_sources(&root)?);
    // 物理路径回流
    let gate_path = root.join("crates/infra/gate");
    if gate_path.is_dir() {
        findings.push(Finding {
            rule: "GATE-PATH-001",
            path: "crates/infra/gate".into(),
            detail: "directory reintroduced; runtime gate crate must stay deleted".into(),
        });
    }

    let ok = findings.is_empty();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&Report {
                ok,
                findings: findings.clone(),
            })?
        );
    } else {
        println!("no-new-gate: PLAN-GATE-RETIRE-001 anti-reintroduction");
        if ok {
            println!("no-new-gate: PASS (0 findings)");
        } else {
            for f in &findings {
                println!("FAIL [{}] {}: {}", f.rule, f.path, f.detail);
            }
            println!("no-new-gate: FAIL ({} findings)", findings.len());
        }
    }

    if !ok {
        bail!(
            "no-new-gate: runtime gate reintroduction detected ({} finding(s))",
            findings.len()
        );
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let meta = MetadataCommand::new().no_deps().exec()?;
    Ok(meta.workspace_root.into_std_path_buf())
}

fn scan_packages_and_deps() -> Result<Vec<Finding>> {
    let meta = MetadataCommand::new().no_deps().exec()?;
    let members: std::collections::HashSet<_> = meta.workspace_members.iter().cloned().collect();
    let mut out = Vec::new();

    for pkg in &meta.packages {
        if !members.contains(&pkg.id) {
            continue;
        }
        if is_runtime_gate_package(pkg) {
            out.push(Finding {
                rule: "GATE-PKG-001",
                path: pkg.manifest_path.to_string(),
                detail: format!(
                    "workspace package `{}` is runtime gate; must not be a member after retirement",
                    pkg.name
                ),
            });
        }
        for dep in &pkg.dependencies {
            let name = dep.name.as_str();
            let path_hit = dep
                .path
                .as_ref()
                .map(|p| {
                    let s = p.as_str().replace('\\', "/");
                    s.contains("/infra/gate") || s.ends_with("infra/gate")
                })
                .unwrap_or(false);
            if name == "xhyper-gate" || name == "gate" || path_hit {
                out.push(Finding {
                    rule: "GATE-DEP-001",
                    path: pkg.manifest_path.to_string(),
                    detail: format!(
                        "package `{}` depends on runtime gate (`{}`)",
                        pkg.name, name
                    ),
                });
            }
        }
    }
    Ok(out)
}

fn is_runtime_gate_package(pkg: &Package) -> bool {
    if pkg.name.as_str() == "xhyper-gate" {
        return true;
    }
    // lib name gate at infra/gate path
    let manifest = pkg.manifest_path.as_str().replace('\\', "/");
    manifest.contains("/crates/infra/gate/")
}

fn scan_sources(root: &Path) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    let mut files = Vec::new();
    for sub in ["crates", "tools", "apps", "services"] {
        let base = root.join(sub);
        if base.is_dir() {
            collect_rs(&base, &mut files);
        }
    }

    for path in files {
        let rel = match path.strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if META_ALLOWLIST_FILES.iter().any(|p| rel == *p) {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for (lineno, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if let Some(rule) = match_forbidden_line(line) {
                out.push(Finding {
                    rule,
                    path: format!("{rel}:{}", lineno + 1),
                    detail: line.trim().to_string(),
                });
            }
        }
    }
    Ok(out)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn needle_use_gate() -> String {
    format!("use {}::", "gate")
}
fn needle_gate_gate() -> String {
    format!("{}::Gate", "gate")
}
fn needle_gate_cap() -> String {
    format!("{}::Capability", "gate")
}
fn needle_register() -> String {
    format!("{}{}", "register", "_capability")
}

fn match_forbidden_line(line: &str) -> Option<&'static str> {
    if line.contains(&needle_use_gate()) {
        return Some("GATE-SRC-001");
    }
    if line.contains(&needle_gate_gate()) || line.contains(&needle_gate_cap()) {
        return Some("GATE-SRC-002");
    }
    if line.contains(&needle_register()) {
        return Some("GATE-SRC-003");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_flag_venue_safety_gate() {
        let line = "let mut g = VenueSafetyGate::new(100, 10, Duration::from_secs(3600));";
        assert_eq!(match_forbidden_line(line), None);
    }

    #[test]
    fn does_not_flag_archgate_wording() {
        assert_eq!(
            match_forbidden_line("cargo run -p xhyper-archgate -- --json"),
            None
        );
        assert_eq!(match_forbidden_line("see .agent/gates/runner.sh"), None);
    }

    #[test]
    fn flags_use_gate_and_register() {
        let use_line = format!("{}{{Capability, Gate}};", needle_use_gate());
        assert_eq!(match_forbidden_line(&use_line), Some("GATE-SRC-001"));
        let gate_new = format!("{}::new();", needle_gate_gate());
        assert_eq!(match_forbidden_line(&gate_new), Some("GATE-SRC-002"));
        let reg = format!(".{}(cap)", needle_register());
        assert_eq!(match_forbidden_line(&reg), Some("GATE-SRC-003"));
    }

    #[test]
    fn live_workspace_passes_anti_reintroduction() {
        run(false).expect("no-new-gate must PASS after runtime gate retirement");
    }
}
