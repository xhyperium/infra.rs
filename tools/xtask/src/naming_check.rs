//! Package 命名检查（PLAN-ARCH-NAMING-QUANT / NAMING_STANDARD Approved）。
//!
//! - 读取 `.architecture/naming.toml`（机器投影，不得创造政策）
//! - 对照 `cargo metadata` workspace members
//! - 默认 `--mode shadow`：打印 findings，**exit 0**（不伪绿宣称 strict）
//! - `--mode strict`：任一 ERROR → 非 0
//!
//! 权威：`docs/standards/NAMING_STANDARD.md`（Approved）+ `CRATE_STANDARD` §3.1.1。
//! 使用 serde + TOML 严格解析 registry；包名前缀/kebab 规则保持显式机器校验。

use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_PREFIX: &str = "xhyper-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Shadow,
    Strict,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Shadow => write!(f, "shadow"),
            Mode::Strict => write!(f, "strict"),
        }
    }
}

impl Mode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "shadow" => Ok(Self::Shadow),
            "strict" => Ok(Self::Strict),
            other => bail!("unknown naming-check mode '{other}' (use shadow|strict)"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: String,
    pub path: String,
    pub package: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub mode: String,
    pub passed: bool,
    pub cargo_members: usize,
    pub registry_entries: usize,
    pub findings: Vec<Finding>,
    pub note: String,
}

#[derive(Debug)]
struct RegistryPackage {
    path: String,
    current_package: String,
    target_package: String,
    status: String,
    /// 解析期校验；保留供后续 lib/bin 对照扩展。
    #[allow(dead_code)]
    lib_name_short: Option<String>,
    #[allow(dead_code)]
    commit_scope: Option<String>,
    #[allow(dead_code)]
    tag_prefix: Option<String>,
}

