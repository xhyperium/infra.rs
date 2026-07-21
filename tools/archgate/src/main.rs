//! archgate —— 架构门禁 CLI。
//!
//! 只读检查 workspace 的 public API 泄漏、时间源与依赖边等架构约束；
//! 不修改业务 crate，不替代 `lint-deps` / `crate-standard` 的职责。
//!
//! SPEC-KERNEL-002 §12.2 命名规则见 [`kernel_rules`]。

mod kernel_rules;
mod registry;

use anyhow::{bail, Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::Parser;
use registry::{DependencyPolicy, WorkspaceRegistry};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    json: bool,
}

#[derive(Default)]
struct Diagnostics {
    public_api_leaks: Vec<String>,
    provider_api_leaks: Vec<String>,
    external_time_calls: Vec<String>,
    time_call_keys: Vec<String>,
    unapproved_time_calls: Vec<String>,
    stale_time_call_exceptions: Vec<String>,
    undeclared_edges: Vec<String>,
    /// kernel 外部依赖白名单违规（仅允许 thiserror）。
    kernel_external_deps: Vec<String>,
    /// kernel 源码 use/pub 行中的禁止依赖标识。
    kernel_forbidden_tokens: Vec<String>,
}

/// kernel 源码 use/pub 行禁止出现的标识。
const KERNEL_FORBIDDEN_TOKENS: &[&str] = &["anyhow", "serde", "tokio", "chrono", "tracing"];

fn is_allowed_kernel_external_dep(name: &str) -> bool {
    kernel_rules::is_allowed_kernel_external_dep(name)
}

fn parse_normal_dependency_names(manifest: &str) -> Vec<String> {
    kernel_rules::parse_normal_dependency_names(manifest)
}

fn collect_kernel_external_dep_violations(manifest: &str) -> Vec<String> {
    parse_normal_dependency_names(manifest)
        .into_iter()
        .filter(|name| !is_allowed_kernel_external_dep(name))
        .collect()
}

/// 判断是否为 use / pub 行（trim 后以 `use ` / `pub ` 开头，含 `pub use`）。
fn is_use_or_pub_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("use ") || trimmed.starts_with("pub ")
}

/// 在 use/pub 行中查找禁止 token；命中则返回 token 名。
fn forbidden_token_in_use_or_pub_line(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || !is_use_or_pub_line(line) {
        return None;
    }
    for token in KERNEL_FORBIDDEN_TOKENS {
        // 按路径/标识片段匹配：`use anyhow::`、`pub use serde`、`use tokio as _`
        if contains_ident_token(trimmed, token) {
            return Some(*token);
        }
    }
    None
}

