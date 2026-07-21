//! INFRA-003：Evidence 记录校验器（草稿 schema + 脱敏 + 自测）。
//!
//! 对齐 `schemas/jsonschema/evidence-record.schema.json`：
//! - 必填字段 / 枚举 / 类型（手写 + 轻量 schema 子集匹配）
//! - 疑似 secret 形态拒绝入库
//! - `--self-test`：缺字段、篡改字段、secret 样本必须失败
//!
//! 完整签名 / WORM / AC-E-01..08 全量矩阵仍为 TARGET INTERFACE。
//! 默认扫描 `*.evidence.json`（含 fixtures 与 `evidence/`）；
//! 负向样本放在 `fixtures/negative/*.json`（非 `.evidence.json` 后缀）。

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::schema_lite::{is_rfc3339_utc, json_schema_matches};

const REQUIRED: &[&str] = &[
    "schema_version",
    "work_package",
    "acceptance_criterion",
    "repository",
    "branch",
    "commit",
    "dirty_state",
    "command",
    "started_at",
    "ended_at",
    "exit_code",
    "result",
    "sensitivity",
    "retention",
];

/// 禁止进入 Evidence 正文的敏感子串（INFRA-003 AC-E-20 方向）。
const SECRET_PATTERNS: &[&str] = &[
    "ghp_",
    "ghr_",
    "gho_",
    "github_pat_",
    "ANTHROPIC_AUTH_TOKEN=",
    "A22Z",
    "AKIA",
    "-----BEGIN PRIVATE KEY-----",
    "-----BEGIN RSA PRIVATE KEY-----",
    "-----BEGIN OPENSSH PRIVATE KEY-----",
    "password=",
    "PASSWORD=",
    "secret=",
    "SECRET=",
    "api_key=",
    "API_KEY=",
    "aws_secret_access_key",
    "xoxb-",
    "xoxp-",
    "sk-ant-",
    "sk-proj-",
];

const SCHEMA_REL: &str = "schemas/jsonschema/evidence-record.schema.json";

#[derive(Debug, Serialize)]
struct Finding {
    path: String,
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct Report {
    passed: bool,
    checked: usize,
    findings: Vec<Finding>,
    self_test_passed: Option<bool>,
    schema_loaded: bool,
    /// 对 `evidence/infrastructure` 的非破坏性扫描摘要（若目录存在）。
    infrastructure_scan: Option<InfrastructureScan>,
}

#[derive(Debug, Serialize)]
struct InfrastructureScan {
    scanned: usize,
    invalid: usize,
    note: &'static str,
}

pub fn run(json: bool, path: Option<PathBuf>, self_test: bool) -> Result<()> {
    let root = workspace_root()?;
    let schema = load_schema(&root);
    let schema_loaded = schema.is_some();
    let mut findings = Vec::new();
    let mut checked = 0usize;

    let targets = resolve_targets(&root, path)?;
    for target in &targets {
        checked += 1;
        validate_file(target, &root, schema.as_ref(), &mut findings)?;
    }

    // 可选：非破坏性扫描 evidence/infrastructure 下的机器记录（不把 md 当 evidence）
    let infrastructure_scan = scan_infrastructure_optional(&root, schema.as_ref(), &mut findings)?;

    let mut self_test_passed = None;
    if self_test {
        self_test_passed = Some(run_self_test(schema.as_ref())?);
        if self_test_passed == Some(false) {
            findings.push(Finding {
                path: "<self-test>".into(),
                code: "self-test-failed",
                message: "自测未通过：合法样本应通过，缺字段/篡改/secret 样本必须失败".into(),
            });
        }
    }

    // infrastructure 扫描 finding 已写入 findings；仅 markdown 时 scanned=0 不阻断
    let report = Report {
        passed: findings.is_empty(),
        checked,
        findings,
        self_test_passed,
        schema_loaded,
        infrastructure_scan,
    };

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "evidence-check: checked={} findings={} self_test={:?} schema_loaded={}",
            report.checked,
            report.findings.len(),
            report.self_test_passed,
            report.schema_loaded
        );
        if let Some(scan) = &report.infrastructure_scan {
            println!(
                "  infrastructure_scan: scanned={} invalid={} ({})",
                scan.scanned, scan.invalid, scan.note
            );
        }
        for f in &report.findings {
            println!("  {} [{}]: {}", f.path, f.code, f.message);
        }
        if report.passed {
            println!("evidence-check: PASS");
        } else {
            println!("evidence-check: FAIL");
        }
    }

    if !report.passed {
        bail!("evidence-check found invalid evidence records");
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let meta = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    Ok(meta.workspace_root.into_std_path_buf())
}

