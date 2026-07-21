//! INFRA-004：inventory / contract / topology SSOT 只读校验。
//!
//! 复用现有 `.architecture/`、`configs/`、`deploy/`、`schemas/`、`evidence/`，
//! 不新建平行控制面。仅做漂移检测，**不自动修复**（INFRA-060 同约束）。
//!
//! 检测项摘要：
//! - SSOT 五面入口 + 禁止 `.infrastructure/` 平行树
//! - architecture unit path 重复 / 空 path / 登记路径缺失
//! - cargo members ↔ architecture 集合差
//! - dependency.toml 相对 R3/R2.1 不得放宽
//! - SQL/schema 金融字段禁止 float
//! - `.cargo/config.toml` target-dir 策略 + scripts/workflows 硬编码 `./target/` 扫描

use crate::architecture_toml::{load_dependency_allowances, WorkspaceRegistry};
use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Finding {
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub passed: bool,
    pub architecture_units: usize,
    pub cargo_members: usize,
    pub findings: Vec<Finding>,
    /// INFRA-060：本命令覆盖的故意/可预期漂移类别（只读声明，非自动修复清单）。
    pub intentional_categories: Vec<&'static str>,
}

/// inventory / drift-detect 共用的故意类别说明（只读 taxonomy）。
pub const INTENTIONAL_CATEGORIES: &[&str] = &[
    "ssot-roots",
    "parallel-control-plane",
    "architecture-unit-ids",
    "cargo-vs-architecture",
    "dependency-policy-width",
    "sql-float",
    "schema-financial-float",
    "target-dir-config",
    "target-dir-hardcode-scan",
    "exclude-or-quarantine-warn",
];

pub fn run(json: bool) -> Result<()> {
    let report = collect(json)?;
    emit(&report, json)?;
    if !report.passed {
        bail!("inventory-ssot found blocking drift");
    }
    Ok(())
}

/// 供 `drift-detect` 复用：收集 findings 而不直接退出。
pub fn collect(json_mode_hint: bool) -> Result<Report> {
    let _ = json_mode_hint;
    let root = workspace_root()?;
    let mut findings = Vec::new();

    check_ssot_roots(&root, &mut findings);
    let arch_paths = load_architecture_paths(&root, &mut findings)?;
    let cargo_paths = load_cargo_member_paths(&root, &mut findings)?;
    check_path_sets(&arch_paths, &cargo_paths, &mut findings);
    check_dependency_policy_strictness(&root, &mut findings)?;
    check_sql_decimal(&root, &mut findings)?;
    check_schema_financial_float(&root, &mut findings)?;
    check_target_dir_policy(&root, &mut findings)?;
    check_target_hardcode_scan(&root, &mut findings)?;

    Ok(Report {
        passed: findings.iter().all(|f| f.severity != "error"),
        architecture_units: arch_paths.len(),
        cargo_members: cargo_paths.len(),
        findings,
        intentional_categories: INTENTIONAL_CATEGORIES.to_vec(),
    })
}

fn emit(report: &Report, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(report)?);
    } else {
        println!(
            "inventory-ssot: architecture_units={} cargo_members={} findings={}",
            report.architecture_units,
            report.cargo_members,
            report.findings.len()
        );
        for f in &report.findings {
            println!("  [{}] {}: {}", f.severity, f.code, f.message);
        }
        if report.passed {
            println!("inventory-ssot: PASS");
        } else {
            println!("inventory-ssot: FAIL");
        }
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let meta = MetadataCommand::new().no_deps().exec()?;
    Ok(meta.workspace_root.into_std_path_buf())
}

fn check_ssot_roots(root: &Path, findings: &mut Vec<Finding>) {
    // D-01 推荐：复用既有五面，不默认建 .infrastructure/
    const REQUIRED: &[&str] = &[
        ".architecture/workspace.toml",
        ".architecture/policies/dependency.toml",
        "configs/README.md",
        "deploy/README.md",
        "schemas/README.md",
        "evidence/README.md",
    ];
    for rel in REQUIRED {
        if !root.join(rel).is_file() {
            findings.push(Finding {
                code: "ssot-root-missing",
                severity: "error",
                message: format!("缺少 SSOT 入口文件: {rel}"),
            });
        }
    }
    if root.join(".infrastructure").exists() {
        findings.push(Finding {
            code: "parallel-control-plane",
            severity: "error",
            message: "检测到 .infrastructure/ 平行控制面；D-01 未批准前禁止默认创建".into(),
        });
    }
}