/// F-03：真 TOML 顶层 + package 字段；未知字段拒绝。
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NamingToml {
    schema_version: u32,
    status: String,
    required_package_prefix: String,
    package_pattern: String,
    new_package_enforcement: String,
    legacy_enforcement: String,
    lib_name_policy: String,
    source_plan: String,
    #[serde(default)]
    note: Option<String>,
    #[serde(default, rename = "package")]
    packages: Vec<NamingPackageToml>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NamingPackageToml {
    path: String,
    current_package: String,
    target_package: String,
    status: String,
    #[serde(default)]
    lib_name_short: Option<String>,
    #[serde(default)]
    commit_scope: Option<String>,
    #[serde(default)]
    tag_prefix: Option<String>,
}

pub fn run(json: bool, mode: Mode) -> Result<()> {
    let report = collect(mode)?;
    emit(&report, json)?;
    if mode == Mode::Strict && !report.passed {
        bail!("naming-check strict: found ERROR findings");
    }
    Ok(())
}

pub fn collect(mode: Mode) -> Result<Report> {
    let root = workspace_root()?;
    let reg_path = root.join(".architecture/naming.toml");
    if !reg_path.is_file() {
        bail!(
            "missing {} — create PR-0 registry before naming-check",
            reg_path.display()
        );
    }
    let raw =
        fs::read_to_string(&reg_path).with_context(|| format!("read {}", reg_path.display()))?;
    let registry = parse_naming_toml(&raw).context("parse .architecture/naming.toml")?;

    let meta = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("cargo metadata --no-deps")?;

    let mut by_path: BTreeMap<String, String> = BTreeMap::new();
    for pkg in &meta.packages {
        if !meta.workspace_members.contains(&pkg.id) {
            continue;
        }
        let path = strip_manifest(&root, pkg.manifest_path.as_std_path())?;
        by_path.insert(path, pkg.name.to_string());
    }

    let mut findings = Vec::new();
    let mut reg_paths = BTreeSet::new();

    for entry in &registry {
        reg_paths.insert(entry.path.clone());

        if entry.target_package.starts_with("xhyper-xhyper-") {
            findings.push(Finding {
                rule_id: "PKG-NAME-009".into(),
                severity: "error".into(),
                path: entry.path.clone(),
                package: entry.target_package.clone(),
                message: "target_package has double xhyper- prefix".into(),
            });
        }
        if !entry.target_package.is_empty() && !is_valid_xhyper_package(&entry.target_package) {
            findings.push(Finding {
                rule_id: "PKG-NAME-001".into(),
                severity: "error".into(),
                path: entry.path.clone(),
                package: entry.target_package.clone(),
                message: "target_package is not valid xhyper- kebab package name".into(),
            });
        }

        match by_path.get(&entry.path) {
            None => findings.push(Finding {
                rule_id: "PKG-NAME-003".into(),
                severity: "error".into(),
                path: entry.path.clone(),
                package: entry.current_package.clone(),
                message: "registry path not present in cargo workspace members".into(),
            }),
            Some(live) if live != &entry.current_package => findings.push(Finding {
                rule_id: "PKG-NAME-003".into(),
                severity: "error".into(),
                path: entry.path.clone(),
                package: live.clone(),
                message: format!(
                    "registry current_package='{}' != cargo metadata name='{live}'",
                    entry.current_package
                ),
            }),
            Some(live) => {
                if live.starts_with("xhyper-xhyper-") {
                    findings.push(Finding {
                        rule_id: "PKG-NAME-009".into(),
                        severity: "error".into(),
                        path: entry.path.clone(),
                        package: live.clone(),
                        message: "live package has double xhyper- prefix".into(),
                    });
                } else if !live.starts_with(REQUIRED_PREFIX) {
                    // shadow+legacy_warn → WARN（迁移期可审阅）
                    // strict 下缺前缀一律 ERROR，确保 cutover 前 `strict` 非 0（naming-gate）
                    // shadow+compliant 却无前缀 → ERROR（registry 不一致）
                    let sev = match (mode, entry.status.as_str()) {
                        (Mode::Shadow, "legacy_warn") => "warn",
                        (Mode::Shadow, s) if s != "compliant" && !s.is_empty() => "warn",
                        _ => "error",
                    };
                    findings.push(Finding {
                        rule_id: "PKG-NAME-001".into(),
                        severity: sev.into(),
                        path: entry.path.clone(),
                        package: live.clone(),
                        message: format!(
                            "package lacks '{REQUIRED_PREFIX}' prefix (registry status={}; mode={mode})",
                            entry.status
                        ),
                    });
                } else if !is_valid_xhyper_package(live) {
                    findings.push(Finding {
                        rule_id: "PKG-NAME-002".into(),
                        severity: "error".into(),
                        path: entry.path.clone(),
                        package: live.clone(),
                        message: "package name fails kebab pattern after xhyper- prefix".into(),
                    });
                }
            }
        }
    }

    for (path, name) in &by_path {
        if !reg_paths.contains(path) {
            findings.push(Finding {
                rule_id: "PKG-NAME-003".into(),
                severity: "error".into(),
                path: path.clone(),
                package: name.clone(),
                message: "cargo member missing from .architecture/naming.toml".into(),
            });
            if !name.starts_with(REQUIRED_PREFIX) {
                findings.push(Finding {
                    rule_id: "PKG-NAME-018".into(),
                    severity: "error".into(),
                    path: path.clone(),
                    package: name.clone(),
                    message: "unregistered package without xhyper- prefix (fail closed for new)"
                        .into(),
                });
            }
        }
    }

    // PKG-NAME-008：path dep 过渡 alias（short = { package = "xhyper-…", path = … }）须清零。
    for alias in scan_transitional_package_aliases(&root)? {
        findings.push(Finding {
            rule_id: "PKG-NAME-008".into(),
            severity: "error".into(),
            path: alias.manifest_rel,
            package: alias.package.clone(),
            message: format!(
                "transitional dependency alias key '{}' → package '{}'; end-state key must equal package name (drop package=)",
                alias.dep_key, alias.package
            ),
        });
    }

    let has_error = findings.iter().any(|f| f.severity == "error");
    let passed = match mode {
        Mode::Shadow => true,
        Mode::Strict => !has_error,
    };

    Ok(Report {
        mode: match mode {
            Mode::Shadow => "shadow".into(),
            Mode::Strict => "strict".into(),
        },
        passed,
        cargo_members: by_path.len(),
        registry_entries: registry.len(),
        findings,
        note: match mode {
            Mode::Shadow => {
                "shadow mode: exit 0 even with ERROR findings; not a cutover/strict pass".into()
            }
            Mode::Strict => {
                "strict mode: ERROR findings fail (prefix + PKG-NAME-008 alias end-state)".into()
            }
        },
    })
}

#[derive(Debug)]
struct TransitionalAlias {
    manifest_rel: String,
    dep_key: String,
    package: String,
}

/// 扫描 workspace 内 `Cargo.toml` 中 `package = "xhyper-…"` 过渡 alias（W4 终态应为空）。
fn scan_transitional_package_aliases(root: &Path) -> Result<Vec<TransitionalAlias>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for ent in entries.flatten() {
            let path = ent.path();
            let name = ent.file_name();
            let name_str = name.to_string_lossy();
            if path.is_dir() {
                if matches!(
                    name_str.as_ref(),
                    ".git"
                        | "target"
                        | ".target-xhyper.rs"
                        | "node_modules"
                        | ".worktree"
                        | ".cargo"
                ) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if name_str != "Cargo.toml" {
                continue;
            }
            let text =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.extend(parse_transitional_aliases_in_toml(&rel, &text));
        }
    }
    out.sort_by(|a, b| {
        (&a.manifest_rel, &a.dep_key, &a.package).cmp(&(&b.manifest_rel, &b.dep_key, &b.package))
    });
    Ok(out)
}

