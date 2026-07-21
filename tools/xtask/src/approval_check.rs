//! IG-1 / INFRA 决策审批 registry 只读校验。
//!
//! - 强制 D-01..D-19（含 D-06a/D-06b）集合完整
//! - `AWAITING_APPROVAL` 合法；`APPROVED` 必须有完整 approver 字段
//! - 默认 `strict_raci`：AI/bot 不得当 approver；同 handle 不得跨角色
//! - `single_accountable_owner`（schema_version≥2 + approval_automation）：
//!   允许单一自然人跨角色 + `machine://` 副署 ticket/review，降低人工 RACI 摩擦
//! - subject SHA-256 绑定；INFRA-009 readback 可由 automation 配置放宽 gate_ready
//!
//! AI CLI 仍不得自任 Accountable Owner handle；owner 必须是自然人 GitHub handle。

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const IG1_DECISIONS: [&str; 20] = [
    "D-01", "D-02", "D-03", "D-04", "D-05", "D-06a", "D-06b", "D-07", "D-08", "D-09", "D-10",
    "D-11", "D-12", "D-13", "D-14", "D-15", "D-16", "D-17", "D-18", "D-19",
];
const IG1_EXIT_DECISIONS: [&str; 14] = [
    "D-01", "D-02", "D-03", "D-04", "D-05", "D-06a", "D-10", "D-11", "D-12", "D-13", "D-14",
    "D-15", "D-16", "D-19",
];
const IG1_PROPOSALS: [&str; 9] = [
    "P-01-control-plane",
    "P-02-lifecycle",
    "P-03-storage-contracts-v2",
    "P-04-data-authority",
    "P-05-taos",
    "P-06-evidence",
    "P-07-integration-harness",
    "P-08-release",
    "P-09-l2-service",
];
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

#[derive(Debug, Deserialize)]
struct Registry {
    schema_version: u32,
    gate: String,
    registry_status: String,
    #[serde(default)]
    approval_automation: ApprovalAutomation,
    role_bindings: Vec<RoleBinding>,
    required_proposals: Vec<Proposal>,
    decisions: Vec<Decision>,
}

/// 审批自动化策略（默认 strict，保持历史 fail-closed 行为）。
///
/// 最优解（仲裁共识）：`single_accountable_owner` + 常设授权 + 机器副署。
/// - AI 可作为 **执行器**（invoke `approval-auto`），**不得**冒充最终人类 Approver handle
/// - T0 设计车道：Owner 常设授权下机器/AI 自动写入
/// - T2 生产：`production_gate_ready` 独立，硬依赖 external readback
#[derive(Debug, Clone, Deserialize)]
struct ApprovalAutomation {
    #[serde(default = "default_strict_mode")]
    mode: String,
    #[serde(default = "default_unassigned")]
    accountable_owner_handle: String,
    #[serde(default)]
    allow_owner_multi_role: bool,
    #[serde(default)]
    machine_attestation_accepted: bool,
    /// 为 false 时，**design** gate_ready 不再硬依赖 INFRA-009 external readback。
    #[serde(default = "default_true")]
    gate_ready_requires_external_readback: bool,
    /// Owner 一次授权 AI/机器反复执行 approval-auto（可撤销：改 handle 或关 standing）
    #[serde(default)]
    standing_authorization: bool,
    /// 文档/策略位：允许 AI agent 调用 approval-auto（不把 AI 写成 approver_handle）
    #[serde(default)]
    ai_may_invoke_auto: bool,
    /// 审批有效天数（optimal 默认 90；过长如 2099 在仲裁中被指风险）
    #[serde(default = "default_validity_days")]
    approval_validity_days: u32,
    /// apply 前关键门禁必须绿（lint-deps/inventory 等）
    #[serde(default)]
    require_gates_green: bool,
    #[serde(default)]
    dual_control_decision_ids: Vec<String>,
    #[serde(default)]
    dual_control_proposal_ids: Vec<String>,
}

fn default_strict_mode() -> String {
    "strict_raci".into()
}
fn default_unassigned() -> String {
    "UNASSIGNED".into()
}
fn default_true() -> bool {
    true
}
fn default_validity_days() -> u32 {
    90
}

impl Default for ApprovalAutomation {
    fn default() -> Self {
        Self {
            mode: default_strict_mode(),
            accountable_owner_handle: default_unassigned(),
            allow_owner_multi_role: false,
            machine_attestation_accepted: false,
            gate_ready_requires_external_readback: true,
            standing_authorization: false,
            ai_may_invoke_auto: false,
            approval_validity_days: 90,
            require_gates_green: false,
            dual_control_decision_ids: Vec::new(),
            dual_control_proposal_ids: Vec::new(),
        }
    }
}

impl ApprovalAutomation {
    fn single_owner_mode(&self) -> bool {
        self.mode == "single_accountable_owner"
            && self.allow_owner_multi_role
            && self.accountable_owner_handle != "UNASSIGNED"
            && !self.accountable_owner_handle.is_empty()
            && !is_non_human_handle(&self.accountable_owner_handle)
    }
}

#[derive(Debug, Deserialize)]
struct RoleBinding {
    role: String,
    owner_handle: String,
    backup_handle: String,
}

#[derive(Debug, Deserialize)]
struct Decision {
    id: String,
    revision: u32,
    subject_ref: String,
    subject_sha256: String,
    status: String,
    required_roles: Vec<String>,
    proposal_authors: Vec<String>,
    depends_on_decisions: Vec<String>,
    approvals: Vec<Approval>,
    evidence_refs: Vec<EvidenceReference>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct EvidenceReference {
    path: String,
    sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum D06EvidenceKind {
    RestRoundtrip,
    NativeRoundtrip,
    NativeSafety,
}

impl D06EvidenceKind {
    const ALL: [Self; 3] = [
        Self::RestRoundtrip,
        Self::NativeRoundtrip,
        Self::NativeSafety,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::RestRoundtrip => "rest-roundtrip",
            Self::NativeRoundtrip => "native-roundtrip",
            Self::NativeSafety => "native-safety",
        }
    }
}

struct ApprovalValidation<'a> {
    id: &'a str,
    status: &'a str,
    required_roles: &'a [String],
    proposal_authors: &'a [String],
    approvals: &'a [Approval],
    subject_revision: u32,
    subject_sha256: &'a str,
    finding_prefix: &'a str,
}

