//! CI SSOT 执行引擎（SPEC-CI-SSOT-001 / GOAL-CI-SSOT-001）— Wave1–5 agent-safe Shadow。
//!
//! **Shadow only / DRAFT**：不激活生产 Required Checks，不修改 GitHub Ruleset /
//! secrets / Runner。`plan` 做变更分类；`run`/`aggregate`/`reconcile`/`metrics`
//! 提供 **dry-run** 合同输出（不执行真实编译测试、不 apply 控制面）。
//! 完整 `just ci` 对等仍属过渡期，由 `local` 子命令显式声明。

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub mod autoresearch;
pub mod chaos;
pub mod domain_gates;
pub mod drift;
pub mod fingerprint;
pub mod flake;
pub mod graph;
pub mod locks;
pub mod verify_runner;

/// `cargo xtask ci <sub>` 子命令。
#[derive(Debug, Subcommand)]
pub enum CiCommand {
    /// 打印 Shadow plan JSON（变更分类；缺省 unknown → full）
    Plan {
        /// 变更路径列表文件（每行一条相对路径）
        #[arg(long)]
        paths_file: Option<PathBuf>,
        /// 逗号分隔路径（脚本/测试）
        #[arg(long, value_delimiter = ',')]
        paths: Option<Vec<String>>,
    },
    /// 校验 `.github/ci/baseline.toml` 最小必填键
    Verify,
    /// 先 verify，再提示本地完整对等仍为过渡（不重实现 just ci）
    Local,
    /// 报告 baseline / schema / xtask 接线 / just recipes 就绪度
    Doctor,
    /// 渲染 generated 投影（Shadow dry-run；写入 workflow-contract / policy-table）
    Render,
    /// Lane dry-run：按 plan 输出 lane 决策，不执行真实 command
    Run {
        /// 强制 dry-run（默认 true；生产执行未授权）
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
        /// 可选 plan JSON；缺省现场 `plan`
        #[arg(long)]
        plan_file: Option<PathBuf>,
    },
    /// Aggregate dry-run：Expected/Actual + 状态机（always 语义）
    Aggregate {
        /// lane 决策 JSON 对象，如 {"fast":"RUN_PASS","build_test":"RUN_PASS"}
        /// **必填**（除非 `--synthetic-smoke`）；缺省不再默认全绿（PHASE-1-07 / AC-04 fail-closed）
        #[arg(long)]
        decisions_file: Option<PathBuf>,
        /// 期望 lanes，逗号分隔；缺省取 baseline lanes
        #[arg(long)]
        expected: Option<String>,
        /// 显式本地 smoke：为全部 expected lane 合成 RUN_PASS（**非**裁决；生产/shadow workflow 禁止依赖）
        #[arg(long, default_value_t = false)]
        synthetic_smoke: bool,
        /// REUSED attestation 的唯一信任根；只接受根内相对路径。
        #[arg(long)]
        attestation_root: Option<PathBuf>,
    },
    /// 只读对账 Desired vs Observed（dry-run；不 apply）
    Reconcile {
        /// observed JSON；缺省 synthetic MATCH 骨架
        #[arg(long)]
        observed_file: Option<PathBuf>,
    },
    /// 输出 CI metrics 事件（schema 对齐；可写样例）
    Metrics {
        /// 写出仓库内样例 JSONL（evidence/ci/samples/）
        #[arg(long, default_value_t = false)]
        write_sample: bool,
        /// 校验周报/cadence 模板存在
        #[arg(long, default_value_t = false)]
        weekly_template: bool,
    },
    /// Affected graph + reverse dependents
    Graph {
        #[arg(long)]
        paths_file: Option<PathBuf>,
        #[arg(long, value_delimiter = ',')]
        paths: Option<Vec<String>>,
    },
    /// Generated drift fail-closed
    Drift {
        /// 仅覆盖 drift 合同根目录（测试/离线验证）；缺省仍自动发现 workspace。
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Fingerprint {
        /// 完整 typed FingerprintInputV1 JSON；旧 lane/plan 快捷输入已移除。
        #[arg(long)]
        input: PathBuf,
    },
    Reuse {
        #[arg(long, default_value_t = false)]
        want_reused: bool,
        /// 可信根目录；`attestation` 只能是其下相对 UTF-8 POSIX 路径。
        #[arg(long)]
        attestation_root: Option<PathBuf>,
        #[arg(long)]
        attestation: Option<String>,
        /// 未验证的当前 observation；只用于 structural predicates。
        #[arg(long)]
        context: Option<PathBuf>,
    },
    VerifyRunner {
        #[arg(long)]
        class: String,
        /// 从 root-owned attestation 与本机资源/工具现场采集；禁止 workflow 常量自报。
        #[arg(long, default_value_t = false)]
        observe_current: bool,
    },
    Locks {
        /// 仅覆盖 lock 合同根目录（测试/离线验证）；缺省仍自动发现 workspace。
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Flake {
        /// 显式 UTC 日期（测试/回放）；缺省从系统 UTC clock 读取。
        #[arg(long)]
        today: Option<String>,
        /// 可选 registry 文件（默认 `.github/ci/flakes.toml`；负向 fixture 注入）
        #[arg(long)]
        registry_file: Option<PathBuf>,
    },
    FailureFp {
        #[arg(long)]
        lane: String,
        #[arg(long)]
        command: String,
        #[arg(long, default_value = "unknown")]
        error_class: String,
    },
    EvidenceRoot,
    Taxonomy,
    /// Determinism digest（Transitional / AC-11 升级路径）
    Determinism {
        #[arg(long, default_value = r#"{"fixture":true,"v":1}"#)]
        payload: String,
    },
    /// No-lookahead property 骨架（AC-12）
    NoLookahead {
        /// 事件 fixture JSON；缺省使用内置合法样例
        #[arg(long)]
        fixture: Option<PathBuf>,
    },
    /// 领域 Gate 升级路径骨架检查
    DomainGates,
    /// AutoResearch Shadow 报告（PHASE-5-01；默认不 apply）
    Autoresearch {
        #[arg(long, default_value_t = false)]
        write: bool,
    },
    /// Spec §26 可执行负向子集（PHASE-5-07）
    Chaos,
}

#[derive(Debug, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub baseline: String,
    pub missing_keys: Vec<String>,
    pub mode: &'static str,
}

#[derive(Debug, Serialize)]
pub struct PlanReport {
    pub schema_version: u32,
    pub change_class: String,
    pub unknown_change_behavior: &'static str,
    pub mode: &'static str,
    pub note: String,
    pub lanes: serde_json::Value,
    pub path_count: usize,
    pub fast_lane: serde_json::Value,
    pub shards: serde_json::Value,
    pub special_matrix: serde_json::Value,
    pub affected: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub mode: &'static str,
    pub baseline_present: bool,
    pub schema_present: bool,
    pub evidence_schemas_present: bool,
    pub xtask_ci_wired: bool,
    pub just_recipes_present: bool,
    pub details: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LocalReport {
    pub ok: bool,
    pub mode: &'static str,
    pub verify_ok: bool,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RenderReport {
    pub ok: bool,
    pub mode: &'static str,
    pub baseline_digest: String,
    pub written: Vec<String>,
    pub note: &'static str,
}

#[cfg(test)]
#[derive(Debug, Serialize)]
pub struct StubReport {
    pub ok: bool,
    pub command: &'static str,
    pub mode: &'static str,
    pub status: &'static str,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RunReport {
    pub ok: bool,
    pub mode: &'static str,
    pub dry_run: bool,
    pub change_class: String,
    pub lanes: serde_json::Value,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AggregateReport {
    pub ok: bool,
    pub mode: &'static str,
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub run_id: String,
    pub expected_lanes: Vec<String>,
    pub actual_lanes: Vec<String>,
    pub decisions: serde_json::Map<String, serde_json::Value>,
    pub missing_lanes: Vec<String>,
    pub unexpected_lanes: Vec<String>,
    pub not_applicable_reasons: serde_json::Map<String, serde_json::Value>,
    pub reused_sources: serde_json::Map<String, serde_json::Value>,
    pub final_decision: String,
    pub baseline_digest: String,
    pub ruleset_context: String,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ReconcileReport {
    pub ok: bool,
    pub mode: &'static str,
    pub status: String,
    pub desired: serde_json::Value,
    pub observed: serde_json::Value,
    pub drifts: Vec<String>,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
pub struct MetricsReport {
    pub ok: bool,
    pub mode: &'static str,
    pub events: Vec<serde_json::Value>,
    pub written: Vec<String>,
    pub note: &'static str,
}

/// Metrics schema 相对路径。
pub const METRICS_EVENT_SCHEMA_REL: &str = "evidence/schemas/ci/metrics-event.schema.json";

/// baseline.toml 相对仓库根的路径。
pub const BASELINE_REL: &str = ".github/ci/baseline.toml";
/// schema.json 相对路径。
pub const SCHEMA_REL: &str = ".github/ci/schema.json";
/// Lane attestation schema 相对路径。
pub const LANE_ATTESTATION_SCHEMA_REL: &str = "evidence/schemas/ci/lane-attestation.schema.json";
/// Aggregate decision schema 相对路径。
pub const AGGREGATE_DECISION_SCHEMA_REL: &str =
    "evidence/schemas/ci/aggregate-decision.schema.json";

/// SPEC §4.1 最小必填：顶层键 / 段 / 段内键（去注释后按 section 检查；无完整 TOML crate）。
const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &["schema_version", "baseline_id", "baseline_version"];

/// (section header without brackets, required keys inside that section)
const REQUIRED_SECTIONS: &[(&str, &[&str])] = &[
    ("merge", &["required_context", "shadow_context"]),
    ("toolchains", &["primary", "msrv"]),
    ("budgets", &[]),
    ("reuse", &["enabled"]),
    ("planner", &["unknown_change_behavior"]),
    ("lanes.fast", &["command", "runner_class"]),
    ("lanes.build_test", &["command", "runner_class"]),
];

/// 入口：分发 `ci` 子命令。
pub fn run(json: bool, action: CiCommand) -> Result<()> {
    let root = workspace_root()?;
    match action {
        CiCommand::Verify => {
            let report = verify(&root)?;
            emit_verify(json, &report)?;
            if !report.ok {
                bail!(
                    "ci verify: baseline incomplete ({} missing)",
                    report.missing_keys.len()
                );
            }
            Ok(())
        }
        CiCommand::Plan { paths_file, paths } => {
            let report = plan_from_inputs(paths_file.as_deref(), paths.as_deref())?;
            emit_plan(json, &report)?;
            Ok(())
        }
        CiCommand::Local => {
            let report = local(&root)?;
            emit_local(json, &report)?;
            if !report.ok {
                bail!("ci local: verify failed");
            }
            Ok(())
        }
        CiCommand::Doctor => {
            let report = doctor(&root)?;
            emit_doctor(json, &report)?;
            if !report.ok {
                bail!("ci doctor: one or more checks failed");
            }
            Ok(())
        }
        CiCommand::Render => {
            let report = render(&root)?;
            emit_render(json, &report)?;
            if !report.ok {
                bail!("ci render: failed");
            }
            Ok(())
        }
        CiCommand::Run { dry_run, plan_file } => {
            if !dry_run {
                bail!("ci run: non-dry-run execution is not authorized in shadow mode");
            }
            let report = run_lanes_dry(&root, plan_file.as_deref())?;
            emit_run(json, &report)?;
            if !report.ok {
                bail!("ci run: dry-run failed");
            }
            Ok(())
        }
        CiCommand::Aggregate {
            decisions_file,
            expected,
            synthetic_smoke,
            attestation_root,
        } => {
            let report = aggregate_dry_with_attestation_root(
                &root,
                decisions_file.as_deref(),
                expected.as_deref(),
                synthetic_smoke,
                attestation_root.as_deref(),
            )?;
            emit_aggregate(json, &report)?;
            // Aggregate always emits; non-PASS exits non-zero (fail-closed)
            if report.final_decision != "PASS" {
                bail!("ci aggregate: final_decision={}", report.final_decision);
            }
            Ok(())
        }
        CiCommand::Reconcile { observed_file } => {
            let report = reconcile_dry(&root, observed_file.as_deref())?;
            emit_reconcile(json, &report)?;
            if !report.ok {
                bail!("ci reconcile: DRIFT or failure");
            }
            Ok(())
        }
        CiCommand::Metrics {
            write_sample,
            weekly_template,
        } => {
            let report = metrics_events(&root, write_sample, weekly_template)?;
            emit_metrics(json, &report)?;
            if !report.ok {
                bail!("ci metrics: failed");
            }
            Ok(())
        }
        CiCommand::Graph { paths_file, paths } => {
            let mut all = Vec::new();
            if let Some(list) = paths {
                all.extend(list);
            }
            if let Some(file) = paths_file {
                let body = fs::read_to_string(&file)?;
                for line in body.lines() {
                    let tt = line.trim();
                    if !tt.is_empty() && !tt.starts_with('#') {
                        all.push(tt.to_string());
                    }
                }
            }
            let report = graph::affected_from_paths(&root, &all)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci graph: full_fallback={} packages={}",
                    report.full_fallback,
                    report.affected_packages.len()
                );
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            Ok(())
        }
        CiCommand::Drift {
            root: root_override,
        } => {
            let drift_root = match root_override {
                Some(path) => path
                    .canonicalize()
                    .with_context(|| format!("canonicalize drift root {}", path.display()))?,
                None => root.clone(),
            };
            let report = drift::drift_or_bail(&drift_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci drift: {} ok={}", report.status, report.ok);
            }
            Ok(())
        }
        CiCommand::Fingerprint { input } => {
            let report = fingerprint::fingerprint_from_file(&input)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci fingerprint: {} {}", report.lane, report.fingerprint);
            }
            Ok(())
        }
        CiCommand::Reuse {
            want_reused,
            attestation_root,
            attestation,
            context,
        } => {
            let report = fingerprint::evaluate_reuse(
                &root,
                want_reused,
                attestation_root.as_deref(),
                attestation.as_deref(),
                context.as_deref(),
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci reuse: enabled={} decision={}",
                    report.reuse_enabled, report.decision
                );
            }
            Ok(())
        }
        CiCommand::VerifyRunner {
            class,
            observe_current,
        } => {
            if !observe_current {
                anyhow::bail!(
                    "ci verify-runner: INFRA_FAILURE --observe-current is required; direct workflow observations are forbidden"
                );
            }
            let obs = verify_runner::observe_current(&root, &class)
                .map_err(|error| anyhow::anyhow!("ci verify-runner: INFRA_FAILURE {error:#}"))?;
            let report = verify_runner::verify_runner_or_bail(&root, &obs)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci verify-runner: {}", report.status);
            }
            Ok(())
        }
        CiCommand::Locks {
            root: root_override,
        } => {
            let lock_root = match root_override {
                Some(path) => path
                    .canonicalize()
                    .with_context(|| format!("canonicalize lock root {}", path.display()))?,
                None => root.clone(),
            };
            let report = locks::check_locks_or_bail(&lock_root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci locks: ok msrv={}", report.msrv);
            }
            Ok(())
        }
        CiCommand::Flake {
            today,
            registry_file,
        } => {
            let today = today.map_or_else(flake::utc_today, Ok)?;
            let report = if let Some(rf) = registry_file {
                let raw = fs::read_to_string(&rf)
                    .with_context(|| format!("read flake registry {}", rf.display()))?;
                flake::check_flake_registry_text(&raw, &today)?
            } else {
                flake::check_flake_registry(&root, &today)?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci flake: open={} expired={:?}",
                    report.open_count, report.expired
                );
            }
            if !report.ok {
                bail!("ci flake: expired open flakes {:?}", report.expired);
            }
            Ok(())
        }
        CiCommand::FailureFp {
            lane,
            command,
            error_class,
        } => {
            let report = flake::failure_fingerprint(&lane, &command, &error_class);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci failure-fp: {}", report.fingerprint);
            }
            Ok(())
        }
        CiCommand::EvidenceRoot => {
            let report = evidence_root_check(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci evidence-root: ok={}", report["ok"]);
            }
            if report.get("ok") != Some(&serde_json::Value::Bool(true)) {
                bail!("ci evidence-root: failed");
            }
            Ok(())
        }
        CiCommand::Taxonomy => {
            let report = taxonomy_read(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci taxonomy: profiles={:?}", report.get("profiles"));
            }
            if report.get("ok") != Some(&serde_json::Value::Bool(true)) {
                bail!("ci taxonomy: failed");
            }
            Ok(())
        }
        CiCommand::Determinism { payload } => {
            let report = domain_gates::check_determinism_twice(&payload);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci determinism: {} digest={}",
                    report.status, report.digest_a
                );
            }
            if !report.ok {
                bail!("ci determinism: FAIL");
            }
            Ok(())
        }
        CiCommand::NoLookahead { fixture } => {
            let report = if let Some(f) = fixture {
                domain_gates::no_lookahead_from_fixture(&f)?
            } else {
                // built-in legal sample
                domain_gates::check_no_lookahead(&[domain_gates::TimelineEvent {
                    observed_at: 10,
                    available_at: 10,
                    effective_at: 9,
                    decision_as_of: 10,
                }])
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci no-lookahead: {} violations={:?}",
                    report.status, report.violations
                );
            }
            if !report.ok {
                bail!("ci no-lookahead: FAIL {:?}", report.violations);
            }
            Ok(())
        }
        CiCommand::DomainGates => {
            let report = domain_gates::domain_gate_or_bail(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci domain-gates: ok={} upgrade={} neg={}",
                    report.ok, report.upgrade_path_present, report.negative_fixture_present
                );
            }
            Ok(())
        }
        CiCommand::Autoresearch { write } => {
            let report = autoresearch::generate_shadow_report(&root, write)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "ci autoresearch: shadow={} recs={}",
                    report.shadow,
                    report.recommendations.len()
                );
                for w in &report.written {
                    println!("  wrote: {w}");
                }
            }
            Ok(())
        }
        CiCommand::Chaos => {
            let report = chaos::run_negative_subset(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("ci chaos: ok={} cases={}", report.ok, report.cases.len());
                for c in &report.cases {
                    println!(
                        "  - {} expected_fail_ok={} {}",
                        c.id, c.ok_expected_fail, c.detail
                    );
                }
            }
            if !report.ok {
                bail!("ci chaos: subset failed");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
fn not_implemented_report(command: &'static str) -> StubReport {
    StubReport {
        ok: false,
        command,
        mode: "shadow",
        status: "NOT_IMPLEMENTED",
        note: "command surface reserved by PLAN-CI-SSOT-001; implementation and production activation are not available",
    }
}

/// 校验 baseline 文件存在且包含最小必填键（去注释 + section 路径）。
pub fn verify(root: &Path) -> Result<VerifyReport> {
    let path = root.join(BASELINE_REL);
    if !path.is_file() {
        let mut missing: Vec<String> = REQUIRED_TOP_LEVEL_KEYS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        for (sec, keys) in REQUIRED_SECTIONS {
            missing.push(format!("[{sec}]"));
            for k in *keys {
                missing.push(format!("{sec}.{k}"));
            }
        }
        return Ok(VerifyReport {
            ok: false,
            baseline: path.display().to_string(),
            missing_keys: missing,
            mode: "shadow",
        });
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cleaned = strip_toml_comments(&text);
    let mut missing = verify_baseline_structure(&cleaned);

    // primary/msrv 值应为 1.94.1（MSRV 对齐）— 仅在 toolchains section 内检查
    if let Some(sec) = section_body(&cleaned, "toolchains") {
        if !sec.contains("1.94.1") {
            missing.push("toolchains.primary/msrv=1.94.1".into());
        }
    }
    // reuse.enabled = false
    if let Some(sec) = section_body(&cleaned, "reuse") {
        if key_value_is(sec, "enabled", "true") {
            missing.push("reuse.enabled must be false in skeleton".into());
        }
    }
    // planner unknown → full
    if let Some(sec) = section_body(&cleaned, "planner") {
        if !key_value_is(sec, "unknown_change_behavior", "full") {
            missing.push("planner.unknown_change_behavior=full".into());
        }
    }

    Ok(VerifyReport {
        ok: missing.is_empty(),
        baseline: path.display().to_string(),
        missing_keys: missing,
        mode: "shadow",
    })
}

/// 去掉 TOML 行注释与行尾注释（字符串内的 `#` 保留）。不实现完整 TOML 解析。
fn strip_toml_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out.push('\n');
            continue;
        }
        let mut in_str = false;
        let mut cleaned = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '"' {
                // 简单处理：不支持转义引号嵌套；baseline 字面量足够
                in_str = !in_str;
                cleaned.push(c);
            } else if c == '#' && !in_str {
                break;
            } else {
                cleaned.push(c);
            }
            i += 1;
        }
        out.push_str(cleaned.trim_end());
        out.push('\n');
    }
    out
}