fn load_schema(root: &Path) -> Option<Value> {
    let path = root.join(SCHEMA_REL);
    let bytes = fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn resolve_targets(root: &Path, path: Option<PathBuf>) -> Result<Vec<PathBuf>> {
    if let Some(p) = path {
        let full = if p.is_absolute() { p } else { root.join(p) };
        if full.is_file() {
            return Ok(vec![full]);
        }
        if full.is_dir() {
            // 显式 path 时：目录内全部 .json（含负向样本）
            return collect_json_files(&full);
        }
        bail!("path not found: {}", full.display());
    }
    // 默认：扫描 evidence/**/*.evidence.json 与 fixtures 合法样本
    let mut out = collect_json_files(&root.join("evidence"))?;
    let fixture = root.join("schemas/jsonschema/fixtures");
    if fixture.is_dir() {
        out.extend(collect_json_files(&fixture)?);
    }
    Ok(out
        .into_iter()
        .filter(|p| {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            name.ends_with(".evidence.json")
        })
        .collect())
}

fn collect_json_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                out.push(path);
            }
        }
        Ok(())
    }
    walk(dir, &mut out)?;
    out.sort();
    Ok(out)
}

/// 非破坏性扫描 `evidence/infrastructure/**/*.evidence.json`。
/// 目录仅有 markdown 时报告 scanned=0，不失败。
fn scan_infrastructure_optional(
    root: &Path,
    schema: Option<&Value>,
    findings: &mut Vec<Finding>,
) -> Result<Option<InfrastructureScan>> {
    let dir = root.join("evidence/infrastructure");
    if !dir.is_dir() {
        return Ok(None);
    }
    let files: Vec<PathBuf> = collect_json_files(&dir)?
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with(".evidence.json"))
        })
        .collect();
    // 已在 resolve_targets 默认扫描过 evidence/**，避免重复计入 checked；
    // 此处仅提供摘要，finding 若已存在则不再重复 push。
    let mut invalid = 0usize;
    for path in &files {
        let before = findings.len();
        // 只在 findings 中尚无该 path 时补充校验
        let rel = path
            .strip_prefix(root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());
        let already = findings.iter().any(|f| f.path == rel);
        if !already {
            validate_file(path, root, schema, findings)?;
        }
        let after = findings.iter().filter(|f| f.path == rel).count();
        if after > 0 || (already && findings[before..].iter().any(|f| f.path == rel)) {
            // recount for this file
        }
        let file_findings = findings.iter().filter(|f| f.path == rel).count();
        if file_findings > 0 {
            invalid += 1;
        }
    }
    Ok(Some(InfrastructureScan {
        scanned: files.len(),
        invalid,
        note: "readonly; auto-repair disabled; markdown-only dirs report scanned=0",
    }))
}

fn validate_file(
    path: &Path,
    root: &Path,
    schema: Option<&Value>,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let rel = path
        .strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            findings.push(Finding {
                path: rel,
                code: "invalid-json",
                message: e.to_string(),
            });
            return Ok(());
        }
    };
    validate_value(&value, &rel, schema, findings);
    Ok(())
}