#[derive(Debug, Serialize)]
pub(crate) struct Report {
    registry_valid: bool,
    /// 设计车道就绪（IG-1 design）。**不等于** production_gate_ready。
    gate_ready: bool,
    /// 生产车道就绪：需 external readback + 无生产 blocker；当前恒 false 直至 INFRA-009 真实现。
    production_gate_ready: bool,
    decision_count: usize,
    proposal_count: usize,
    trusted_review_readback: bool,
    automation_mode: String,
    standing_authorization: bool,
    ai_may_invoke_auto: bool,
    approval_validity_days: u32,
    require_gates_green: bool,
    findings: Vec<String>,
    blockers: Vec<String>,
}

pub(crate) fn run(json: bool, registry_only: bool) -> Result<()> {
    let root = workspace_root()?;
    let registry_path = root.join("docs/plans/infra-ig1-decisions.json");
    let report = validate_registry(&root, &registry_path)?;

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "approval-check: registry_valid={} design_gate_ready={} production_gate_ready={} mode={} decisions={} proposals={} findings={}",
            report.registry_valid,
            report.gate_ready,
            report.production_gate_ready,
            report.automation_mode,
            report.decision_count,
            report.proposal_count,
            report.findings.len()
        );
        for finding in &report.findings {
            println!("- {finding}");
        }
        for blocker in &report.blockers {
            println!("- BLOCKED: {blocker}");
        }
    }

    if !report.registry_valid {
        bail!("approval registry is invalid");
    }
    if !registry_only && !report.gate_ready {
        bail!("IG-1 is not ready: natural-person approvals remain open");
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn validate_registry(root: &Path, registry_path: &Path) -> Result<Report> {
    let text = fs::read_to_string(registry_path)
        .with_context(|| format!("read approval registry {}", registry_path.display()))?;
    let registry: Registry = serde_json::from_str(&text)
        .with_context(|| format!("parse approval registry {}", registry_path.display()))?;
    let mut findings = Vec::new();
    let now_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .context("system clock is before Unix epoch")?;
    let evidence_schema = load_evidence_schema(root, &mut findings);
    if !(registry.schema_version == 1 || registry.schema_version == 2) {
        findings.push(format!(
            "unsupported schema_version {} (expected 1 or 2)",
            registry.schema_version
        ));
    }
    let automation = registry.approval_automation.clone();
    if automation.single_owner_mode() && is_non_human_handle(&automation.accountable_owner_handle) {
        findings.push(format!(
            "automation-owner-not-human:{}",
            automation.accountable_owner_handle
        ));
    }
    if registry.gate != "IG-1" {
        findings.push(format!(
            "registry gate {} does not match fixed IG-1 policy",
            registry.gate
        ));
    }
    if !["DRAFT", "ACTIVE", "SUPERSEDED"].contains(&registry.registry_status.as_str()) {
        findings.push(format!(
            "registry-status-invalid:{}",
            registry.registry_status
        ));
    }
    let mut seen = BTreeSet::new();
    for decision in &registry.decisions {
        if !seen.insert(decision.id.as_str()) {
            findings.push(format!("decision-id-duplicate:{}", decision.id));
        }
    }
    let decision_statuses = registry
        .decisions
        .iter()
        .map(|decision| (decision.id.as_str(), decision.status.as_str()))
        .collect::<BTreeMap<_, _>>();
    let decisions_by_id = registry
        .decisions
        .iter()
        .map(|decision| (decision.id.as_str(), decision))
        .collect::<BTreeMap<_, _>>();
    for decision in &registry.decisions {
        if has_dependency_cycle(
            decision.id.as_str(),
            &decisions_by_id,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        ) {
            findings.push(format!("decision-dependency-cycle:{}", decision.id));
        }
    }
    {
        let expected = IG1_DECISIONS.into_iter().collect::<BTreeSet<_>>();
        for missing in expected.difference(&seen) {
            findings.push(format!("decision-set-missing:{missing}"));
        }
        for extra in seen.difference(&expected) {
            findings.push(format!("decision-set-extra:{extra}"));
        }
        let mut seen_proposals = BTreeSet::new();
        for proposal in &registry.required_proposals {
            if !seen_proposals.insert(proposal.id.as_str()) {
                findings.push(format!("proposal-id-duplicate:{}", proposal.id));
            }
        }
        let expected_proposals = IG1_PROPOSALS.into_iter().collect::<BTreeSet<_>>();
        for missing in expected_proposals.difference(&seen_proposals) {
            findings.push(format!("proposal-set-missing:{missing}"));
        }
        for extra in seen_proposals.difference(&expected_proposals) {
            findings.push(format!("proposal-set-extra:{extra}"));
        }
    }
    for decision in &registry.decisions {
        if decision.revision == 0 {
            findings.push(format!("decision-revision-invalid:{}", decision.id));
        }
        if let Some((required_roles, dependencies)) = decision_policy(&decision.id) {
            if !same_strings(&decision.required_roles, required_roles) {
                findings.push(format!(
                    "decision-required-roles-policy-mismatch:{}",
                    decision.id
                ));
            }
            if !same_strings(&decision.depends_on_decisions, dependencies) {
                findings.push(format!(
                    "decision-dependencies-policy-mismatch:{}",
                    decision.id
                ));
            }
        }
        validate_authors(
            "decision",
            &decision.id,
            &decision.proposal_authors,
            &mut findings,
        );
    }
    for proposal in &registry.required_proposals {
        if proposal.revision == 0 {
            findings.push(format!("proposal-revision-invalid:{}", proposal.id));
        }
        if let Some(required_roles) = proposal_policy(&proposal.id) {
            if !same_strings(&proposal.required_roles, required_roles) {
                findings.push(format!(
                    "proposal-required-roles-policy-mismatch:{}",
                    proposal.id
                ));
            }
        }
        validate_authors(
            "proposal",
            &proposal.id,
            &proposal.proposal_authors,
            &mut findings,
        );
    }
    let mut bound_roles = BTreeSet::new();
    let mut role_handles = BTreeMap::new();
    let mut assigned_handles = BTreeMap::<String, &str>::new();
    for binding in &registry.role_bindings {
        if !bound_roles.insert(binding.role.as_str()) {
            findings.push(format!("role-binding-duplicate:{}", binding.role));
        }
        for handle in [&binding.owner_handle, &binding.backup_handle] {
            let normalized = normalized_handle(handle);
            if normalized.is_empty() {
                findings.push(format!("role-binding-handle-empty:{}", binding.role));
            } else if handle != "UNASSIGNED" && is_non_human_handle(handle) {
                findings.push(format!(
                    "role-binding-actor-not-human:{}:{handle}",
                    binding.role
                ));
            }
            if handle != "UNASSIGNED" {
                if let Some(previous_role) =
                    assigned_handles.insert(normalized.clone(), &binding.role)
                {
                    if previous_role != binding.role {
                        let owner_norm = normalized_handle(&automation.accountable_owner_handle);
                        let multi_ok = automation.single_owner_mode() && normalized == owner_norm;
                        if !multi_ok {
                            findings.push(format!(
                                "role-binding-handle-reused:{normalized}:{previous_role}:{}",
                                binding.role
                            ));
                        }
                    }
                }
            }
        }
        if binding.owner_handle != "UNASSIGNED"
            && normalized_handle(&binding.owner_handle) == normalized_handle(&binding.backup_handle)
        {
            // single-owner automation may leave backup UNASSIGNED; same owner/backup still banned
            findings.push(format!(
                "role-binding-owner-backup-same:{}:{}",
                binding.role, binding.owner_handle
            ));
        }
        role_handles.insert(
            binding.role.as_str(),
            [
                binding.owner_handle.as_str(),
                binding.backup_handle.as_str(),
            ],
        );
    }
    let governance_roles = GOVERNANCE_ROLES.into_iter().collect::<BTreeSet<_>>();
    for missing in governance_roles.difference(&bound_roles) {
        findings.push(format!("role-binding-missing:{missing}"));
    }
    for extra in bound_roles.difference(&governance_roles) {
        findings.push(format!("role-binding-extra:{extra}"));
    }
    for decision in &registry.decisions {
        for role in &decision.required_roles {
            if !bound_roles.contains(role.as_str()) {
                findings.push(format!("required-role-undefined:{}:{role}", decision.id));
            }
        }
    }
    for proposal in &registry.required_proposals {
        for role in &proposal.required_roles {
            if !bound_roles.contains(role.as_str()) {
                findings.push(format!("proposal-role-undefined:{}:{role}", proposal.id));
            }
        }
        validate_subject(
            root,
            &proposal.id,
            &proposal.subject_ref,
            &proposal.subject_sha256,
            "proposal-",
            &mut findings,
        );
        match fs::read_to_string(root.join(&proposal.subject_ref)) {
            Ok(subject) if declared_proposal_status(&subject) == Some(proposal.status.as_str()) => {
            }
            Ok(_) => findings.push(format!(
                "proposal-subject-status-mismatch:{}:{}",
                proposal.id, proposal.status
            )),
            Err(_) => {}
        }
    }
    let allowed_proposal_statuses = [
        "DRAFT",
        "APPROVED",
        "REJECTED",
        "NEEDS_REVISION",
        "SUPERSEDED",
    ];
    for proposal in &registry.required_proposals {
        if !allowed_proposal_statuses.contains(&proposal.status.as_str()) {
            findings.push(format!(
                "proposal-status-invalid:{}:{}",
                proposal.id, proposal.status
            ));
        }
        validate_approval_records(
            ApprovalValidation {
                id: &proposal.id,
                status: &proposal.status,
                required_roles: &proposal.required_roles,
                proposal_authors: &proposal.proposal_authors,
                approvals: &proposal.approvals,
                subject_revision: proposal.revision,
                subject_sha256: &proposal.subject_sha256,
                finding_prefix: "proposal-",
            },
            &role_handles,
            &automation,
            now_epoch_seconds,
            &mut findings,
        );
        if proposal.status == "APPROVED"
            && automation
                .dual_control_proposal_ids
                .iter()
                .any(|id| id == &proposal.id)
        {
            enforce_dual_control(
                &proposal.id,
                "proposal-",
                &proposal.approvals,
                &mut findings,
            );
        }
    }
    let allowed_statuses = [
        "AWAITING_APPROVAL",
        "APPROVED",
        "REJECTED",
        "NEEDS_REVISION",
        "SUPERSEDED",
    ];
    for decision in &registry.decisions {
        if !allowed_statuses.contains(&decision.status.as_str()) {
            findings.push(format!(
                "decision-status-invalid:{}:{}",
                decision.id, decision.status
            ));
        }
        validate_approval_records(
            ApprovalValidation {
                id: &decision.id,
                status: &decision.status,
                required_roles: &decision.required_roles,
                proposal_authors: &decision.proposal_authors,
                approvals: &decision.approvals,
                subject_revision: decision.revision,
                subject_sha256: &decision.subject_sha256,
                finding_prefix: "",
            },
            &role_handles,
            &automation,
            now_epoch_seconds,
            &mut findings,
        );
        if decision.status == "APPROVED"
            && automation
                .dual_control_decision_ids
                .iter()
                .any(|id| id == &decision.id)
        {
            enforce_dual_control(&decision.id, "", &decision.approvals, &mut findings);
        }
        if decision.status == "APPROVED" {
            for dependency in &decision.depends_on_decisions {
                match decision_statuses.get(dependency.as_str()) {
                    Some(status) if *status == "APPROVED" => {}
                    Some(_) => findings.push(format!(
                        "decision-dependency-not-approved:{}:{dependency}",
                        decision.id
                    )),
                    None => findings.push(format!(
                        "decision-dependency-unknown:{}:{dependency}",
                        decision.id
                    )),
                }
            }
            if decision.id == "D-06b" && decision.evidence_refs.is_empty() {
                findings.push("decision-evidence-required:D-06b".into());
            }
        }
        for dependency in &decision.depends_on_decisions {
            if !decision_statuses.contains_key(dependency.as_str()) {
                findings.push(format!(
                    "decision-dependency-unknown:{}:{dependency}",
                    decision.id
                ));
            }
        }
        let mut d06_evidence_kinds = BTreeSet::new();
        for evidence_ref in &decision.evidence_refs {
            if let Some(kind) = validate_evidence_reference(
                root,
                &decision.id,
                evidence_ref,
                decision.id == "D-06b",
                evidence_schema.as_ref(),
                &mut findings,
            ) {
                d06_evidence_kinds.insert(kind);
            }
        }
        if decision.id == "D-06b" && decision.status == "APPROVED" {
            for required in D06EvidenceKind::ALL {
                if !d06_evidence_kinds.contains(&required) {
                    findings.push(format!(
                        "decision-evidence-kind-missing:D-06b:{}",
                        required.label()
                    ));
                }
            }
        }
    }
    for decision in &registry.decisions {
        validate_subject(
            root,
            &decision.id,
            &decision.subject_ref,
            &decision.subject_sha256,
            "",
            &mut findings,
        );
    }
    findings.sort();
    findings.dedup();
    let registry_valid = findings.is_empty();
    let mut blockers = Vec::new();
    if registry.registry_status != "ACTIVE" {
        blockers.push(format!("registry-not-active:{}", registry.registry_status));
    }
    let ig1_exit_decisions = IG1_EXIT_DECISIONS.into_iter().collect::<BTreeSet<_>>();
    for decision in &registry.decisions {
        if !ig1_exit_decisions.contains(decision.id.as_str()) {
            continue;
        }
        if decision.status != "APPROVED" {
            blockers.push(format!(
                "decision-not-approved:{}:{}",
                decision.id, decision.status
            ));
        }
        for role in &decision.required_roles {
            if let Some(handles) = role_handles.get(role.as_str()) {
                // single-owner 模式：仅要求 owner 绑定；backup 可为 UNASSIGNED
                let owner = handles[0];
                let unassigned = if automation.single_owner_mode() {
                    owner == "UNASSIGNED"
                } else {
                    handles.contains(&"UNASSIGNED")
                };
                if unassigned {
                    blockers.push(format!("role-unassigned:{role}"));
                }
            }
        }
    }
    for proposal in &registry.required_proposals {
        if proposal.status != "APPROVED" {
            blockers.push(format!(
                "proposal-not-approved:{}:{}",
                proposal.id, proposal.status
            ));
        }
        for role in &proposal.required_roles {
            if let Some(handles) = role_handles.get(role.as_str()) {
                let owner = handles[0];
                let unassigned = if automation.single_owner_mode() {
                    owner == "UNASSIGNED"
                } else {
                    handles.contains(&"UNASSIGNED")
                };
                if unassigned {
                    blockers.push(format!("role-unassigned:{role}"));
                }
            }
        }
    }
    // INFRA-009 真实 API readback 仍未实现；strict 模式 gate_ready 永假。
    // single_accountable_owner 可配置放宽（设计决策自动化车道）。
    let trusted_review_readback = false;
    if !trusted_review_readback && automation.gate_ready_requires_external_readback {
        blockers.push("external-review-readback-not-implemented:INFRA-009".into());
    }
    blockers.sort();
    blockers.dedup();
    let design_gate_ready = registry_valid
        && !registry.decisions.is_empty()
        && (!automation.gate_ready_requires_external_readback || trusted_review_readback)
        && blockers.is_empty();
    // 生产门禁：显式分离，防止把 design gate_ready 误读为生产获批（仲裁 RISKS）
    // 始终要求可信 external readback；与 design 是否放宽无关。
    let production_gate_ready = registry_valid && trusted_review_readback && blockers.is_empty();
    Ok(Report {
        registry_valid,
        gate_ready: design_gate_ready,
        production_gate_ready,
        decision_count: registry.decisions.len(),
        proposal_count: registry.required_proposals.len(),
        trusted_review_readback,
        automation_mode: automation.mode.clone(),
        standing_authorization: automation.standing_authorization,
        ai_may_invoke_auto: automation.ai_may_invoke_auto,
        approval_validity_days: automation.approval_validity_days,
        require_gates_green: automation.require_gates_green,
        findings,
        blockers,
    })
}

fn enforce_dual_control(
    id: &str,
    finding_prefix: &str,
    approvals: &[Approval],
    findings: &mut Vec<String>,
) {
    let humans: BTreeSet<_> = approvals
        .iter()
        .filter(|a| !is_non_human_handle(&a.approver_handle))
        .map(|a| normalized_handle(&a.approver_handle))
        .collect();
    if humans.len() < 2 {
        findings.push(format!(
            "{finding_prefix}dual-control-insufficient:{id}:need_2_distinct_humans"
        ));
    }
}

fn is_non_human_handle(handle: &str) -> bool {
    crate::human_actor::is_non_human_handle(handle)
}

fn normalized_handle(handle: &str) -> String {
    crate::human_actor::normalized_handle(handle)
}

fn same_strings(actual: &[String], expected: &[&str]) -> bool {
    actual
        .iter()
        .map(String::as_str)
        .eq(expected.iter().copied())
}

fn validate_authors(kind: &str, id: &str, authors: &[String], findings: &mut Vec<String>) {
    if authors.is_empty() {
        findings.push(format!("{kind}-authors-empty:{id}"));
        return;
    }
    let mut seen = BTreeSet::new();
    for author in authors {
        let normalized = normalized_handle(author);
        if normalized.is_empty() {
            findings.push(format!("{kind}-author-empty:{id}"));
        } else if !seen.insert(normalized) {
            findings.push(format!("{kind}-author-duplicate:{id}:{author}"));
        }
    }
}

fn decision_policy(id: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    Some(match id {
        "D-01" => (&["Architecture Owner"], &[]),
        "D-02" | "D-03" => (&["Architecture Owner"], &["D-01"]),
        "D-04" => (&["Data Owner", "Architecture Owner"], &["D-01"]),
        "D-05" => (&["Data Owner", "Architecture Owner"], &["D-03", "D-04"]),
        "D-06a" => (&["Data Owner", "Architecture Owner"], &["D-01"]),
        "D-06b" => (&["Data Owner", "Architecture Owner"], &["D-06a"]),
        "D-07" => (&["Data Owner"], &["D-06a"]),
        "D-08" => (
            &["Data Owner", "Security Owner", "Architecture Owner"],
            &["D-04", "D-05"],
        ),
        "D-09" => (&["Data Owner"], &["D-04"]),
        "D-10" => (&["Data Owner", "Release Owner", "Risk Owner"], &[]),
        "D-11" => (&["Data Owner", "Release Owner", "Risk Owner"], &["D-10"]),
        "D-12" => (&["Security Owner"], &[]),
        "D-13" => (&["Security Owner"], &["D-12"]),
        "D-14" => (&["Release Owner", "Security Owner"], &["D-12", "D-13"]),
        "D-15" => (&["Maintainer"], &[]),
        "D-16" => (
            &["Maintainer", "Security Owner", "Release Owner"],
            &["D-12"],
        ),
        "D-17" => (&["Data Owner", "Security Owner"], &["D-04", "D-13"]),
        "D-18" => (
            &[
                "Maintainer",
                "Security Owner",
                "Risk Owner",
                "Production Operator",
            ],
            &["D-16"],
        ),
        "D-19" => (
            &["Architecture Owner", "Data Owner"],
            &["D-01", "D-03", "D-04", "D-05"],
        ),
        _ => return None,
    })
}

fn proposal_policy(id: &str) -> Option<&'static [&'static str]> {
    Some(match id {
        "P-01-control-plane" | "P-02-lifecycle" | "P-03-storage-contracts-v2" => {
            &["Architecture Owner"]
        }
        "P-04-data-authority" => &["Data Owner", "Architecture Owner", "Security Owner"],
        "P-05-taos" => &["Data Owner", "Architecture Owner"],
        "P-06-evidence" => &["Security Owner", "Release Owner"],
        "P-07-integration-harness" => &["Maintainer", "Security Owner", "Data Owner"],
        "P-08-release" => &["Release Owner", "Security Owner"],
        "P-09-l2-service" => &["Architecture Owner", "Data Owner"],
        _ => return None,
    })
}