/// 提取 `[name]` section 正文（到下一 section 之前）。
fn section_body<'a>(cleaned: &'a str, name: &str) -> Option<&'a str> {
    let header = format!("[{name}]");
    let start = cleaned.find(&header)?;
    let after = start + header.len();
    let rest = &cleaned[after..];
    let end = rest.find("\n[").map(|i| after + i).unwrap_or(cleaned.len());
    Some(&cleaned[after..end])
}

fn top_level_body(cleaned: &str) -> &str {
    cleaned
        .find("\n[")
        .map(|i| &cleaned[..i])
        .unwrap_or(cleaned)
}

fn section_has_key(section: &str, key: &str) -> bool {
    for line in section.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(key) {
            if rest.trim_start().starts_with('=') {
                return true;
            }
        }
    }
    false
}

fn key_value_is(section: &str, key: &str, want: &str) -> bool {
    for line in section.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(val) = rest.strip_prefix('=') else {
            continue;
        };
        let val = val.trim().trim_matches('"');
        if val == want {
            return true;
        }
    }
    false
}

fn verify_baseline_structure(cleaned: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let top = top_level_body(cleaned);
    for k in REQUIRED_TOP_LEVEL_KEYS {
        if !section_has_key(top, k) {
            missing.push((*k).to_string());
        }
    }
    for (sec, keys) in REQUIRED_SECTIONS {
        match section_body(cleaned, sec) {
            None => {
                missing.push(format!("[{sec}]"));
                for k in *keys {
                    missing.push(format!("{sec}.{k}"));
                }
            }
            Some(body) => {
                for k in *keys {
                    if !section_has_key(body, k) {
                        missing.push(format!("{sec}.{k}"));
                    }
                }
            }
        }
    }
    missing
}

