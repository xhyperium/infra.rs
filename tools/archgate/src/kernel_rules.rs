//! SPEC-KERNEL-002 §12.2 — 命名 KERNEL-* 机器规则。
//!
//! 与 residual RES-GATE-* 对齐；失败项进入 archgate 总诊断。

use anyhow::{Context, Result};
use cargo_metadata::{Dependency, DependencyKind};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CARGO_PUBLIC_API_VERSION: &str = "cargo-public-api 0.52.0";

// `kernel-public-api.baseline.txt` 是已发布 0.1.1 的冻结面。这个指纹使同时修改
// baseline 与 snapshot 不能绕过 API-002；重设冻结面必须显式修改门禁代码并走审查。
// FNV-1a 不是安全摘要，只承担确定性的意外漂移检测；删除/签名变化仍由集合差异拒绝。
const KERNEL_API_BASELINE_FNV1A64: u64 = 0x557e_fdc3_5a9d_070f;
const KERNEL_API_BASELINE_LINES: usize = 508;

/// 单条命名规则结果。
#[derive(Debug, Clone)]
pub struct KernelRuleResult {
    pub id: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// KERNEL 规则扫描汇总。
#[derive(Debug, Default)]
pub struct KernelRulesReport {
    pub results: Vec<KernelRuleResult>,
    /// 失败明细（含规则 ID 前缀），供 archgate 退出失败。
    pub violations: Vec<String>,
    /// 生产侧 `XError::internal` 调用计数（ERR-001）。
    pub internal_count: usize,
}

impl KernelRulesReport {
    fn push(&mut self, id: &'static str, ok: bool, detail: impl Into<String>) {
        let detail = detail.into();
        if !ok {
            self.violations.push(format!("{id}: {detail}"));
        }
        self.results.push(KernelRuleResult { id, ok, detail });
    }
}

/// 从 Cargo.toml 解析 `[dependencies]` 键名（不含 dev/build/target）。
pub fn parse_normal_dependency_names(manifest: &str) -> Vec<String> {
    let mut in_deps = false;
    let mut names = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(key) = trimmed.split('=').next() else {
            continue;
        };
        let name = key.trim().trim_matches('"');
        if !name.is_empty() {
            names.push(name.to_owned());
        }
    }
    names
}

/// 解析 `[features]` 中除注释外的键名。
pub fn parse_feature_keys(manifest: &str) -> Vec<String> {
    let mut in_features = false;
    let mut keys = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_features = trimmed == "[features]";
            continue;
        }
        if !in_features || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(key) = trimmed.split('=').next() else {
            continue;
        };
        let name = key.trim().trim_matches('"');
        if !name.is_empty() {
            keys.push(name.to_owned());
        }
    }
    keys
}