fn is_github_review_url(value: &str) -> bool {
    const PREFIX: &str = "https://github.com/xhyperium/xhyper.rs/pull/";
    let Some(rest) = value.strip_prefix(PREFIX) else {
        return false;
    };
    let Some((pull, review)) = rest.split_once("#pullrequestreview-") else {
        return false;
    };
    positive_integer(pull) && positive_integer(review)
}

fn is_github_issue_url(value: &str) -> bool {
    const PREFIX: &str = "https://github.com/xhyperium/xhyper.rs/issues/";
    value.strip_prefix(PREFIX).is_some_and(positive_integer)
}

fn is_machine_ticket_url(value: &str) -> bool {
    value.starts_with("machine://xtask/approval-auto/ticket/") && value.len() > 40
}

fn is_machine_review_url(value: &str) -> bool {
    value.starts_with("machine://xtask/approval-auto/attestation/") && value.len() > 50
}

fn positive_integer(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|number| number > 0)
}

fn is_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let number = |range: std::ops::Range<usize>| -> Option<u32> {
        std::str::from_utf8(&bytes[range]).ok()?.parse().ok()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
        number(17..19),
    ) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days).contains(&day)
}

fn validate_evidence_reference(
    root: &Path,
    decision_id: &str,
    evidence: &EvidenceReference,
    require_d06b_compatibility: bool,
    schema: Option<&serde_json::Value>,
    findings: &mut Vec<String>,
) -> Option<D06EvidenceKind> {
    if !evidence.path.ends_with(".evidence.json") {
        findings.push(format!(
            "decision-evidence-not-record:{decision_id}:{}",
            evidence.path
        ));
        return None;
    }
    let Some(path) = repo_file(root, &evidence.path) else {
        findings.push(format!(
            "decision-evidence-invalid:{decision_id}:{}",
            evidence.path
        ));
        return None;
    };
    let Ok(bytes) = fs::read(&path) else {
        findings.push(format!(
            "decision-evidence-invalid:{decision_id}:{}",
            evidence.path
        ));
        return None;
    };
    if sha256_hex(&bytes) != evidence.sha256 {
        findings.push(format!(
            "decision-evidence-hash-mismatch:{decision_id}:{}",
            evidence.path
        ));
        return None;
    }
    let Ok(record) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        findings.push(format!(
            "decision-evidence-json-invalid:{decision_id}:{}",
            evidence.path
        ));
        return None;
    };
    if !schema.is_some_and(|schema| json_schema_matches(&record, schema)) {
        findings.push(format!(
            "decision-evidence-schema-invalid:{decision_id}:{}",
            evidence.path
        ));
        return None;
    }
    if require_d06b_compatibility {
        if let Some(kind) = classify_d06b_evidence(&record) {
            return Some(kind);
        }
        findings.push(format!(
            "decision-evidence-semantic-invalid:D-06b:{}",
            evidence.path
        ));
    }
    None
}