/// 兼容入口：无路径 → unknown → full。
pub fn plan() -> PlanReport {
    plan_from_paths(&[]).expect("empty plan")
}

/// 从 CLI 输入构造 plan。
pub fn plan_from_inputs(paths_file: Option<&Path>, paths: Option<&[String]>) -> Result<PlanReport> {
    let mut all = Vec::new();
    if let Some(list) = paths {
        all.extend(list.iter().cloned());
    }
    if let Some(file) = paths_file {
        let body = fs::read_to_string(file)
            .with_context(|| format!("read paths_file {}", file.display()))?;
        for line in body.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            all.push(t.to_string());
        }
    }
    let mut report = plan_from_paths(&all)?;
    if let Ok(root) = workspace_root() {
        if let Ok(g) = graph::affected_from_paths(&root, &all) {
            report.affected = serde_json::to_value(&g).unwrap_or(serde_json::Value::Null);
            if g.full_fallback && report.change_class != "docs_only" {
                report.note = format!("{}; graph full_fallback", report.note);
            }
        }
    }
    Ok(report)
}

/// 变更分类 + Lane 决策矩阵（Shadow；Recall 安全：不确定 → full）。
pub fn plan_from_paths(paths: &[String]) -> Result<PlanReport> {
    let class = classify_change_paths(paths);
    let change_class = class.as_str().to_string();
    let (lanes, note) = match class {
        ChangeClass::DocsOnly => (
            serde_json::json!({
                "fast": { "decision": "RUN", "reason": "docs_policy_and_naming" },
                "build_test": { "decision": "NOT_APPLICABLE", "reason": "docs_only_no_rust_compile" }
            }),
            "docs-only: skip rust compile/test lane; still run fast policy path",
        ),
        ChangeClass::InfraCi => (
            serde_json::json!({
                "fast": { "decision": "RUN", "reason": "ci_policy" },
                "build_test": { "decision": "RUN", "reason": "ci_or_xtask_may_affect_build" }
            }),
            "infra/ci change: full lanes (safe over-run)",
        ),
        ChangeClass::Manifest => (
            serde_json::json!({
                "fast": { "decision": "RUN", "reason": "manifest" },
                "build_test": { "decision": "RUN", "reason": "manifest_or_lock_requires_full" }
            }),
            "manifest/lock/toolchain: full",
        ),
        ChangeClass::RustCode => (
            serde_json::json!({
                "fast": { "decision": "RUN", "reason": "rust_touched" },
                "build_test": { "decision": "RUN", "reason": "rust_touched" }
            }),
            "rust code change: full build/test (affected graph Phase-1-03 still expands precision)",
        ),
        ChangeClass::Unknown => (
            serde_json::json!({
                "fast": { "decision": "RUN", "reason": "unknown_change_behavior_full" },
                "build_test": { "decision": "RUN", "reason": "unknown_change_behavior_full" }
            }),
            "Shadow plan: no paths or unclassified → unknown → full (HC-03)",
        ),
    };
    let fast_lane = lanes
        .get("fast")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"decision": "RUN"}));
    let shards = match class {
        ChangeClass::DocsOnly => serde_json::json!({
            "compile_once": false,
            "shards": [],
            "reason": "docs_only_skip_rust"
        }),
        _ => serde_json::json!({
            "compile_once": true,
            "order": ["build", "test"],
            "shards": [
                {"id": "shard-0", "packages": "from_affected_or_full", "role": "build"},
                {"id": "shard-0-test", "packages": "same_target", "role": "test", "depends_on": "shard-0"}
            ],
            "note": "shadow shard plan; real cargo partitioning deferred"
        }),
    };
    let special_matrix = serde_json::json!({
        "loom": {"when": "paths_match:loom", "decision": "conditional"},
        "testkit": {"when": "paths_match:testkit", "decision": "conditional"},
        "kafka_real": {"when": "paths_match:kafkax|feature:real", "decision": "conditional"},
        "default": "NOT_APPLICABLE"
    });
    Ok(PlanReport {
        schema_version: 1,
        change_class,
        unknown_change_behavior: "full",
        mode: "shadow",
        note: note.to_string(),
        lanes,
        path_count: paths.len(),
        fast_lane,
        shards,
        special_matrix,
        affected: serde_json::Value::Null,
    })
}

fn evidence_root_check(root: &Path) -> Result<serde_json::Value> {
    let policy = root.join("evidence/ROOT_POLICY.md");
    let schemas = root.join("evidence/schemas/ci");
    let ok = policy.is_file() && schemas.is_dir();
    Ok(serde_json::json!({
        "ok": ok,
        "mode": "shadow",
        "policy_present": policy.is_file(),
        "schemas_present": schemas.is_dir(),
        "note": "evidence/ is sole compliance root; presence≠compliance"
    }))
}