/// 解析单文件中 `key = { … package = "xhyper-…" … }`（单行表；无 regex 依赖）。
fn parse_transitional_aliases_in_toml(manifest_rel: &str, text: &str) -> Vec<TransitionalAlias> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some(pkg_idx) = trimmed.find("package") else {
            continue;
        };
        let after_pkg = &trimmed[pkg_idx..];
        let Some(eq) = after_pkg.find('=') else {
            continue;
        };
        let rest = after_pkg[eq + 1..].trim_start();
        if !rest.starts_with('"') {
            continue;
        }
        let rest = &rest[1..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let package = &rest[..end];
        if !package.starts_with("xhyper-") {
            continue;
        }
        let Some(key_eq) = trimmed.find('=') else {
            continue;
        };
        let key = trimmed[..key_eq].trim();
        if key.is_empty() || key.contains('[') || key == "package" {
            continue;
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        out.push(TransitionalAlias {
            manifest_rel: manifest_rel.to_string(),
            dep_key: key.to_string(),
            package: package.to_string(),
        });
    }
    out
}

/// `^xhyper-[a-z][a-z0-9]*(?:-[a-z0-9]+)*$` 且禁止双前缀 `xhyper-xhyper-`。
pub fn is_valid_xhyper_package(name: &str) -> bool {
    if name.starts_with("xhyper-xhyper-") {
        return false;
    }
    let Some(rest) = name.strip_prefix(REQUIRED_PREFIX) else {
        return false;
    };
    if rest.is_empty() || rest.starts_with("xhyper-") || rest.contains('_') || rest.contains("--") {
        return false;
    }
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut prev_hyphen = false;
    for c in chars {
        if c == '-' {
            if prev_hyphen {
                return false;
            }
            prev_hyphen = true;
            continue;
        }
        prev_hyphen = false;
        if !(c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return false;
        }
    }
    !prev_hyphen
}