fn repo_file(root: &Path, reference: &str) -> Option<PathBuf> {
    let relative = Path::new(reference);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    let mut lexical_path = root.to_path_buf();
    for component in relative.components() {
        lexical_path.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&lexical_path).ok()?;
        if metadata.file_type().is_symlink() {
            return None;
        }
    }
    let canonical_root = fs::canonicalize(root).ok()?;
    let canonical_path = fs::canonicalize(lexical_path).ok()?;
    (canonical_path.starts_with(canonical_root) && canonical_path.is_file())
        .then_some(canonical_path)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn load_evidence_schema(root: &Path, findings: &mut Vec<String>) -> Option<serde_json::Value> {
    let path = "schemas/jsonschema/evidence-record.schema.json";
    let Some(path) = repo_file(root, path) else {
        findings.push(
            "evidence-schema-unreadable:schemas/jsonschema/evidence-record.schema.json".into(),
        );
        return None;
    };
    match fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        Some(schema) => Some(schema),
        None => {
            findings.push(
                "evidence-schema-invalid:schemas/jsonschema/evidence-record.schema.json".into(),
            );
            None
        }
    }
}

fn json_schema_matches(value: &serde_json::Value, schema: &serde_json::Value) -> bool {
    if let Some(types) = schema.get("type") {
        let type_matches = match types {
            serde_json::Value::String(kind) => json_type_matches(value, kind),
            serde_json::Value::Array(kinds) => kinds
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|kind| json_type_matches(value, kind)),
            _ => false,
        };
        if !type_matches {
            return false;
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(serde_json::Value::as_array) {
        if !allowed.contains(value) {
            return false;
        }
    }
    if let Some(text) = value.as_str() {
        if schema
            .get("minLength")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|minimum| text.chars().count() < minimum as usize)
        {
            return false;
        }
        if schema
            .get("pattern")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|pattern| !known_pattern_matches(text, pattern))
        {
            return false;
        }
        if schema
            .get("format")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|format| !known_format_matches(text, format))
        {
            return false;
        }
    }
    if schema
        .get("minimum")
        .and_then(serde_json::Value::as_i64)
        .is_some_and(|minimum| value.as_i64().is_none_or(|number| number < minimum))
    {
        return false;
    }
    if let Some(object) = value.as_object() {
        let properties = schema
            .get("properties")
            .and_then(serde_json::Value::as_object);
        if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
            if required
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|field| !object.contains_key(field))
            {
                return false;
            }
        }
        for (field, field_value) in object {
            if let Some(field_schema) = properties.and_then(|known| known.get(field)) {
                if !json_schema_matches(field_value, field_schema) {
                    return false;
                }
                continue;
            }
            match schema.get("additionalProperties") {
                Some(serde_json::Value::Bool(false)) => return false,
                Some(additional @ serde_json::Value::Object(_))
                    if !json_schema_matches(field_value, additional) =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }
    if let Some(items) = value.as_array() {
        if let Some(item_schema) = schema.get("items") {
            if items
                .iter()
                .any(|item| !json_schema_matches(item, item_schema))
            {
                return false;
            }
        }
    }
    true
}