fn validate_value(value: &Value, rel: &str, schema: Option<&Value>, findings: &mut Vec<Finding>) {
    let Some(obj) = value.as_object() else {
        findings.push(Finding {
            path: rel.into(),
            code: "not-object",
            message: "Evidence 根必须是 JSON object".into(),
        });
        return;
    };

    for key in REQUIRED {
        if !obj.contains_key(*key) {
            findings.push(Finding {
                path: rel.into(),
                code: "missing-required",
                message: format!("缺少必填字段: {key}"),
            });
        }
    }

    if let Some(v) = obj.get("result") {
        let ok = matches!(
            v.as_str(),
            Some("PASS" | "FAIL" | "SKIP" | "BLOCKED" | "OPEN" | "UNKNOWN")
        );
        if !ok {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-result",
                message: "result 必须是 PASS|FAIL|SKIP|BLOCKED|OPEN|UNKNOWN".into(),
            });
        }
    }

    if let Some(v) = obj.get("sensitivity") {
        let ok = matches!(
            v.as_str(),
            Some("public" | "internal" | "confidential" | "secret_ref_only")
        );
        if !ok {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-sensitivity",
                message: "sensitivity 枚举非法".into(),
            });
        }
    }

    if let Some(ret) = obj.get("retention") {
        if !ret.is_object() || ret.get("class").and_then(|c| c.as_str()).is_none() {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-retention",
                message: "retention 必须是含 class 的 object".into(),
            });
        } else if let Some(class) = ret.get("class").and_then(|c| c.as_str()) {
            if !matches!(class, "ephemeral" | "standard" | "audit" | "legal_hold") {
                findings.push(Finding {
                    path: rel.into(),
                    code: "invalid-retention-class",
                    message: format!("retention.class 非法: {class}"),
                });
            }
        }
    }

    if let Some(c) = obj.get("commit").and_then(|v| v.as_str()) {
        if c.len() < 7 || !c.chars().all(|ch| ch.is_ascii_hexdigit()) {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-commit",
                message: "commit 须为 7–40 位 hex SHA".into(),
            });
        }
    }

    if let Some(d) = obj.get("dirty_state") {
        if !d.is_boolean() {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-dirty-state",
                message: "dirty_state 必须是 boolean".into(),
            });
        }
    }

    if let Some(code) = obj.get("exit_code") {
        if !code.is_i64() && !code.is_u64() {
            findings.push(Finding {
                path: rel.into(),
                code: "invalid-exit-code",
                message: "exit_code 必须是 integer".into(),
            });
        }
    }

    for ts_field in ["started_at", "ended_at"] {
        if let Some(v) = obj.get(ts_field).and_then(|v| v.as_str()) {
            if !is_rfc3339_utc(v) {
                findings.push(Finding {
                    path: rel.into(),
                    code: "invalid-timestamp",
                    message: format!("{ts_field} 须为 RFC3339 UTC（YYYY-MM-DDTHH:MM:SSZ）"),
                });
            }
        }
    }

    // 若声明 signature.payload_hash，粗检格式（sha256: 前缀或 64 hex）
    if let Some(sig) = obj.get("signature") {
        if let Some(ph) = sig.get("payload_hash").and_then(|v| v.as_str()) {
            if !is_digest_like(ph) {
                findings.push(Finding {
                    path: rel.into(),
                    code: "invalid-payload-hash",
                    message: "signature.payload_hash 须为 sha256:hex 或 64 位 hex".into(),
                });
            }
        }
    }
    if let Some(tc) = obj.get("toolchain") {
        if let Some(d) = tc.get("cargo_lock_digest").and_then(|v| v.as_str()) {
            if !is_digest_like(d) {
                findings.push(Finding {
                    path: rel.into(),
                    code: "invalid-lock-digest",
                    message: "toolchain.cargo_lock_digest 须为 digest 形态".into(),
                });
            }
        }
    }

    // 轻量 JSON Schema 子集（若 schema 可读）
    if let Some(schema) = schema {
        if !json_schema_matches(value, schema) {
            findings.push(Finding {
                path: rel.into(),
                code: "schema-mismatch",
                message: format!("未通过 {SCHEMA_REL} 子集校验（required/type/enum/pattern/additionalProperties）"),
            });
        }
    }

    // 脱敏：记录中不得出现常见 secret 形态
    reject_secret_like(value, rel, findings);
}

