use anyhow::{bail, Context, Result};
use cargo_metadata::{MetadataCommand, Package, TargetKind};
use serde::Serialize;
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Debug, Default, Serialize)]
struct Report {
    workspace: Vec<CrateRecord>,
    legacy: Vec<CrateRecord>,
    findings: Vec<Finding>,
}

#[derive(Debug, Serialize)]
struct CrateRecord {
    name: String,
    manifest_path: String,
    scope: &'static str,
    target_kind: String,
    publish: Option<Vec<String>>,
    features: Vec<String>,
    structure: Structure,
    #[serde(skip)]
    missing_target_roots: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Structure {
    readme: bool,
    changelog: bool,
    agents: bool,
    docs: bool,
    target_root: bool,
}

#[derive(Debug, Serialize)]
struct Finding {
    rule_id: &'static str,
    level: &'static str,
    evidence_path: String,
    evidence_line: Option<u32>,
    suggested_action: String,
}

pub fn run(json: bool, check: bool) -> Result<()> {
    let report = inventory(&std::env::current_dir()?)?;
    if json {
        println!("{}", render_json(&report)?);
    } else {
        print!("{}", render_markdown(&report));
    }
    if check && has_blocking_errors(&report) {
        bail!("crate-standard found machine-checkable errors");
    }
    Ok(())
}

fn inventory(root: &Path) -> Result<Report> {
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("cargo metadata failed")?;
    let members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let mut report = Report::default();

    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
    {
        let record = workspace_record(root, package);
        add_required_file_findings(root, &record, &mut report.findings);
        add_independent_version_findings(root, package, &mut report.findings);
        add_name_prefix_findings(root, package, &mut report.findings);
        add_changelog_unreleased_findings(root, &record, &mut report.findings);
        add_root_docs_findings(root, package, &mut report.findings);
        add_readme_fields_findings(root, &record, &mut report.findings);
        report.workspace.push(record);
    }

    let workspace_manifests: BTreeSet<_> = report
        .workspace
        .iter()
        .map(|record| record.manifest_path.clone())
        .collect();
    let mut legacy_manifests = Vec::new();
    collect_manifests(&root.join("legacy"), &mut legacy_manifests);
    for manifest in legacy_manifests {
        let relative = relative(root, &manifest);
        if !workspace_manifests.contains(&relative) {
            let record = legacy_record(root, &manifest);
            report.findings.push(Finding {
                rule_id: "legacy-quarantine-review",
                level: "MANUAL",
                evidence_path: relative.clone(),
                evidence_line: Some(1),
                suggested_action: format!(
                    "review this {} crate separately; workspace crate rules are not applied",
                    record.scope
                ),
            });
            report.legacy.push(record);
        }
    }

    report
        .workspace
        .sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    report
        .legacy
        .sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    report.findings.sort_by(|left, right| {
        (&left.evidence_path, left.rule_id).cmp(&(&right.evidence_path, right.rule_id))
    });
    Ok(report)
}

fn workspace_record(root: &Path, package: &Package) -> CrateRecord {
    let crate_root = package
        .manifest_path
        .parent()
        .expect("package manifest has a parent")
        .as_std_path();
    let has_lib = package.targets.iter().any(|target| {
        target.kind.iter().any(|kind| {
            matches!(
                kind,
                TargetKind::Lib
                    | TargetKind::RLib
                    | TargetKind::DyLib
                    | TargetKind::CDyLib
                    | TargetKind::StaticLib
                    | TargetKind::ProcMacro
            )
        })
    });
    let has_bin = package
        .targets
        .iter()
        .any(|target| target.kind.contains(&TargetKind::Bin));
    let missing_target_roots: Vec<_> = package
        .targets
        .iter()
        .filter(|target| {
            target.kind.contains(&TargetKind::Bin)
                || target.kind.iter().any(|kind| {
                    matches!(
                        kind,
                        TargetKind::Lib
                            | TargetKind::RLib
                            | TargetKind::DyLib
                            | TargetKind::CDyLib
                            | TargetKind::StaticLib
                            | TargetKind::ProcMacro
                    )
                })
        })
        .filter(|target| !target.src_path.is_file())
        .map(|target| relative(root, target.src_path.as_std_path()))
        .collect();
    let target_root = missing_target_roots.is_empty();
    make_record(RecordParts {
        root,
        crate_root,
        name: package.name.to_string(),
        scope: "workspace",
        target_kind: target_kind(has_lib, has_bin),
        publish: package.publish.clone(),
        features: package.features.keys().cloned().collect(),
        target_root,
        missing_target_roots,
    })
}

fn legacy_record(root: &Path, manifest: &Path) -> CrateRecord {
    let crate_root = manifest.parent().expect("legacy manifest has a parent");
    let text = fs::read_to_string(manifest).unwrap_or_default();
    let name = manifest_value(&text, "package", "name").unwrap_or_else(|| {
        crate_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    let publish = match manifest_value(&text, "package", "publish").as_deref() {
        Some("false") => Some(Vec::new()),
        _ => None,
    };
    let features = section_keys(&text, "features");
    let has_lib = crate_root.join("src/lib.rs").is_file();
    let has_bin = crate_root.join("src/main.rs").is_file();
    let scope = if has_section(&text, "workspace") {
        "legacy-standalone"
    } else {
        "legacy-quarantine"
    };
    make_record(RecordParts {
        root,
        crate_root,
        name,
        scope,
        target_kind: target_kind(has_lib, has_bin),
        publish,
        features,
        target_root: has_lib || has_bin,
        missing_target_roots: Vec::new(),
    })
}

struct RecordParts<'a> {
    root: &'a Path,
    crate_root: &'a Path,
    name: String,
    scope: &'static str,
    target_kind: String,
    publish: Option<Vec<String>>,
    features: Vec<String>,
    target_root: bool,
    missing_target_roots: Vec<String>,
}

fn make_record(parts: RecordParts<'_>) -> CrateRecord {
    CrateRecord {
        name: parts.name,
        manifest_path: relative(parts.root, &parts.crate_root.join("Cargo.toml")),
        scope: parts.scope,
        target_kind: parts.target_kind,
        publish: parts.publish,
        features: parts.features,
        structure: Structure {
            readme: parts.crate_root.join("README.md").is_file(),
            changelog: parts.crate_root.join("CHANGELOG.md").is_file(),
            agents: parts.crate_root.join("AGENTS.md").is_file(),
            docs: parts.crate_root.join("docs").is_dir(),
            target_root: parts.target_root,
        },
        missing_target_roots: parts.missing_target_roots,
    }
}

fn add_required_file_findings(root: &Path, record: &CrateRecord, findings: &mut Vec<Finding>) {
    let crate_root = root.join(
        Path::new(&record.manifest_path)
            .parent()
            .expect("manifest path has a parent"),
    );
    let required = [
        ("README.md", record.structure.readme),
        ("CHANGELOG.md", record.structure.changelog),
        ("AGENTS.md", record.structure.agents),
        ("docs", record.structure.docs),
    ];
    for (path, exists) in required {
        if !exists {
            findings.push(missing_file(root, &crate_root.join(path)));
        }
    }
    if !record.structure.target_root {
        if record.missing_target_roots.is_empty() {
            let path = match record.target_kind.as_str() {
                "bin" => "src/main.rs",
                _ => "src/lib.rs",
            };
            findings.push(missing_file(root, &crate_root.join(path)));
        } else {
            findings.extend(
                record
                    .missing_target_roots
                    .iter()
                    .map(|path| missing_file(root, &root.join(path))),
            );
        }
    }
}

fn missing_file(root: &Path, path: &Path) -> Finding {
    Finding {
        rule_id: "crate-required-file",
        level: "ERROR",
        evidence_path: relative(root, path),
        evidence_line: None,
        suggested_action: format!(
            "add required path `{}` with crate-specific content",
            relative(root, path)
        ),
    }
}

/// 必须以 Cargo.toml 字面量为准：`version.workspace = true` 时 cargo metadata 仍可能解析出版本。
fn add_independent_version_findings(root: &Path, package: &Package, findings: &mut Vec<Finding>) {
    let manifest = package.manifest_path.as_std_path();
    let text = match fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(_) => {
            findings.push(Finding {
                rule_id: "crate-independent-version",
                level: "ERROR",
                evidence_path: relative(root, manifest),
                evidence_line: None,
                suggested_action: format!(
                    "set independent three-part version literal in `{}` `[package]` (do not use version.workspace)",
                    relative(root, manifest)
                ),
            });
            return;
        }
    };
    if let Some(finding) = independent_version_finding(root, manifest, &text) {
        findings.push(finding);
    }
}

fn independent_version_finding(root: &Path, manifest: &Path, text: &str) -> Option<Finding> {
    let mut current = "";
    let mut workspace_inherited = false;
    let mut has_literal_triple = false;
    let mut evidence_line: Option<u32> = None;

    for (idx, line) in text.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let stripped = line.split('#').next().unwrap_or_default().trim();
        if stripped.starts_with('[') && stripped.ends_with(']') {
            current = stripped.trim_matches(['[', ']']);
            continue;
        }
        if current != "package" {
            continue;
        }
        let Some((raw_key, raw_value)) = stripped.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value.trim();

        // `version.workspace = true`
        if key == "version.workspace" {
            workspace_inherited = value == "true";
            evidence_line = Some(line_no);
            continue;
        }
        if key != "version" {
            continue;
        }
        // `version = { workspace = true }` 等继承写法
        if value.contains("workspace") {
            workspace_inherited = true;
            evidence_line = Some(line_no);
            continue;
        }
        let literal = value.trim_matches('"').trim();
        if is_semver_triple(literal) {
            has_literal_triple = true;
        } else {
            evidence_line = Some(line_no);
        }
    }