/// 是否为允许的生产外部依赖（KERNEL-DEP-002）。
pub fn is_allowed_kernel_external_dep(name: &str) -> bool {
    name == "thiserror"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KernelDepKind {
    Normal,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KernelDependency {
    package: String,
    rename: Option<String>,
    kind: KernelDepKind,
    target: Option<String>,
    is_path: bool,
}

impl KernelDependency {
    fn from_metadata(dependency: &Dependency) -> Option<Self> {
        let kind = match dependency.kind {
            DependencyKind::Normal => KernelDepKind::Normal,
            DependencyKind::Build => KernelDepKind::Build,
            DependencyKind::Development => return None,
            _ => return None,
        };
        Some(Self {
            package: dependency.name.to_string(),
            rename: dependency.rename.clone(),
            kind,
            target: dependency.target.as_ref().map(ToString::to_string),
            is_path: dependency.source.is_none(),
        })
    }

    fn display(&self) -> String {
        format!(
            "package={} rename={} kind={} target={}",
            self.package,
            self.rename.as_deref().unwrap_or("-"),
            match self.kind {
                KernelDepKind::Normal => "normal",
                KernelDepKind::Build => "build",
            },
            self.target.as_deref().unwrap_or("-")
        )
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct KernelDependencyEvaluation {
    workspace_or_path: Vec<String>,
    disallowed_external: Vec<String>,
    allowed_external: Vec<String>,
}

fn is_exact_kernel_external_allow(dependency: &KernelDependency) -> bool {
    let thiserror = dependency.package == "thiserror"
        && dependency.rename.is_none()
        && dependency.kind == KernelDepKind::Normal
        && dependency.target.is_none();
    let loom = dependency.package == "loom"
        && dependency.rename.is_none()
        && dependency.kind == KernelDepKind::Normal
        && dependency.target.as_deref() == Some("cfg(loom)");
    thiserror || loom
}

fn evaluate_kernel_dependencies(
    dependencies: &[KernelDependency],
    workspace_package_names: &HashSet<String>,
) -> KernelDependencyEvaluation {
    let mut result = KernelDependencyEvaluation::default();
    for dependency in dependencies {
        if dependency.is_path || workspace_package_names.contains(&dependency.package) {
            result.workspace_or_path.push(dependency.display());
        } else if is_exact_kernel_external_allow(dependency) {
            result.allowed_external.push(dependency.display());
        } else {
            result.disallowed_external.push(dependency.display());
        }
    }
    result.workspace_or_path.sort();
    result.disallowed_external.sort();
    result.allowed_external.sort();
    result
}

fn collect_rs_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_comment_or_attr_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
        || t.starts_with("///")
        || t.starts_with("//!")
        || t.starts_with("#[")
        || t.starts_with("#!")
}

/// 真正的 unsafe 用法（排除 `forbid(unsafe_code)` 等属性字面）。
pub fn line_has_unsafe_usage(line: &str) -> bool {
    if is_comment_or_attr_line(line) {
        return false;
    }
    let t = line.trim_start();
    t.starts_with("unsafe ")
        || t.contains(" unsafe ")
        || t.contains("\tunsafe ")
        || t.starts_with("unsafe{")
        || t.contains(" unsafe{")
}

fn path_allowed(path_rel: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|p| path_rel == *p || path_rel.starts_with(p))
}

/// Instant::now 仅允许 kernel 生产实现（SystemClock origin）。
const INSTANT_NOW_ALLOW: &[&str] = &["crates/kernel/"];

/// from_unix_nanos：kernel/testkit + 测试模块内 FixedClock 字面墙钟（cfg(test) 同文件）。
/// evidence adapters：持久化还原墙钟 + 同文件 `#[cfg(test)]` FixedClock（SPEC-EVIDENCE-002）。
const FROM_UNIX_NANOS_ALLOW: &[&str] = &[
    "crates/kernel/",
    "crates/testkit/",
    "crates/domain/macro/",
    "crates/evidence/src/",
    "crates/adapters/evidence/",
    // 测试替身 FixedClock 与生产同文件（#[cfg(test)]）；非运行时路径
    "crates/adapters/exchange/binance/src/rest.rs",
    "crates/adapters/exchange/okx/src/rest.rs",
];

/// from_clock_elapsed：仅允许 kernel/testkit 与已知 FixedClock 测试替身（KERNEL-TIME-004）。
const FROM_CLOCK_ELAPSED_ALLOW: &[&str] = &[
    "crates/kernel/",
    "crates/testkit/",
    "crates/adapters/evidence/",
    "crates/adapters/exchange/binance/src/rest.rs",
    "crates/adapters/exchange/okx/src/rest.rs",
];

/// `XError::internal` 业务 crate 调用上界（KERNEL-ERR-001 棘轮；只减不增）。
/// 统计范围：`crates/**/src/**` 且排除 `crates/kernel/`（自测/定义不计入消费侧棘轮）。
pub const XERROR_INTERNAL_BASELINE: usize = 8;

/// 运行全部命名 KERNEL-* 规则。
pub fn evaluate_kernel_rules(
    root: &Path,
    workspace_package_names: &HashSet<String>,
    kernel_dependencies: &[Dependency],
) -> Result<KernelRulesReport> {
    let mut report = KernelRulesReport::default();
    let kernel_dir = root.join("crates/kernel");
    let manifest_path = kernel_dir.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;

    // --- KERNEL-DEP-001 / DEP-002 ---
    let production_dependencies: Vec<_> = kernel_dependencies
        .iter()
        .filter_map(KernelDependency::from_metadata)
        .collect();
    let dependency_evaluation =
        evaluate_kernel_dependencies(&production_dependencies, workspace_package_names);
    report.push(
        "KERNEL-DEP-001",
        dependency_evaluation.workspace_or_path.is_empty(),
        if dependency_evaluation.workspace_or_path.is_empty() {
            "workspace/local path deps across normal/build/target tables = 0".into()
        } else {
            format!(
                "workspace/local path deps present: {:?}",
                dependency_evaluation.workspace_or_path
            )
        },
    );
    report.push(
        "KERNEL-DEP-002",
        dependency_evaluation.disallowed_external.is_empty(),
        if dependency_evaluation.disallowed_external.is_empty() {
            format!(
                "production external deps exactly allowed: {:?}",
                dependency_evaluation.allowed_external
            )
        } else {
            format!(
                "disallowed production deps: {:?}",
                dependency_evaluation.disallowed_external
            )
        },
    );

    // --- KERNEL-FEATURE-001 ---
    let features = parse_feature_keys(&manifest);
    let feature_ok = features.is_empty() || features == ["default".to_owned()];
    // 进一步：default 必须为 []
    let default_empty = manifest.lines().any(|l| {
        let t = l.trim();
        t == "default = []" || t.starts_with("default=[]")
    });
    let feat_pass = feature_ok && (features.is_empty() || default_empty);
    report.push(
        "KERNEL-FEATURE-001",
        feat_pass,
        if feat_pass {
            "features only default=[] (or absent)".into()
        } else {
            format!("unexpected features keys={features:?} default_empty={default_empty}")
        },
    );

    // --- KERNEL-API-001 ---
    let api_path = root.join(".architecture/api/kernel-public-api.txt");
    let api_text = fs::read_to_string(&api_path).unwrap_or_default();
    let normalized_snapshot = normalize_public_api(&api_text);
    let current_api = generate_current_public_api(root);
    let mut api_issues: Vec<String> = Vec::new();
    if !api_path.is_file() || normalized_snapshot.is_empty() {
        api_issues.push("missing or empty .architecture/api/kernel-public-api.txt".to_string());
    }
    match &current_api {
        Ok(current) => {
            if current != &normalized_snapshot {
                api_issues.push(describe_api_snapshot_drift(current, &normalized_snapshot));
            }
            if current.contains("pub trait kernel::Component")
                || current.contains("pub trait kernel::lifecycle::Component")
            {
                api_issues.push("Component trait appears on current public API".to_string());
            }
            if current.contains("Serialize") || current.contains("Deserialize") {
                api_issues.push("serde traits appear on current public API".to_string());
            }
        }
        Err(error) => api_issues.push(error.clone()),
    }
    report.push(
        "KERNEL-API-001",
        api_issues.is_empty(),
        if api_issues.is_empty() {
            format!(
                "current source matches normalized public API snapshot ({} lines; {})",
                normalized_snapshot.lines().count(),
                CARGO_PUBLIC_API_VERSION
            )
        } else {
            api_issues.join("; ")
        },
    );

    // --- KERNEL-API-002：删除一律拒绝；新增逐行登记 Approved RFC ---
    let api002 = evaluate_kernel_api_002(root, current_api.as_ref().ok().map(String::as_str));
    report.push("KERNEL-API-002", api002.ok, api002.detail);

    // --- KERNEL-PUBLISH-001：Cargo / workspace registry / spec 三方一致 ---
    let publish = evaluate_kernel_publish_001(root);
    report.push("KERNEL-PUBLISH-001", publish.ok, publish.detail);

    // 扫描 workspace src（生产）
    let mut all_src = Vec::new();
    for prefix in ["crates", "apps", "tools"] {
        collect_rs_under(&root.join(prefix), &mut all_src);
    }
    // 去掉 tests 目录与 examples
    all_src.retain(|p| {
        let s = rel(root, p);
        !s.contains("/tests/") && !s.contains("/examples/") && !s.contains("/benches/")
    });

    let mut kernel_src = Vec::new();
    collect_rs_under(&kernel_dir.join("src"), &mut kernel_src);

    // --- KERNEL-TIME-001/002/003/004 + ERR-001/002 + SERDE/ASYNC/UNSAFE ---
    let mut system_time_hits = Vec::new();
    let mut instant_hits = Vec::new();
    let mut from_unix_hits = Vec::new();
    let mut from_clock_elapsed_hits = Vec::new();
    let mut internal_count = 0usize;
    let mut err_string_class = Vec::new();
    let mut serde_hits = Vec::new();
    let mut async_hits = Vec::new();
    let mut unsafe_hits = Vec::new();

    for file in &all_src {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        let path_rel = rel(root, file);
        let in_kernel = path_rel.starts_with("crates/kernel/");

        for (i, line) in content.lines().enumerate() {
            let ln = i + 1;
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }

            if !in_kernel
                && (line.contains("SystemTime::now") || line.contains("Utc::now"))
                && !path_rel.starts_with("tools/")
            {
                // tools 侧时间调用不纳入 TIME-001 生产边界（与既有 archgate 一致）
                system_time_hits.push(format!("{path_rel}:{ln}"));
            }

            // tools/ 非生产交易路径：TIME-002/003 不扫（避免门禁实现自引用）
            if !path_rel.starts_with("tools/") {
                if line.contains("Instant::now") && !path_allowed(&path_rel, INSTANT_NOW_ALLOW) {
                    instant_hits.push(format!("{path_rel}:{ln}"));
                }
                if line.contains("from_unix_nanos")
                    && !path_allowed(&path_rel, FROM_UNIX_NANOS_ALLOW)
                {
                    from_unix_hits.push(format!("{path_rel}:{ln}"));
                }
                if line.contains("from_clock_elapsed")
                    && !path_allowed(&path_rel, FROM_CLOCK_ELAPSED_ALLOW)
                {
                    from_clock_elapsed_hits.push(format!("{path_rel}:{ln}"));
                }
            }

            // ERR-001：业务 crate 消费侧（排除 kernel 自身与 tools）
            if line.contains("XError::internal")
                && !path_rel.contains("/tests/")
                && path_rel.starts_with("crates/")
                && !path_rel.starts_with("crates/kernel/")
            {
                internal_count += 1;
            }

            // ERR-002：用字符串子串决定错误分类的常见坏模式
            if (line.contains("ErrorKind::") || line.contains("XError::"))
                && (line.contains("\"not_found\"")
                    || line.contains("\"not found\"")
                    || line.contains("\"other\"")
                    || line.contains("contains(\"not_found\")")
                    || line.contains("contains(\"not found\")"))
            {
                err_string_class.push(format!("{path_rel}:{ln}"));
            }

            if in_kernel {
                if !is_comment_or_attr_line(line)
                    && (line.contains("Serialize")
                        || line.contains("Deserialize")
                        || contains_ident(line, "serde"))
                {
                    serde_hits.push(format!("{path_rel}:{ln}"));
                }
                if !is_comment_or_attr_line(line)
                    && (contains_ident(line, "tokio")
                        || contains_ident(line, "async_std")
                        || line.contains("async-std"))
                {
                    async_hits.push(format!("{path_rel}:{ln}"));
                }
                if line_has_unsafe_usage(line) {
                    unsafe_hits.push(format!("{path_rel}:{ln}"));
                }
            }
        }
    }
    report.internal_count = internal_count;

    // TIME-001：kernel 外生产 SystemTime（tools 已排除）
    report.push(
        "KERNEL-TIME-001",
        system_time_hits.is_empty(),
        if system_time_hits.is_empty() {
            "no SystemTime::now/Utc::now outside kernel production src".into()
        } else {
            format!("unapproved: {system_time_hits:?}")
        },
    );
    report.push(
        "KERNEL-TIME-002",
        instant_hits.is_empty(),
        if instant_hits.is_empty() {
            "Instant::now only under crates/kernel/".into()
        } else {
            format!("unapproved Instant::now: {instant_hits:?}")
        },
    );
    report.push(
        "KERNEL-TIME-003",
        from_unix_hits.is_empty(),
        if from_unix_hits.is_empty() {
            "from_unix_nanos only on allowlist".into()
        } else {
            format!("unapproved from_unix_nanos: {from_unix_hits:?}")
        },
    );
    report.push(
        "KERNEL-TIME-004",
        from_clock_elapsed_hits.is_empty(),
        if from_clock_elapsed_hits.is_empty() {
            "from_clock_elapsed only on allowlist".into()
        } else {
            format!("unapproved from_clock_elapsed: {from_clock_elapsed_hits:?}")
        },
    );

    let err001_ok = internal_count <= XERROR_INTERNAL_BASELINE;
    report.push(
        "KERNEL-ERR-001",
        err001_ok,
        format!(
            "XError::internal production count={internal_count} baseline≤{XERROR_INTERNAL_BASELINE}"
        ),
    );
    report.push(
        "KERNEL-ERR-002",
        err_string_class.is_empty(),
        if err_string_class.is_empty() {
            "no string-class ErrorKind/XError patterns detected".into()
        } else {
            format!("string-class classification: {err_string_class:?}")
        },
    );

    report.push(
        "KERNEL-SERDE-001",
        serde_hits.is_empty(),
        if serde_hits.is_empty() {
            "kernel src has no serde surface".into()
        } else {
            format!("serde hits: {serde_hits:?}")
        },
    );
    report.push(
        "KERNEL-ASYNC-001",
        async_hits.is_empty(),
        if async_hits.is_empty() {
            "kernel src has no tokio/async-std tokens".into()
        } else {
            format!("async hits: {async_hits:?}")
        },
    );
    report.push(
        "KERNEL-UNSAFE-001",
        unsafe_hits.is_empty(),
        if unsafe_hits.is_empty() {
            "kernel unsafe usage count = 0".into()
        } else {
            format!("unsafe hits: {unsafe_hits:?}")
        },
    );

    // --- KERNEL-LIFECYCLE-001：loom 测试资产存在（CI 负责执行）---
    let loom_test = kernel_dir.join("tests/lifecycle_concurrency_loom.rs");
    let loom_body = fs::read_to_string(&loom_test).unwrap_or_default();
    let loom_ok = loom_test.is_file()
        && loom_body.contains("loom::model")
        && loom_body.contains("ShutdownSignal")
        && loom_body.contains("cfg(loom)");
    report.push(
        "KERNEL-LIFECYCLE-001",
        loom_ok,
        if loom_ok {
            "loom suite present (tests/lifecycle_concurrency_loom.rs); CI must execute with --cfg loom"
                .to_string()
        } else {
            "missing or incomplete loom suite for ShutdownSignal".to_string()
        },
    );

    Ok(report)
}

/// KERNEL-API-002 评估结果。
struct Api002Result {
    ok: bool,
    detail: String,
}

fn normalize_public_api(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("pub ")
                || line.starts_with("impl ")
                || line.starts_with("unsafe impl ")
                || line.starts_with("#[")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn command_stderr(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr.trim().chars().take(800).collect()
}

fn generate_current_public_api(root: &Path) -> std::result::Result<String, String> {
    generate_current_public_api_with_program(root, Path::new("cargo"))
}

fn generate_current_public_api_with_program(
    root: &Path,
    cargo_program: &Path,
) -> std::result::Result<String, String> {
    let version = Command::new(cargo_program)
        .args(["public-api", "--version"])
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .map_err(|error| format!("cargo-public-api unavailable: {error}"))?;
    if !version.status.success() {
        return Err(format!(
            "cargo-public-api version check failed (status={}): {}",
            version.status,
            command_stderr(&version)
        ));
    }
    let actual_version = String::from_utf8(version.stdout)
        .map_err(|error| format!("cargo-public-api version is not UTF-8: {error}"))?;
    if actual_version.trim() != CARGO_PUBLIC_API_VERSION {
        return Err(format!(
            "cargo-public-api version mismatch: expected {CARGO_PUBLIC_API_VERSION:?}, got {:?}",
            actual_version.trim()
        ));
    }

    let output = Command::new(cargo_program)
        .args([
            "public-api",
            "--manifest-path",
            "crates/kernel/Cargo.toml",
            "--simplified",
            "--color=never",
        ])
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .map_err(|error| format!("execute cargo-public-api: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo-public-api generation failed (status={}): {}",
            output.status,
            command_stderr(&output)
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("cargo-public-api output is not UTF-8: {error}"))?;
    let normalized = normalize_public_api(&stdout);
    if normalized.is_empty() {
        return Err("cargo-public-api returned no normalized API lines".into());
    }
    Ok(normalized)
}

fn describe_api_snapshot_drift(current: &str, snapshot: &str) -> String {
    let current_lines: HashSet<_> = current.lines().collect();
    let snapshot_lines: HashSet<_> = snapshot.lines().collect();
    let mut source_only: Vec<_> = current_lines.difference(&snapshot_lines).copied().collect();
    let mut snapshot_only: Vec<_> = snapshot_lines.difference(&current_lines).copied().collect();
    source_only.sort_unstable();
    snapshot_only.sort_unstable();
    format!(
        "stale public API snapshot: source_only={:?} snapshot_only={:?}{}",
        source_only.iter().take(5).collect::<Vec<_>>(),
        snapshot_only.iter().take(5).collect::<Vec<_>>(),
        if source_only.is_empty() && snapshot_only.is_empty() {
            " (line order or multiplicity differs)"
        } else {
            ""
        }
    )
}

fn fnv1a64(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 相对冻结 baseline：删除/签名变化 fail closed；新增逐行登记 Approved RFC。
fn evaluate_kernel_api_002(root: &Path, current_api: Option<&str>) -> Api002Result {
    let baseline_path = root.join(".architecture/api/kernel-public-api.baseline.txt");
    let rfc_path = root.join(".architecture/api/kernel-api-rfc.toml");

    if !baseline_path.is_file() {
        return Api002Result {
            ok: false,
            detail: "missing .architecture/api/kernel-public-api.baseline.txt".into(),
        };
    }
    let Some(current_api) = current_api else {
        return Api002Result {
            ok: false,
            detail: "current source API unavailable; API-002 cannot use committed snapshot as a fallback"
                .into(),
        };
    };
    let baseline = normalize_public_api(&fs::read_to_string(&baseline_path).unwrap_or_default());
    let baseline_line_count = baseline.lines().count();
    let baseline_hash = fnv1a64(&baseline);
    if baseline_line_count != KERNEL_API_BASELINE_LINES
        || baseline_hash != KERNEL_API_BASELINE_FNV1A64
    {
        return Api002Result {
            ok: false,
            detail: format!(
                "frozen baseline drift: expected lines={} fnv1a64={:#018x}, got lines={} fnv1a64={:#018x}; baseline re-freeze is not supported by this rule",
                KERNEL_API_BASELINE_LINES,
                KERNEL_API_BASELINE_FNV1A64,
                baseline_line_count,
                baseline_hash
            ),
        };
    }

    let baseline_lines: HashSet<&str> = baseline.lines().collect();
    let current_lines: HashSet<&str> = current_api.lines().collect();
    let mut removals: Vec<_> = baseline_lines.difference(&current_lines).copied().collect();
    let mut additions: Vec<_> = current_lines.difference(&baseline_lines).copied().collect();
    removals.sort_unstable();
    additions.sort_unstable();

    if !removals.is_empty() {
        return Api002Result {
            ok: false,
            detail: format!(
                "breaking public API removal/signature change is forbidden (sample): {:?}",
                removals.iter().take(8).collect::<Vec<_>>()
            ),
        };
    }

    if additions.is_empty() {
        return Api002Result {
            ok: true,
            detail: format!(
                "no public API additions beyond baseline ({} lines)",
                baseline_lines.len()
            ),
        };
    }

    let rfc_text = fs::read_to_string(&rfc_path).unwrap_or_default();
    if rfc_text.trim().is_empty() && !rfc_path.is_file() {
        return Api002Result {
            ok: false,
            detail: format!(
                "{} addition(s) beyond baseline but missing kernel-api-rfc.toml: {:?}",
                additions.len(),
                additions.iter().take(5).collect::<Vec<_>>()
            ),
        };
    }
    let allows = parse_api_rfc_allows(&rfc_text);
    let mut unregistered = Vec::new();
    let mut bad_rfc = Vec::new();

    for add in &additions {
        let Some(entry) = allows.iter().find(|a| api_line_matches(add, &a.pattern)) else {
            unregistered.push((*add).to_string());
            continue;
        };
        match resolve_rfc_approved(root, &entry.rfc) {
            Ok(true) => {}
            Ok(false) => bad_rfc.push(format!("{add} → rfc={} (not Approved)", entry.rfc)),
            Err(e) => bad_rfc.push(format!("{add} → rfc={}: {e}", entry.rfc)),
        }
    }

    if unregistered.is_empty() && bad_rfc.is_empty() {
        return Api002Result {
            ok: true,
            detail: format!(
                "{} addition(s) beyond baseline all registered with Approved RFC",
                additions.len()
            ),
        };
    }

    let mut parts = Vec::new();
    if !unregistered.is_empty() {
        parts.push(format!(
            "unregistered additions (sample): {:?}",
            unregistered.iter().take(8).collect::<Vec<_>>()
        ));
    }
    if !bad_rfc.is_empty() {
        parts.push(format!(
            "rfc issues: {:?}",
            bad_rfc.iter().take(5).collect::<Vec<_>>()
        ));
    }
    Api002Result {
        ok: false,
        detail: parts.join("; "),
    }
}

struct PublishResult {
    ok: bool,
    detail: String,
}

fn parse_cargo_publish(text: &str) -> std::result::Result<bool, String> {
    let value: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
    value
        .get("package")
        .and_then(|package| package.get("publish"))
        .and_then(toml::Value::as_bool)
        .ok_or_else(|| "[package].publish must be an explicit boolean".into())
}

fn parse_workspace_kernel_publish(text: &str) -> std::result::Result<bool, String> {
    let value: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
    let units = value
        .get("unit")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "workspace.toml has no [[unit]] array".to_string())?;
    let matches: Vec<_> = units
        .iter()
        .filter(|unit| unit.get("path").and_then(toml::Value::as_str) == Some("crates/kernel"))
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "expected exactly one [[unit]] path=crates/kernel, got {}",
            matches.len()
        ));
    }
    matches[0]
        .get("publish")
        .and_then(toml::Value::as_bool)
        .or_else(|| {
            value
                .get("defaults")
                .and_then(|defaults| defaults.get("publish"))
                .and_then(toml::Value::as_bool)
        })
        .ok_or_else(|| {
            "kernel unit publish and defaults.publish are both missing/non-boolean".into()
        })
}