fn contains_ident_token(line: &str, token: &str) -> bool {
    let bytes = line.as_bytes();
    let t = token.as_bytes();
    if t.is_empty() || bytes.len() < t.len() {
        return false;
    }
    for i in 0..=bytes.len() - t.len() {
        if &bytes[i..i + t.len()] != t {
            continue;
        }
        let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
        let after = i + t.len();
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_kernel_public_api_leak(entry: &str) -> bool {
    entry.starts_with("crates/kernel/") || entry.contains("/crates/kernel/")
}

fn metadata() -> Result<Metadata> {
    MetadataCommand::new()
        .no_deps()
        .exec()
        .context("cargo metadata failed")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn package_path(root: &Path, package: &cargo_metadata::Package) -> String {
    let manifest = package.manifest_path.as_std_path();
    manifest
        .parent()
        .unwrap_or(manifest)
        .strip_prefix(root)
        .unwrap_or(manifest)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_under(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

/// F-04：exceptions.toml 严格 schema。空 registry 合法；非空条目必须带治理字段。
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExceptionsToml {
    schema_version: u32,
    #[serde(default)]
    exception: Vec<ExceptionEntryToml>,
    /// 兼容历史 `[[exceptions]]` 表名（与 `exception` 二选一聚合）。
    #[serde(default, alias = "exceptions")]
    exceptions_legacy: Vec<ExceptionEntryToml>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExceptionEntryToml {
    /// 兼容极简 time-call allowlist 形状。
    rule: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    clause: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    risk: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    compensation: Option<String>,
    #[serde(default)]
    expires: Option<String>,
    #[serde(default)]
    tracking: Option<String>,
    #[serde(default)]
    approved_by: Vec<String>,
}

fn parse_exceptions_registry(text: &str) -> Result<ExceptionsToml> {
    let trimmed = text
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n");
    if trimmed.is_empty() {
        return Ok(ExceptionsToml {
            schema_version: 1,
            exception: Vec::new(),
            exceptions_legacy: Vec::new(),
        });
    }
    let raw: ExceptionsToml =
        toml::from_str(text).context("exceptions.toml TOML deserialize (strict schema)")?;
    if raw.schema_version != 1 {
        bail!(
            "unsupported exceptions.toml schema_version={} (supported=1)",
            raw.schema_version
        );
    }
    let all: Vec<&ExceptionEntryToml> = raw
        .exception
        .iter()
        .chain(raw.exceptions_legacy.iter())
        .collect();
    for (i, e) in all.iter().enumerate() {
        if e.rule.trim().is_empty() {
            bail!("exceptions entry[{i}] has empty rule");
        }
        // 非空 registry 的每个条目都必须满足 EXCEPTION_POLICY 核心字段（F-04）。
        // 历史 rule-only 形状不再授予豁免，避免永久、无批准的 allowlist。
        for (field, val) in [
            ("id", e.id.as_deref()),
            ("owner", e.owner.as_deref()),
            ("expires", e.expires.as_deref()),
            ("reason", e.reason.as_deref()),
            ("risk", e.risk.as_deref()),
            ("scope", e.scope.as_deref()),
            ("clause", e.clause.as_deref()),
            ("compensation", e.compensation.as_deref()),
            ("tracking", e.tracking.as_deref()),
        ] {
            if val.map(str::trim).unwrap_or("").is_empty() {
                bail!(
                    "exceptions entry rule={:?}: governance field `{field}` is required",
                    e.rule
                );
            }
        }
        if e.approved_by.is_empty() || e.approved_by.iter().any(|name| name.trim().is_empty()) {
            bail!(
                "exceptions entry rule={:?}: approved_by must contain only non-empty identities (AI cannot approve)",
                e.rule
            );
        }
        // expires 形如 YYYY-MM-DD
        let exp = e.expires.as_deref().unwrap_or("");
        if exp.len() != 10 || exp.as_bytes()[4] != b'-' || exp.as_bytes()[7] != b'-' {
            bail!(
                "exceptions entry rule={:?}: expires must be YYYY-MM-DD, got {exp:?}",
                e.rule
            );
        }
    }
    Ok(raw)
}

fn parse_time_call_allowlist(text: &str) -> Result<HashSet<String>> {
    let reg = parse_exceptions_registry(text)?;
    let mut locations = HashSet::new();
    for e in reg.exception.iter().chain(reg.exceptions_legacy.iter()) {
        let Some(location) = e.rule.strip_prefix("time-call:") else {
            continue;
        };
        if location.is_empty() {
            bail!("time-call exception must include a location");
        }
        locations.insert(location.to_owned());
    }
    Ok(locations)
}

fn time_call_allowlist(root: &Path) -> Result<HashSet<String>> {
    let text = fs::read_to_string(root.join(".architecture/exceptions.toml"))?;
    parse_time_call_allowlist(&text)
}

fn classify_time_calls(
    observed: &[String],
    allowed: &HashSet<String>,
) -> (Vec<String>, Vec<String>) {
    let observed_set: HashSet<_> = observed.iter().cloned().collect();
    let mut unapproved: Vec<_> = observed
        .iter()
        .filter(|location| !allowed.contains(*location))
        .cloned()
        .collect();
    let mut stale: Vec<_> = allowed.difference(&observed_set).cloned().collect();
    unapproved.sort();
    stale.sort();
    (unapproved, stale)
}

fn diagnostics(root: &Path, metadata: &Metadata) -> Result<Diagnostics> {
    let mut result = Diagnostics::default();
    let mut paths = HashMap::new();
    for package in &metadata.packages {
        paths.insert(package.id.clone(), package_path(root, package));
    }

    for package in &metadata.packages {
        let path = paths
            .get(&package.id)
            .expect("package path populated above")
            .clone();
        let is_kernel = package.name.as_str() == "kernel"
            || package.name.as_str() == "xhyper-kernel"
            || path == "crates/kernel";

        // R7：kernel 外部依赖仅允许 thiserror（解析 Cargo.toml [dependencies]）。
        if is_kernel {
            let manifest_text = fs::read_to_string(package.manifest_path.as_std_path())
                .with_context(|| format!("read kernel manifest {}", package.manifest_path))?;
            for name in collect_kernel_external_dep_violations(&manifest_text) {
                result
                    .kernel_external_deps
                    .push(format!("crates/kernel: disallowed dependency `{name}`"));
            }
        }

        let src = root.join(&path).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src, &mut files);
        for file in files {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            for (line_no, line) in content.lines().enumerate() {
                let line_number = line_no + 1;
                if line.contains("anyhow::Error")
                    && (line.contains("pub ") || line.contains("impl From<"))
                {
                    result.public_api_leaks.push(format!(
                        "{}:{line_number}: {line}",
                        file.strip_prefix(root).unwrap_or(&file).display()
                    ));
                }
                if !path.starts_with("crates/adapters/exchange/")
                    && !path.starts_with("legacy/")
                    && !path.starts_with("tools/")
                    && line.contains("pub ")
                    && ["Binance", "Okx", "BinanceAdapter", "OkxAdapter"]
                        .iter()
                        .any(|name| line.contains(name))
                {
                    result.provider_api_leaks.push(format!(
                        "{}/{}:{line_number}: {line}",
                        path,
                        file.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
                if !is_under(&path, "crates/kernel")
                    && !is_under(&path, "tools")
                    && (line.contains("SystemTime::now") || line.contains("Utc::now"))
                {
                    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
                    let file_path = format!("{path}/{file_name}");
                    let pattern = if line.contains("SystemTime::now") {
                        "SystemTime::now"
                    } else {
                        "Utc::now"
                    };
                    result
                        .external_time_calls
                        .push(format!("{file_path}:{line_number}"));
                    result.time_call_keys.push(format!("{file_path}:{pattern}"));
                }
                // R7：kernel 源码 use/pub 行禁止 anyhow/serde/tokio/chrono/tracing。
                if is_kernel {
                    if let Some(token) = forbidden_token_in_use_or_pub_line(line) {
                        let rel = file.strip_prefix(root).unwrap_or(&file).display();
                        result.kernel_forbidden_tokens.push(format!(
                            "{rel}:{line_number}: forbidden token `{token}`: {line}"
                        ));
                    }
                }
            }
        }
    }
    let allowlist = time_call_allowlist(root)?;
    (
        result.unapproved_time_calls,
        result.stale_time_call_exceptions,
    ) = classify_time_calls(&result.time_call_keys, &allowlist);

    let full = cargo_metadata::MetadataCommand::new().exec()?;
    let registered_layers = WorkspaceRegistry::load(root)?.layers();
    let allowed_layers = DependencyPolicy::load(root)?.allowances;
    let by_id: HashMap<_, _> = full.packages.iter().map(|p| (&p.id, p)).collect();
    let members: HashSet<_> = full.workspace_members.iter().collect();
    for id in &members {
        let Some(package) = by_id.get(id) else {
            continue;
        };
        let from_path = package_path(root, package);
        let from = registered_layers
            .get(&from_path)
            .map(String::as_str)
            .unwrap_or("unknown");
        for dependency in &package.dependencies {
            let Some(target) = full.packages.iter().find(|p| p.name == dependency.name) else {
                continue;
            };
            if !members.contains(&target.id)
                || dependency.kind != cargo_metadata::DependencyKind::Normal
            {
                continue;
            }
            let to_path = package_path(root, target);
            let to = registered_layers
                .get(&to_path)
                .map(String::as_str)
                .unwrap_or("unknown");
            // R3.1 / ADR-005：bootstrap 是唯一 L1 组装豁免，可依赖其他 L1。
            // 与 lint-deps 一致：policy 层禁止 infra→infra，仅 bootstrap 包名豁免。
            let bootstrap_l1_exempt = (package.name == "bootstrap"
                || package.name == "xhyper-bootstrap")
                && from == "infra"
                && to == "infra"
                && package.name != target.name;
            let allowed = bootstrap_l1_exempt
                || allowed_layers
                    .get(from)
                    .is_some_and(|layers| layers.contains(to));
            if !allowed {
                result.undeclared_edges.push(format!(
                    "{} ({from}) -> {} ({to})",
                    package.name, target.name
                ));
            }
        }
    }
    result.public_api_leaks.sort();
    result.provider_api_leaks.sort();
    result.external_time_calls.sort();
    result.undeclared_edges.sort();
    result.kernel_external_deps.sort();
    result.kernel_forbidden_tokens.sort();
    Ok(result)
}
/// Cargo `package.publish`：`None` = 未限制（可发布）；`Some([])` = publish=false。
fn cargo_allows_publish(package: &cargo_metadata::Package) -> bool {
    match &package.publish {
        None => true,
        Some(regs) => !regs.is_empty(),
    }
}

/// registry.publish 与 Cargo.toml publish 不一致的 unit 路径列表。
fn collect_publish_drift(
    packages: &[cargo_metadata::Package],
    root: &Path,
    registered: &HashMap<String, bool>,
) -> Vec<String> {
    let mut drift = Vec::new();
    for package in packages {
        let path = package_path(root, package);
        let Some(&want) = registered.get(&path) else {
            continue;
        };
        let actual = cargo_allows_publish(package);
        if !publish_flags_match(want, actual) {
            drift.push(format!(
                "{path}: registry.publish={want} cargo.allows_publish={actual}"
            ));
        }
    }
    drift.sort();
    drift
}

/// 单条 status 边裁决（可单测）。`None` = 允许。
fn status_edge_violation(
    from_name: &str,
    from_layer: &str,
    from_status: &str,
    to_name: &str,
    to_status: &str,
) -> Option<String> {
    if to_status == "quarantined" && from_layer != "tools" {
        return Some(format!(
            "{from_name} ({from_status}/{from_layer}) -> {to_name} ({to_status}): normal dep on quarantined"
        ));
    }
    if matches!(from_layer, "tools" | "test-support") {
        return None;
    }
    if from_status == "stable" && matches!(to_status, "incubating" | "experimental" | "quarantined")
    {
        return Some(format!(
            "{from_name} ({from_status}/{from_layer}) -> {to_name} ({to_status}): stable must not normal-depend on {to_status}"
        ));
    }
    None
}

/// 生产向 status 边：stable 不得 normal 依赖 incubating/experimental/quarantined。
/// tools / test-support 作为依赖方豁免（工具链与测试平面常态依赖孵化 crate）。
/// 任意非 tools 层 normal 依赖 quarantined 亦禁止。
fn collect_status_edges(
    packages: &[cargo_metadata::Package],
    root: &Path,
    layers: &HashMap<String, String>,
    statuses: &HashMap<String, String>,
) -> Vec<String> {
    let mut edges = Vec::new();
    for package in packages {
        let from_path = package_path(root, package);
        let from_layer = layers
            .get(&from_path)
            .map(String::as_str)
            .unwrap_or("unknown");
        let from_status = statuses
            .get(&from_path)
            .map(String::as_str)
            .unwrap_or("incubating");

        for dep in &package.dependencies {
            if dep.kind != cargo_metadata::DependencyKind::Normal {
                continue;
            }
            let Some(target) = packages.iter().find(|p| p.name == dep.name) else {
                continue;
            };
            let to_path = package_path(root, target);
            let to_status = statuses
                .get(&to_path)
                .map(String::as_str)
                .unwrap_or("incubating");
            if let Some(v) = status_edge_violation(
                package.name.as_str(),
                from_layer,
                from_status,
                target.name.as_str(),
                to_status,
            ) {
                edges.push(v);
            }
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

/// registry vs cargo publish 是否一致（可单测）。
fn publish_flags_match(registry_publish: bool, cargo_allows: bool) -> bool {
    registry_publish == cargo_allows
}

fn run(json: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let registry = WorkspaceRegistry::load(&root)?;
    let units = registry.path_set();
    let registered_len = registry.units.len();
    let m = metadata()?;
    let expected: HashSet<_> = m
        .packages
        .iter()
        .filter_map(|p| {
            p.manifest_path.parent().map(|x| {
                x.to_string()
                    .replace(root.to_str().unwrap_or(""), "")
                    .trim_start_matches('/')
                    .to_owned()
            })
        })
        .collect();
    let missing: Vec<_> = expected.difference(&units).cloned().collect();
    let stale: Vec<_> = units.difference(&expected).cloned().collect();
    let duplicate = registered_len != units.len();
    let diagnostics = diagnostics(&root, &m)?;
    let invalid = registry.invalid_statuses();
    let paths: HashMap<_, _> = m
        .packages
        .iter()
        .filter_map(|p| {
            p.manifest_path.parent().map(|x| {
                (
                    p.name.to_string(),
                    x.to_string()
                        .replace(root.to_str().unwrap_or(""), "")
                        .trim_start_matches('/')
                        .to_owned(),
                )
            })
        })
        .collect();
    let lm = registry.layers();
    let dep_policy = DependencyPolicy::load(&root)?;
    let rules = &dep_policy.forbidden;
    let mut bad = Vec::new();
    for p in &m.packages {
        if let Some(src) = paths.get(p.name.as_str()).and_then(|x| lm.get(x)) {
            for d in &p.dependencies {
                if let Some(dst) = paths.get(d.name.as_str()).and_then(|x| lm.get(x)) {
                    if rules.iter().any(|(a, b)| a == src && b == dst) {
                        bad.push(format!("{} -> {}", p.name, d.name));
                    }
                }
            }
        }
    }
    let kernel_public_api_leaks: Vec<_> = diagnostics
        .public_api_leaks
        .iter()
        .filter(|entry| is_kernel_public_api_leak(entry))
        .cloned()
        .collect();

    let workspace_names: HashSet<String> = m.packages.iter().map(|p| p.name.to_string()).collect();
    let kernel_package = m
        .packages
        .iter()
        .find(|package| package_path(&root, package) == "crates/kernel")
        .context("workspace metadata is missing crates/kernel")?;
    let kernel_rules_report =
        kernel_rules::evaluate_kernel_rules(&root, &workspace_names, &kernel_package.dependencies)?;
    let kernel_rules_json: serde_json::Map<String, serde_json::Value> = kernel_rules_report
        .results
        .iter()
        .map(|r| {
            (
                r.id.to_string(),
                serde_json::json!({ "ok": r.ok, "detail": r.detail }),
            )
        })
        .collect();

    let reg_publish = registry.publish_flags();
    let publish_drift = collect_publish_drift(&m.packages, &root, &reg_publish);
    let reg_status = registry.statuses();
    let status_edges = collect_status_edges(&m.packages, &root, &lm, &reg_status);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "registered":registered_len,
                "schema_version":registry.schema_version,
                "members":expected.len(),
                "missing":missing,
                "stale":stale,
                "duplicate":duplicate,
                "invalid_status":invalid,
                "forbidden":bad,
                "publish_drift":publish_drift,
                "status_edges":status_edges,
                "public_api_leaks":diagnostics.public_api_leaks,
                "provider_api_leaks":diagnostics.provider_api_leaks,
                "external_time_calls":diagnostics.external_time_calls,
                "unapproved_time_calls":diagnostics.unapproved_time_calls,
                "stale_time_call_exceptions":diagnostics.stale_time_call_exceptions,
                "undeclared_edges":diagnostics.undeclared_edges,
                "kernel_external_deps":diagnostics.kernel_external_deps,
                "kernel_forbidden_tokens":diagnostics.kernel_forbidden_tokens,
                "kernel_public_api_leaks":kernel_public_api_leaks,
                "kernel_rules": kernel_rules_json,
                "kernel_internal_count": kernel_rules_report.internal_count,
            })
        );
    }
    if !missing.is_empty()
        || !stale.is_empty()
        || duplicate
        || !invalid.is_empty()
        || !bad.is_empty()
        || !publish_drift.is_empty()
        || !status_edges.is_empty()
        || !diagnostics.unapproved_time_calls.is_empty()
        || !diagnostics.stale_time_call_exceptions.is_empty()
        || !diagnostics.undeclared_edges.is_empty()
        || !diagnostics.kernel_external_deps.is_empty()
        || !diagnostics.kernel_forbidden_tokens.is_empty()
        || !kernel_public_api_leaks.is_empty()
        || !kernel_rules_report.violations.is_empty()
    {
        bail!(
            "architecture drift: missing={missing:?} stale={stale:?} duplicate={duplicate} invalid_status={invalid:?} forbidden={bad:?} publish_drift={publish_drift:?} status_edges={status_edges:?} unapproved_time_calls={:?} stale_time_call_exceptions={:?} kernel_external_deps={:?} kernel_forbidden_tokens={:?} kernel_public_api_leaks={:?} kernel_rules={:?}",
            diagnostics.unapproved_time_calls,
            diagnostics.stale_time_call_exceptions,
            diagnostics.kernel_external_deps,
            diagnostics.kernel_forbidden_tokens,
            kernel_public_api_leaks,
            kernel_rules_report.violations
        );
    }
    if !json {
        println!(
            "archgate: PASS ({} workspace units registered; kernel_rules={})",
            expected.len(),
            kernel_rules_report.results.len()
        );
    }
    Ok(())
}
fn main() -> Result<()> {
    run(Cli::parse().json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_path_boundary_is_exact() {
        assert!(is_under("crates/kernel", "crates/kernel"));
        assert!(is_under("crates/kernel/src/lib.rs", "crates/kernel"));
        assert!(!is_under("crates/kernelish", "crates/kernel"));
    }

    #[test]
    fn kernel_external_dep_whitelist_only_thiserror() {
        assert!(is_allowed_kernel_external_dep("thiserror"));
        assert!(!is_allowed_kernel_external_dep("anyhow"));
        assert!(!is_allowed_kernel_external_dep("serde"));
        assert!(!is_allowed_kernel_external_dep("tokio"));
        assert!(!is_allowed_kernel_external_dep("chrono"));
        assert!(!is_allowed_kernel_external_dep("tracing"));
    }

    #[test]
    fn parse_normal_dependency_names_reads_dependencies_section_only() {
        let manifest = r#"
[package]
name = "kernel"

[dependencies]
thiserror = { workspace = true }
anyhow = "1"

[dev-dependencies]
serde = "1"

[build-dependencies]
tokio = "1"
"#;
        let names = parse_normal_dependency_names(manifest);
        assert_eq!(names, vec!["thiserror", "anyhow"]);
        let violations = collect_kernel_external_dep_violations(manifest);
        assert_eq!(violations, vec!["anyhow"]);
    }

    #[test]
    fn forbidden_tokens_detected_on_use_or_pub_lines() {
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use anyhow::Result;"),
            Some("anyhow")
        );
        assert_eq!(
            forbidden_token_in_use_or_pub_line("pub use serde::Serialize;"),
            Some("serde")
        );
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use tokio::time;"),
            Some("tokio")
        );
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use chrono::Utc;"),
            Some("chrono")
        );
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use tracing::info;"),
            Some("tracing")
        );
        // 注释行不报
        assert_eq!(
            forbidden_token_in_use_or_pub_line("// use anyhow::Error"),
            None
        );
        // 允许 thiserror
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use thiserror::Error;"),
            None
        );
        // 子串不误报（标识边界）
        assert_eq!(
            forbidden_token_in_use_or_pub_line("use my_serde_helpers::X;"),
            None
        );
    }

    #[test]
    fn kernel_public_api_leak_path_filter() {
        // 用 concat 拆开字面量，避免 archgate 自身被 public_api_leaks 扫描误报
        // （扫描条件：同行同时含 anyhow Error 类型路径与 pub 可见性关键字 / From 实现）。
        let kernel_entry = concat!(
            "crates/kernel/src/error.rs:10: ",
            "pub type E = ",
            "anyhow::Error"
        );
        let bootstrap_entry = concat!(
            "crates/infra/bootstrap/src/lib.rs:1: ",
            "pub type E = ",
            "anyhow::Error"
        );
        assert!(is_kernel_public_api_leak(kernel_entry));
        assert!(!is_kernel_public_api_leak(bootstrap_entry));
    }

    #[test]
    fn time_call_exceptions_are_machine_readable_and_exact() {
        let text = r#"
schema_version = 1
[[exceptions]]
rule = "time-call:crates/adapters/exchange/binance/rest.rs:SystemTime::now"
id = "EXC-001"
clause = "CONSTITUTION §10"
reason = "integration test fixture"
risk = "low test-only"
scope = "tools/archgate tests"
owner = "architecture-owner"
compensation = "time-call allowlist remains exact"
expires = "2026-10-01"
tracking = "https://example.invalid/issues/1"
approved_by = ["maintainer", "risk-owner"]
"#;
        let allowlist = parse_time_call_allowlist(text).expect("valid exception registry");
        assert!(allowlist.contains("crates/adapters/exchange/binance/rest.rs:SystemTime::now"));
        assert!(!allowlist.contains("crates/adapters/exchange/binance/rest.rs:Utc::now"));
    }

    #[test]
    fn exceptions_empty_registry_ok() {
        assert!(parse_exceptions_registry("# empty\n").is_ok());
        assert!(parse_exceptions_registry("").is_ok());
        assert!(parse_exceptions_registry("schema_version = 1\n").is_ok());
    }

    #[test]
    fn exceptions_reject_partial_governance_and_unknown_fields() {
        let missing_schema = r#"
[[exception]]
rule = "time-call:foo.rs:now"
"#;
        assert!(parse_exceptions_registry(missing_schema).is_err());

        let rule_only = r#"
schema_version = 1
[[exception]]
rule = "time-call:foo.rs:now"
"#;
        assert!(parse_exceptions_registry(rule_only).is_err());

        let blank_rule = r#"
schema_version = 1
[[exception]]
rule = "   "
"#;
        assert!(parse_exceptions_registry(blank_rule).is_err());

        let partial = r#"
schema_version = 1
[[exception]]
rule = "time-call:foo.rs:now"
owner = "alice"
"#;
        assert!(parse_exceptions_registry(partial).is_err());

        for missing in ["id", "scope"] {
            let full = r#"
schema_version = 1
[[exception]]
rule = "time-call:foo.rs:SystemTime::now"
id = "EXC-001"
clause = "CONSTITUTION §10"
reason = "integration test fixture"
risk = "low test-only"
scope = "tools/archgate tests"
owner = "architecture-owner"
compensation = "time-call allowlist still scanned"
expires = "2026-10-01"
tracking = "https://example.invalid/issues/1"
approved_by = ["maintainer", "risk-owner"]
"#;
            let without_required = full
                .lines()
                .filter(|line| !line.trim_start().starts_with(&format!("{missing} =")))
                .collect::<Vec<_>>()
                .join("\n");
            let error = parse_exceptions_registry(&without_required)
                .expect_err("missing governance field must fail closed")
                .to_string();
            assert!(
                error.contains(&format!("`{missing}`")),
                "missing {missing} diagnostic must name the field: {error}"
            );
        }

        let unknown = r#"
schema_version = 1
[[exception]]
rule = "time-call:foo.rs:now"
typo = 1
"#;
        assert!(parse_exceptions_registry(unknown).is_err());

        let full = r#"
schema_version = 1
[[exception]]
rule = "time-call:foo.rs:SystemTime::now"
id = "EXC-001"
clause = "CONSTITUTION §10"
reason = "integration test fixture"
risk = "low test-only"
scope = "tools/archgate tests"
owner = "architecture-owner"
compensation = "time-call allowlist still scanned"
expires = "2026-10-01"
tracking = "https://example.invalid/issues/1"
approved_by = ["maintainer", "risk-owner"]
"#;
        let allow = parse_time_call_allowlist(full).expect("full governance ok");
        assert!(allow.contains("foo.rs:SystemTime::now"));

        let blank_approver = full.replace(
            "approved_by = [\"maintainer\", \"risk-owner\"]",
            "approved_by = [\"maintainer\", \"   \"]",
        );
        assert!(parse_exceptions_registry(&blank_approver).is_err());
    }

    #[test]
    fn unknown_and_stale_time_calls_are_rejected() {
        let allowed = HashSet::from(["known.rs:1".to_owned()]);
        let observed = vec!["known.rs:1".to_owned(), "new.rs:2".to_owned()];
        let (unapproved, stale) = classify_time_calls(&observed, &allowed);
        assert_eq!(unapproved, vec!["new.rs:2"]);
        assert!(stale.is_empty());
    }

    #[test]
    fn status_edge_rules_cover_stable_and_quarantined() {
        let v = status_edge_violation(
            "xhyper-kernel",
            "kernel",
            "stable",
            "xhyper-decimalx",
            "incubating",
        )
        .expect("stable→incubating");
        assert!(v.contains("stable must not normal-depend on incubating"));

        let q = status_edge_violation(
            "xhyper-domainx",
            "domain",
            "incubating",
            "legacy-foo",
            "quarantined",
        )
        .expect("quarantined");
        assert!(q.contains("normal dep on quarantined"));

        // tools / test-support 豁免 stable→incubating
        assert!(status_edge_violation(
            "xhyper-xtask",
            "tools",
            "stable",
            "xhyper-decimalx",
            "incubating",
        )
        .is_none());
        assert!(status_edge_violation(
            "xhyper-testkit",
            "test-support",
            "stable",
            "xhyper-decimalx",
            "incubating",
        )
        .is_none());
        // tools 可依赖 quarantined
        assert!(status_edge_violation(
            "xhyper-xtask",
            "tools",
            "incubating",
            "legacy-foo",
            "quarantined",
        )
        .is_none());
    }

    #[test]
    fn publish_flags_match_detects_drift() {
        assert!(publish_flags_match(true, true));
        assert!(publish_flags_match(false, false));
        assert!(!publish_flags_match(true, false));
        assert!(!publish_flags_match(false, true));
    }

    #[test]
    fn collect_publish_drift_empty_when_aligned() {
        // 空 packages → 无漂移（正例）
        let empty: Vec<cargo_metadata::Package> = Vec::new();
        let root = Path::new(".");
        let reg = HashMap::from([("crates/kernel".to_string(), true)]);
        assert!(collect_publish_drift(&empty, root, &reg).is_empty());
    }

    #[test]
    fn real_workspace_registry_loads_via_toml() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let reg = WorkspaceRegistry::load(&root).expect("load workspace.toml");
        assert_eq!(reg.schema_version, 1);
        assert_eq!(reg.units.len(), 39);
        let kernel = reg
            .units
            .iter()
            .find(|u| u.path == "crates/kernel")
            .expect("kernel unit");
        assert_eq!(kernel.status, "stable");
        assert!(kernel.publish);
        let decimal = reg
            .units
            .iter()
            .find(|u| u.path == "crates/types/decimal")
            .expect("decimal unit");
        assert_eq!(decimal.status, "incubating");
        assert!(!decimal.publish);
        let pol = DependencyPolicy::load(&root).expect("load dependency.toml");
        assert!(pol.allowances.get("tools").unwrap().contains("types"));
    }
}
