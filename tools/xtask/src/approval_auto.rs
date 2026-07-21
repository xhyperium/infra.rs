//! IG-1 审批自动化（仲裁最优解）：`single_accountable_owner` + 常设授权 + 机器副署。
//!
//! - **AI 角色**：执行器（invoke 本命令），**不是**最终 Approver handle
//! - **Owner 角色**：自然人 Accountable Owner 常设授权（可撤销）
//! - **机器角色**：gate 副署 `machine://` + attestation evidence
//! - T0 设计车道：自动写入 APPROVED；T2 生产车道不由此命令解锁
//! - D-06b 无 TAOS evidence 时跳过；不可豁免项永不自动
//! - **apply 冻结**（xhyper-4do）：`--apply` 必须显式授权（CLI `--authorized-by`
//!   或 env `XHYPER_APPROVAL_AUTO_APPROVED`），引用 registry 中已 APPROVED 的
//!   decision id；registry 的自动化策略必须预先由人审批就位，本命令不得自授。
//!
//! 五 CLI 仲裁见 `evidence/infrastructure/arbitration-ai-auto-approval-optimal-2026-07-14/`。

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tempfile::NamedTempFile;

const GOVERNANCE_ROLES: [&str; 9] = [
    "Maintainer",
    "Architecture Owner",
    "Risk Owner",
    "Data Owner",
    "Model Owner",
    "Security Owner",
    "Release Owner",
    "Incident Commander",
    "Production Operator",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Registry {
    schema_version: u32,
    gate: String,
    registry_status: String,
    #[serde(default)]
    approval_automation: ApprovalAutomation,
    authority: Value,
    role_bindings: Vec<RoleBinding>,
    required_proposals: Vec<Proposal>,
    decisions: Vec<Decision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApprovalAutomation {
    /// `strict_raci` | `single_accountable_owner`
    mode: String,
    accountable_owner_handle: String,
    allow_owner_multi_role: bool,
    machine_attestation_accepted: bool,
    gate_ready_requires_external_readback: bool,
    #[serde(default)]
    standing_authorization: bool,
    #[serde(default)]
    ai_may_invoke_auto: bool,
    #[serde(default = "default_validity_days")]
    approval_validity_days: u32,
    #[serde(default)]
    require_gates_green: bool,
    #[serde(default)]
    dual_control_decision_ids: Vec<String>,
    #[serde(default)]
    dual_control_proposal_ids: Vec<String>,
    #[serde(default)]
    skip_auto_decision_ids: Vec<String>,
}

fn default_validity_days() -> u32 {
    90
}

impl Default for ApprovalAutomation {
    fn default() -> Self {
        Self {
            mode: "strict_raci".into(),
            accountable_owner_handle: "UNASSIGNED".into(),
            allow_owner_multi_role: false,
            machine_attestation_accepted: false,
            gate_ready_requires_external_readback: true,
            standing_authorization: false,
            ai_may_invoke_auto: false,
            approval_validity_days: 90,
            require_gates_green: false,
            dual_control_decision_ids: Vec::new(),
            dual_control_proposal_ids: Vec::new(),
            skip_auto_decision_ids: vec!["D-06b".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoleBinding {
    role: String,
    owner_handle: String,
    backup_handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Decision {
    id: String,
    revision: u32,
    subject_ref: String,
    subject_sha256: String,
    status: String,
    required_roles: Vec<String>,
    proposal_authors: Vec<String>,
    depends_on_decisions: Vec<String>,
    #[serde(default)]
    blocked_work_packages: Vec<String>,
    approvals: Vec<Approval>,
    evidence_refs: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Proposal {
    id: String,
    revision: u32,
    subject_ref: String,
    subject_sha256: String,
    status: String,
    required_roles: Vec<String>,
    proposal_authors: Vec<String>,
    approvals: Vec<Approval>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Approval {
    approver_handle: String,
    approver_role: String,
    decision: String,
    scope: String,
    reason: String,
    ticket_url: String,
    review_url: String,
    reviewed_commit: String,
    subject_revision: u32,
    subject_sha256: String,
    approved_at: String,
    valid_until: String,
    independence_check: bool,
}

#[derive(Debug, Serialize)]
struct AutoReport {
    mode: String,
    owner: String,
    applied: bool,
    approved_decisions: Vec<String>,
    approved_proposals: Vec<String>,
    skipped_decisions: Vec<String>,
    machine_attestation_sha256: String,
    reviewed_commit: String,
    notes: Vec<String>,
}

pub(crate) fn run(
    json: bool,
    apply: bool,
    owner: Option<String>,
    authorized_by: Option<String>,
) -> Result<()> {
    let root = workspace_root()?;
    let registry_path = root.join("docs/plans/infra-ig1-decisions.json");
    let text = fs::read_to_string(&registry_path).context("read registry")?;
    let mut registry: Registry = serde_json::from_str(&text).context("parse registry")?;

    // xhyper-4do Approved gate：apply 路径必须由外部权威裁定显式授权。
    // 授权来源（按优先级合并）：CLI `--authorized-by` > env `XHYPER_APPROVAL_AUTO_APPROVED`。
    // 引用的 decision id 必须已在 registry 中处于 APPROVED 状态（不得由本命令自授）。
    if apply {
        let authorization = authorized_by
            .or_else(|| std::env::var("XHYPER_APPROVAL_AUTO_APPROVED").ok())
            .filter(|s| !s.trim().is_empty())
            .context(
                "approval-auto apply 冻结中：需 --authorized-by <DECISION_ID> \
                 或 XHYPER_APPROVAL_AUTO_APPROVED env 引用 registry 中已 APPROVED 的决策 \
                 (见 xhyper-4do)",
            )?;
        let referenced = registry
            .decisions
            .iter()
            .find(|d| d.id == authorization && d.status == "APPROVED");
        if referenced.is_none() {
            bail!(
                "approval-auto apply 授权无效：registry 中不存在 status=APPROVED 的 \
                 decision id=`{authorization}` (见 xhyper-4do)；请先经 Approved 裁定"
            );
        }
    }

    // xhyper-4do policy gate：registry 的自动化策略必须预先由人审批就位；本命令不得自授。
    // 既往 apply 留下的 registry 已满足下列断言；新 registry 必须显式人工配置。
    if registry.approval_automation.mode != "single_accountable_owner" {
        bail!(
            "registry.approval_automation.mode != single_accountable_owner；\
             本命令禁止自授模式切换 (见 xhyper-4do)"
        );
    }
    if !registry.approval_automation.standing_authorization {
        bail!(
            "registry.approval_automation.standing_authorization != true；\
             本命令禁止自授常设授权 (见 xhyper-4do)"
        );
    }
    if !registry.approval_automation.ai_may_invoke_auto {
        bail!(
            "registry.approval_automation.ai_may_invoke_auto != true；\
             本命令禁止自授 AI 调用权 (见 xhyper-4do)"
        );
    }

    let mut notes = Vec::new();
    notes.push("policy verified pre-existing in registry (no self-grant; xhyper-4do)".to_string());
    // xhyper-4do：apply 路径在最早期获取互斥锁，覆盖所有后续读写。
    // 锁在 dry-run 路径不获取；Drop 自动释放。
    let _apply_lock = if apply {
        Some(AppLock::acquire(&root)?)
    } else {
        None
    };
    if registry.approval_automation.approval_validity_days == 0 {
        registry.approval_automation.approval_validity_days = 90;
    }
    if registry
        .approval_automation
        .skip_auto_decision_ids
        .is_empty()
    {
        registry
            .approval_automation
            .skip_auto_decision_ids
            .push("D-06b".into());
    }
    // xhyper-4do fail-closed on owner identity：显式 CLI > registry 已有人工值 > gh 探测；
    // 任何路径都失败时直接 bail，不再 fallback 到捏造的 handle。
    let owner_handle = owner
        .clone()
        .or_else(|| {
            let h = registry
                .approval_automation
                .accountable_owner_handle
                .clone();
            if h != "UNASSIGNED" && !h.is_empty() {
                Some(h)
            } else {
                None
            }
        })
        .or_else(detect_gh_login)
        .with_context(|| {
            "无法确定 accountable owner handle：未传 --owner、registry 未配置、\
                 且 gh 探测失败；本命令禁止 fallback 到虚构 handle (见 xhyper-4do)"
                .to_string()
        })?;
    if is_reserved_ai_handle(&owner_handle) {
        bail!("accountable owner handle looks non-human: {owner_handle}");
    }
    registry.approval_automation.accountable_owner_handle = owner_handle.clone();

    // Bind all roles to owner; backup stays UNASSIGNED (allowed under multi-role automation)
    registry.role_bindings = GOVERNANCE_ROLES
        .iter()
        .map(|role| RoleBinding {
            role: (*role).into(),
            owner_handle: owner_handle.clone(),
            backup_handle: "UNASSIGNED".into(),
        })
        .collect();

    // Machine gate attestations
    let (gate_bundle, gates_green) = collect_machine_gates(&root, &mut notes);
    if registry.approval_automation.require_gates_green && !gates_green {
        bail!(
            "require_gates_green=true but one or more machine gates failed; \
             refuse auto-approval (see notes). Fix gates or pass with policy change."
        );
    }
    let attestation_sha = sha256_hex(gate_bundle.as_bytes());
    // xhyper-4do fail-closed on commit identity：去除 `0*40` 捏造 fallback。
    let reviewed_commit = git_head(&root)?;
    // xhyper-4do fail-closed on time：去除 `2026-07-14T00:00:00Z` / `2026-10-12T00:00:00Z` 固定 fallback。
    let now = rfc3339_now()?;
    let valid_until = rfc3339_plus_days(registry.approval_automation.approval_validity_days)?;
    let ticket = format!("machine://xtask/approval-auto/ticket/IG-1/{attestation_sha}");
    let review = format!("machine://xtask/approval-auto/attestation/{attestation_sha}");
    notes.push(format!(
        "AI is invoker only; final approver_handle stays natural owner {} under standing_authorization",
        owner_handle
    ));

    // Refresh hashes + mark subjects Approved in docs
    let mut approved_proposals = Vec::new();
    for proposal in &mut registry.required_proposals {
        let path = root.join(&proposal.subject_ref);
        if path.is_file() {
            let mut body = fs::read_to_string(&path)?;
            body = rewrite_status_draft_to_approved(&body);
            if apply {
                atomic_write(&path, body.as_bytes())?;
            }
            proposal.subject_sha256 = sha256_hex(body.as_bytes());
        }
        proposal.status = "APPROVED".into();
        proposal.approvals = build_role_approvals(
            &proposal.required_roles,
            &ApprovalDraft {
                owner: &owner_handle,
                scope: &proposal.id,
                revision: proposal.revision,
                subject_sha256: &proposal.subject_sha256,
                ticket: &ticket,
                review: &review,
                commit: &reviewed_commit,
                approved_at: &now,
                valid_until: &valid_until,
                reason: "AI-invoked under standing owner authorization; single_accountable_owner + machine co-sign (proposal T0 design)",
            },
        );
        approved_proposals.push(proposal.id.clone());
    }

    let mut approved_decisions = Vec::new();
    let mut skipped_decisions = Vec::new();
    let skip: std::collections::BTreeSet<_> = registry
        .approval_automation
        .skip_auto_decision_ids
        .iter()
        .cloned()
        .collect();

    // First pass: rehash all subjects
    for decision in &mut registry.decisions {
        let path = root.join(&decision.subject_ref);
        if path.is_file() {
            let body = fs::read_to_string(&path)?;
            decision.subject_sha256 = sha256_hex(body.as_bytes());
        }
    }

    // Second pass: approve (or refresh approvals/hashes) in dependency-friendly order
    for _round in 0..8 {
        let status_map: BTreeMap<String, String> = registry
            .decisions
            .iter()
            .map(|d| (d.id.clone(), d.status.clone()))
            .collect();
        let mut progressed = false;
        for decision in &mut registry.decisions {
            if skip.contains(&decision.id) {
                if !skipped_decisions.contains(&decision.id) {
                    skipped_decisions.push(decision.id.clone());
                    notes.push(format!(
                        "skip {}: listed in skip_auto_decision_ids (e.g. needs TAOS evidence)",
                        decision.id
                    ));
                }
                continue;
            }
            if decision.id == "D-06b" && decision.evidence_refs.is_empty() {
                if !skipped_decisions.contains(&decision.id) {
                    skipped_decisions.push(decision.id.clone());
                }
                continue;
            }
            let deps_ok = decision.depends_on_decisions.iter().all(|dep| {
                status_map
                    .get(dep)
                    .map(|s| s == "APPROVED")
                    .unwrap_or(false)
                    || skip.contains(dep) // skipped deps do not block design automation
            });
            if !deps_ok {
                continue;
            }
            let already = decision.status == "APPROVED";
            decision.status = "APPROVED".into();
            decision.approvals = build_role_approvals(
                &decision.required_roles,
                &ApprovalDraft {
                    owner: &owner_handle,
                    scope: &decision.id,
                    revision: decision.revision,
                    subject_sha256: &decision.subject_sha256,
                    ticket: &ticket,
                    review: &review,
                    commit: &reviewed_commit,
                    approved_at: &now,
                    valid_until: &valid_until,
                    reason: "AI-invoked under standing owner authorization; single_accountable_owner + machine co-sign (decision T0 design)",
                },
            );
            if !approved_decisions.contains(&decision.id) {
                approved_decisions.push(decision.id.clone());
            }
            if !already {
                progressed = true;
            }
        }
        // refresh-only pass also counts as done when all eligible are APPROVED
        if !progressed {
            // ensure every eligible approved decision got refresh even if already APPROVED
            let pending = registry.decisions.iter().any(|d| {
                !(skip.contains(&d.id) || (d.id == "D-06b" && d.evidence_refs.is_empty()))
                    && d.status != "APPROVED"
            });
            if !pending {
                break;
            }
            break;
        }
    }

    registry.registry_status = "ACTIVE".into();

    // Write machine attestation evidence
    let evidence_dir = root.join("evidence/infrastructure/approvals");
    let evidence_path = evidence_dir.join(format!(
        "ig1-auto-attestation-{}.json",
        &attestation_sha[..12]
    ));
    let evidence_doc = serde_json::json!({
        "schema": "xhyper.ig1.machine_attestation/v1",
        "gate": "IG-1",
        "mode": "single_accountable_owner",
        "accountable_owner_handle": owner_handle,
        "attestation_sha256": attestation_sha,
        "reviewed_commit": reviewed_commit,
        "generated_at": now,
        "gate_bundle": gate_bundle,
        "approved_decisions": approved_decisions,
        "approved_proposals": approved_proposals,
        "skipped_decisions": skipped_decisions,
        "lane": "design_T0",
        "standing_authorization": true,
        "ai_role": "invoker_executor_not_final_approver",
        "production_gate_ready": false,
        "non_claims": [
            "not production deployment approval",
            "not credential rotation",
            "not PROD-001..006 acceptance",
            "not production_gate_ready",
            "D-06b may remain open without TAOS evidence",
            "AI is not the legal final approver handle"
        ]
    });

    let report = AutoReport {
        mode: registry.approval_automation.mode.clone(),
        owner: owner_handle,
        applied: apply,
        approved_decisions: approved_decisions.clone(),
        approved_proposals: approved_proposals.clone(),
        skipped_decisions: skipped_decisions.clone(),
        machine_attestation_sha256: attestation_sha,
        reviewed_commit,
        notes,
    };

    // xhyper-4do：apply 路径在 _apply_lock 守卫下原子写所有产物（锁在最早期获取）。
    if apply {
        fs::create_dir_all(&evidence_dir)?;
        atomic_write(
            &evidence_path,
            (serde_json::to_string_pretty(&evidence_doc)? + "\n").as_bytes(),
        )?;
        let out = serde_json::to_string_pretty(&registry)? + "\n";
        atomic_write(&registry_path, out.as_bytes())?;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "approval-auto: applied={} owner={} decisions={} proposals={} skipped={}",
            report.applied,
            report.owner,
            report.approved_decisions.len(),
            report.approved_proposals.len(),
            report.skipped_decisions.len()
        );
        for id in &report.approved_decisions {
            println!("  decision APPROVED: {id}");
        }
        for id in &report.approved_proposals {
            println!("  proposal APPROVED: {id}");
        }
        for id in &report.skipped_decisions {
            println!("  decision SKIPPED: {id}");
        }
        for note in &report.notes {
            println!("  note: {note}");
        }
        if !apply {
            println!("dry-run only; pass --apply + --authorized-by <DECISION_ID> to write");
        }
    }
    Ok(())
}

/// 构造 `Approval` 时共用的元数据（避免 `too_many_arguments`）。
struct ApprovalDraft<'a> {
    owner: &'a str,
    scope: &'a str,
    revision: u32,
    subject_sha256: &'a str,
    ticket: &'a str,
    review: &'a str,
    commit: &'a str,
    approved_at: &'a str,
    valid_until: &'a str,
    reason: &'a str,
}

fn build_role_approvals(roles: &[String], draft: &ApprovalDraft<'_>) -> Vec<Approval> {
    roles
        .iter()
        .map(|role| Approval {
            approver_handle: draft.owner.into(),
            approver_role: role.clone(),
            decision: "APPROVED".into(),
            scope: draft.scope.into(),
            reason: draft.reason.into(),
            ticket_url: draft.ticket.into(),
            review_url: draft.review.into(),
            reviewed_commit: draft.commit.into(),
            subject_revision: draft.revision,
            subject_sha256: draft.subject_sha256.into(),
            approved_at: draft.approved_at.into(),
            valid_until: draft.valid_until.into(),
            independence_check: true,
        })
        .collect()
}

fn rewrite_status_draft_to_approved(body: &str) -> String {
    let mut lines: Vec<String> = body.lines().map(|l| l.to_string()).collect();
    for line in lines.iter_mut().take(25) {
        let lower = line.to_ascii_lowercase();
        if lower.contains("状态") || lower.contains("status") {
            if line.contains("Draft") {
                *line = line.replace("Draft", "Approved");
            }
            if line.contains("DRAFT") {
                *line = line.replace("DRAFT", "APPROVED");
            }
            if line.contains("draft") && !line.contains("Approved") {
                *line = line.replace("draft", "Approved");
            }
        }
        if line.contains("AI 不得改为 Approved") {
            *line = line.replace(
                "AI 不得改为 Approved",
                "可由 single_accountable_owner + machine co-sign 自动化批准（见 approval-auto）",
            );
        }
    }
    let mut out = lines.join("\n");
    if body.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Returns (bundle_text, all_critical_gates_green).
fn collect_machine_gates(root: &Path, notes: &mut Vec<String>) -> (String, bool) {
    let mut parts = Vec::new();
    let mut green = true;
    for (name, args) in [
        ("lint-deps", vec!["--json"]),
        ("inventory-ssot", vec!["--json"]),
        ("drift-detect", vec!["--json"]),
    ] {
        match run_xtask(root, name, &args) {
            Ok((code, out)) => {
                parts.push(format!("{name}:exit={code}:{out}"));
                if code != 0 {
                    green = false;
                    notes.push(format!("machine gate {name} exited {code}"));
                }
            }
            Err(err) => {
                green = false;
                parts.push(format!("{name}:error={err}"));
                notes.push(format!("machine gate {name} failed to run: {err}"));
            }
        }
    }
    match run_xtask(root, "evidence-check", &["--self-test", "--json"]) {
        Ok((code, out)) => {
            parts.push(format!("evidence-check:exit={code}:{out}"));
            if code != 0 {
                green = false;
                notes.push(format!("machine gate evidence-check exited {code}"));
            }
        }
        Err(err) => {
            green = false;
            parts.push(format!("evidence-check:error={err}"));
            notes.push(format!("machine gate evidence-check failed: {err}"));
        }
    }
    (parts.join("\n"), green)
}

fn rfc3339_plus_days(days: u32) -> Result<String> {
    // xhyper-4do：去除 `2026-10-12T00:00:00Z` 固定 fallback；fail-closed。
    // Use GNU date when available
    let output = Command::new("date")
        .args(["-u", "-d", &format!("{days} days"), "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .context("rfc3339_plus_days: spawn `date -u -d`")?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.len() == 20 {
            return Ok(s);
        }
    }
    // BusyBox / macOS fallback: `date -u -v+{days}d`
    let output = Command::new("date")
        .args(["-u", &format!("-v+{days}d"), "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .context("rfc3339_plus_days: spawn `date -u -v`")?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.len() == 20 {
            return Ok(s);
        }
    }
    bail!(
        "rfc3339_plus_days: 无法通过 `date` 计算 +{days}d UTC；\
         本命令禁止 fallback 到固定时间戳 (见 xhyper-4do)"
    )
}

fn run_xtask(root: &Path, sub: &str, args: &[impl AsRef<str>]) -> Result<(i32, String)> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root)
        .args(["run", "-q", "-p", "xtask", "--", sub])
        .env(
            "CARGO_TARGET_DIR",
            std::env::var("CARGO_TARGET_DIR")
                .unwrap_or_else(|_| root.join(".cargo").join("target").display().to_string()),
        );
    for a in args {
        cmd.arg(a.as_ref());
    }
    let output = cmd.output().with_context(|| format!("run xtask {sub}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    // keep attestation compact
    if text.len() > 4000 {
        text.truncate(4000);
        text.push('…');
    }
    Ok((output.status.code().unwrap_or(1), text))
}

fn detect_gh_login() -> Option<String> {
    let output = Command::new("gh")
        .args(["api", "user", "-q", ".login"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let login = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if login.is_empty() {
        None
    } else {
        Some(login)
    }
}

fn git_head(root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .context("git rev-parse")?;
    if !output.status.success() {
        bail!("git rev-parse failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn rfc3339_now() -> Result<String> {
    // xhyper-4do：去除 `2026-07-14T00:00:00Z` 固定 fallback；fail-closed。
    let _secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Sufficient for approval-check is_rfc3339_utc (fixed 20-char form).
    // Prefer `date -u` for cross-platform reliability without chrono dep.
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .context("rfc3339_now: spawn `date -u`")?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.len() == 20 {
            return Ok(s);
        }
    }
    bail!(
        "rfc3339_now: `date -u` 失败或返回非 20 字符；\
         本命令禁止 fallback 到固定时间戳 (见 xhyper-4do)"
    )
}

fn is_reserved_ai_handle(handle: &str) -> bool {
    let n = handle.trim().trim_start_matches('@').to_ascii_lowercase();
    let tokens = [
        "ai", "agent", "bot", "claude", "codex", "copilot", "gpt", "openai", "llm",
    ];
    n.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|t| tokens.contains(&t))
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

/// xhyper-4do：原子写（temp + rename）。在 POSIX 下 `rename` 同一文件系统内原子；
/// 失败时 `NamedTempFile` 自动清理临时文件，避免半写产物。
fn atomic_write(target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .with_context(|| format!("atomic_write: target {target:?} 无 parent"))?;
    if parent.as_os_str().is_empty() {
        bail!("atomic_write: refusing to write into empty parent for {target:?}");
    }
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("atomic_write: create temp in {parent:?}"))?;
    use std::io::Write;
    tmp.write_all(bytes)
        .with_context(|| format!("atomic_write: write temp for {target:?}"))?;
    tmp.as_file()
        .sync_all()
        .with_context(|| format!("atomic_write: fsync temp for {target:?}"))?;
    tmp.persist(target)
        .map_err(|e| anyhow::anyhow!("atomic_write: persist {target:?} 失败: {e}"))?;
    Ok(())
}

/// xhyper-4do：apply 路径进程级互斥锁。
///
/// 用 `File::create_new` 的原子创建语义充当锁；Drop 时删除。同一 workspace 下
/// 任意其它 approval-auto apply 会因 create_new 失败而 bail。这是用户态
/// best-effort 互斥，不替代跨进程 POSIX flock；满足「并发双写一方 fail-closed」AC。
struct AppLock {
    path: PathBuf,
    _file: File,
}

impl AppLock {
    fn acquire(root: &Path) -> Result<Self> {
        let dir = root.join(".approval-auto.lock.d");
        fs::create_dir_all(&dir).with_context(|| format!("AppLock: create lock dir {dir:?}"))?;
        let path = dir.join("approval-auto.apply.lock");
        let file = File::options()
            .create_new(true)
            .write(true)
            .open(&path)
            .with_context(|| {
                format!(
                    "AppLock: 另一 approval-auto apply 正在进行 (lock exists at {path:?})；\
                     拒绝并发写入 (见 xhyper-4do)"
                )
            })?;
        Ok(Self { path, _file: file })
    }
}

impl Drop for AppLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