fn load_architecture_paths(root: &Path, findings: &mut Vec<Finding>) -> Result<BTreeSet<String>> {
    let registry = match WorkspaceRegistry::load(root) {
        Ok(r) => r,
        Err(err) => {
            findings.push(Finding {
                code: "architecture-parse-error",
                severity: "error",
                message: format!(".architecture/workspace.toml TOML 解析失败: {err:#}"),
            });
            return Ok(BTreeSet::new());
        }
    };

    let mut paths = BTreeSet::new();
    for unit in &registry.units {
        if unit.path.is_empty() {
            findings.push(Finding {
                code: "architecture-unit-empty-path",
                severity: "error",
                message: format!(".architecture unit id={} path 为空", unit.id),
            });
            continue;
        }
        let full = root.join(&unit.path);
        if !full.exists() {
            findings.push(Finding {
                code: "architecture-path-missing",
                severity: "error",
                message: format!(".architecture 登记路径不存在: {}", unit.path),
            });
        }
        paths.insert(unit.path.clone());
    }
    if paths.is_empty() {
        findings.push(Finding {
            code: "architecture-empty",
            severity: "error",
            message: ".architecture/workspace.toml 未解析到任何 path".into(),
        });
    }
    Ok(paths)
}

fn load_cargo_member_paths(root: &Path, findings: &mut Vec<Finding>) -> Result<BTreeSet<String>> {
    let meta = MetadataCommand::new().no_deps().exec()?;
    let mut paths = BTreeSet::new();
    for id in &meta.workspace_members {
        let pkg = meta
            .packages
            .iter()
            .find(|p| &p.id == id)
            .context("package missing from metadata")?;
        let manifest = PathBuf::from(pkg.manifest_path.as_str());
        let dir = manifest
            .parent()
            .context("manifest has no parent")?
            .strip_prefix(root)
            .with_context(|| format!("manifest outside workspace: {}", pkg.manifest_path))?
            .to_string_lossy()
            .replace('\\', "/");
        paths.insert(dir);
    }
    if paths.is_empty() {
        findings.push(Finding {
            code: "cargo-members-empty",
            severity: "error",
            message: "cargo metadata 未返回 workspace members".into(),
        });
    }
    Ok(paths)
}

fn check_path_sets(arch: &BTreeSet<String>, cargo: &BTreeSet<String>, findings: &mut Vec<Finding>) {
    // architecture 可包含 quarantine/exclude 包；cargo 必须被 architecture 覆盖
    for p in cargo {
        if !arch.contains(p) {
            findings.push(Finding {
                code: "member-not-in-architecture",
                severity: "error",
                message: format!("cargo member 未登记到 .architecture: {p}"),
            });
        }
    }
    // architecture 多出来的可能是 exclude/quarantine——标 warn 而非 error
    for p in arch {
        if !cargo.contains(p) {
            findings.push(Finding {
                code: "architecture-not-member",
                severity: "warn",
                message: format!(
                    ".architecture 登记但非 active cargo member（可能 exclude/quarantine）: {p}"
                ),
            });
        }
    }
}