fn json_type_matches(value: &serde_json::Value, kind: &str) -> bool {
    match kind {
        "null" => value.is_null(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        _ => false,
    }
}

fn known_pattern_matches(value: &str, pattern: &str) -> bool {
    match pattern {
        "^[0-9]+\\.[0-9]+\\.[0-9]+(-[A-Za-z0-9.-]+)?$" => {
            let (core, suffix) = value
                .split_once('-')
                .map_or((value, None), |(core, suffix)| (core, Some(suffix)));
            core.split('.').count() == 3
                && core
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                && suffix.is_none_or(|suffix| {
                    !suffix.is_empty()
                        && suffix
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
                })
        }
        "^[0-9a-f]{7,40}$" => {
            (7..=40).contains(&value.len())
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }
        _ => false,
    }
}

fn known_format_matches(value: &str, format: &str) -> bool {
    match format {
        "date-time" => is_rfc3339_utc(value),
        "date" => is_iso_date(value),
        "uri" => value
            .split_once("://")
            .is_some_and(|(scheme, rest)| !scheme.is_empty() && !rest.is_empty()),
        _ => false,
    }
}

fn is_iso_date(value: &str) -> bool {
    let candidate = format!("{value}T00:00:00Z");
    is_rfc3339_utc(&candidate)
}

fn classify_d06b_evidence(record: &serde_json::Value) -> Option<D06EvidenceKind> {
    let service_is_bound = record
        .pointer("/environment/services")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|services| {
            services.iter().any(|service| {
                service
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| name.eq_ignore_ascii_case("tdengine"))
                    && service
                        .get("image_digest")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(is_sha256_digest)
                    && service
                        .get("client_version")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.is_empty())
            })
        });
    let criterion = record
        .get("acceptance_criterion")
        .and_then(serde_json::Value::as_str)
        .map(str::to_ascii_lowercase)?;
    let command = record
        .get("command")
        .and_then(serde_json::Value::as_str)
        .map(str::to_ascii_lowercase)?;
    let commit_is_full = record
        .get("commit")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|commit| {
            commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    let common = record
        .get("work_package")
        .and_then(serde_json::Value::as_str)
        == Some("INFRA-012")
        && criterion.contains("d-06b")
        && !command.contains("cargo check")
        && record.get("result").and_then(serde_json::Value::as_str) == Some("PASS")
        && record.get("exit_code").and_then(serde_json::Value::as_i64) == Some(0)
        && record
            .get("dirty_state")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
        && record
            .pointer("/verifier/status")
            .and_then(serde_json::Value::as_str)
            == Some("PASS")
        && service_is_bound
        && commit_is_full;
    if !common {
        return None;
    }
    let runtime_test = command.contains("cargo test")
        || command.contains("nextest")
        || command.contains("roundtrip")
        || command.contains("harness");
    let oracle_passes = record
        .pointer("/data_oracle/precision_ok")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
        && record
            .pointer("/data_oracle/count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0);
    if criterion.contains("roundtrip") && runtime_test && oracle_passes {
        if criterion.contains("rest") && command.contains("rest") {
            return Some(D06EvidenceKind::RestRoundtrip);
        }
        if criterion.contains("native") && command.contains("native") {
            return Some(D06EvidenceKind::NativeRoundtrip);
        }
    }
    let safety_criterion = ["sanitizer", "fuzz", "ffi-safety", "ffi safety"]
        .iter()
        .any(|marker| criterion.contains(marker));
    let safety_command = ["sanitizer", "cargo fuzz", "fuzz", "miri", "asan"]
        .iter()
        .any(|marker| command.contains(marker));
    (criterion.contains("native")
        && command.contains("native")
        && safety_criterion
        && safety_command)
        .then_some(D06EvidenceKind::NativeSafety)
}

fn is_sha256_digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn declared_proposal_status(subject: &str) -> Option<&'static str> {
    let status_line = subject
        .lines()
        .take(20)
        .find(|line| line.contains("状态") || line.to_ascii_lowercase().contains("status"))?
        .to_ascii_uppercase();
    if status_line.contains("NEEDS_REVISION") || status_line.contains("NEEDS REVISION") {
        Some("NEEDS_REVISION")
    } else if status_line.contains("SUPERSEDED") {
        Some("SUPERSEDED")
    } else if status_line.contains("REJECTED") {
        Some("REJECTED")
    } else if status_line.contains("APPROVED") {
        Some("APPROVED")
    } else if status_line.contains("DRAFT") {
        Some("DRAFT")
    } else {
        None
    }
}