fn taxonomy_read(root: &Path) -> Result<serde_json::Value> {
    let nextest = root.join(".config/nextest.toml");
    let tax = root.join(".config/test-taxonomy.toml");
    let mut profiles = Vec::new();
    if nextest.is_file() {
        for line in fs::read_to_string(&nextest)?.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("[profile.") {
                if let Some(name) = rest.strip_suffix(']') {
                    profiles.push(name.to_string());
                }
            }
        }
    }
    let mut classes = Vec::new();
    if tax.is_file() {
        for line in fs::read_to_string(&tax)?.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("id = ") {
                classes.push(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    let ok = profiles.iter().any(|p| p == "ci" || p.starts_with("ci"));
    Ok(serde_json::json!({
        "ok": ok,
        "mode": "shadow",
        "profiles": profiles,
        "taxonomy_classes": classes,
        "nextest_path": ".config/nextest.toml",
        "taxonomy_path": ".config/test-taxonomy.toml"
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeClass {
    DocsOnly,
    RustCode,
    InfraCi,
    Manifest,
    Unknown,
}

impl ChangeClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::DocsOnly => "docs_only",
            Self::RustCode => "rust_code",
            Self::InfraCi => "infra_ci",
            Self::Manifest => "manifest",
            Self::Unknown => "unknown",
        }
    }
}

fn is_docs_path(p: &str) -> bool {
    let p = p.replace('\\', "/");
    if p.ends_with(".md") || p.ends_with(".mdx") {
        return true;
    }
    p.starts_with("docs/")
        || p.starts_with("doc/")
        || p == "LICENSE"
        || p == "LICENSE-APACHE"
        || p == "LICENSE-MIT"
        || p == "README.md"
        || p.starts_with("README")
}

/// 分类变更路径。空列表 → Unknown（强制 full）。
fn classify_change_paths(paths: &[String]) -> ChangeClass {
    if paths.is_empty() {
        return ChangeClass::Unknown;
    }
    let mut only_docs = true;
    let mut has_manifest = false;
    let mut has_ci = false;
    let mut has_rust = false;
    for raw in paths {
        let p = raw.replace('\\', "/");
        let base = p.rsplit('/').next().unwrap_or(&p);
        if base == "Cargo.toml"
            || base == "Cargo.lock"
            || base == "rust-toolchain.toml"
            || base.starts_with("rust-toolchain")
            || p == "rust-toolchain.toml"
        {
            has_manifest = true;
            only_docs = false;
            continue;
        }
        if p.starts_with(".github/")
            || p.starts_with(".agent/gates/")
            || p.starts_with("tools/xtask/")
            || p.starts_with(".github/ci/")
        {
            has_ci = true;
            only_docs = false;
            continue;
        }
        if is_docs_path(&p) {
            continue;
        }
        only_docs = false;
        if p.ends_with(".rs")
            || p.starts_with("crates/")
            || p.starts_with("apps/")
            || p.starts_with("tools/")
        {
            has_rust = true;
        } else {
            // 未知扩展：Recall 安全 → 当 rust/full
            has_rust = true;
        }
    }
    if has_manifest {
        ChangeClass::Manifest
    } else if only_docs {
        ChangeClass::DocsOnly
    } else if has_ci && !has_rust {
        ChangeClass::InfraCi
    } else if has_rust {
        ChangeClass::RustCode
    } else {
        ChangeClass::Unknown
    }
}

fn baseline_digest(root: &Path) -> Result<String> {
    let path = root.join(BASELINE_REL);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cleaned = strip_toml_comments(&raw);
    Ok(format!("{:x}", Sha256::digest(cleaned.as_bytes())))
}

/// Lane dry-run：把 plan 决策物化为将执行的 lane 列表（不 spawn 进程）。
pub fn run_lanes_dry(root: &Path, plan_file: Option<&Path>) -> Result<RunReport> {
    let plan_report = if let Some(pf) = plan_file {
        let body = fs::read_to_string(pf).with_context(|| format!("read plan {}", pf.display()))?;
        serde_json::from_str::<serde_json::Value>(&body).context("parse plan JSON")?;
        // 宽松：从 JSON 取字段
        let v: serde_json::Value = serde_json::from_str(&body)?;
        PlanReport {
            schema_version: v["schema_version"].as_u64().unwrap_or(1) as u32,
            change_class: v["change_class"].as_str().unwrap_or("unknown").to_string(),
            unknown_change_behavior: "full",
            mode: "shadow",
            note: v["note"].as_str().unwrap_or("from plan_file").to_string(),
            lanes: v["lanes"].clone(),
            path_count: v["path_count"].as_u64().unwrap_or(0) as usize,
            fast_lane: v.get("fast_lane").cloned().unwrap_or(serde_json::json!({})),
            shards: v.get("shards").cloned().unwrap_or(serde_json::json!({})),
            special_matrix: v
                .get("special_matrix")
                .cloned()
                .unwrap_or(serde_json::json!({})),
            affected: v
                .get("affected")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        }
    } else {
        plan()
    };
    // verify baseline so dry-run is grounded
    let v = verify(root)?;
    if !v.ok {
        return Ok(RunReport {
            ok: false,
            mode: "shadow",
            dry_run: true,
            change_class: plan_report.change_class,
            lanes: plan_report.lanes,
            note: "baseline verify failed; refuse dry-run run",
        });
    }
    Ok(RunReport {
        ok: true,
        mode: "shadow",
        dry_run: true,
        change_class: plan_report.change_class,
        lanes: plan_report.lanes,
        note:
            "dry-run only: lane decisions emitted; commands not executed; not production activation",
    })
}

/// Aggregate dry-run：Expected/Actual + missing/unknown fail-closed。
///
/// **fail-closed（PHASE-1-07 / AC-04）**：
/// - 无 `decisions_file` 且未 `synthetic_smoke` → **FAIL**（禁止默认全绿）
/// - `NOT_APPLICABLE` 必须带 reason；`REUSED` 必须带 attestation
/// - `SKIP` / 裸 `UNKNOWN` → FAIL
pub fn aggregate_dry(
    root: &Path,
    decisions_file: Option<&Path>,
    expected: Option<&str>,
    synthetic_smoke: bool,
) -> Result<AggregateReport> {
    aggregate_dry_with_attestation_root(root, decisions_file, expected, synthetic_smoke, None)
}

fn aggregate_dry_with_attestation_root(
    root: &Path,
    decisions_file: Option<&Path>,
    expected: Option<&str>,
    synthetic_smoke: bool,
    attestation_root: Option<&Path>,
) -> Result<AggregateReport> {
    let cleaned = {
        let path = root.join(BASELINE_REL);
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        strip_toml_comments(&raw)
    };
    let digest = format!("{:x}", Sha256::digest(cleaned.as_bytes()));
    let shadow_context = baseline_string_value(&cleaned, "shadow_context")
        .unwrap_or_else(|| "CI / required-shadow".into());
    let requested_expected_lanes: Vec<String> = if let Some(e) = expected {
        e.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        baseline_lane_names(&cleaned)
    };
    let mut expected_seen = HashSet::new();
    let mut duplicate_expected = false;
    let expected_lanes: Vec<String> = requested_expected_lanes
        .into_iter()
        .filter(|lane| {
            let inserted = expected_seen.insert(lane.clone());
            duplicate_expected |= !inserted;
            inserted
        })
        .collect();

    let mut raw_decisions: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut note = "Shadow structural validation; N/A is not Plan-bound; REUSED reference schema/basic semantics are validated but full eligibility and rollout remain disabled; NOT GitHub Ruleset required";
    let mut force_fail = duplicate_expected;

    if let Some(df) = decisions_file {
        let body = fs::read_to_string(df).with_context(|| format!("read {}", df.display()))?;
        let v: serde_json::Value = serde_json::from_str(&body).context("parse decisions JSON")?;
        let obj = v
            .as_object()
            .with_context(|| "decisions_file must be a JSON object of lane→decision")?;
        for (k, val) in obj {
            raw_decisions.insert(k.clone(), val.clone());
        }
    } else if synthetic_smoke {
        // 仅显式 --synthetic-smoke：本地 smoke，不代表裁决
        for lane in &expected_lanes {
            raw_decisions.insert(lane.clone(), serde_json::Value::String("RUN_PASS".into()));
        }
        note = "SYNTHETIC_SMOKE: invented RUN_PASS for expected lanes; NOT adjudicating; not for shadow required context";
    } else {
        // 禁止无 decisions_file 默认全绿
        force_fail = true;
        note = "FAIL-CLOSED: decisions_file required (no default all-green); use --synthetic-smoke only for local smoke";
    }

    let actual_lanes: Vec<String> = raw_decisions.keys().cloned().collect();
    let mut missing = Vec::new();
    for e in &expected_lanes {
        if !raw_decisions.contains_key(e) {
            missing.push(e.clone());
        }
    }
    let mut unexpected = Vec::new();
    for a in &actual_lanes {
        if !expected_lanes.iter().any(|e| e == a) {
            unexpected.push(a.clone());
        }
    }

    force_fail |= !missing.is_empty() || !unexpected.is_empty();
    let mut infra_failure = false;
    let mut decisions = serde_json::Map::new();
    let mut not_applicable_reasons = serde_json::Map::new();
    let mut reused_sources = serde_json::Map::new();
    for (lane, raw) in &raw_decisions {
        let mut parsed = parse_lane_decision(raw);
        if !VALID_LANE_DECISIONS.contains(&parsed.decision.as_str()) {
            parsed = ParsedLaneDecision::invalid("unknown_or_invalid_decision");
        }

        let mut normalized_reason = parsed.reason.clone();
        let mut normalized_attestation = None;
        match parsed.decision.as_str() {
            "RUN_PASS" => {}
            "NOT_APPLICABLE" => {
                let reason = parsed.reason.as_deref().map(str::trim).unwrap_or("");
                if reason.is_empty() {
                    force_fail = true;
                    normalized_reason = Some("missing_not_applicable_reason".into());
                } else {
                    normalized_reason = Some(reason.to_string());
                    not_applicable_reasons
                        .insert(lane.clone(), serde_json::Value::String(reason.to_string()));
                }
            }
            "REUSED" => {
                // Aggregate 无当前 observation context，永远不得把候选升级为 REUSED。
                force_fail = true;
                let validation = fingerprint::validate_reuse_reference(
                    root,
                    attestation_root,
                    parsed.attestation.as_deref(),
                    lane,
                    None,
                );
                if let (true, Some(path), Some(digest)) = (
                    validation.reference_valid,
                    validation.source_path,
                    validation.source_digest,
                ) {
                    normalized_attestation = Some(digest.clone());
                    normalized_reason =
                        Some("reuse_reference_validated_activation_disabled".into());
                    reused_sources.insert(
                        lane.clone(),
                        serde_json::json!({"path": path, "digest": digest}),
                    );
                } else {
                    normalized_reason = validation.reasons.first().cloned();
                }
            }
            "INFRA_FAILURE" => infra_failure = true,
            "RUN_FAIL" | "MISSING" | "UNEXPECTED_SKIP" | "CANCELLED" | "UNKNOWN" => {
                force_fail = true;
            }
            _ => unreachable!("decision allowlist checked above"),
        }
        decisions.insert(
            lane.clone(),
            serde_json::json!({
                "decision": parsed.decision,
                "reason": normalized_reason,
                "attestation": normalized_attestation,
            }),
        );
    }
    // Empty expected → fail-closed (empty plan)
    if expected_lanes.is_empty() {
        force_fail = true;
    }

    let final_decision = if infra_failure {
        "INFRA_FAILURE"
    } else if force_fail {
        "FAIL"
    } else {
        "PASS"
    };
    let report = AggregateReport {
        ok: final_decision == "PASS",
        mode: "shadow",
        schema_version: 2,
        kind: "ci-aggregate-decision",
        run_id: "dry-run-local".into(),
        expected_lanes,
        actual_lanes,
        decisions,
        missing_lanes: missing,
        unexpected_lanes: unexpected,
        not_applicable_reasons,
        reused_sources,
        final_decision: final_decision.into(),
        baseline_digest: format!("sha256:{digest}"),
        ruleset_context: shadow_context,
        note,
    };
    validate_aggregate_report_schema(root, &report)?;
    Ok(report)
}

const VALID_LANE_DECISIONS: &[&str] = &[
    "RUN_PASS",
    "RUN_FAIL",
    "NOT_APPLICABLE",
    "REUSED",
    "INFRA_FAILURE",
    "MISSING",
    "UNEXPECTED_SKIP",
    "CANCELLED",
    "UNKNOWN",
];

/// 解析 lane 决策输入；任何未知字段或类型漂移均归一为 UNKNOWN。
#[derive(Debug)]
struct ParsedLaneDecision {
    decision: String,
    reason: Option<String>,
    attestation: Option<String>,
}

impl ParsedLaneDecision {
    fn invalid(reason: &str) -> Self {
        Self {
            decision: "UNKNOWN".into(),
            reason: Some(reason.into()),
            attestation: None,
        }
    }
}

fn parse_lane_decision(val: &serde_json::Value) -> ParsedLaneDecision {
    if let Some(s) = val.as_str() {
        return ParsedLaneDecision {
            decision: s.to_string(),
            reason: None,
            attestation: None,
        };
    }
    if let Some(obj) = val.as_object() {
        let allowed = ["decision", "reason", "attestation", "attestation_file"];
        if obj.keys().any(|key| !allowed.contains(&key.as_str())) {
            return ParsedLaneDecision::invalid("invalid_decision_shape");
        }
        let Some(decision) = obj.get("decision").and_then(|value| value.as_str()) else {
            return ParsedLaneDecision::invalid("invalid_decision_shape");
        };
        let reason = match obj.get("reason") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str() {
                Some(reason) => Some(reason.to_string()),
                None => return ParsedLaneDecision::invalid("invalid_reason_type"),
            },
            None => None,
        };
        if obj.contains_key("attestation") && obj.contains_key("attestation_file") {
            return ParsedLaneDecision::invalid("ambiguous_attestation_reference");
        }
        let attestation = match obj
            .get("attestation")
            .or_else(|| obj.get("attestation_file"))
        {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str() {
                Some(reference) => Some(reference.to_string()),
                None => return ParsedLaneDecision::invalid("invalid_attestation_type"),
            },
            None => None,
        };
        return ParsedLaneDecision {
            decision: decision.to_string(),
            reason,
            attestation,
        };
    }
    ParsedLaneDecision::invalid("invalid_decision_type")
}

fn validate_aggregate_report_schema(root: &Path, report: &AggregateReport) -> Result<()> {
    let schema_path = root.join(AGGREGATE_DECISION_SCHEMA_REL);
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&schema_path)
            .with_context(|| format!("read aggregate schema {}", schema_path.display()))?,
    )
    .context("parse aggregate decision schema")?;
    let value = serde_json::to_value(report).context("serialize aggregate report")?;
    if !crate::schema_lite::json_schema_matches(&value, &schema) {
        bail!("ci aggregate: produced report does not match aggregate-decision schema");
    }
    Ok(())
}