    if workspace_inherited || !has_literal_triple {
        Some(Finding {
            rule_id: "crate-independent-version",
            level: "ERROR",
            evidence_path: relative(root, manifest),
            evidence_line,
            suggested_action: format!(
                "set independent three-part version literal in `{}` `[package]` (do not use version.workspace)",
                relative(root, manifest)
            ),
        })
    } else {
        None
    }
}

/// 前缀迁移 allowlist：cutover 后应为空。未命中且无 `xhyper-` 前缀 → ERROR。
/// 禁止再向此列表添加新 crate。
const EXISTING_NON_PREFIXED: &[&str] = &[];

/// `[package] name` 必须 `xhyper-<short-name>` 前缀（CRATE_STANDARD §3.1.1）。
/// lib/bin name 通过 `[lib] name` / `[[bin]] name` 解耦，不在本规则检查范围。
fn add_name_prefix_findings(root: &Path, package: &Package, findings: &mut Vec<Finding>) {
    let name = package.name.as_str();
    if name.starts_with("xhyper-") {
        return;
    }
    let manifest = package.manifest_path.as_std_path();
    let evidence_line = package_name_line(manifest);
    let level = if EXISTING_NON_PREFIXED.contains(&name) {
        "WARN"
    } else {
        "ERROR"
    };
    findings.push(Finding {
        rule_id: "crate-name-prefix",
        level,
        evidence_path: relative(root, manifest),
        evidence_line,
        suggested_action: format!(
            "rename `[package] name` from `{name}` to `xhyper-{name}` (keep lib/bin short name via `[lib] name`); see CRATE_STANDARD §3.1.1",
        ),
    });
}