fn is_digest_like(value: &str) -> bool {
    if let Some(hex) = value.strip_prefix("sha256:") {
        return hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit());
    }
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn reject_secret_like(value: &Value, rel: &str, findings: &mut Vec<Finding>) {
    let dump = value.to_string();
    for pat in SECRET_PATTERNS {
        if dump.contains(pat) {
            findings.push(Finding {
                path: rel.into(),
                code: "possible-secret",
                message: format!("Evidence 内容疑似包含敏感模式: {pat}*（禁止入库）"),
            });
        }
    }
}

fn sample_valid() -> Value {
    serde_json::json!({
        "schema_version": "0.1.0-draft",
        "work_package": "INFRA-003",
        "acceptance_criterion": "self-test-valid",
        "repository": "xhyperium/xhyper.rs",
        "branch": "docs/infrastructure-production-plan",
        "commit": "deadbeef",
        "dirty_state": true,
        "command": "cargo run -p xhyper-xtask -- evidence-check --self-test",
        "started_at": "2026-07-14T00:00:00Z",
        "ended_at": "2026-07-14T00:00:01Z",
        "exit_code": 0,
        "result": "PASS",
        "sensitivity": "internal",
        "retention": { "class": "ephemeral", "worm": false }
    })
}

fn run_self_test(schema: Option<&Value>) -> Result<bool> {
    let mut findings = Vec::new();
    let valid = sample_valid();
    validate_value(&valid, "self-test-valid", schema, &mut findings);
    if !findings.is_empty() {
        return Ok(false);
    }

    // 1) 缺字段：删除 commit → 必须失败
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("commit");
    findings.clear();
    validate_value(&missing, "self-test-missing-commit", schema, &mut findings);
    if findings.is_empty() || !findings.iter().any(|f| f.code == "missing-required") {
        return Ok(false);
    }

    // 2) 篡改 result → 必须失败
    let mut bad_result = valid.clone();
    bad_result
        .as_object_mut()
        .unwrap()
        .insert("result".into(), Value::String("YES".into()));
    findings.clear();
    validate_value(&bad_result, "self-test-bad-result", schema, &mut findings);
    if findings.is_empty() {
        return Ok(false);
    }

    // 3) 篡改 hash 字段形态 → 必须失败
    let mut bad_hash = valid.clone();
    bad_hash.as_object_mut().unwrap().insert(
        "toolchain".into(),
        serde_json::json!({ "cargo_lock_digest": "not-a-digest" }),
    );
    findings.clear();
    validate_value(&bad_hash, "self-test-bad-hash", schema, &mut findings);
    if findings.is_empty()
        || !findings
            .iter()
            .any(|f| f.code == "invalid-lock-digest" || f.code == "schema-mismatch")
    {
        return Ok(false);
    }

    // 4) 篡改 exit_code 类型 → 必须失败
    let mut bad_exit = valid.clone();
    bad_exit
        .as_object_mut()
        .unwrap()
        .insert("exit_code".into(), Value::String("0".into()));
    findings.clear();
    validate_value(&bad_exit, "self-test-bad-exit", schema, &mut findings);
    if findings.is_empty() {
        return Ok(false);
    }

    // 5) secret-like 字段拒绝
    let mut with_secret = valid.clone();
    with_secret.as_object_mut().unwrap().insert(
        "notes".into(),
        Value::String("token=ghp_EXAMPLESECRETVALUE000000000000".into()),
    );
    findings.clear();
    validate_value(&with_secret, "self-test-secret", schema, &mut findings);
    if findings.is_empty() || !findings.iter().any(|f| f.code == "possible-secret") {
        return Ok(false);
    }

    // 6) AKIA / private key 形态
    let mut akia = valid.clone();
    akia.as_object_mut()
        .unwrap()
        .insert("notes".into(), Value::String("AKIAIOSFODNN7EXAMPLE".into()));
    findings.clear();
    validate_value(&akia, "self-test-akia", schema, &mut findings);
    if !findings.iter().any(|f| f.code == "possible-secret") {
        return Ok(false);
    }

    // 7) 额外未知字段在 schema additionalProperties=false 时失败
    if schema.is_some() {
        let mut extra = valid.clone();
        extra
            .as_object_mut()
            .unwrap()
            .insert("unreviewed_override".into(), Value::Bool(true));
        findings.clear();
        validate_value(&extra, "self-test-extra-field", schema, &mut findings);
        if !findings.iter().any(|f| f.code == "schema-mismatch") {
            return Ok(false);
        }
    }

    // 磁盘负向 fixtures
    if !assert_negative_fixtures_fail(schema)? {
        return Ok(false);
    }

    Ok(true)
}