/// 只读 Reconcile：Desired（baseline 投影）vs Observed。
pub fn reconcile_dry(root: &Path, observed_file: Option<&Path>) -> Result<ReconcileReport> {
    let v = verify(root)?;
    let cleaned = {
        let path = root.join(BASELINE_REL);
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        strip_toml_comments(&raw)
    };
    let digest = format!("{:x}", Sha256::digest(cleaned.as_bytes()));
    let desired = serde_json::json!({
        "baseline_rel": BASELINE_REL,
        "baseline_digest_sha256": digest,
        "required_context": baseline_string_value(&cleaned, "required_context"),
        "shadow_context": baseline_string_value(&cleaned, "shadow_context"),
        "lanes": baseline_lane_names(&cleaned),
    });
    let observed = if let Some(of) = observed_file {
        let body = fs::read_to_string(of).with_context(|| format!("read {}", of.display()))?;
        serde_json::from_str(&body).context("parse observed JSON")?
    } else {
        // synthetic MATCH：与 desired 对齐的观察面（无外部 API）
        desired.clone()
    };
    let mut drifts = Vec::new();
    if desired["baseline_digest_sha256"] != observed["baseline_digest_sha256"] {
        drifts.push("baseline_digest_sha256".into());
    }
    if desired["shadow_context"] != observed["shadow_context"] {
        drifts.push("shadow_context".into());
    }
    if desired["required_context"] != observed["required_context"] {
        drifts.push("required_context".into());
    }
    if desired["lanes"] != observed["lanes"] {
        drifts.push("lanes".into());
    }
    let status = if drifts.is_empty() {
        "MATCH".to_string()
    } else {
        "DRIFT".to_string()
    };
    Ok(ReconcileReport {
        ok: drifts.is_empty() && v.ok,
        mode: "shadow",
        status,
        desired,
        observed,
        drifts,
        note: "read-only reconcile dry-run; never applies Ruleset/Environment; external API observe is HUMAN/Phase-3",
    })
}

/// Metrics 最小事件流（identity/queue/lane/failure）。
pub fn metrics_events(
    root: &Path,
    write_sample: bool,
    weekly_template: bool,
) -> Result<MetricsReport> {
    let digest = baseline_digest(root).unwrap_or_else(|_| "unknown".into());
    let events = vec![
        serde_json::json!({
            "schema_version": 1,
            "type": "ci-metrics-event",
            "event_class": "runner_identity",
            "run_id": "dry-run-local",
            "ts": "1970-01-01T00:00:00Z",
            "payload": {
                "runner_class": "fast",
                "trust_domain": "pr-untrusted",
                "image_digest": "sha256:PLACEHOLDER_FAST_IMAGE_DIGEST"
            }
        }),
        serde_json::json!({
            "schema_version": 1,
            "type": "ci-metrics-event",
            "event_class": "queue",
            "run_id": "dry-run-local",
            "ts": "1970-01-01T00:00:00Z",
            "payload": {
                "queue_seconds": 0,
                "baseline_digest_sha256": digest
            }
        }),
        serde_json::json!({
            "schema_version": 1,
            "type": "ci-metrics-event",
            "event_class": "lane_duration",
            "run_id": "dry-run-local",
            "ts": "1970-01-01T00:00:00Z",
            "payload": {
                "lane": "fast",
                "duration_seconds": 0,
                "decision": "RUN_PASS"
            }
        }),
        serde_json::json!({
            "schema_version": 1,
            "type": "ci-metrics-event",
            "event_class": "failure_class",
            "run_id": "dry-run-local",
            "ts": "1970-01-01T00:00:00Z",
            "payload": {
                "lane": "build_test",
                "failure_class": "none",
                "fingerprint": "sha256:PLACEHOLDER_FAILURE_FP"
            }
        }),
    ];
    let mut written = Vec::new();
    if write_sample {
        let dir = root.join("evidence/ci/samples");
        fs::create_dir_all(&dir).context("create evidence/ci/samples")?;
        let out = dir.join("metrics-events.sample.jsonl");
        let mut body = String::new();
        for e in &events {
            body.push_str(&serde_json::to_string(e)?);
            body.push('\n');
        }
        fs::write(&out, body).context("write metrics sample")?;
        written.push(out.display().to_string());
    }
    if weekly_template {
        let cadence = root.join("evidence/ci/templates/cadence.toml");
        let weekly = root.join("evidence/ci/templates/metrics-weekly.md");
        if !cadence.is_file() || !weekly.is_file() {
            return Ok(MetricsReport {
                ok: false,
                mode: "shadow",
                events,
                written,
                note: "weekly templates missing under evidence/ci/templates",
            });
        }
        written.push(cadence.display().to_string());
        written.push(weekly.display().to_string());
    }
    // schema 存在性
    let schema_ok = root.join(METRICS_EVENT_SCHEMA_REL).is_file();
    Ok(MetricsReport {
        ok: schema_ok,
        mode: "shadow",
        events,
        written,
        note: if schema_ok {
            "metrics dry-run sample events; not production telemetry pipeline"
        } else {
            "metrics schema missing"
        },
    })
}

/// 先 verify，再声明完整本地对等仍为过渡。
pub fn local(root: &Path) -> Result<LocalReport> {
    let v = verify(root)?;
    Ok(LocalReport {
        ok: v.ok,
        mode: "shadow",
        verify_ok: v.ok,
        message: "full local parity is transitional; use existing `just ci` for fmt/clippy/build/test; this command does not reimplement just ci",
    })
}

/// 从 baseline 渲染 Shadow 投影到 `.github/ci/generated/`（dry-run 合同，不 apply Ruleset）。
pub fn render(root: &Path) -> Result<RenderReport> {
    render_to(root, &root.join(".github/ci/generated"))
}

/// 渲染到指定目录（测试注入临时目录，避免覆盖仓库 tracked generated）。
pub fn render_to(root: &Path, out_dir: &Path) -> Result<RenderReport> {
    let v = verify(root)?;
    if !v.ok {
        return Ok(RenderReport {
            ok: false,
            mode: "shadow",
            baseline_digest: String::new(),
            written: vec![],
            note: "baseline verify failed; refuse to render",
        });
    }
    let baseline_path = root.join(BASELINE_REL);
    let raw = fs::read_to_string(&baseline_path)
        .with_context(|| format!("read {}", baseline_path.display()))?;
    let cleaned = strip_toml_comments(&raw);
    let digest = format!("{:x}", Sha256::digest(cleaned.as_bytes()));
    let lanes = baseline_lane_names(&cleaned);
    let required_context = baseline_string_value(&cleaned, "required_context")
        .unwrap_or_else(|| "CI / required".into());
    let shadow_context = baseline_string_value(&cleaned, "shadow_context")
        .unwrap_or_else(|| "CI / required-shadow".into());
    fs::create_dir_all(out_dir).with_context(|| format!("create {}", out_dir.display()))?;

    let contract = serde_json::json!({
        "schema_version": 1,
        "mode": "shadow",
        "type": "workflow-contract",
        "baseline_rel": BASELINE_REL,
        "baseline_digest_sha256": digest,
        "required_context": required_context,
        "shadow_context": shadow_context,
        "note": "Shadow dry-run only; NOT applied to GitHub Ruleset; hand-edit forbidden",
        "lanes": lanes,
        "aggregate_context": shadow_context,
    });
    let contract_path = out_dir.join("workflow-contract.json");
    fs::write(
        &contract_path,
        format!("{}\n", serde_json::to_string_pretty(&contract)?),
    )
    .context("write workflow-contract.json")?;

    let mut policy_rows = String::from(
        "| Context | Role | Cadence |\n|---------|------|---------|\n\
         | CI / required-shadow | Aggregate Shadow 裁决（观察） | PR / merge_group / main（未来） |\n\
         | CI / required | 最终 Required（HUMAN 切换） | 未 apply |\n",
    );
    for lane in &lanes {
        policy_rows.push_str(&format!("| {lane} | baseline lane | plan 决定 |\n"));
    }
    let policy_table = format!(
        r#"# CI Policy Table (generated · Shadow)

> **禁止手改**。由 `cargo xtl ci render`（或 `cargo run -p xhyper-xtask -- ci render`）从 `{BASELINE_REL}` 生成。
> baseline_digest_sha256 = `{digest}`
> **≠** Production Ready / Ruleset 已切主。

{policy_rows}
生成时间：机器 render；不代表外部控制面已 Reconcile。
"#
    );
    let policy_path = out_dir.join("policy-table.md");
    fs::write(&policy_path, policy_table).context("write policy-table.md")?;

    Ok(RenderReport {
        ok: true,
        mode: "shadow",
        baseline_digest: digest,
        written: vec![
            contract_path.display().to_string(),
            policy_path.display().to_string(),
        ],
        note: "Shadow render dry-run; generated files must not be hand-edited; Ruleset not applied",
    })
}