/// 扫描 manifest `[package]` section 下 `name = "..."` 的行号；不可读 / 找不到时返回 None。
fn package_name_line(manifest: &Path) -> Option<u32> {
    let text = fs::read_to_string(manifest).ok()?;
    let mut current = "";
    for (idx, line) in text.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let stripped = line.split('#').next().unwrap_or_default().trim();
        if stripped.starts_with('[') && stripped.ends_with(']') {
            current = stripped.trim_matches(['[', ']']);
            continue;
        }
        if current != "package" {
            continue;
        }
        if let Some((key, _)) = stripped.split_once('=') {
            if key.trim() == "name" {
                return Some(line_no);
            }
        }
    }
    None
}

/// 粗匹配三段式 semver：`X.Y.Z`（可带 pre-release / build 后缀）。
fn is_semver_triple(version: &str) -> bool {
    let core = version
        .split_once(['-', '+'])
        .map(|(core, _)| core)
        .unwrap_or(version);
    let mut parts = core.split('.');
    let (Some(major), Some(minor), Some(patch)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    // 仅接受恰好三段数字核心（禁止额外 `.` 段）
    if parts.next().is_some() {
        return false;
    }
    !major.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && !minor.is_empty()
        && minor.chars().all(|c| c.is_ascii_digit())
        && !patch.is_empty()
        && patch.chars().all(|c| c.is_ascii_digit())
}

/// CHANGELOG 存在时要求含 `[Unreleased]`；文件缺失时由 `crate-required-file` 负责。
fn add_changelog_unreleased_findings(
    root: &Path,
    record: &CrateRecord,
    findings: &mut Vec<Finding>,
) {
    if !record.structure.changelog {
        return;
    }
    let crate_root = root.join(
        Path::new(&record.manifest_path)
            .parent()
            .expect("manifest path has a parent"),
    );
    let path = crate_root.join("CHANGELOG.md");
    let text = fs::read_to_string(&path).unwrap_or_default();
    if text.to_ascii_lowercase().contains("[unreleased]") {
        return;
    }
    findings.push(Finding {
        rule_id: "crate-changelog-unreleased",
        level: "WARN",
        evidence_path: relative(root, &path),
        evidence_line: None,
        suggested_action: format!(
            "add a Keep a Changelog `[Unreleased]` section to `{}`",
            relative(root, &path)
        ),
    });
}

/// README 存在时启发式检查「非职责」「限制」字段；文件缺失时由 `crate-required-file` 负责。
///
/// 有意使用轻量关键字匹配（非语义理解），仅作迁移防回归 WARN，不得升 ERROR。
fn add_readme_fields_findings(root: &Path, record: &CrateRecord, findings: &mut Vec<Finding>) {
    if !record.structure.readme {
        return;
    }
    let crate_root = root.join(
        Path::new(&record.manifest_path)
            .parent()
            .expect("manifest path has a parent"),
    );
    let path = crate_root.join("README.md");
    let text = fs::read_to_string(&path).unwrap_or_default();
    let lower = text.to_ascii_lowercase();
    let has_non_goals = text.contains("非职责")
        || lower.contains("non-goals")
        || lower.contains("non_goals")
        || lower.contains("out of scope");
    let has_limits = text.contains("限制")
        || lower.contains("limitations")
        || lower.contains("constraints")
        || text.contains("安全说明");
    if has_non_goals && has_limits {
        return;
    }
    let mut missing = Vec::new();
    if !has_non_goals {
        missing.push("非职责/non-goals");
    }
    if !has_limits {
        missing.push("限制/limitations");
    }
    findings.push(Finding {
        rule_id: "crate-readme-fields",
        level: "WARN",
        evidence_path: relative(root, &path),
        evidence_line: None,
        suggested_action: format!(
            "add README sections covering {} in `{}` (heuristic; see CRATE_STANDARD §6)",
            missing.join(" and "),
            relative(root, &path)
        ),
    });
}

/// 对存在的 lib/bin 入口检查 crate 级 `//!` 文档；文件缺失时不重复报。
fn add_root_docs_findings(root: &Path, package: &Package, findings: &mut Vec<Finding>) {
    for target in &package.targets {
        let is_lib_or_bin = target.kind.contains(&TargetKind::Bin)
            || target.kind.iter().any(|kind| {
                matches!(
                    kind,
                    TargetKind::Lib
                        | TargetKind::RLib
                        | TargetKind::DyLib
                        | TargetKind::CDyLib
                        | TargetKind::StaticLib
                        | TargetKind::ProcMacro
                )
            });
        if !is_lib_or_bin {
            continue;
        }
        let src = target.src_path.as_std_path();
        if !src.is_file() {
            continue;
        }
        let text = fs::read_to_string(src).unwrap_or_default();
        if has_crate_level_docs(&text) {
            continue;
        }
        findings.push(Finding {
            rule_id: "crate-root-docs",
            level: "WARN",
            evidence_path: relative(root, src),
            evidence_line: None,
            suggested_action: format!(
                "add crate-level `//!` documentation at the top of `{}`",
                relative(root, src)
            ),
        });
    }
}

fn has_crate_level_docs(text: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start().starts_with("//!"))
}