fn parse_naming_toml(raw: &str) -> Result<Vec<RegistryPackage>> {
    let doc: NamingToml =
        toml::from_str(raw).context("naming.toml TOML deserialize (deny_unknown_fields)")?;
    if doc.schema_version != 1 {
        bail!(
            "unsupported naming.toml schema_version={} (supported=1)",
            doc.schema_version
        );
    }
    if doc.status != "post_cutover" {
        bail!(
            "naming.toml status must be \"post_cutover\"; got {:?}",
            doc.status
        );
    }
    if doc.required_package_prefix != REQUIRED_PREFIX {
        bail!(
            "naming.toml required_package_prefix={:?} must be {REQUIRED_PREFIX:?}",
            doc.required_package_prefix
        );
    }
    if doc.package_pattern != "^xhyper-[a-z][a-z0-9]*(?:-[a-z0-9]+)*$" {
        bail!(
            "naming.toml package_pattern must match NAMING_STANDARD machine pattern; got {:?}",
            doc.package_pattern
        );
    }
    for field in [
        (
            "new_package_enforcement",
            doc.new_package_enforcement.as_str(),
        ),
        ("legacy_enforcement", doc.legacy_enforcement.as_str()),
    ] {
        if field.1 != "error" {
            bail!(
                "naming.toml {} must be \"error\" post-cutover; got {:?}",
                field.0,
                field.1
            );
        }
    }
    if doc.lib_name_policy != "short_name_L_SHORT" {
        bail!(
            "naming.toml lib_name_policy unsupported: {:?}",
            doc.lib_name_policy
        );
    }
    if doc.source_plan.is_empty() {
        bail!("naming.toml source_plan must be non-empty");
    }
    let _ = &doc.note;

    if doc.packages.is_empty() {
        bail!("no [[package]] entries parsed from naming.toml");
    }

    let mut out = Vec::with_capacity(doc.packages.len());
    let mut seen_paths = BTreeSet::new();
    let mut seen_targets = BTreeSet::new();
    for p in doc.packages {
        if p.path.is_empty() || p.current_package.is_empty() {
            bail!(
                "incomplete [[package]] entry (path/current_package required); got path='{}' current='{}'",
                p.path,
                p.current_package
            );
        }
        if p.target_package.is_empty() {
            bail!(
                "[[package]] path='{}' missing required target_package",
                p.path
            );
        }
        if p.status != "compliant" && p.status != "legacy_warn" {
            bail!(
                "[[package]] path='{}' invalid status='{}' (expected compliant|legacy_warn)",
                p.path,
                p.status
            );
        }
        // F-03：可选字段若存在则做一致性校验（不为空、tag_prefix 对齐 package 名）。
        if let Some(ref lib) = p.lib_name_short {
            if lib.is_empty() || lib.contains('-') {
                bail!(
                    "[[package]] path='{}' lib_name_short must be non-empty short ident without hyphens",
                    p.path
                );
            }
        }
        if let Some(ref scope) = p.commit_scope {
            if scope.is_empty() {
                bail!(
                    "[[package]] path='{}' commit_scope must not be empty when set",
                    p.path
                );
            }
        }
        if let Some(ref tag) = p.tag_prefix {
            if tag != &p.target_package {
                bail!(
                    "[[package]] path='{}' tag_prefix={:?} must equal target_package={:?}",
                    p.path,
                    tag,
                    p.target_package
                );
            }
        }
        if !seen_paths.insert(p.path.clone()) {
            bail!("duplicate registry path '{}'", p.path);
        }
        if !seen_targets.insert(p.target_package.clone()) {
            bail!("duplicate target_package '{}'", p.target_package);
        }
        out.push(RegistryPackage {
            path: p.path,
            current_package: p.current_package,
            target_package: p.target_package,
            status: p.status,
            lib_name_short: p.lib_name_short,
            commit_scope: p.commit_scope,
            tag_prefix: p.tag_prefix,
        });
    }
    Ok(out)
}

fn emit(report: &Report, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "naming-check mode={} members={} registry={} findings={} passed={}",
            report.mode,
            report.cargo_members,
            report.registry_entries,
            report.findings.len(),
            report.passed
        );
        for f in &report.findings {
            println!(
                "  [{}] {} {} @ {} — {}",
                f.severity, f.rule_id, f.package, f.path, f.message
            );
        }
        println!("note: {}", report.note);
    }
    Ok(())
}