/// 读取 baseline 清理文本中 `key = "value"`（顶层或任意 section；取首次）。
fn baseline_string_value(cleaned: &str, key: &str) -> Option<String> {
    for line in cleaned.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let val = rest.trim().trim_matches('"').trim();
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    None
}

/// 从 baseline 提取 `[lanes.<name>]`（不含 aggregate 裁决 context）。
fn baseline_lane_names(cleaned: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in cleaned.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("[lanes.") {
            if let Some(name) = rest.strip_suffix(']') {
                let name = name.trim();
                if !name.is_empty() && !names.iter().any(|n| n == name) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

fn emit_render(json: bool, report: &RenderReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else if report.ok {
        println!(
            "ci render: PASS (shadow) baseline_digest={}",
            report.baseline_digest
        );
        for w in &report.written {
            println!("  wrote: {w}");
        }
        println!("ci render: {}", report.note);
    } else {
        println!("ci render: FAIL — {}", report.note);
    }
    Ok(())
}

/// 就绪度检查。
pub fn doctor(root: &Path) -> Result<DoctorReport> {
    let baseline_present = root.join(BASELINE_REL).is_file();
    let schema_present = root.join(SCHEMA_REL).is_file();
    let evidence_schemas_present = root.join(LANE_ATTESTATION_SCHEMA_REL).is_file()
        && root.join(AGGREGATE_DECISION_SCHEMA_REL).is_file();
    // 若本模块已编译运行，则 xtask ci 已接线
    let xtask_ci_wired = true;
    let justfile = root.join("justfile");
    let just_text = if justfile.is_file() {
        fs::read_to_string(&justfile).unwrap_or_default()
    } else {
        String::new()
    };
    let just_recipes_present = just_text.contains("ci-plan")
        && just_text.contains("ci-verify")
        && just_text.contains("ci-doctor");

    let mut details = Vec::new();
    if baseline_present {
        details.push(format!("baseline: present ({BASELINE_REL})"));
    } else {
        details.push(format!("baseline: MISSING ({BASELINE_REL})"));
    }
    if schema_present {
        details.push(format!("schema: present ({SCHEMA_REL})"));
    } else {
        details.push(format!("schema: MISSING ({SCHEMA_REL})"));
    }
    if evidence_schemas_present {
        details.push("evidence schemas: lane + aggregate present".into());
    } else {
        details.push("evidence schemas: MISSING lane or aggregate schema".into());
    }
    details.push("xtask ci: wired (this binary)".into());
    if just_recipes_present {
        details.push("just recipes: ci-plan / ci-verify / ci-doctor present".into());
    } else {
        details.push("just recipes: MISSING ci-plan/ci-verify/ci-doctor".into());
    }
    details.push("mode: shadow (not production ready)".into());

    let ok = baseline_present
        && schema_present
        && evidence_schemas_present
        && xtask_ci_wired
        && just_recipes_present;
    Ok(DoctorReport {
        ok,
        mode: "shadow",
        baseline_present,
        schema_present,
        evidence_schemas_present,
        xtask_ci_wired,
        just_recipes_present,
        details,
    })
}

fn emit_verify(json: bool, report: &VerifyReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else if report.ok {
        println!("ci verify: PASS (shadow) baseline={}", report.baseline);
    } else {
        println!("ci verify: FAIL (shadow) baseline={}", report.baseline);
        for k in &report.missing_keys {
            println!("  missing: {k}");
        }
    }
    Ok(())
}

fn emit_plan(json: bool, report: &PlanReport) -> Result<()> {
    // plan 始终以 JSON 为主合同；非 json 模式也打印可读 JSON 骨架
    let pretty = serde_json::to_string_pretty(report)?;
    if json {
        println!("{pretty}");
    } else {
        println!(
            "ci plan: Shadow plan change_class={} path_count={}",
            report.change_class, report.path_count
        );
        println!("{pretty}");
    }
    Ok(())
}

fn emit_local(json: bool, report: &LocalReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "ci local: verify_ok={} mode={}",
            report.verify_ok, report.mode
        );
        println!("ci local: {}", report.message);
    }
    Ok(())
}

fn emit_doctor(json: bool, report: &DoctorReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!("ci doctor: mode={}", report.mode);
        for d in &report.details {
            println!("  - {d}");
        }
        if report.ok {
            println!("ci doctor: PASS (skeleton ready; not production activation)");
        } else {
            println!("ci doctor: FAIL");
        }
    }
    Ok(())
}

fn emit_run(json: bool, report: &RunReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "ci run: dry_run={} change_class={} ok={}",
            report.dry_run, report.change_class, report.ok
        );
        println!("{}", serde_json::to_string_pretty(&report.lanes)?);
        println!("ci run: {}", report.note);
    }
    Ok(())
}

fn emit_aggregate(json: bool, report: &AggregateReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "ci aggregate: final_decision={} missing={} unexpected={}",
            report.final_decision,
            report.missing_lanes.len(),
            report.unexpected_lanes.len()
        );
        println!("{}", serde_json::to_string_pretty(report)?);
        println!("ci aggregate: {}", report.note);
    }
    Ok(())
}

fn emit_reconcile(json: bool, report: &ReconcileReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "ci reconcile: status={} drifts={:?}",
            report.status, report.drifts
        );
        println!("ci reconcile: {}", report.note);
    }
    Ok(())
}

fn emit_metrics(json: bool, report: &MetricsReport) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "ci metrics: events={} ok={}",
            report.events.len(),
            report.ok
        );
        for w in &report.written {
            println!("  wrote: {w}");
        }
        println!("ci metrics: {}", report.note);
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    // 优先 cargo metadata；测试与无 cargo 场景回退 CARGO_MANIFEST_DIR
    if let Ok(meta) = cargo_metadata::MetadataCommand::new().no_deps().exec() {
        return Ok(meta.workspace_root.into_std_path_buf());
    }
    Ok(repo_root_from_manifest())
}