fn validate_subject(
    root: &Path,
    id: &str,
    subject_ref: &str,
    subject_sha256: &str,
    finding_prefix: &str,
    findings: &mut Vec<String>,
) {
    let Some(subject_path) = repo_file(root, subject_ref) else {
        findings.push(format!(
            "{finding_prefix}subject-ref-unsafe:{id}:{subject_ref}"
        ));
        return;
    };
    match fs::read(subject_path) {
        Ok(bytes) => {
            let actual = sha256_hex(&bytes);
            if actual != subject_sha256 {
                findings.push(format!(
                    "{finding_prefix}subject-hash-mismatch:{id}:{subject_ref}"
                ));
            }
        }
        Err(error) => findings.push(format!(
            "{finding_prefix}subject-ref-unreadable:{id}:{subject_ref}:{error}"
        )),
    }
}

fn validate_approval_records<'a>(
    subject: ApprovalValidation<'_>,
    role_handles: &BTreeMap<&'a str, [&'a str; 2]>,
    automation: &ApprovalAutomation,
    now_epoch_seconds: i64,
    findings: &mut Vec<String>,
) {
    let ApprovalValidation {
        id,
        status,
        required_roles,
        proposal_authors,
        approvals,
        subject_revision,
        subject_sha256,
        finding_prefix,
    } = subject;
    let multi = automation.single_owner_mode();
    let owner_norm = normalized_handle(&automation.accountable_owner_handle);
    let mut approval_handles = BTreeSet::new();
    let mut approved_roles = BTreeSet::new();
    for approval in approvals {
        if !required_roles.contains(&approval.approver_role) {
            findings.push(format!(
                "{finding_prefix}approval-role-not-required:{id}:{}",
                approval.approver_role
            ));
        }
        let normalized_approver = normalized_handle(&approval.approver_handle);
        // strict: one handle once per subject; single-owner: same owner may cover many roles
        if !approval_handles.insert(normalized_approver.clone()) {
            let allow_dup = multi && normalized_approver == owner_norm;
            if !allow_dup {
                findings.push(format!(
                    "{finding_prefix}approval-handle-duplicate:{id}:{}",
                    approval.approver_handle
                ));
            }
        }
        approved_roles.insert(approval.approver_role.as_str());
        if is_non_human_handle(&approval.approver_handle) {
            findings.push(format!(
                "{finding_prefix}approval-actor-not-human:{id}:{}",
                approval.approver_handle
            ));
        }
        if proposal_authors
            .iter()
            .any(|author| normalized_handle(author) == normalized_approver)
        {
            // AI_AGENT 作者 + 自然人 owner 审批：允许；作者与 approver 同为 AI 仍禁止
            if !multi || normalized_approver != owner_norm {
                findings.push(format!(
                    "{finding_prefix}approval-self-review:{id}:{}",
                    approval.approver_handle
                ));
            }
        }
        match role_handles.get(approval.approver_role.as_str()) {
            Some(handles)
                if handles.iter().any(|handle| {
                    *handle != "UNASSIGNED" && normalized_handle(handle) == normalized_approver
                }) => {}
            _ => findings.push(format!(
                "{finding_prefix}approval-role-binding-mismatch:{id}:{}:{}",
                approval.approver_role, approval.approver_handle
            )),
        }
        if approval.decision != "APPROVED" {
            findings.push(format!(
                "{finding_prefix}approval-verdict-invalid:{id}:{}",
                approval.decision
            ));
        }
        if approval.scope != id {
            findings.push(format!(
                "{finding_prefix}approval-scope-mismatch:{id}:{}",
                approval.scope
            ));
        }
        if approval.reason.trim().is_empty() {
            findings.push(format!("{finding_prefix}approval-reason-empty:{id}"));
        }
        let ticket_ok = is_github_issue_url(&approval.ticket_url)
            || (automation.machine_attestation_accepted
                && is_machine_ticket_url(&approval.ticket_url));
        if !ticket_ok {
            findings.push(format!(
                "{finding_prefix}approval-ticket-url-invalid:{id}:{}",
                approval.ticket_url
            ));
        }
        let review_ok = is_github_review_url(&approval.review_url)
            || (automation.machine_attestation_accepted
                && is_machine_review_url(&approval.review_url));
        if !review_ok {
            findings.push(format!(
                "{finding_prefix}approval-review-url-invalid:{id}:{}",
                approval.review_url
            ));
        }
        if approval.reviewed_commit.len() != 40
            || !approval
                .reviewed_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            findings.push(format!(
                "{finding_prefix}approval-commit-invalid:{id}:{}",
                approval.reviewed_commit
            ));
        }
        if approval.subject_revision != subject_revision {
            findings.push(format!(
                "{finding_prefix}approval-subject-revision-mismatch:{id}"
            ));
        }
        if approval.subject_sha256 != subject_sha256 {
            findings.push(format!(
                "{finding_prefix}approval-subject-hash-mismatch:{id}"
            ));
        }
        if !is_rfc3339_utc(&approval.approved_at) {
            findings.push(format!(
                "{finding_prefix}approval-time-invalid:{id}:{}",
                approval.approved_at
            ));
        }
        let approved_epoch = rfc3339_utc_epoch_seconds(&approval.approved_at);
        let valid_until_epoch = rfc3339_utc_epoch_seconds(&approval.valid_until);
        if valid_until_epoch.is_none()
            || approved_epoch.is_none()
            || valid_until_epoch <= approved_epoch
            || valid_until_epoch.is_some_and(|valid_until| valid_until <= now_epoch_seconds)
        {
            findings.push(format!(
                "{finding_prefix}approval-valid-until-invalid:{id}:{}",
                approval.valid_until
            ));
        }
        if !approval.independence_check {
            findings.push(format!("{finding_prefix}approval-independence-failed:{id}"));
        }
    }
    if status == "APPROVED" {
        if approvals.is_empty() {
            findings.push(format!("{finding_prefix}approved-without-approvals:{id}"));
        }
        // 机器/AI 不得把状态写成 APPROVED 却无合格自然人审批
        let human_approvals = approvals
            .iter()
            .filter(|approval| !is_non_human_handle(&approval.approver_handle))
            .count();
        if human_approvals == 0 {
            findings.push(format!("{finding_prefix}machine-approved-forbidden:{id}"));
        }
        for role in required_roles {
            if !approved_roles.contains(role.as_str()) {
                findings.push(format!("{finding_prefix}approval-role-missing:{id}:{role}"));
            }
        }
        // 每个 APPROVED 记录必须具备核心 provenance 字段（空串即失败）
        for approval in approvals {
            for (field, value) in [
                ("approver_handle", approval.approver_handle.as_str()),
                ("approver_role", approval.approver_role.as_str()),
                ("ticket_url", approval.ticket_url.as_str()),
                ("review_url", approval.review_url.as_str()),
                ("reviewed_commit", approval.reviewed_commit.as_str()),
                ("approved_at", approval.approved_at.as_str()),
                ("valid_until", approval.valid_until.as_str()),
                ("reason", approval.reason.as_str()),
            ] {
                if value.trim().is_empty() {
                    findings.push(format!(
                        "{finding_prefix}approval-required-field-empty:{id}:{field}"
                    ));
                }
            }
        }
    }
}