fn check_dependency_policy_strictness(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let layer_allows = match load_dependency_allowances(root) {
        Ok(m) => m,
        Err(err) => {
            findings.push(Finding {
                code: "dependency-parse-error",
                severity: "error",
                message: format!("dependency.toml TOML 解析失败: {err:#}"),
            });
            return Ok(());
        }
    };
    if let Some(allows) = layer_allows.get("infra") {
        if allows.contains("infra") {
            findings.push(Finding {
                code: "policy-infra-self",
                severity: "error",
                message: "dependency.toml: infra may_depend_on 含 infra（违反 R3；bootstrap 豁免在 archgate/lint-deps）"
                    .into(),
            });
        }
    } else {
        findings.push(Finding {
            code: "policy-infra-missing",
            severity: "warn",
            message: "dependency.toml 未找到 infra 层 may_depend_on".into(),
        });
    }
    if let Some(allows) = layer_allows.get("adapters") {
        if allows.contains("domain") {
            findings.push(Finding {
                code: "policy-adapters-domain",
                severity: "error",
                message: "dependency.toml: adapters may_depend_on 含 domain（违反 R2.1）".into(),
            });
        }
    }
    Ok(())
}

fn check_sql_decimal(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let sql_dir = root.join("schemas/sql");
    if !sql_dir.is_dir() {
        findings.push(Finding {
            code: "sql-schema-missing",
            severity: "warn",
            message: "schemas/sql 目录不存在".into(),
        });
        return Ok(());
    }
    for entry in fs::read_dir(&sql_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        for (i, line) in text.lines().enumerate() {
            let code = line.split("--").next().unwrap_or(line);
            let upper = code.to_uppercase();
            // 粗检：列类型使用 DOUBLE/FLOAT/REAL（忽略 SQL 注释）
            let tokens: Vec<&str> = upper.split_whitespace().collect();
            let has_float_type = tokens.iter().any(|t| {
                matches!(
                    *t,
                    "DOUBLE"
                        | "FLOAT"
                        | "REAL"
                        | "DOUBLE,"
                        | "FLOAT,"
                        | "REAL,"
                        | "DOUBLE)"
                        | "FLOAT)"
                        | "REAL)"
                )
            });
            if has_float_type {
                findings.push(Finding {
                    code: "sql-float-type",
                    severity: "error",
                    message: format!(
                        "{}:{} 使用浮点列类型（ADR-006 禁止价格/数量用 f64/DOUBLE）",
                        path.strip_prefix(root).unwrap_or(&path).display(),
                        i + 1
                    ),
                });
            }
        }
    }
    Ok(())
}

/// 金融金额/价格/数量类字段名（小写匹配）。
fn is_financial_field_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    matches!(
        n.as_str(),
        "bid"
            | "ask"
            | "price"
            | "volume"
            | "amount"
            | "qty"
            | "quantity"
            | "size"
            | "notional"
            | "fee"
            | "balance"
            | "funds"
            | "cost"
            | "value"
    ) || n.ends_with("_price")
        || n.ends_with("_qty")
        || n.ends_with("_volume")
        || n.ends_with("_amount")
        || n.ends_with("_bid")
        || n.ends_with("_ask")
        || n.ends_with("_fee")
        || n.ends_with("_balance")
}

/// 扫描 schemas/ 下 protobuf / jsonschema / openapi：价格数量类字段禁止 float/double/number。
fn check_schema_financial_float(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    check_protobuf_financial_float(root, findings)?;
    check_json_schema_financial_number(root, findings)?;
    Ok(())
}

/// schemas/protobuf/**/*.proto：price/qty 类字段禁止 double/float。
fn check_protobuf_financial_float(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let proto_dir = root.join("schemas/protobuf");
    if !proto_dir.is_dir() {
        findings.push(Finding {
            code: "proto-schema-missing",
            severity: "warn",
            message: "schemas/protobuf 目录不存在".into(),
        });
        return Ok(());
    }
    walk_files(&proto_dir, "proto", &mut |path| {
        let text = fs::read_to_string(path)?;
        for (i, line) in text.lines().enumerate() {
            // 去掉行尾注释
            let code = line.split("//").next().unwrap_or(line).trim();
            if code.is_empty() {
                continue;
            }
            // 字段形如: [repeated] double|float <name> = N;
            let tokens: Vec<&str> = code.trim_end_matches(';').split_whitespace().collect();
            if tokens.len() < 3 {
                continue;
            }
            let (ty, name) = if tokens[0] == "repeated" && tokens.len() >= 4 {
                (tokens[1], tokens[2])
            } else {
                (tokens[0], tokens[1])
            };
            let is_float = matches!(ty, "double" | "float");
            if is_float && is_financial_field_name(name) {
                findings.push(Finding {
                    code: "proto-float-financial",
                    severity: "error",
                    message: format!(
                        "{}:{} 金融字段 `{name}` 使用 {ty}（ADR-006 禁止 float/double；应使用 string 十进制）",
                        path.strip_prefix(root).unwrap_or(path).display(),
                        i + 1
                    ),
                });
            }
        }
        Ok(())
    })?;
    Ok(())
}