/// 自 `tools/xtask` 的 CARGO_MANIFEST_DIR 上溯仓库根。
pub fn repo_root_from_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools/")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_aggregate_schema_valid(root: &Path, report: &AggregateReport) {
        let schema: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(AGGREGATE_DECISION_SCHEMA_REL))
                .expect("read aggregate schema"),
        )
        .expect("parse aggregate schema");
        let value = serde_json::to_value(report).expect("serialize aggregate report");
        assert!(
            crate::schema_lite::json_schema_matches(&value, &schema),
            "aggregate report must match shipped schema: {value}"
        );
    }

    fn valid_lane_attestation(lane: &str, decision: &str) -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1,
            "type": "ci-lane-attestation",
            "run_id": "source-run",
            "lane": lane,
            "decision": decision,
            "base_sha": "a".repeat(40),
            "head_sha": "b".repeat(40),
            "plan_digest": format!("sha256:{}", "c".repeat(64)),
            "fingerprint": format!("sha256:{}", "d".repeat(64)),
            "runner_class": "fast",
            "runner_image_digest": format!("sha256:{}", "e".repeat(64)),
            "toolchain_digest": format!("sha256:{}", "f".repeat(64)),
            "started_at": "2026-07-17T00:00:00Z",
            "finished_at": "2026-07-17T00:01:00Z",
            "result_digest": format!("sha256:{}", "1".repeat(64))
        })
    }

    #[test]
    fn verify_succeeds_against_real_baseline() {
        let root = repo_root_from_manifest();
        let report = verify(&root).expect("verify runs");
        assert!(
            report.ok,
            "verify must succeed on real baseline; missing={:?}",
            report.missing_keys
        );
        assert!(
            root.join(BASELINE_REL).is_file(),
            "baseline path must exist under {:?}",
            root
        );
    }

    #[test]
    fn plan_json_contains_schema_version() {
        let report = plan();
        let v = serde_json::to_value(&report).expect("serialize plan");
        assert_eq!(v["schema_version"], 1);
        assert!(
            v.get("schema_version").is_some(),
            "plan must contain schema_version"
        );
        assert_eq!(v["lanes"]["fast"]["decision"], "RUN");
        assert_eq!(v["change_class"], "unknown");
        let s = serde_json::to_string(&report).unwrap();
        assert!(
            s.contains("schema_version"),
            "serialized plan has schema_version"
        );
    }

    #[test]
    fn planner_docs_only_skips_build_test() {
        let report = plan_from_paths(&["docs/architecture/spec.md".into(), "README.md".into()])
            .expect("plan");
        assert_eq!(report.change_class, "docs_only");
        assert_eq!(report.lanes["fast"]["decision"], "RUN");
        assert_eq!(report.lanes["build_test"]["decision"], "NOT_APPLICABLE");
    }

    #[test]
    fn planner_manifest_forces_full() {
        let report = plan_from_paths(&["Cargo.lock".into(), "docs/x.md".into()]).expect("plan");
        assert_eq!(report.change_class, "manifest");
        assert_eq!(report.lanes["build_test"]["decision"], "RUN");
    }

    #[test]
    fn planner_empty_paths_unknown_full() {
        let report = plan_from_paths(&[]).expect("plan");
        assert_eq!(report.change_class, "unknown");
        assert_eq!(report.lanes["build_test"]["decision"], "RUN");
    }

    #[test]
    fn aggregate_missing_lane_fails() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        // only fast; build_test missing
        fs::write(&dec, r#"{"fast":"RUN_PASS"}"#).unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "FAIL");
        assert!(report.missing_lanes.iter().any(|l| l == "build_test"));
    }

    #[test]
    fn aggregate_without_decisions_file_fails_closed() {
        let root = repo_root_from_manifest();
        let report = aggregate_dry(&root, None, None, false).expect("agg");
        assert_eq!(report.final_decision, "FAIL", "{report:?}");
        assert!(!report.ok);
        assert!(
            report.note.contains("decisions_file") || report.note.contains("FAIL-CLOSED"),
            "must explain fail-closed: {}",
            report.note
        );
    }

    #[test]
    fn aggregate_synthetic_smoke_pass() {
        let root = repo_root_from_manifest();
        let report = aggregate_dry(&root, None, None, true).expect("agg");
        assert_eq!(report.final_decision, "PASS", "{report:?}");
        assert!(report.missing_lanes.is_empty());
        assert!(
            report.note.contains("SYNTHETIC_SMOKE"),
            "must mark synthetic: {}",
            report.note
        );
    }

    #[test]
    fn aggregate_all_pass_with_decisions_file() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        fs::write(&dec, r#"{"fast":"RUN_PASS","build_test":"RUN_PASS"}"#).unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "PASS", "{report:?}");
        assert!(report.missing_lanes.is_empty());
        assert!(report.decisions["fast"].is_object());
        assert_eq!(report.decisions["fast"]["decision"], "RUN_PASS");
        assert_aggregate_schema_valid(&root, &report);
    }

    #[test]
    fn aggregate_v2_decisions_roundtrip_preserves_pass_and_na() {
        let root = repo_root_from_manifest();
        for original in [
            serde_json::json!({"fast": "RUN_PASS", "build_test": "RUN_PASS"}),
            serde_json::json!({
                "fast": "RUN_PASS",
                "build_test": {"decision": "NOT_APPLICABLE", "reason": "docs_only"}
            }),
        ] {
            let tmp = tempfile::tempdir().expect("tmp");
            let first_input = tmp.path().join("first.json");
            fs::write(&first_input, serde_json::to_vec(&original).unwrap()).unwrap();
            let first =
                aggregate_dry(&root, Some(&first_input), Some("fast,build_test"), false).unwrap();
            assert_eq!(first.final_decision, "PASS", "{first:?}");

            let second_input = tmp.path().join("second.json");
            fs::write(&second_input, serde_json::to_vec(&first.decisions).unwrap()).unwrap();
            let second =
                aggregate_dry(&root, Some(&second_input), Some("fast,build_test"), false).unwrap();
            assert_eq!(second.final_decision, "PASS", "{second:?}");
            assert_eq!(second.decisions, first.decisions);
        }
    }

    #[test]
    fn aggregate_v2_nullable_reused_fields_stay_reused_but_fail_closed() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("decisions.json");
        fs::write(
            &dec,
            r#"{
                "fast":{"decision":"REUSED","reason":null,"attestation":null},
                "build_test":{"decision":"RUN_PASS","reason":null,"attestation":null}
            }"#,
        )
        .unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.decisions["fast"]["decision"], "REUSED");
        assert_eq!(report.final_decision, "FAIL");
        assert_aggregate_schema_valid(&root, &report);
    }

    #[test]
    fn aggregate_invalid_na_without_reason_fails() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        // bare NOT_APPLICABLE string has no reason
        fs::write(&dec, r#"{"fast":"RUN_PASS","build_test":"NOT_APPLICABLE"}"#).unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "FAIL", "{report:?}");
    }

    #[test]
    fn aggregate_na_with_reason_ok() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        fs::write(
            &dec,
            r#"{"fast":"RUN_PASS","build_test":{"decision":"NOT_APPLICABLE","reason":"docs_only"}}"#,
        )
        .unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "PASS", "{report:?}");
        assert_aggregate_schema_valid(&root, &report);
    }

    #[test]
    fn aggregate_unknown_and_type_drift_fail_with_schema_valid_reports() {
        let root = repo_root_from_manifest();
        for body in [
            r#"{"fast":"TYPO_PASS","build_test":"RUN_PASS"}"#,
            r#"{"fast":42,"build_test":"RUN_PASS"}"#,
            r#"{"fast":null,"build_test":"RUN_PASS"}"#,
            r#"{"fast":[],"build_test":"RUN_PASS"}"#,
            r#"{"fast":{"reason":"missing decision"},"build_test":"RUN_PASS"}"#,
            r#"{"fast":{"decision":"RUN_PASS","extra":true},"build_test":"RUN_PASS"}"#,
        ] {
            let tmp = tempfile::tempdir().expect("tmp");
            let dec = tmp.path().join("dec.json");
            fs::write(&dec, body).unwrap();
            let report =
                aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
            assert_eq!(
                report.final_decision, "FAIL",
                "body={body} report={report:?}"
            );
            assert_eq!(report.decisions["fast"]["decision"], "UNKNOWN");
            assert_aggregate_schema_valid(&root, &report);
        }
    }

    #[test]
    fn aggregate_na_rejects_blank_reason() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        fs::write(
            &dec,
            r#"{"fast":"RUN_PASS","build_test":{"decision":"NOT_APPLICABLE","reason":"   "}}"#,
        )
        .unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL", "{report:?}");
        assert_aggregate_schema_valid(&root, &report);
    }

    #[test]
    fn aggregate_reused_reference_is_validated_but_full_eligibility_stays_disabled() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        fs::write(
            tmp.path().join("fast.json"),
            serde_json::to_vec(&valid_lane_attestation("fast", "RUN_PASS")).unwrap(),
        )
        .unwrap();
        let dec = tmp.path().join("dec.json");
        fs::write(
            &dec,
            r#"{"fast":{"decision":"REUSED","attestation":"fast.json"},"build_test":"RUN_PASS"}"#,
        )
        .unwrap();
        let report = aggregate_dry_with_attestation_root(
            &root,
            Some(&dec),
            Some("fast,build_test"),
            false,
            Some(tmp.path()),
        )
        .expect("aggregate");
        assert_eq!(report.final_decision, "FAIL", "reuse remains disabled");
        assert!(report.decisions["fast"]["attestation"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:")));
        assert_eq!(
            report.decisions["fast"]["reason"],
            "reuse_reference_validated_activation_disabled"
        );
        assert_aggregate_schema_valid(&root, &report);
    }

    #[test]
    fn aggregate_reused_rejects_untrusted_paths_and_symlink_escape() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let outside = tempfile::tempdir().expect("outside");
        fs::write(
            outside.path().join("outside.json"),
            serde_json::to_vec(&valid_lane_attestation("fast", "RUN_PASS")).unwrap(),
        )
        .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            outside.path().join("outside.json"),
            tmp.path().join("escape.json"),
        )
        .unwrap();
        let absolute = outside.path().join("outside.json");
        let references = [
            absolute.to_string_lossy().into_owned(),
            "../outside.json".into(),
            "escape.json".into(),
        ];
        for reference in references {
            let dec = tmp.path().join("dec.json");
            fs::write(
                &dec,
                serde_json::to_vec(&serde_json::json!({
                    "fast": {"decision": "REUSED", "attestation": reference},
                    "build_test": "RUN_PASS"
                }))
                .unwrap(),
            )
            .unwrap();
            let report = aggregate_dry_with_attestation_root(
                &root,
                Some(&dec),
                Some("fast,build_test"),
                false,
                Some(tmp.path()),
            )
            .unwrap();
            assert_eq!(report.final_decision, "FAIL", "{report:?}");
            assert!(report.decisions["fast"]["attestation"].is_null());
            assert_aggregate_schema_valid(&root, &report);
        }
    }

    #[test]
    fn aggregate_reused_rejects_invalid_attestation_semantics() {
        let root = repo_root_from_manifest();
        for (name, attestation) in [
            ("lane-mismatch", valid_lane_attestation("other", "RUN_PASS")),
            ("not-pass", valid_lane_attestation("fast", "RUN_FAIL")),
            ("placeholder", {
                let mut value = valid_lane_attestation("fast", "RUN_PASS");
                value["fingerprint"] = serde_json::json!("sha256:PLACEHOLDER_FP");
                value
            }),
            ("bad-schema", serde_json::json!({"lane": "fast"})),
        ] {
            let tmp = tempfile::tempdir().expect("tmp");
            fs::write(
                tmp.path().join("attestation.json"),
                serde_json::to_vec(&attestation).unwrap(),
            )
            .unwrap();
            let dec = tmp.path().join("dec.json");
            fs::write(
                &dec,
                r#"{"fast":{"decision":"REUSED","attestation":"attestation.json"},"build_test":"RUN_PASS"}"#,
            )
            .unwrap();
            let report = aggregate_dry_with_attestation_root(
                &root,
                Some(&dec),
                Some("fast,build_test"),
                false,
                Some(tmp.path()),
            )
            .unwrap();
            assert_eq!(report.final_decision, "FAIL", "case={name} {report:?}");
            assert!(report.decisions["fast"]["attestation"].is_null());
            assert_aggregate_schema_valid(&root, &report);
        }
    }

    #[test]
    fn aggregate_reused_without_attestation_fails() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        fs::write(&dec, r#"{"fast":"REUSED","build_test":"RUN_PASS"}"#).unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "FAIL", "{report:?}");
    }

    #[test]
    fn aggregate_unexpected_skip_fails() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tmp");
        let dec = tmp.path().join("dec.json");
        fs::write(&dec, r#"{"fast":"SKIP","build_test":"RUN_PASS"}"#).unwrap();
        let report = aggregate_dry(&root, Some(&dec), Some("fast,build_test"), false).expect("agg");
        assert_eq!(report.final_decision, "FAIL", "{report:?}");
    }

    #[test]
    fn run_dry_ok() {
        let root = repo_root_from_manifest();
        let report = run_lanes_dry(&root, None).expect("run");
        assert!(report.ok);
        assert!(report.dry_run);
    }

    #[test]
    fn reconcile_match_synthetic() {
        let root = repo_root_from_manifest();
        let report = reconcile_dry(&root, None).expect("reconcile");
        assert_eq!(report.status, "MATCH");
        assert!(report.ok);
    }

    #[test]
    fn metrics_schema_and_events() {
        let root = repo_root_from_manifest();
        let report = metrics_events(&root, false, false).expect("metrics");
        assert!(report.ok, "metrics schema must exist");
        assert_eq!(report.events.len(), 4);
        assert_eq!(report.events[0]["event_class"], "runner_identity");
    }

    #[test]
    fn doctor_detects_baseline_file() {
        let root = repo_root_from_manifest();
        let report = doctor(&root).expect("doctor runs");
        assert!(
            report.baseline_present,
            "doctor must detect real baseline at {}",
            root.join(BASELINE_REL).display()
        );
        assert!(report.schema_present, "schema.json must be present");
        assert!(
            report.evidence_schemas_present,
            "lane and aggregate evidence schemas must both be present"
        );
        assert!(report.xtask_ci_wired);
    }

    #[test]
    fn aggregate_schema_covers_spec_20_2() {
        let root = repo_root_from_manifest();
        let body = fs::read_to_string(root.join(AGGREGATE_DECISION_SCHEMA_REL))
            .expect("read aggregate schema");
        let schema: serde_json::Value = serde_json::from_str(&body).expect("valid JSON schema");
        let required = schema["required"].as_array().expect("required array");
        for field in [
            "expected_lanes",
            "actual_lanes",
            "decisions",
            "missing_lanes",
            "final_decision",
            "baseline_digest",
            "ruleset_context",
        ] {
            assert!(
                required.iter().any(|value| value.as_str() == Some(field)),
                "aggregate schema missing Spec §20.2 field {field}"
            );
        }
    }

    #[test]
    fn non_dry_run_run_is_rejected_by_contract() {
        // 通过 run() 分发验证：dry_run=false 必须失败关闭
        let err = run(
            true,
            CiCommand::Run {
                dry_run: false,
                plan_file: None,
            },
        );
        assert!(err.is_err(), "non-dry-run must fail closed");
    }

    #[test]
    fn not_implemented_helper_still_explicit() {
        let report = not_implemented_report("legacy");
        assert!(!report.ok);
        assert_eq!(report.status, "NOT_IMPLEMENTED");
    }

    #[test]
    fn verify_ignores_markers_only_in_comments() {
        // 注释里塞入全部必填 token 不得伪通过
        let fake = r#"
# schema_version baseline_id baseline_version
# [merge] required_context shadow_context
# [toolchains] primary msrv 1.94.1
# [budgets] [reuse] enabled [planner] unknown_change_behavior
# [lanes.fast] [lanes.build_test] command runner_class
schema_version = 1
"#;
        let cleaned = strip_toml_comments(fake);
        let missing = verify_baseline_structure(&cleaned);
        assert!(
            missing.iter().any(|m| m.contains("baseline_id")),
            "comment-only markers must not satisfy baseline_id; missing={missing:?}"
        );
        assert!(
            missing
                .iter()
                .any(|m| m.contains("[merge]") || m.contains("merge")),
            "missing real [merge] section; missing={missing:?}"
        );
    }

    #[test]
    fn render_writes_generated_projections_from_real_baseline() {
        let root = repo_root_from_manifest();
        let tmp = tempfile::tempdir().expect("tempdir");
        let out = tmp.path().join("generated");
        let report = render_to(&root, &out).expect("render_to runs");
        assert!(report.ok, "render must succeed: {}", report.note);
        assert_eq!(report.baseline_digest.len(), 64, "sha256 hex length");
        let contract = out.join("workflow-contract.json");
        let policy = out.join("policy-table.md");
        assert!(contract.is_file(), "workflow-contract.json");
        assert!(policy.is_file(), "policy-table.md");
        let body = fs::read_to_string(&contract).expect("read contract");
        assert!(body.contains(&report.baseline_digest));
        assert!(body.contains("shadow") || body.contains("Shadow"));
        // lanes 必须来自 baseline，不得硬编码 aggregate
        let v: serde_json::Value = serde_json::from_str(&body).expect("json");
        let lanes = v["lanes"].as_array().expect("lanes array");
        assert!(
            !lanes.iter().any(|x| x.as_str() == Some("aggregate")),
            "aggregate must not appear in lanes projection"
        );
        assert!(
            lanes.iter().any(|x| x.as_str() == Some("fast")),
            "baseline lane fast must be projected"
        );
        // 与仓库内 golden 对比：digest 一致且 lanes 不含 aggregate
        let golden = root.join(".github/ci/generated/workflow-contract.json");
        if golden.is_file() {
            let g = fs::read_to_string(&golden).expect("golden");
            assert!(
                g.contains(&report.baseline_digest),
                "checked-in golden must match current baseline digest (re-run ci render)"
            );
            assert!(
                !g.contains("\"aggregate\"") || g.contains("aggregate_context"),
                "golden should not list aggregate as a lane name only"
            );
            // 更严：golden lanes 数组不得含 aggregate 元素
            if let Ok(gv) = serde_json::from_str::<serde_json::Value>(&g) {
                if let Some(gl) = gv["lanes"].as_array() {
                    assert!(
                        !gl.iter().any(|x| x.as_str() == Some("aggregate")),
                        "golden lanes must not include aggregate; re-render and commit"
                    );
                }
            }
        }
        let again = render_to(&root, &out).expect("second render");
        assert_eq!(again.baseline_digest, report.baseline_digest);
    }

    #[test]
    fn negative_fixture_inventory_lists_spec26_items() {
        let root = repo_root_from_manifest();
        let inv = root.join("tools/xtask/tests/ci_negative_fixtures.md");
        assert!(
            inv.is_file(),
            "PHASE-1-13 inventory must exist at {}",
            inv.display()
        );
        let body = fs::read_to_string(&inv).expect("read fixtures inventory");
        // Spec §26 权威 20 项（字面）
        let required = [
            "Missing Lane",
            "Cancelled Lane",
            "Unexpected Skip",
            "Invalid N/A",
            "Invalid Reused Attestation",
            "Fingerprint Mismatch",
            "Runner Digest Mismatch",
            "Tool Version Mismatch",
            "Disk Insufficient",
            "Planner Unknown",
            "Cargo Graph Parse Failure",
            "Merge Group Event",
            "Fork PR",
            "GitHub API 429/5xx",
            "Generated Drift",
            "Ruleset Drift",
            "Cache Corruption",
            "External Cleanup Failure",
            "Flake Expiry",
            "Aggregate Unknown State",
        ];
        for (i, name) in required.iter().enumerate() {
            assert!(
                body.contains(name),
                "fixture inventory missing Spec §26 item {}: {name}",
                i + 1
            );
        }
        assert!(
            body.contains("STUB") || body.contains("Stub"),
            "fixtures still stub status"
        );
    }

    #[test]
    fn plan_includes_shards_and_fast_lane() {
        let report = plan();
        assert!(
            report.shards.get("compile_once").is_some() || report.shards.get("shards").is_some()
        );
        assert!(report.fast_lane.get("decision").is_some());
        assert!(report.special_matrix.get("loom").is_some());
    }

    #[test]
    fn taxonomy_lists_ci_profile() {
        let root = repo_root_from_manifest();
        let v = taxonomy_read(&root).unwrap();
        assert_eq!(v["ok"], true, "{v}");
        let profiles = v["profiles"].as_array().unwrap();
        assert!(profiles.iter().any(|p| p.as_str() == Some("ci")));
    }

    #[test]
    fn evidence_root_ok() {
        let root = repo_root_from_manifest();
        let v = evidence_root_check(&root).unwrap();
        assert_eq!(v["ok"], true, "{v}");
    }

    #[test]
    fn negative_missing_lane_fixture_fails_aggregate() {
        let root = repo_root_from_manifest();
        let fixture = root.join("tools/xtask/tests/ci_negative/fixtures/missing_lane.json");
        assert!(fixture.is_file(), "executable negative fixture must exist");
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
        assert!(report.missing_lanes.iter().any(|l| l == "build_test"));
    }

    #[test]
    fn negative_invalid_na_fixture_fails_aggregate() {
        let root = repo_root_from_manifest();
        let fixture = root.join("tools/xtask/tests/ci_negative/fixtures/invalid_na.json");
        assert!(fixture.is_file(), "invalid_na fixture must exist");
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
    }

    #[test]
    fn negative_missing_reused_attestation_fixture_fails() {
        let root = repo_root_from_manifest();
        let fixture =
            root.join("tools/xtask/tests/ci_negative/fixtures/missing_reused_attestation.json");
        assert!(fixture.is_file());
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
    }

    #[test]
    fn negative_unexpected_skip_fixture_fails() {
        let root = repo_root_from_manifest();
        let fixture = root.join("tools/xtask/tests/ci_negative/fixtures/unexpected_skip.json");
        assert!(fixture.is_file());
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
    }

    #[test]
    fn negative_cancelled_lane_fixture_fails() {
        let root = repo_root_from_manifest();
        let fixture = root.join("tools/xtask/tests/ci_negative/fixtures/cancelled_lane.json");
        assert!(fixture.is_file());
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
    }

    #[test]
    fn negative_aggregate_unknown_state_fixture_fails() {
        let root = repo_root_from_manifest();
        let fixture =
            root.join("tools/xtask/tests/ci_negative/fixtures/aggregate_unknown_state.json");
        assert!(fixture.is_file());
        let report = aggregate_dry(&root, Some(&fixture), Some("fast,build_test"), false).unwrap();
        assert_eq!(report.final_decision, "FAIL");
    }

    #[test]
    fn verify_requires_section_keys_not_prose() {
        let body = r#"
schema_version = 1
baseline_id = "x"
baseline_version = "0"
[merge]
# required_context is only in comment
shadow_context = "CI / required-shadow"
[toolchains]
primary = "1.94.1"
msrv = "1.94.1"
[budgets]
[reuse]
enabled = false
[planner]
unknown_change_behavior = "full"
[lanes.fast]
command = "cargo xtl ci plan"
runner_class = "fast"
[lanes.build_test]
command = "cargo xtl ci verify"
runner_class = "build"
"#;
        let cleaned = strip_toml_comments(body);
        let missing = verify_baseline_structure(&cleaned);
        assert!(
            missing.iter().any(|m| m == "merge.required_context"),
            "must fail when required_context only in comment; missing={missing:?}"
        );
    }
}