fn bool_prefix(text: &str) -> Option<bool> {
    let value = text.trim_start();
    for (token, parsed) in [("true", true), ("false", false)] {
        if let Some(rest) = value.strip_prefix(token) {
            if rest
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
            {
                return Some(parsed);
            }
        }
    }
    None
}

fn parse_spec_publish(text: &str) -> std::result::Result<bool, String> {
    let mut declarations = Vec::new();
    let mut invalid = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim().trim_start_matches("- ").trim();
        let value = trimmed
            .strip_prefix("Publish:")
            .or_else(|| trimmed.strip_prefix("publish ="));
        let Some(value) = value else {
            continue;
        };
        match bool_prefix(value) {
            Some(value) => declarations.push((index + 1, value)),
            None => invalid.push(index + 1),
        }
    }
    if !invalid.is_empty() {
        return Err(format!(
            "non-boolean publish declaration(s) at lines {invalid:?}"
        ));
    }
    let Some((_, expected)) = declarations.first().copied() else {
        return Err("spec has no `Publish:` or `publish =` boolean declaration".into());
    };
    let conflicts: Vec<_> = declarations
        .iter()
        .filter(|(_, value)| *value != expected)
        .copied()
        .collect();
    if !conflicts.is_empty() {
        return Err(format!(
            "conflicting spec publish declarations: {declarations:?}"
        ));
    }
    Ok(expected)
}