/// schemas/jsonschema 与 schemas/openapi：bid/ask/price/volume 等禁止 type:number。
fn check_json_schema_financial_number(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for rel in ["schemas/jsonschema", "schemas/openapi"] {
        let dir = root.join(rel);
        if !dir.is_dir() {
            findings.push(Finding {
                code: "json-schema-dir-missing",
                severity: "warn",
                message: format!("{rel} 目录不存在"),
            });
            continue;
        }
        walk_files(&dir, "json", &mut |path| {
            // 跳过 fixtures（样本数据，非 schema 定义）
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            if rel_path.contains("/fixtures/") {
                return Ok(());
            }
            let text = fs::read_to_string(path)?;
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                // 非合法 JSON 不在此门禁处理
                return Ok(());
            };
            scan_json_for_financial_number(&value, &rel_path, "", findings);
            Ok(())
        })?;
    }
    Ok(())
}

/// 递归扫描 JSON：属性名为金融字段且 type==number 则报错。
fn scan_json_for_financial_number(
    value: &serde_json::Value,
    file: &str,
    path_hint: &str,
    findings: &mut Vec<Finding>,
) {
    match value {
        serde_json::Value::Object(map) => {
            // OpenAPI/JSON Schema properties 块
            if let Some(serde_json::Value::Object(props)) = map.get("properties") {
                for (name, prop) in props {
                    if is_financial_field_name(name) && json_prop_is_number(prop) {
                        findings.push(Finding {
                            code: "schema-number-financial",
                            severity: "error",
                            message: format!(
                                "{file} 属性 `{name}`{hint} 使用 type number（ADR-006 禁止；应使用 string 十进制）",
                                hint = if path_hint.is_empty() {
                                    String::new()
                                } else {
                                    format!(" @ {path_hint}")
                                }
                            ),
                        });
                    }
                    let child_hint = if path_hint.is_empty() {
                        name.clone()
                    } else {
                        format!("{path_hint}.{name}")
                    };
                    scan_json_for_financial_number(prop, file, &child_hint, findings);
                }
            }
            for (k, v) in map {
                if k == "properties" {
                    continue; // 已处理
                }
                let child_hint = if path_hint.is_empty() {
                    k.clone()
                } else {
                    format!("{path_hint}.{k}")
                };
                scan_json_for_financial_number(v, file, &child_hint, findings);
            }
        }
        serde_json::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let child_hint = format!("{path_hint}[{i}]");
                scan_json_for_financial_number(item, file, &child_hint, findings);
            }
        }
        _ => {}
    }
}

/// property schema 是否声明 type number（含 type 数组含 number）。
fn json_prop_is_number(prop: &serde_json::Value) -> bool {
    match prop.get("type") {
        Some(serde_json::Value::String(s)) => s == "number",
        Some(serde_json::Value::Array(arr)) => arr.iter().any(|v| v.as_str() == Some("number")),
        _ => false,
    }
}

/// 递归遍历目录中指定扩展名的文件。
fn walk_files(dir: &Path, ext: &str, f: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, ext, f)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            f(&path)?;
        }
    }
    Ok(())
}

fn check_target_dir_policy(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let cargo_config = root.join(".cargo/config.toml");
    if !cargo_config.is_file() {
        findings.push(Finding {
            code: "cargo-config-missing",
            severity: "warn",
            message: ".cargo/config.toml 不存在，无法校验 target-dir".into(),
        });
        return Ok(());
    }
    let text = fs::read_to_string(cargo_config)?;
    if !text.contains("target-dir") || !text.contains(".cargo/target") {
        findings.push(Finding {
            code: "target-dir-policy",
            severity: "error",
            message: ".cargo/config.toml 未配置 .cargo/target/（禁止恢复 ./target/ 假设）".into(),
        });
    }
    Ok(())
}

