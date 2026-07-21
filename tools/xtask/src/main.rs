//! xtask —— xhyper.rs 内部工具链。
//!
//! 子命令：
//! - `lint-deps`：校验 workspace 依赖图是否符合 spec §2 R1–R6。
//! - `gen-structure` / `migration` / `crate-standard`：结构与标准盘点。
//! - `inventory-ssot`：INFRA-004 SSOT/拓扑漂移只读检测。
//! - `evidence-check`：INFRA-003 Evidence schema/脱敏/自测校验。
//! - `approval-check`：IG-1 审批 registry 只读校验（支持 single_accountable_owner 自动化策略）。
//! - `approval-auto`：IG-1 最大自动化批准（单责任人 + 机器副署；`--apply` 写入）。
//! - `semver-check`：INFRA-008 cargo-semver-checks 探测（工具缺失 fail-closed）。
//! - `drift-detect`：INFRA-060 只读漂移检测（禁止自动修复）。
//! - `test-graph-check`：SPEC-TESTKIT-002 测试支持平面生产图隔离。
//! - `naming-check`：NAMING_STANDARD / package 前缀 registry 对照（默认 shadow）。
//! - `no-new-gate`：PLAN-GATE-RETIRE-001 Phase 0 冻结 runtime gate 新增使用。
//! - `ci`：CI SSOT（Shadow only；plan/run/aggregate/reconcile/metrics dry-run；非 dry-run fail-closed）。
//! - `authority-check`：Authority Plane Shadow（Registry 唯一校验 / Risk Tier / subject 绑定 / AI·bot 非人类）。