fn has_blocking_errors(report: &Report) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.level == "ERROR")
}

fn collect_manifests(dir: &Path, manifests: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, manifests);
        } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            manifests.push(path);
        }
    }
}

fn manifest_value(text: &str, section: &str, key: &str) -> Option<String> {
    let mut current = "";
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.starts_with('[') && line.ends_with(']') {
            current = line.trim_matches(['[', ']']);
        } else if current == section {
            if let Some((candidate, value)) = line.split_once('=') {
                if candidate.trim() == key {
                    return Some(value.trim().trim_matches('"').to_owned());
                }
            }
        }
    }
    None
}

fn section_keys(text: &str, section: &str) -> Vec<String> {
    let mut current = "";
    let mut keys = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.starts_with('[') && line.ends_with(']') {
            current = line.trim_matches(['[', ']']);
        } else if current == section {
            if let Some((key, _)) = line.split_once('=') {
                keys.push(key.trim().to_owned());
            }
        }
    }
    keys.sort();
    keys
}

fn has_section(text: &str, section: &str) -> bool {
    text.lines()
        .any(|line| line.split('#').next().unwrap_or_default().trim() == format!("[{section}]"))
}

fn target_kind(has_lib: bool, has_bin: bool) -> String {
    match (has_lib, has_bin) {
        (true, true) => "lib+bin",
        (true, false) => "lib",
        (false, true) => "bin",
        (false, false) => "other",
    }
    .to_owned()
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn render_json(report: &Report) -> Result<String> {
    serde_json::to_string_pretty(report).context("serialize crate-standard report")
}

fn render_markdown(report: &Report) -> String {
    let errors = report
        .findings
        .iter()
        .filter(|finding| finding.level == "ERROR")
        .count();
    let warnings = report
        .findings
        .iter()
        .filter(|finding| finding.level == "WARN")
        .count();
    let manual = report
        .findings
        .iter()
        .filter(|finding| finding.level == "MANUAL")
        .count();
    let mut out = format!(
        "# Crate standard report\n\nWorkspace crates: {}  \nDiscovered legacy crates: {}  \nFindings: {errors} ERROR, {warnings} WARN, {manual} MANUAL\n\n",
        report.workspace.len(),
        report.legacy.len()
    );
    for (title, records) in [
        ("Workspace", report.workspace.as_slice()),
        ("Legacy / quarantine", report.legacy.as_slice()),
    ] {
        out.push_str(&format!("## {title}\n\n| Crate | Manifest | Scope | Target | Publish | Features | Structure |\n|---|---|---|---|---|---|---|\n"));
        for record in records {
            let publish = match &record.publish {
                Some(registries) if registries.is_empty() => "false".to_owned(),
                Some(registries) => registries.join(", "),
                None => "default".to_owned(),
            };
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} | {} | {} | readme={}, changelog={}, agents={}, docs={}, target={} |\n",
                record.name,
                record.manifest_path,
                record.scope,
                record.target_kind,
                publish,
                record.features.join(", "),
                yes_no(record.structure.readme),
                yes_no(record.structure.changelog),
                yes_no(record.structure.agents),
                yes_no(record.structure.docs),
                yes_no(record.structure.target_root)
            ));
        }
        out.push('\n');
    }
    out.push_str("## Findings\n\n");
    if report.findings.is_empty() {
        out.push_str("No findings.\n");
    } else {
        for finding in &report.findings {
            out.push_str(&format!(
                "- **{}** `{}` at `{}` — {}\n",
                finding.level, finding.rule_id, finding.evidence_path, finding.suggested_action
            ));
        }
    }
    out
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "xhyper-crate-standard-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before unix epoch")
                .as_nanos()
        ))
    }

    fn write(path: &std::path::Path, text: &str) {
        fs::create_dir_all(path.parent().expect("file parent")).expect("create parent");
        fs::write(path, text).expect("write fixture");
    }

    #[test]
    fn inventories_workspace_and_legacy_with_stable_output() {
        let root = temp_root();
        // Path with spaces exercises stable relative paths.
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/a lib\", \"apps/tool\", \"legacy/member\"]\n",
        );
        write(
            &root.join("crates/a lib/Cargo.toml"),
            "[package]\nname = \"xhyper-a-lib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("crates/a lib/src/lib.rs"), "//! library\n");
        write(
            &root.join("apps/tool/Cargo.toml"),
            "[package]\nname = \"xhyper-tool\"\nversion = \"0.1.0\"\nedition = \"2021\"\npublish = false\n\n[[bin]]\nname = \"tool\"\npath = \"cmd/tool.rs\"\n\n[[bin]]\nname = \"missing\"\npath = \"cmd/missing.rs\"\n\n[features]\ndefault = []\n",
        );
        // Complete bin entry includes crate-level docs to avoid WARN noise.
        write(
            &root.join("apps/tool/cmd/tool.rs"),
            "//! tool binary\nfn main() {}\n",
        );
        // Workspace member under a legacy path: complete structure → no required-file ERROR.
        write(
            &root.join("legacy/member/Cargo.toml"),
            "[package]\nname = \"xhyper-legacy-member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("legacy/member/src/lib.rs"), "//! member\n");
        write(
            &root.join("legacy/member/README.md"),
            "# member\n\n## 非职责\n\n- none\n\n## 限制\n\n- none\n",
        );
        write(
            &root.join("legacy/member/CHANGELOG.md"),
            "# changelog\n\n## [Unreleased]\n\n",
        );
        write(&root.join("legacy/member/AGENTS.md"), "# agents\n");
        write(&root.join("legacy/member/docs/note.md"), "member docs\n");
        // Non-member legacy packages: MANUAL only.
        write(
            &root.join("legacy/standalone/Cargo.toml"),
            "[package]\nname = \"standalone\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        );
        write(
            &root.join("legacy/standalone/src/lib.rs"),
            "//! standalone\n",
        );
        write(
            &root.join("legacy/quarantine/Cargo.toml"),
            "[package]\nname = \"quarantine\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            &root.join("legacy/quarantine/src/lib.rs"),
            "//! quarantine\n",
        );

        let report = inventory(&root).expect("inventory");
        assert_eq!(report.workspace.len(), 3);
        assert_eq!(report.legacy.len(), 2);
        let tool = report
            .workspace
            .iter()
            .find(|record| record.name == "xhyper-tool")
            .expect("explicit bin package");
        assert_eq!(tool.target_kind, "bin");
        assert!(!tool.structure.target_root);
        assert_eq!(tool.publish.as_deref(), Some(&[][..]));
        let library = report
            .workspace
            .iter()
            .find(|record| record.manifest_path == "crates/a lib/Cargo.toml")
            .expect("library package with spaced path");
        assert_eq!(library.name, "xhyper-a-lib");
        assert_eq!(library.target_kind, "lib");
        let legacy_member = report
            .workspace
            .iter()
            .find(|record| record.name == "xhyper-legacy-member" && record.scope == "workspace")
            .expect("legacy-path workspace member");
        assert!(legacy_member.structure.readme);
        assert!(report
            .legacy
            .iter()
            .any(|record| record.name == "quarantine" && record.scope == "legacy-quarantine"));
        assert!(report
            .legacy
            .iter()
            .any(|record| record.name == "standalone" && record.scope == "legacy-standalone"));
        assert!(render_markdown(&report)
            .contains("readme=no, changelog=no, agents=no, docs=no, target=yes"));

        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "legacy-quarantine-review" && finding.level == "MANUAL"
        }));
        // Non-workspace legacy packages never get workspace required-file ERROR rules.
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-required-file"
                && (finding.evidence_path.starts_with("legacy/standalone/")
                    || finding.evidence_path.starts_with("legacy/quarantine/"))
        }));
        // Complete workspace member under legacy/ has no required-file ERROR and no policy WARN.
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-required-file"
                && finding.evidence_path.starts_with("legacy/member/")
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.level == "WARN" && finding.evidence_path.starts_with("legacy/member/")
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-independent-version"
                && finding.evidence_path.starts_with("legacy/member/")
        }));
        // Incomplete workspace members still produce ERROR findings (including paths with spaces).
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-required-file"
                && finding.level == "ERROR"
                && finding.evidence_path == "crates/a lib/README.md"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-required-file"
                && finding.level == "ERROR"
                && finding.evidence_path == "apps/tool/README.md"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-required-file"
                && finding.level == "ERROR"
                && finding.evidence_path == "apps/tool/cmd/missing.rs"
        }));
        // Incomplete packages: missing CHANGELOG must not also emit unreleased WARN.
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-changelog-unreleased"
                && (finding.evidence_path.starts_with("crates/a lib/")
                    || finding.evidence_path.starts_with("apps/tool/"))
        }));
        // Present entries with `//!` should not emit root-docs WARN.
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-root-docs"
                && (finding.evidence_path == "crates/a lib/src/lib.rs"
                    || finding.evidence_path == "apps/tool/cmd/tool.rs"
                    || finding.evidence_path == "legacy/member/src/lib.rs")
        }));
        assert!(has_blocking_errors(&report));

        // Stable render: identical bytes across two serializations.
        assert_eq!(render_markdown(&report), render_markdown(&report));
        assert_eq!(render_json(&report).unwrap(), render_json(&report).unwrap());
        let again = inventory(&root).expect("second inventory");
        assert_eq!(
            render_json(&report).unwrap(),
            render_json(&again).unwrap(),
            "duplicate inventory must be byte-stable"
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn empty_report_is_renderable_and_missing_input_fails() {
        let report = Report::default();
        assert!(render_markdown(&report).contains("Workspace crates: 0"));
        assert!(render_markdown(&report).contains("No findings."));
        assert_eq!(render_json(&report).unwrap(), render_json(&report).unwrap());
        assert!(inventory(&temp_root()).is_err());
    }

    #[test]
    fn check_blocks_only_errors() {
        let mut report = Report::default();
        report.findings.push(Finding {
            rule_id: "legacy-quarantine-review",
            level: "MANUAL",
            evidence_path: "legacy/Cargo.toml".into(),
            evidence_line: Some(1),
            suggested_action: "review".into(),
        });
        report.findings.push(Finding {
            rule_id: "crate-changelog-unreleased",
            level: "WARN",
            evidence_path: "some/CHANGELOG.md".into(),
            evidence_line: None,
            suggested_action: "optional".into(),
        });
        report.findings.push(Finding {
            rule_id: "crate-root-docs",
            level: "WARN",
            evidence_path: "some/src/lib.rs".into(),
            evidence_line: None,
            suggested_action: "add //!".into(),
        });
        assert!(
            !has_blocking_errors(&report),
            "WARN/MANUAL alone must not block --check"
        );
        report.findings.push(Finding {
            rule_id: "crate-independent-version",
            level: "ERROR",
            evidence_path: "Cargo.toml".into(),
            evidence_line: Some(3),
            suggested_action: "set independent version".into(),
        });
        assert!(has_blocking_errors(&report));
    }

    #[test]
    fn version_workspace_inheritance_is_error() {
        let root = temp_root();
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/ws_ver\"]\n\n[workspace.package]\nversion = \"0.1.0\"\n",
        );
        write(
            &root.join("crates/ws_ver/Cargo.toml"),
            "[package]\nname = \"xhyper-ws-ver\"\nversion.workspace = true\nedition = \"2021\"\n",
        );
        write(&root.join("crates/ws_ver/src/lib.rs"), "//! docs\n");
        write(
            &root.join("crates/ws_ver/README.md"),
            "# ws\n\n## 非职责\n\n- n/a\n\n## 限制\n\n- n/a\n",
        );
        write(
            &root.join("crates/ws_ver/CHANGELOG.md"),
            "## [Unreleased]\n",
        );
        write(&root.join("crates/ws_ver/AGENTS.md"), "# agents\n");
        write(&root.join("crates/ws_ver/docs/.gitkeep"), "");

        let report = inventory(&root).expect("inventory");
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-independent-version"
                && finding.level == "ERROR"
                && finding.evidence_path == "crates/ws_ver/Cargo.toml"
        }));
        assert!(has_blocking_errors(&report));
        // cargo metadata may still resolve a version; rule is file-literal based.
        assert!(independent_version_finding(
            &root,
            &root.join("crates/ws_ver/Cargo.toml"),
            &fs::read_to_string(root.join("crates/ws_ver/Cargo.toml")).unwrap()
        )
        .is_some());
        assert!(independent_version_finding(
            &root,
            Path::new("Cargo.toml"),
            "[package]\nname = \"ok\"\nversion = \"1.2.3\"\n"
        )
        .is_none());
        assert!(independent_version_finding(
            &root,
            Path::new("Cargo.toml"),
            "[package]\nname = \"bad\"\nversion = \"1.2\"\n"
        )
        .is_some());

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn changelog_without_unreleased_and_missing_crate_docs_are_warn_only() {
        let root = temp_root();
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/warn_pkg\"]\n",
        );
        write(
            &root.join("crates/warn_pkg/Cargo.toml"),
            "[package]\nname = \"xhyper-warn-pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        // No crate-level `//!`.
        write(&root.join("crates/warn_pkg/src/lib.rs"), "pub fn f() {}\n");
        write(&root.join("crates/warn_pkg/README.md"), "# warn\n");
        write(
            &root.join("crates/warn_pkg/CHANGELOG.md"),
            "# Changelog\n\n## 0.1.0\n\n- initial\n",
        );
        write(&root.join("crates/warn_pkg/AGENTS.md"), "# agents\n");
        write(&root.join("crates/warn_pkg/docs/.gitkeep"), "");

        let report = inventory(&root).expect("inventory");
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-changelog-unreleased"
                && finding.level == "WARN"
                && finding.evidence_path == "crates/warn_pkg/CHANGELOG.md"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-root-docs"
                && finding.level == "WARN"
                && finding.evidence_path == "crates/warn_pkg/src/lib.rs"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule_id == "crate-readme-fields"
                && finding.level == "WARN"
                && finding.evidence_path == "crates/warn_pkg/README.md"
        }));
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.level == "ERROR"),
            "complete structure with only content gaps must not ERROR"
        );
        assert!(
            !has_blocking_errors(&report),
            "WARN-only findings must not block --check"
        );

        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn readme_with_non_goals_and_limits_skips_fields_warn() {
        let root = temp_root();
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/ok_readme\"]\n",
        );
        write(
            &root.join("crates/ok_readme/Cargo.toml"),
            "[package]\nname = \"xhyper-ok-readme\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("crates/ok_readme/src/lib.rs"), "//! docs\n");
        write(
            &root.join("crates/ok_readme/README.md"),
            "# ok\n\n## 非职责\n\n- not this\n\n## 限制与安全\n\n- limited\n",
        );
        write(
            &root.join("crates/ok_readme/CHANGELOG.md"),
            "## [Unreleased]\n",
        );
        write(&root.join("crates/ok_readme/AGENTS.md"), "# agents\n");
        write(&root.join("crates/ok_readme/docs/.gitkeep"), "");

        let report = inventory(&root).expect("inventory");
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == "crate-readme-fields"),
            "complete README fields must not WARN"
        );
        assert!(!has_blocking_errors(&report));
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn name_prefix_rule_classifies_new_and_existing_crates() {
        let root = temp_root();
        // cutover 后 EXISTING_NON_PREFIXED 为空：
        //   - `xhyper-new`      → 合规，不报
        //   - `xtask`           → 缺前缀 → ERROR
        //   - `brand_new_crate` → 缺前缀 → ERROR
        write(
            &root.join("Cargo.toml"),
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/new\", \"crates/existing\", \"crates/rogue\"]\n",
        );
        write(
            &root.join("crates/new/Cargo.toml"),
            "[package]\nname = \"xhyper-new\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("crates/new/src/lib.rs"), "//! docs\n");
        write(
            &root.join("crates/new/README.md"),
            "# new\n\n## 非职责\n\n- n/a\n\n## 限制\n\n- n/a\n",
        );
        write(&root.join("crates/new/CHANGELOG.md"), "## [Unreleased]\n");
        write(&root.join("crates/new/AGENTS.md"), "# agents\n");
        write(&root.join("crates/new/docs/.gitkeep"), "");

        write(
            &root.join("crates/existing/Cargo.toml"),
            "[package]\nname = \"xtask\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("crates/existing/src/lib.rs"), "//! docs\n");
        write(
            &root.join("crates/existing/README.md"),
            "# existing\n\n## 非职责\n\n- n/a\n\n## 限制\n\n- n/a\n",
        );
        write(
            &root.join("crates/existing/CHANGELOG.md"),
            "## [Unreleased]\n",
        );
        write(&root.join("crates/existing/AGENTS.md"), "# agents\n");
        write(&root.join("crates/existing/docs/.gitkeep"), "");

        write(
            &root.join("crates/rogue/Cargo.toml"),
            "[package]\nname = \"brand_new_crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(&root.join("crates/rogue/src/lib.rs"), "//! docs\n");
        write(
            &root.join("crates/rogue/README.md"),
            "# rogue\n\n## 非职责\n\n- n/a\n\n## 限制\n\n- n/a\n",
        );
        write(&root.join("crates/rogue/CHANGELOG.md"), "## [Unreleased]\n");
        write(&root.join("crates/rogue/AGENTS.md"), "# agents\n");
        write(&root.join("crates/rogue/docs/.gitkeep"), "");

        let report = inventory(&root).expect("inventory");

        // 合规 crate：不报 crate-name-prefix
        assert!(!report.findings.iter().any(|finding| {
            finding.rule_id == "crate-name-prefix"
                && finding.evidence_path == "crates/new/Cargo.toml"
        }));

        // cutover 后 allowlist 为空：旧短名亦 ERROR
        let existing = report.findings.iter().find(|finding| {
            finding.rule_id == "crate-name-prefix"
                && finding.evidence_path == "crates/existing/Cargo.toml"
        });
        let existing = existing.expect("unprefixed package should emit ERROR finding");
        assert_eq!(existing.level, "ERROR");
        assert_eq!(existing.evidence_line, Some(2));

        // 新 crate 缺前缀 → ERROR
        let rogue = report.findings.iter().find(|finding| {
            finding.rule_id == "crate-name-prefix"
                && finding.evidence_path == "crates/rogue/Cargo.toml"
        });
        let rogue = rogue.expect("rogue crate should emit ERROR finding");
        assert_eq!(rogue.level, "ERROR");
        assert_eq!(rogue.evidence_line, Some(2));
        assert!(rogue.suggested_action.contains(
            "rename `[package] name` from `brand_new_crate` to `xhyper-brand_new_crate`"
        ));

        // ERROR 阻断 --check
        assert!(has_blocking_errors(&report));

        fs::remove_dir_all(root).expect("remove fixture");
    }
}