/// 扫描 scripts/ 与 .github/workflows/ 中非注释的 `./target/` 硬编码。
/// 注释/文档中出现「禁止 ./target/」等说明性文字不记 error。
fn check_target_hardcode_scan(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    const ROOTS: &[&str] = &["scripts", ".github/workflows", ".github/actions"];
    const EXTS: &[&str] = &["sh", "bash", "yml", "yaml", "mjs", "js", "ts", "toml", "rs"];

    for rel in ROOTS {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        walk_files_multi(&dir, EXTS, &mut |path| {
            let text = fs::read_to_string(path)?;
            let display = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            for (i, line) in text.lines().enumerate() {
                if line_is_comment_or_doc(line, path) {
                    continue;
                }
                // 允许字符串中出现在否定上下文（仍建议注释说明）；硬编码路径片段：
                // `./target/` `./target"` 以及 YAML `publish_dir: ./target`
                if line_has_target_hardcode(line) {
                    findings.push(Finding {
                        code: "target-hardcode",
                        severity: "error",
                        message: format!(
                            "{display}:{} 疑似硬编码 ./target/（应使用 CARGO_TARGET_DIR / .cargo/target/）",
                            i + 1
                        ),
                    });
                }
            }
            Ok(())
        })?;
    }
    Ok(())
}

fn line_is_comment_or_doc(line: &str, path: &Path) -> bool {
    let t = line.trim_start();
    if t.is_empty() {
        return true;
    }
    // 通用注释
    if t.starts_with('#') || t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
        return true;
    }
    // markdown 中的说明（若误扫）
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "md" {
        return true;
    }
    // 行内：仅含注释部分出现 ./target/ 时，若代码侧无硬编码则跳过
    // 简化：若 trim 后整行以 echo/printf 打印禁止说明，仍可能命中——下面 hardcode 检测更严
    false
}

fn line_has_target_hardcode(line: &str) -> bool {
    // 去掉行尾 shell/yaml 注释再判断
    let code = strip_inline_comment(line);
    if code.contains("./target/")
        || code.contains("./target\"")
        || code.contains("./target'")
        || code.contains("publish_dir: ./target")
        || code.contains("publish_dir:./target")
    {
        // 排除明确的否定/迁移说明字符串（仍在代码中）
        let lower = code.to_ascii_lowercase();
        if lower.contains("禁止")
            || lower.contains("avoid")
            || lower.contains("do not")
            || lower.contains("don't")
            || lower.contains("instead of ./target")
            || lower.contains("not ./target")
        {
            return false;
        }
        return true;
    }
    false
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\'' && !in_double {
            in_single = !in_single;
        } else if b == b'"' && !in_single {
            in_double = !in_double;
        } else if !in_single && !in_double {
            if b == b'#' {
                return &line[..i];
            }
            if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                return &line[..i];
            }
        }
        i += 1;
    }
    line
}

fn walk_files_multi(
    dir: &Path,
    exts: &[&str],
    f: &mut dyn FnMut(&Path) -> Result<()>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_files_multi(&path, exts, f)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e))
        {
            f(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardcode_detector_flags_path_usage() {
        assert!(line_has_target_hardcode("cp ./target/debug/foo /tmp/foo"));
        assert!(line_has_target_hardcode("publish_dir: ./target/doc"));
        assert!(!line_has_target_hardcode("# 禁止写死 ./target/"));
        assert!(!line_has_target_hardcode(
            "echo \"avoid ./target/ — use CARGO_TARGET_DIR\""
        ));
    }

    #[test]
    fn intentional_categories_non_empty() {
        assert!(INTENTIONAL_CATEGORIES.contains(&"parallel-control-plane"));
        assert!(INTENTIONAL_CATEGORIES.contains(&"target-dir-hardcode-scan"));
    }
}