fn evaluate_kernel_publish_001(root: &Path) -> PublishResult {
    let read_and_parse = |path: &Path,
                          parser: fn(&str) -> std::result::Result<bool, String>|
     -> std::result::Result<bool, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("read: {error}"))?;
        parser(&text).map_err(|error| format!("parse: {error}"))
    };
    let cargo = read_and_parse(&root.join("crates/kernel/Cargo.toml"), parse_cargo_publish);
    let workspace = read_and_parse(
        &root.join(".architecture/workspace.toml"),
        parse_workspace_kernel_publish,
    );
    let spec = read_and_parse(
        &root.join(".agent/SSOT/kernel/spec/spec.md"),
        parse_spec_publish,
    );
    let ok = publish_sources_agree(&cargo, &workspace, &spec);
    let render = |value: &std::result::Result<bool, String>| match value {
        Ok(value) => value.to_string(),
        Err(error) => format!("ERROR({error})"),
    };
    PublishResult {
        ok,
        detail: format!(
            "cargo={} workspace={} spec={}",
            render(&cargo),
            render(&workspace),
            render(&spec)
        ),
    }
}

fn publish_sources_agree(
    cargo: &std::result::Result<bool, String>,
    workspace: &std::result::Result<bool, String>,
    spec: &std::result::Result<bool, String>,
) -> bool {
    matches!((cargo, workspace, spec), (Ok(a), Ok(b), Ok(c)) if a == b && b == c)
}