/// 加载 `schemas/jsonschema/fixtures/negative/*.json`，断言每个文件至少产生一条 finding。
fn assert_negative_fixtures_fail(schema: Option<&Value>) -> Result<bool> {
    let root = workspace_root()?;
    let dir = root.join("schemas/jsonschema/fixtures/negative");
    if !dir.is_dir() {
        return Ok(true);
    }
    let files = collect_json_files(&dir)?;
    if files.is_empty() {
        return Ok(false);
    }
    for path in files {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("read negative fixture {}", path.display()))?;
        let value: Value = serde_json::from_str(&text)
            .with_context(|| format!("parse negative fixture {}", path.display()))?;
        let rel = path
            .strip_prefix(&root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());
        let mut findings = Vec::new();
        validate_value(&value, &rel, schema, &mut findings);
        if findings.is_empty() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_valid_passes_without_schema() {
        let mut findings = Vec::new();
        validate_value(&sample_valid(), "t", None, &mut findings);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn missing_commit_is_detected() {
        let mut v = sample_valid();
        v.as_object_mut().unwrap().remove("commit");
        let mut findings = Vec::new();
        validate_value(&v, "t", None, &mut findings);
        assert!(findings.iter().any(|f| f.code == "missing-required"));
    }

    #[test]
    fn secret_patterns_are_rejected() {
        let mut v = sample_valid();
        v.as_object_mut()
            .unwrap()
            .insert("notes".into(), Value::String("password=hunter2".into()));
        let mut findings = Vec::new();
        validate_value(&v, "t", None, &mut findings);
        assert!(findings.iter().any(|f| f.code == "possible-secret"));
    }

    #[test]
    fn tampered_lock_digest_fails() {
        let mut v = sample_valid();
        v.as_object_mut().unwrap().insert(
            "toolchain".into(),
            serde_json::json!({ "cargo_lock_digest": "zz" }),
        );
        let mut findings = Vec::new();
        validate_value(&v, "t", None, &mut findings);
        assert!(findings.iter().any(|f| f.code == "invalid-lock-digest"));
    }

    #[test]
    fn repo_schema_accepts_valid_fixture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let schema = load_schema(root).expect("schema present");
        let text = fs::read_to_string(
            root.join("schemas/jsonschema/fixtures/evidence-record.valid.evidence.json"),
        )
        .unwrap();
        let value: Value = serde_json::from_str(&text).unwrap();
        let mut findings = Vec::new();
        validate_value(&value, "fixture", Some(&schema), &mut findings);
        assert!(findings.is_empty(), "{findings:?}");
    }
}