mod allowed_matrix;
mod approval_auto;
mod approval_check;
mod architecture_toml;
mod authority_plane;
mod ci;
mod classify;
mod crate_standard;
mod drift_detect;
mod evidence_check;
mod gen_structure;
mod human_actor;
mod inventory_ssot;
mod lint_deps;
mod migration;
mod naming_check;
mod no_new_gate;
mod schema_lite;
mod semver_check;
mod test_graph_check;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xtask", about = "xhyper.rs 内部工具链")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// 输出 JSON 格式（机器可读）
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// 校验 workspace 依赖图是否符合 spec §2 R1–R6
    LintDeps,
    /// Generate the canonical workspace structure snapshot.
    GenStructure {
        #[arg(long)]
        check: bool,
    },
    /// Validate the frozen P2 physical-migration registry.
    Migration {
        #[arg(long)]
        check: bool,
    },
    /// Inventory workspace and legacy crates against the crate standard.
    CrateStandard {
        #[arg(long)]
        check: bool,
    },
    /// INFRA-004：校验 architecture/configs/deploy/schemas/evidence SSOT 漂移
    InventorySsot,
    /// INFRA-003：校验 Evidence JSON（schema 子集 + 脱敏；可选 --self-test）
    EvidenceCheck {
        /// 单文件或目录；默认扫描 evidence/**/*.evidence.json 与 schema fixtures
        #[arg(long)]
        path: Option<PathBuf>,
        /// 运行内存自测：合法样本通过，缺字段/篡改/secret 样本必须失败
        #[arg(long)]
        self_test: bool,
    },
    /// IG-1：校验唯一审批 registry；默认 fail-closed 判断 gate 是否可退出
    ApprovalCheck {
        /// 只校验 registry 结构/引用，不要求全部决策已批准
        #[arg(long)]
        registry_only: bool,
    },
    /// IG-1：single_accountable_owner + 机器副署自动化（dry-run 默认；--apply 写入）
    ApprovalAuto {
        /// 写入 registry / subject 状态 / attestation evidence
        #[arg(long)]
        apply: bool,
        /// 责任人 GitHub handle（默认：gh api user 或 registry 已有值）
        #[arg(long)]
        owner: Option<String>,
        /// 授权 apply 的 APPROVED decision id（也可用 env XHYPER_APPROVAL_AUTO_APPROVED）
        #[arg(long)]
        authorized_by: Option<String>,
    },
    /// INFRA-008：探测 cargo-semver-checks；工具缺失或无 baseline 时非 0（不伪绿）
    SemverCheck,
    /// INFRA-060：只读 drift 检测 + 类别清单；禁止自动修复
    DriftDetect,
    /// SPEC-TESTKIT-002 §14：test-support 不得进入 production normal/build 图
    TestGraphCheck,
    /// NAMING_STANDARD：对照 naming.toml 与 cargo metadata（默认 shadow，不阻断）
    NamingCheck {
        /// shadow（默认，exit 0）| strict（ERROR 非 0）
        #[arg(long, default_value = "shadow")]
        mode: String,
    },
    /// PLAN-GATE-RETIRE-001：冻结 runtime xhyper-gate 新增依赖/源码使用
    NoNewGate,
    /// CI SSOT 骨架（Shadow only；非生产激活）
    Ci {
        #[command(subcommand)]
        action: ci::CiCommand,
    },
    /// Authority Plane Shadow：Registry 唯一校验 / Risk Tier·operation class / subject 绑定 approval
    ///
    /// 永不授予 live 合并权（`live_authorization=false`）。
    AuthorityCheck {
        /// 只校验 `.architecture/authority-registry.target.toml` 结构与唯一性
        #[arg(long)]
        registry_only: bool,
        /// 变更路径（可重复）；用于 Risk Tier / operation class 计算
        #[arg(long = "path")]
        paths: Vec<String>,
        /// Change Contract 声明的 risk tier（可抬高，不可低于机器计算）
        #[arg(long)]
        declared_risk_tier: Option<String>,
        /// 评估 final-head approvals 的 JSON 文件（reviews + optional risk_reviews + subject fields）
        #[arg(long)]
        eval_json: Option<PathBuf>,
        /// 允许加载 live `authority-registry.toml`（默认拒绝；仅 RFC Effective 后由人类打开）
        #[arg(long)]
        allow_live_registry: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::LintDeps => lint_deps::run(cli.json),
        Command::GenStructure { check } => gen_structure::run(check),
        Command::Migration { check } => migration::run(check),
        Command::CrateStandard { check } => crate_standard::run(cli.json, check),
        Command::InventorySsot => inventory_ssot::run(cli.json),
        Command::EvidenceCheck { path, self_test } => {
            evidence_check::run(cli.json, path, self_test)
        }
        Command::ApprovalCheck { registry_only } => approval_check::run(cli.json, registry_only),
        Command::ApprovalAuto {
            apply,
            owner,
            authorized_by,
        } => approval_auto::run(cli.json, apply, owner, authorized_by),
        Command::SemverCheck => semver_check::run(cli.json),
        Command::DriftDetect => drift_detect::run(cli.json),
        Command::TestGraphCheck => test_graph_check::run(cli.json),
        Command::NamingCheck { mode } => {
            let mode = naming_check::Mode::parse(&mode)?;
            naming_check::run(cli.json, mode)
        }
        Command::NoNewGate => no_new_gate::run(cli.json),
        Command::Ci { action } => ci::run(cli.json, action),
        Command::AuthorityCheck {
            registry_only,
            paths,
            declared_risk_tier,
            eval_json,
            allow_live_registry,
        } => {
            let root = authority_plane::workspace_root();
            let mut args = authority_plane::AuthorityCheckArgs {
                registry_only,
                changed_paths: paths,
                declared_risk_tier,
                allow_live_registry,
                ..Default::default()
            };
            if let Some(path) = eval_json {
                let raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read eval-json {}", path.display()))?;
                let v: serde_json::Value = serde_json::from_str(&raw)
                    .with_context(|| format!("parse eval-json {}", path.display()))?;
                if let Some(author) = v.get("pr_author").and_then(|x| x.as_str()) {
                    args.pr_author = Some(author.to_string());
                }
                if let Some(paths) = v.get("paths").and_then(|x| x.as_array()) {
                    for p in paths {
                        if let Some(s) = p.as_str() {
                            args.changed_paths.push(s.to_string());
                        }
                    }
                }
                if let Some(tier) = v.get("declared_risk_tier").and_then(|x| x.as_str()) {
                    args.declared_risk_tier = Some(tier.to_string());
                }
                if let Some(subj) = v.get("subject") {
                    args.subject = Some(serde_json::from_value(subj.clone())?);
                }
                if let Some(reviews) = v.get("reviews") {
                    args.reviews = serde_json::from_value(reviews.clone())?;
                }
                if let Some(risk) = v.get("risk_reviews") {
                    args.risk_reviews = serde_json::from_value(risk.clone())?;
                }
            }
            let report = authority_plane::run(&root, args)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "authority-check mode={} ok={} live_authorization={}",
                    report.mode, report.ok, report.live_authorization
                );
                if let Some(reg) = &report.registry {
                    println!(
                        "  registry: path={} entries={} live_ssot={} ok={}",
                        reg.path, reg.entry_count, reg.live_ssot, reg.ok
                    );
                }
                if let Some(risk) = &report.risk {
                    println!(
                        "  risk: tier={} primary_class={} classes={:?} human_approvals_needed={} risk_review={}",
                        risk.computed_risk_tier,
                        risk.primary_operation_class,
                        risk.operation_classes,
                        risk.required_human_approvals,
                        risk.require_independent_risk_review
                    );
                }
                if let Some(d) = &report.subject_digest {
                    println!("  subject_digest: {d}");
                }
                if let Some(a) = &report.approvals {
                    println!(
                        "  approvals: human={} risk={} quorum={} risk_ok={} subject_bound={}",
                        a.human_approvals_counted,
                        a.risk_reviews_counted,
                        a.meets_human_quorum,
                        a.meets_risk_review,
                        a.subject_bound_ok
                    );
                    for r in &a.rejected {
                        println!("    rejected: {r}");
                    }
                }
                for f in &report.findings {
                    println!("  finding: {f}");
                }
            }
            if report.ok {
                Ok(())
            } else {
                anyhow::bail!(
                    "authority-check failed with {} finding(s)",
                    report.findings.len()
                )
            }
        }
    }
}