#[derive(Debug, Clone)]
struct ApiRfcAllow {
    pattern: String,
    rfc: String,
}

/// 极简解析 `[[allow]]` 块（避免 archgate 新增 toml 依赖）。
fn parse_api_rfc_allows(text: &str) -> Vec<ApiRfcAllow> {
    let mut out = Vec::new();
    let mut in_allow = false;
    let mut pattern: Option<String> = None;
    let mut rfc: Option<String> = None;

    let flush =
        |out: &mut Vec<ApiRfcAllow>, pattern: &mut Option<String>, rfc: &mut Option<String>| {
            if let (Some(p), Some(r)) = (pattern.take(), rfc.take()) {
                if !p.is_empty() && !r.is_empty() {
                    out.push(ApiRfcAllow { pattern: p, rfc: r });
                }
            } else {
                *pattern = None;
                *rfc = None;
            }
        };

    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if t == "[[allow]]" {
            flush(&mut out, &mut pattern, &mut rfc);
            in_allow = true;
            continue;
        }
        if t.starts_with('[') {
            flush(&mut out, &mut pattern, &mut rfc);
            in_allow = false;
            continue;
        }
        if !in_allow {
            continue;
        }
        if let Some(rest) = t.strip_prefix("pattern") {
            let v = rest
                .trim_start()
                .trim_start_matches('=')
                .trim()
                .trim_matches('"')
                .to_string();
            pattern = Some(v);
        } else if let Some(rest) = t.strip_prefix("rfc") {
            let v = rest
                .trim_start()
                .trim_start_matches('=')
                .trim()
                .trim_matches('"')
                .to_string();
            rfc = Some(v);
        }
    }
    flush(&mut out, &mut pattern, &mut rfc);
    out
}