fn strip_manifest(root: &Path, manifest: &Path) -> Result<String> {
    let parent = manifest
        .parent()
        .with_context(|| format!("manifest parent {}", manifest.display()))?;
    let rel = parent
        .strip_prefix(root)
        .with_context(|| format!("strip prefix {} from {}", root.display(), parent.display()))?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn workspace_root() -> Result<PathBuf> {
    let meta = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("cargo metadata for workspace_root")?;
    Ok(PathBuf::from(meta.workspace_root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_pattern_accepts_kebab() {
        assert!(is_valid_xhyper_package("xhyper-kernel"));
        assert!(is_valid_xhyper_package("xhyper-domain-market"));
        assert!(is_valid_xhyper_package("xhyper-risk-engine"));
        assert!(!is_valid_xhyper_package("kernel"));
        assert!(!is_valid_xhyper_package("xhyper-domain_market"));
        assert!(!is_valid_xhyper_package("xhyper-xhyper-kernel"));
        assert!(!is_valid_xhyper_package("xhyper-"));
        assert!(!is_valid_xhyper_package("xhyper--kernel"));
    }

    #[test]
    fn mode_parse() {
        assert!(matches!(Mode::parse("shadow").unwrap(), Mode::Shadow));
        assert!(matches!(Mode::parse("strict").unwrap(), Mode::Strict));
        assert!(Mode::parse("warn").is_err());
    }

    fn naming_header() -> &'static str {
        r#"
schema_version = 1
status = "post_cutover"
required_package_prefix = "xhyper-"
package_pattern = "^xhyper-[a-z][a-z0-9]*(?:-[a-z0-9]+)*$"
new_package_enforcement = "error"
legacy_enforcement = "error"
lib_name_policy = "short_name_L_SHORT"
source_plan = "PLAN-ARCH-NAMING-QUANT-v1"
"#
    }

    #[test]
    fn parse_minimal_registry() {
        let raw = format!(
            r#"{}
[[package]]
path = "crates/kernel"
current_package = "xhyper-kernel"
target_package = "xhyper-kernel"
status = "compliant"
"#,
            naming_header()
        );
        let pkgs = parse_naming_toml(&raw).unwrap();
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].path, "crates/kernel");
        assert_eq!(pkgs[0].current_package, "xhyper-kernel");
    }

    #[test]
    fn parse_rejects_missing_status() {
        let raw = format!(
            r#"{}
[[package]]
path = "crates/kernel"
current_package = "xhyper-kernel"
target_package = "xhyper-kernel"
"#,
            naming_header()
        );
        assert!(parse_naming_toml(&raw).is_err());
    }

    #[test]
    fn parse_rejects_invalid_top_level_status() {
        for status in ["legacy", "post-cutover", ""] {
            let raw = format!(
                r#"
schema_version = 1
status = "{status}"
required_package_prefix = "xhyper-"
package_pattern = "^xhyper-[a-z][a-z0-9]*(?:-[a-z0-9]+)*$"
new_package_enforcement = "error"
legacy_enforcement = "error"
lib_name_policy = "short_name_L_SHORT"
source_plan = "PLAN-ARCH-NAMING-QUANT-v1"

[[package]]
path = "crates/kernel"
current_package = "xhyper-kernel"
target_package = "xhyper-kernel"
status = "compliant"
"#
            );
            assert!(
                parse_naming_toml(&raw).is_err(),
                "invalid top-level status {status:?} must fail closed"
            );
        }
    }

    #[test]
    fn parse_rejects_unknown_top_level_and_bad_tag() {
        let unknown = format!(
            r#"{}
typo_field = true
[[package]]
path = "crates/kernel"
current_package = "xhyper-kernel"
target_package = "xhyper-kernel"
status = "compliant"
"#,
            naming_header()
        );
        assert!(parse_naming_toml(&unknown).is_err());

        let bad_tag = format!(
            r#"{}
[[package]]
path = "crates/kernel"
current_package = "xhyper-kernel"
target_package = "xhyper-kernel"
tag_prefix = "wrong"
status = "compliant"
"#,
            naming_header()
        );
        assert!(parse_naming_toml(&bad_tag).is_err());
    }

    #[test]
    fn transitional_alias_detected() {
        let raw = r#"
[dependencies]
kernel = { package = "xhyper-kernel", path = "../kernel" }
xhyper-contracts = { path = "../contracts" }
"#;
        let found = parse_transitional_aliases_in_toml("crates/foo/Cargo.toml", raw);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].dep_key, "kernel");
        assert_eq!(found[0].package, "xhyper-kernel");
    }

    #[test]
    fn transitional_alias_negative_clean() {
        let raw = r#"
[dependencies]
xhyper-kernel = { path = "../kernel" }
anyhow = { workspace = true }
"#;
        assert!(parse_transitional_aliases_in_toml("crates/foo/Cargo.toml", raw).is_empty());
    }

    /// 负向：无前缀 package 名必须被 PKG-NAME-001 规则判定为非法。
    #[test]
    fn negative_unprefixed_package_rejected_by_pattern() {
        assert!(!is_valid_xhyper_package("ledger"));
        assert!(!is_valid_xhyper_package("testkit"));
        assert!(is_valid_xhyper_package("xhyper-ledger"));
    }
}