fn rfc3339_utc_epoch_seconds(value: &str) -> Option<i64> {
    if !is_rfc3339_utc(value) {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;

    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    Some(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn has_dependency_cycle<'a>(
    id: &'a str,
    decisions: &BTreeMap<&'a str, &'a Decision>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> bool {
    if visiting.contains(id) {
        return true;
    }
    if !visited.insert(id) {
        return false;
    }
    let Some(decision) = decisions.get(id) else {
        return false;
    };
    visiting.insert(id);
    let cyclic = decision
        .depends_on_decisions
        .iter()
        .any(|dependency| has_dependency_cycle(dependency.as_str(), decisions, visiting, visited));
    visiting.remove(id);
    cyclic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_schema_accepts_fixture_and_rejects_unknown_fields() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let schema: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("schemas/jsonschema/evidence-record.schema.json")).unwrap(),
        )
        .unwrap();
        let mut record: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("schemas/jsonschema/fixtures/evidence-record.valid.evidence.json"))
                .unwrap(),
        )
        .unwrap();
        assert!(json_schema_matches(&record, &schema));
        record["unreviewed_override"] = serde_json::json!(true);
        assert!(!json_schema_matches(&record, &schema));
    }

    #[cfg(unix)]
    #[test]
    fn repo_file_rejects_file_and_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("actual")).unwrap();
        fs::write(root.join("actual/evidence.json"), "{}").unwrap();
        symlink(
            root.join("actual/evidence.json"),
            root.join("file-link.json"),
        )
        .unwrap();
        symlink(root.join("actual"), root.join("dir-link")).unwrap();

        assert!(repo_file(root, "file-link.json").is_none());
        assert!(repo_file(root, "dir-link/evidence.json").is_none());
        assert!(repo_file(root, "actual/evidence.json").is_some());
    }
}