fn api_line_matches(line: &str, pattern: &str) -> bool {
    !pattern.is_empty() && !pattern.contains('*') && line == pattern
}

/// 将 rfc 引用解析为文件并检查是否 Approved。
fn resolve_rfc_approved(root: &Path, rfc: &str) -> Result<bool, String> {
    let path = resolve_rfc_path(root, rfc)?;
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if rfc_text_is_approved(&text) {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn resolve_rfc_path(root: &Path, rfc: &str) -> Result<PathBuf, String> {
    let r = rfc.trim();
    if r.is_empty() {
        return Err("empty rfc id".into());
    }
    // F-08：拒绝 absolute / `..` / 试图逃逸 repo 的路径（不依赖 canonicalize 前的 symlink 展开）。
    let as_path = Path::new(r);
    if as_path.is_absolute() {
        return Err(format!(
            "rfc path must be repo-relative (absolute rejected): {r}"
        ));
    }
    if as_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(format!("rfc path must not contain '..': {r}"));
    }

    // 直接路径
    let direct = root.join(r);
    if direct.is_file() {
        return ensure_rfc_contained(root, &direct);
    }
    // 常见别名
    if r.eq_ignore_ascii_case("SPEC-KERNEL-002") || r.eq_ignore_ascii_case("KERNEL-002") {
        let p = root.join(".agent/SSOT/kernel/spec/spec.md");
        if p.is_file() {
            return ensure_rfc_contained(root, &p);
        }
    }
    // docs/specs/<id>.md（id 本身不得含路径分隔或 `..`）
    if r.contains('/') || r.contains('\\') {
        return Err(format!(
            "cannot resolve rfc '{r}' to an existing file (try path like docs/specs/foo.md or SPEC-KERNEL-002)"
        ));
    }
    let cand = root.join("docs/specs").join(format!("{r}.md"));
    if cand.is_file() {
        return ensure_rfc_contained(root, &cand);
    }
    Err(format!(
        "cannot resolve rfc '{r}' to an existing file (try path like docs/specs/foo.md or SPEC-KERNEL-002)"
    ))
}

/// 将解析结果约束在 `root` 之下（canonical containment）。
fn ensure_rfc_contained(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root_canon = root
        .canonicalize()
        .map_err(|e| format!("canonicalize root {}: {e}", root.display()))?;
    let path_canon = path
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {e}", path.display()))?;
    if !path_canon.starts_with(&root_canon) {
        return Err(format!(
            "rfc path escapes repository root: {} (resolved {})",
            path.display(),
            path_canon.display()
        ));
    }
    Ok(path_canon)
}

fn rfc_text_is_approved(text: &str) -> bool {
    for line in text.lines().take(80) {
        let cleaned = line.trim().trim_start_matches("- ").replace("**", "");
        for label in ["Status", "状态", "审批结论"] {
            let Some(rest) = cleaned.strip_prefix(label) else {
                continue;
            };
            let value = rest.trim_start_matches([':', '：']).trim_start();
            if value.split_whitespace().next() == Some("Approved") {
                return true;
            }
        }
    }
    false
}

fn contains_ident(line: &str, token: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    fn dependency(
        package: &str,
        rename: Option<&str>,
        kind: KernelDepKind,
        target: Option<&str>,
        is_path: bool,
    ) -> KernelDependency {
        KernelDependency {
            package: package.into(),
            rename: rename.map(str::to_owned),
            kind,
            target: target.map(str::to_owned),
            is_path,
        }
    }

    fn api_fixture() -> PathBuf {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "xhyper-archgate-kernel-api-{}-{id}",
            std::process::id()
        ));
        let api_dir = root.join(".architecture/api");
        fs::create_dir_all(&api_dir).expect("create API fixture");
        fs::write(
            api_dir.join("kernel-public-api.baseline.txt"),
            include_str!("../../../.architecture/api/kernel-public-api.baseline.txt"),
        )
        .expect("write frozen baseline");
        root
    }

    #[test]
    fn rfc_path_rejects_absolute_and_parent_escape() {
        let root = api_fixture();
        assert!(resolve_rfc_path(&root, "/etc/passwd").is_err());
        assert!(resolve_rfc_path(&root, "../outside.md").is_err());
        assert!(resolve_rfc_path(&root, "docs/../../outside.md").is_err());
        // relative non-existent still errors (cannot resolve)
        assert!(resolve_rfc_path(&root, "docs/specs/does-not-exist-xyz.md").is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn feature_keys_default_only() {
        let m = r#"
[features]
# comment
default = []
"#;
        assert_eq!(parse_feature_keys(m), vec!["default"]);
    }

    #[test]
    fn feature_keys_rejects_extra() {
        let m = r#"
[features]
default = []
mock = []
"#;
        assert_eq!(parse_feature_keys(m), vec!["default", "mock"]);
    }

    #[test]
    fn unsafe_usage_ignores_forbid_attr() {
        assert!(!line_has_unsafe_usage("#![forbid(unsafe_code)]"));
        assert!(!line_has_unsafe_usage("// unsafe block"));
        assert!(line_has_unsafe_usage("unsafe fn f() {}"));
        assert!(line_has_unsafe_usage("let x = unsafe { 1 };"));
    }

    #[test]
    fn dep_whitelist() {
        assert!(is_allowed_kernel_external_dep("thiserror"));
        assert!(!is_allowed_kernel_external_dep("anyhow"));
    }

    #[test]
    fn dependency_gate_covers_build_target_and_rename() {
        let dependencies = vec![
            dependency("thiserror", None, KernelDepKind::Normal, None, false),
            dependency(
                "loom",
                None,
                KernelDepKind::Normal,
                Some("cfg(loom)"),
                false,
            ),
            dependency("thiserror", None, KernelDepKind::Build, None, false),
            dependency(
                "anyhow",
                Some("renamed_anyhow"),
                KernelDepKind::Normal,
                Some("cfg(unix)"),
                false,
            ),
            dependency(
                "xhyper-types",
                Some("types_alias"),
                KernelDepKind::Normal,
                None,
                true,
            ),
        ];
        let workspace = HashSet::from(["xhyper-types".to_owned()]);
        let result = evaluate_kernel_dependencies(&dependencies, &workspace);
        assert_eq!(result.allowed_external.len(), 2);
        assert_eq!(result.disallowed_external.len(), 2);
        assert!(result.disallowed_external[0].contains("anyhow"));
        assert!(result.disallowed_external[1].contains("thiserror"));
        assert_eq!(result.workspace_or_path.len(), 1);
        assert!(result.workspace_or_path[0].contains("types_alias"));
    }

    #[test]
    fn loom_exception_is_exact() {
        let wrong_target = dependency(
            "loom",
            None,
            KernelDepKind::Normal,
            Some("cfg(test)"),
            false,
        );
        let renamed = dependency(
            "loom",
            Some("loom_alias"),
            KernelDepKind::Normal,
            Some("cfg(loom)"),
            false,
        );
        let renamed_thiserror = dependency(
            "thiserror",
            Some("error_derive"),
            KernelDepKind::Normal,
            None,
            false,
        );
        assert!(!is_exact_kernel_external_allow(&wrong_target));
        assert!(!is_exact_kernel_external_allow(&renamed));
        assert!(!is_exact_kernel_external_allow(&renamed_thiserror));
    }

    #[test]
    fn parse_api_rfc_allows_blocks() {
        let t = r#"
# comment
[[allow]]
pattern = "pub fn kernel::error::XError::from_static"
rfc = "SPEC-KERNEL-002"
note = "x"

[[allow]]
pattern = "pub fn kernel::foo::*"
rfc = "docs/specs/example.md"
"#;
        let a = parse_api_rfc_allows(t);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].pattern, "pub fn kernel::error::XError::from_static");
        assert_eq!(a[0].rfc, "SPEC-KERNEL-002");
        assert_eq!(a[1].pattern, "pub fn kernel::foo::*");
    }

    #[test]
    fn api_line_allow_is_exact_and_rejects_wildcards() {
        assert!(api_line_matches(
            "pub fn kernel::error::XError::from_static",
            "pub fn kernel::error::XError::from_static"
        ));
        assert!(!api_line_matches(
            "pub fn kernel::error::XError::from_static(…)",
            "pub fn kernel::error::XError::from_static*"
        ));
        assert!(!api_line_matches("pub fn other", "pub fn kernel::"));
    }

    #[test]
    fn normalization_drops_cargo_noise_but_snapshot_drift_is_visible() {
        let snapshot = normalize_public_api(
            " Documenting kernel v0.1.1 (/tmp/kernel)\n    Finished dev\npub mod kernel\n",
        );
        assert_eq!(snapshot, "pub mod kernel");
        let detail = describe_api_snapshot_drift("pub mod kernel\npub fn kernel::new()", &snapshot);
        assert!(detail.contains("stale public API snapshot"));
        assert!(detail.contains("kernel::new"));
    }

    #[test]
    fn missing_cargo_public_api_is_not_a_pass() {
        let error = generate_current_public_api_with_program(
            Path::new("."),
            Path::new("/definitely/missing/cargo-for-archgate-test"),
        )
        .expect_err("missing tool must fail closed");
        assert!(error.contains("unavailable"));
    }

    #[test]
    fn api_removal_fails_even_when_snapshot_could_be_updated() {
        let root = api_fixture();
        let baseline = normalize_public_api(include_str!(
            "../../../.architecture/api/kernel-public-api.baseline.txt"
        ));
        let current = baseline.lines().skip(1).collect::<Vec<_>>().join("\n");
        let result = evaluate_kernel_api_002(&root, Some(&current));
        assert!(!result.ok);
        assert!(result.detail.contains("removal/signature change"));
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn api_addition_requires_exact_approved_rfc() {
        let root = api_fixture();
        let api_dir = root.join(".architecture/api");
        let spec = root.join("docs/specs/kernel-addition.md");
        fs::create_dir_all(spec.parent().expect("spec parent")).expect("create spec dir");
        fs::write(&spec, "Status: Approved\n").expect("write approved RFC");
        fs::write(
            api_dir.join("kernel-api-rfc.toml"),
            r#"[[allow]]
pattern = "pub fn kernel::approved_addition()"
rfc = "docs/specs/kernel-addition.md"
"#,
        )
        .expect("write API allow");
        let baseline = normalize_public_api(include_str!(
            "../../../.architecture/api/kernel-public-api.baseline.txt"
        ));
        let current = format!("{baseline}\npub fn kernel::approved_addition()");
        let approved = evaluate_kernel_api_002(&root, Some(&current));
        assert!(approved.ok, "{}", approved.detail);

        fs::write(&spec, "Status: Proposed\n").expect("downgrade RFC");
        let proposed = evaluate_kernel_api_002(&root, Some(&current));
        assert!(!proposed.ok);
        assert!(proposed.detail.contains("not Approved"));
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn unregistered_api_addition_fails_closed() {
        let root = api_fixture();
        // No kernel-api-rfc.toml: addition beyond frozen baseline must fail.
        let baseline = normalize_public_api(include_str!(
            "../../../.architecture/api/kernel-public-api.baseline.txt"
        ));
        let current = format!("{baseline}\npub fn kernel::unauthorized_addition()");
        let result = evaluate_kernel_api_002(&root, Some(&current));
        assert!(!result.ok, "unregistered addition must fail");
        assert!(
            result.detail.contains("unregistered additions")
                || result.detail.contains("missing kernel-api-rfc.toml"),
            "detail={}",
            result.detail
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn api_002_does_not_fall_back_to_committed_snapshot() {
        let root = api_fixture();
        let result = evaluate_kernel_api_002(&root, None);
        assert!(!result.ok);
        assert!(
            result.detail.contains("current source API unavailable"),
            "detail={}",
            result.detail
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn frozen_baseline_fingerprint_rejects_tampered_baseline() {
        let root = api_fixture();
        let baseline_path = root
            .join(".architecture/api")
            .join("kernel-public-api.baseline.txt");
        // Drop one line so fingerprint / line count drift.
        let tampered = normalize_public_api(&fs::read_to_string(&baseline_path).unwrap())
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&baseline_path, tampered).expect("tamper baseline");
        let baseline = normalize_public_api(include_str!(
            "../../../.architecture/api/kernel-public-api.baseline.txt"
        ));
        let result = evaluate_kernel_api_002(&root, Some(&baseline));
        assert!(!result.ok);
        assert!(
            result.detail.contains("frozen baseline drift"),
            "detail={}",
            result.detail
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn publish_parsers_cover_consistency_and_spec_conflict() {
        assert_eq!(
            parse_cargo_publish("[package]\nname='xhyper-kernel'\npublish=true\n"),
            Ok(true)
        );
        assert_eq!(
            parse_workspace_kernel_publish(
                "[defaults]\npublish=false\n[[unit]]\npath='crates/kernel'\nlayer='kernel'\npublish=true\n"
            ),
            Ok(true)
        );
        assert_eq!(
            parse_spec_publish("Publish: true\n# 15\npublish = true\n"),
            Ok(true)
        );
        let conflict = parse_spec_publish("Publish: true\n# 15\npublish = false\n")
            .expect_err("conflicting declarations must fail");
        assert!(conflict.contains("conflicting"));
        assert!(publish_sources_agree(&Ok(true), &Ok(true), &Ok(true)));
        assert!(!publish_sources_agree(&Ok(true), &Ok(false), &Ok(true)));
        assert!(!publish_sources_agree(
            &Ok(true),
            &Ok(true),
            &Err("missing".into())
        ));
    }

    #[test]
    fn rfc_approved_detection() {
        assert!(rfc_text_is_approved("Status:         Approved\n"));
        assert!(rfc_text_is_approved("- **状态**：Approved\n"));
        assert!(!rfc_text_is_approved("Status:         Proposed\n"));
        assert!(!rfc_text_is_approved("Status: Not Approved\n"));
    }
}
