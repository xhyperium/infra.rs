//! Authority Plane 机器评估（Shadow）。
//!
//! 覆盖方案 §23 Authority：
//! - A1：Authority Registry 唯一且机器验证
//! - A2：Risk Tier 与 operation class 自动计算
//! - A3：final-head approval / Risk review 绑定 subject
//! - A4：AI/bot 不计为人类 approval
//!
//! **不**授予 live 合并权；`live_ssot=false` 的 target registry 不得作授权输入。

use crate::human_actor::{is_non_human_actor, normalized_handle};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const TARGET_REGISTRY_REL: &str = ".architecture/authority-registry.target.toml";
const LIVE_REGISTRY_REL: &str = ".architecture/authority-registry.toml";
const RISK_RULES_REL: &str = ".github/ai-native/authority-risk-rules.json";

// ---------------------------------------------------------------------------
// Registry (A1)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RegistryFile {
    schema_version: u32,
    status: String,
    live_ssot: bool,
    #[serde(default)]
    rfc: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    authority: Vec<AuthorityEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct AuthorityEntry {
    id: String,
    class: String,
    #[serde(default)]
    canonical: Option<String>,
    #[serde(default)]
    canonical_roots: Vec<String>,
    owner_role: String,
    #[serde(default)]
    #[allow(dead_code)]
    validator: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    projections: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    immutability: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegistryReport {
    pub path: String,
    pub schema_version: u32,
    pub status: String,
    pub live_ssot: bool,
    pub entry_count: usize,
    pub ids: Vec<String>,
    pub findings: Vec<String>,
    pub ok: bool,
}

pub fn validate_registry_at(root: &Path, path: &Path, allow_live: bool) -> Result<RegistryReport> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    validate_registry_text(&rel, &text, allow_live)
}

pub fn validate_registry_text(
    path_label: &str,
    text: &str,
    allow_live: bool,
) -> Result<RegistryReport> {
    let raw: RegistryFile = toml::from_str(text).context("authority-registry TOML deserialize")?;
    let mut findings = Vec::new();

    if raw.schema_version != 1 {
        findings.push(format!(
            "registry-schema-unsupported:{} (supported=1)",
            raw.schema_version
        ));
    }
    if raw.authority.is_empty() {
        findings.push("registry-empty".into());
    }
    if raw.live_ssot && !allow_live {
        findings.push("registry-live-ssot-not-allowed-without-rfc-effective".into());
    }
    if !raw.live_ssot {
        let status = raw.status.to_ascii_lowercase();
        if !(status.contains("shadow") || status.contains("proposed") || status.contains("target"))
        {
            findings.push(format!(
                "registry-status-inconsistent-with-live_ssot=false:{}",
                raw.status
            ));
        }
    }

    let mut seen_ids = BTreeSet::new();
    let mut seen_canonical = BTreeMap::<String, String>::new();
    let mut ids = Vec::new();

    for entry in &raw.authority {
        ids.push(entry.id.clone());
        if entry.id.trim().is_empty() {
            findings.push("authority-id-empty".into());
            continue;
        }
        if !seen_ids.insert(entry.id.clone()) {
            findings.push(format!("authority-id-duplicate:{}", entry.id));
        }
        if entry.class.trim().is_empty() {
            findings.push(format!("authority-class-empty:{}", entry.id));
        }
        if entry.owner_role.trim().is_empty() {
            findings.push(format!("authority-owner-role-empty:{}", entry.id));
        }
        // AI 不得自任 owner_role 文本中的 actor；owner_role 是角色名，不是 handle。
        // 但禁止 owner_role 直接写成 bot/AI handle 伪装。
        if is_non_human_actor(&entry.owner_role, None)
            && !entry.owner_role.contains(' ')
            && !entry.owner_role.contains("Owner")
            && !entry.owner_role.contains("Maintainer")
            && !entry.owner_role.contains("Custodian")
            && !entry.owner_role.contains("Operator")
            && !entry.owner_role.contains("Commander")
        {
            findings.push(format!(
                "authority-owner-role-looks-non-human:{}:{}",
                entry.id, entry.owner_role
            ));
        }

        let has_canonical = entry
            .canonical
            .as_ref()
            .is_some_and(|c| !c.trim().is_empty());
        let has_roots = !entry.canonical_roots.is_empty();
        if has_canonical == has_roots {
            // XOR：恰好一个
            if !has_canonical && !has_roots {
                findings.push(format!(
                    "authority-canonical-missing:{} (need canonical or canonical_roots)",
                    entry.id
                ));
            } else {
                findings.push(format!(
                    "authority-canonical-ambiguous:{} (use exactly one of canonical|canonical_roots)",
                    entry.id
                ));
            }
        }

        if let Some(c) = &entry.canonical {
            let key = c.trim().to_string();
            if let Some(prev) = seen_canonical.insert(key.clone(), entry.id.clone()) {
                findings.push(format!(
                    "authority-canonical-path-collision:{key}:{}:{prev}",
                    entry.id
                ));
            }
        }
        for root in &entry.canonical_roots {
            let key = format!("root:{}", root.trim().trim_end_matches('/'));
            if let Some(prev) = seen_canonical.insert(key.clone(), entry.id.clone()) {
                findings.push(format!(
                    "authority-canonical-root-collision:{key}:{}:{prev}",
                    entry.id
                ));
            }
        }
    }

    // 目标文件路径硬约束：*.target.toml 必须 live_ssot=false
    if path_label.ends_with(".target.toml") && raw.live_ssot {
        findings.push("target-registry-must-not-set-live_ssot=true".into());
    }

    let _ = (raw.rfc, raw.note); // 保留字段反序列化

    Ok(RegistryReport {
        path: path_label.to_string(),
        schema_version: raw.schema_version,
        status: raw.status,
        live_ssot: raw.live_ssot,
        entry_count: raw.authority.len(),
        ids,
        ok: findings.is_empty(),
        findings,
    })
}

// ---------------------------------------------------------------------------
// Risk rules (A2)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RiskRulesFile {
    schema_version: u32,
    #[serde(default)]
    status: String,
    tier_order: Vec<String>,
    default: RiskDefault,
    #[serde(default)]
    required_approvals: BTreeMap<String, TierApprovalReq>,
    rules: Vec<RiskRule>,
}

#[derive(Debug, Deserialize, Clone)]
struct RiskDefault {
    operation_class: String,
    min_risk_tier: String,
}

#[derive(Debug, Deserialize, Clone)]
struct TierApprovalReq {
    human_approvals: u32,
    #[serde(default)]
    independent_risk_review: bool,
    #[serde(default)]
    auto_merge_allowed: bool,
    #[serde(default)]
    #[allow(dead_code)]
    require_non_author: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct RiskRule {
    id: String,
    path_globs: Vec<String>,
    operation_class: String,
    min_risk_tier: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RiskComputation {
    pub computed_risk_tier: String,
    /// 与最高 Risk Tier 对齐的主 operation class（用于 subject_digest）
    pub primary_operation_class: String,
    pub operation_classes: Vec<String>,
    pub matched_rules: Vec<String>,
    pub required_human_approvals: u32,
    pub require_independent_risk_review: bool,
    pub auto_merge_allowed: bool,
    pub findings: Vec<String>,
}

fn load_risk_rules(root: &Path) -> Result<RiskRulesFile> {
    let path = root.join(RISK_RULES_REL);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let rules: RiskRulesFile = serde_json::from_str(&text).context("parse authority-risk-rules")?;
    if rules.schema_version != 1 {
        bail!(
            "unsupported authority-risk-rules schema_version={}",
            rules.schema_version
        );
    }
    Ok(rules)
}

fn tier_rank(order: &[String], tier: &str) -> Option<usize> {
    order.iter().position(|t| t == tier)
}

fn max_tier(order: &[String], a: &str, b: &str) -> Result<String> {
    let ra = tier_rank(order, a).with_context(|| format!("unknown tier {a}"))?;
    let rb = tier_rank(order, b).with_context(|| format!("unknown tier {b}"))?;
    Ok(if ra >= rb {
        a.to_string()
    } else {
        b.to_string()
    })
}

/// 极简 glob：`**` 跨段，`*` 单段内任意；支持 `prefix/**`、`**/mid/**`、`**/file.rs`。
fn path_matches_glob(path: &str, glob: &str) -> bool {
    let path = path.replace('\\', "/");
    let glob = glob.replace('\\', "/");
    if glob == path {
        return true;
    }
    // 将 glob 转为正则风格的段匹配
    glob_match(&path, &glob)
}

fn glob_match(path: &str, glob: &str) -> bool {
    // 算法：按 `**` 分段，顺序搜索。
    let parts: Vec<&str> = glob.split("**").collect();
    if parts.len() == 1 {
        return segment_glob_match(path, glob);
    }

    let mut rest = path;
    for (i, part) in parts.iter().enumerate() {
        let part = part.trim_matches('/');
        if part.is_empty() {
            if i == parts.len() - 1 {
                return true; // trailing **
            }
            continue;
        }
        if i == 0 && !glob.starts_with("**") {
            // 必须以 part 开头（part 可含 * 段）
            if !starts_with_segment_glob(rest, part) {
                return false;
            }
            rest = strip_prefix_segment_glob(rest, part).unwrap_or("");
            rest = rest.trim_start_matches('/');
            continue;
        }
        // 在 rest 中寻找 part
        if i == parts.len() - 1 && !glob.ends_with("**") {
            // 最后一段必须匹配后缀
            return ends_with_segment_glob(rest, part);
        }
        // 中间段：在任意深度找到
        if let Some(idx) = find_segment_glob(rest, part) {
            rest = &rest[idx..];
            rest = strip_prefix_segment_glob(rest, part).unwrap_or("");
            rest = rest.trim_start_matches('/');
        } else {
            return false;
        }
    }
    true
}

fn segment_glob_match(path: &str, pattern: &str) -> bool {
    let path_segs: Vec<&str> = if path.is_empty() {
        vec![]
    } else {
        path.split('/').collect()
    };
    let pat_segs: Vec<&str> = if pattern.is_empty() {
        vec![]
    } else {
        pattern.split('/').collect()
    };
    if path_segs.len() != pat_segs.len() {
        return false;
    }
    path_segs
        .iter()
        .zip(pat_segs.iter())
        .all(|(p, g)| wildcard_match(p, g))
}

fn starts_with_segment_glob(path: &str, pattern: &str) -> bool {
    strip_prefix_segment_glob(path, pattern).is_some()
}

fn strip_prefix_segment_glob<'a>(path: &'a str, pattern: &str) -> Option<&'a str> {
    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    if pat_segs.is_empty() {
        return Some(path);
    }
    let path_segs: Vec<&str> = path.split('/').collect();
    if path_segs.len() < pat_segs.len() {
        return None;
    }
    for (p, g) in path_segs.iter().zip(pat_segs.iter()) {
        if !wildcard_match(p, g) {
            return None;
        }
    }
    let mut used = 0usize;
    for (i, _) in pat_segs.iter().enumerate() {
        used += path_segs[i].len();
        if i + 1 < pat_segs.len() {
            used += 1; // slash
        }
    }
    if used >= path.len() {
        Some("")
    } else if path.as_bytes().get(used) == Some(&b'/') {
        Some(&path[used + 1..])
    } else {
        Some(&path[used..])
    }
}

fn ends_with_segment_glob(path: &str, pattern: &str) -> bool {
    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = if path.is_empty() {
        vec![]
    } else {
        path.split('/').collect()
    };
    if path_segs.len() < pat_segs.len() {
        return false;
    }
    let start = path_segs.len() - pat_segs.len();
    path_segs[start..]
        .iter()
        .zip(pat_segs.iter())
        .all(|(p, g)| wildcard_match(p, g))
}

fn find_segment_glob(path: &str, pattern: &str) -> Option<usize> {
    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = if path.is_empty() {
        vec![]
    } else {
        path.split('/').collect()
    };
    if pat_segs.is_empty() {
        return Some(0);
    }
    if path_segs.len() < pat_segs.len() {
        return None;
    }
    for start in 0..=(path_segs.len() - pat_segs.len()) {
        if path_segs[start..start + pat_segs.len()]
            .iter()
            .zip(pat_segs.iter())
            .all(|(p, g)| wildcard_match(p, g))
        {
            // byte offset of start segment
            let mut off = 0usize;
            for (i, seg) in path_segs.iter().enumerate() {
                if i == start {
                    return Some(off);
                }
                off += seg.len() + 1;
            }
        }
    }
    None
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    // DP for * only (no ?); * does not cross '/' because we match per segment.
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let mut dp = vec![vec![false; p.len() + 1]; t.len() + 1];
    dp[0][0] = true;
    for j in 1..=p.len() {
        if p[j - 1] == '*' {
            dp[0][j] = dp[0][j - 1];
        }
    }
    for i in 1..=t.len() {
        for j in 1..=p.len() {
            if p[j - 1] == '*' {
                dp[i][j] = dp[i][j - 1] || dp[i - 1][j];
            } else if p[j - 1] == t[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }
    dp[t.len()][p.len()]
}

pub(crate) fn compute_risk(
    rules: &RiskRulesFile,
    changed_paths: &[String],
    declared_risk_tier: Option<&str>,
) -> Result<RiskComputation> {
    let mut findings = Vec::new();
    // 不预先 seed default：仅「无规则命中」的路径才用 default，避免 T2 默认压过 T1 evidence。
    let mut tier: Option<String> = None;
    let mut primary_class: Option<String> = None;
    let mut classes = BTreeSet::new();
    let mut matched = Vec::new();

    if changed_paths.is_empty() {
        classes.insert(rules.default.operation_class.clone());
        tier = Some(rules.default.min_risk_tier.clone());
        primary_class = Some(rules.default.operation_class.clone());
    }

    for path in changed_paths {
        let mut hit = false;
        for rule in &rules.rules {
            if rule.path_globs.iter().any(|g| path_matches_glob(path, g)) {
                hit = true;
                matched.push(rule.id.clone());
                classes.insert(rule.operation_class.clone());
                let next_tier = match &tier {
                    None => rule.min_risk_tier.clone(),
                    Some(cur) => max_tier(&rules.tier_order, cur, &rule.min_risk_tier)?,
                };
                // 新的最高 tier 或尚未选定 primary 时更新
                let elevates = match &tier {
                    None => true,
                    Some(cur) => {
                        tier_rank(&rules.tier_order, &rule.min_risk_tier).unwrap_or(0)
                            > tier_rank(&rules.tier_order, cur).unwrap_or(0)
                    }
                };
                if elevates || primary_class.is_none() {
                    primary_class = Some(rule.operation_class.clone());
                }
                tier = Some(next_tier);
            }
        }
        if !hit {
            classes.insert(rules.default.operation_class.clone());
            let next_tier = match &tier {
                None => rules.default.min_risk_tier.clone(),
                Some(cur) => max_tier(&rules.tier_order, cur, &rules.default.min_risk_tier)?,
            };
            let elevates = match &tier {
                None => true,
                Some(cur) => {
                    tier_rank(&rules.tier_order, &rules.default.min_risk_tier).unwrap_or(0)
                        > tier_rank(&rules.tier_order, cur).unwrap_or(0)
                }
            };
            if elevates || primary_class.is_none() {
                primary_class = Some(rules.default.operation_class.clone());
            }
            tier = Some(next_tier);
        }
    }

    let mut tier = tier.unwrap_or_else(|| rules.default.min_risk_tier.clone());
    let primary_operation_class =
        primary_class.unwrap_or_else(|| rules.default.operation_class.clone());

    if let Some(declared) = declared_risk_tier {
        if tier_rank(&rules.tier_order, declared).is_none() {
            findings.push(format!("declared-risk-tier-unknown:{declared}"));
        } else {
            let declared_rank = tier_rank(&rules.tier_order, declared).unwrap();
            let computed_rank = tier_rank(&rules.tier_order, &tier).unwrap();
            if declared_rank < computed_rank {
                findings.push(format!(
                    "declared-risk-tier-below-computed:declared={declared}:computed={tier}"
                ));
            } else if declared_rank > computed_rank {
                // 允许抬高
                tier = declared.to_string();
            }
        }
    }

    matched.sort();
    matched.dedup();

    let req = rules
        .required_approvals
        .get(&tier)
        .cloned()
        .unwrap_or(TierApprovalReq {
            human_approvals: 1,
            independent_risk_review: false,
            auto_merge_allowed: false,
            require_non_author: true,
        });

    Ok(RiskComputation {
        computed_risk_tier: tier,
        primary_operation_class,
        operation_classes: classes.into_iter().collect(),
        matched_rules: matched,
        required_human_approvals: req.human_approvals,
        require_independent_risk_review: req.independent_risk_review,
        auto_merge_allowed: req.auto_merge_allowed,
        findings,
    })
}

// ---------------------------------------------------------------------------
// Subject digest (A3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubjectInput {
    #[serde(default)]
    pub repository_id: String,
    #[serde(default)]
    pub base_sha: String,
    #[serde(default)]
    pub head_sha: String,
    #[serde(default)]
    pub head_tree_sha: String,
    #[serde(default)]
    pub changed_files_digest: String,
    #[serde(default)]
    pub contract_blob_sha: String,
    #[serde(default)]
    pub policy_snapshot_sha: String,
    #[serde(default)]
    pub authority_registry_sha: String,
    #[serde(default)]
    pub operation_class: String,
}

pub fn subject_digest(input: &SubjectInput) -> String {
    // Canonical line protocol — any field change invalidates prior approvals.
    let payload = format!(
        "repository_id={}\nbase_sha={}\nhead_sha={}\nhead_tree_sha={}\nchanged_files_digest={}\ncontract_blob_sha={}\npolicy_snapshot_sha={}\nauthority_registry_sha={}\noperation_class={}\n",
        input.repository_id,
        input.base_sha,
        input.head_sha,
        input.head_tree_sha,
        input.changed_files_digest,
        input.contract_blob_sha,
        input.policy_snapshot_sha,
        input.authority_registry_sha,
        input.operation_class,
    );
    let digest = Sha256::digest(payload.as_bytes());
    format!("sha256:{}", hex_encode(&digest))
}

pub fn sha256_hex_of_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex_encode(&digest))
}

pub fn changed_files_digest(paths: &[String]) -> String {
    let mut sorted = paths.to_vec();
    sorted.sort();
    sorted.dedup();
    let payload = sorted.join("\n") + if sorted.is_empty() { "" } else { "\n" };
    sha256_hex_of_bytes(payload.as_bytes())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

// ---------------------------------------------------------------------------
// Approvals bound to subject (A3 + A4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub login: String,
    pub state: String,
    pub commit_id: String,
    #[serde(default)]
    pub user_type: Option<String>,
    #[serde(default)]
    pub author_association: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub subject_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApprovalEvaluation {
    pub human_approvals_counted: u32,
    pub risk_reviews_counted: u32,
    pub counted_logins: Vec<String>,
    pub rejected: Vec<String>,
    pub findings: Vec<String>,
    pub subject_bound_ok: bool,
    pub meets_human_quorum: bool,
    pub meets_risk_review: bool,
}

/// 评估 final-head reviews：仅 APPROVED + 绑定 head_sha + 绑定 subject + 人类 actor。
pub fn evaluate_approvals(
    head_sha: &str,
    expected_subject: &str,
    pr_author: Option<&str>,
    reviews: &[ReviewRecord],
    risk_reviews: &[ReviewRecord],
    required_human: u32,
    require_risk: bool,
) -> ApprovalEvaluation {
    let mut findings = Vec::new();
    let mut rejected = Vec::new();
    let mut counted_logins = BTreeSet::new();
    let mut human_count = 0u32;
    let author_norm = pr_author.map(normalized_handle);

    for review in reviews {
        let reason = classify_review(review, head_sha, expected_subject, author_norm.as_deref());
        if let Some(r) = reason {
            rejected.push(format!("{}:{r}", review.login));
            continue;
        }
        counted_logins.insert(normalized_handle(&review.login));
        human_count += 1;
    }

    let mut risk_count = 0u32;
    for review in risk_reviews {
        let reason = classify_review(review, head_sha, expected_subject, author_norm.as_deref());
        if let Some(r) = reason {
            rejected.push(format!("risk:{}:{r}", review.login));
            continue;
        }
        // Risk review 不得与已计人类 approval 重复同一人（独立主体）
        let login = normalized_handle(&review.login);
        if counted_logins.contains(&login) {
            rejected.push(format!(
                "risk:{}:not-independent-same-as-approver",
                review.login
            ));
            findings.push(format!("risk-review-not-independent:{}", review.login));
            continue;
        }
        risk_count += 1;
    }

    if human_count < required_human {
        findings.push(format!(
            "human-approval-quorum-unmet:have={human_count}:need={required_human}"
        ));
    }
    if require_risk && risk_count < 1 {
        findings.push("independent-risk-review-missing".into());
    }

    // subject 绑定：每条计入的 review 必须携带匹配 subject（classify 已检查）
    let subject_bound_ok = rejected.iter().all(|r| !r.contains("subject"))
        && reviews
            .iter()
            .chain(risk_reviews.iter())
            .filter(|r| {
                r.state.eq_ignore_ascii_case("APPROVED") || r.state.eq_ignore_ascii_case("ACCEPTED")
            })
            .filter(|r| !is_non_human_actor(&r.login, r.user_type.as_deref()))
            .all(|r| {
                r.subject_digest
                    .as_deref()
                    .is_some_and(|s| s == expected_subject)
                    && r.commit_id == head_sha
            });

    ApprovalEvaluation {
        human_approvals_counted: human_count,
        risk_reviews_counted: risk_count,
        counted_logins: counted_logins.into_iter().collect(),
        rejected,
        findings: findings.clone(),
        subject_bound_ok,
        meets_human_quorum: human_count >= required_human,
        meets_risk_review: !require_risk || risk_count >= 1,
    }
}

fn classify_review(
    review: &ReviewRecord,
    head_sha: &str,
    expected_subject: &str,
    pr_author: Option<&str>,
) -> Option<String> {
    if !review.state.eq_ignore_ascii_case("APPROVED")
        && !review.state.eq_ignore_ascii_case("ACCEPTED")
    {
        return Some(format!("state-not-approved:{}", review.state));
    }
    if is_non_human_actor(&review.login, review.user_type.as_deref()) {
        return Some("non-human-actor".into());
    }
    if review.commit_id != head_sha {
        return Some(format!(
            "stale-or-wrong-head:commit={}:head={head_sha}",
            review.commit_id
        ));
    }
    match &review.subject_digest {
        None => return Some("missing-subject-digest".into()),
        Some(s) if s != expected_subject => {
            return Some(format!("subject-mismatch:got={s}:want={expected_subject}"));
        }
        Some(_) => {}
    }
    if let Some(author) = pr_author {
        if normalized_handle(&review.login) == author {
            return Some("self-approval-forbidden".into());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// CLI orchestration
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AuthorityCheckReport {
    pub mode: String,
    pub registry: Option<RegistryReport>,
    pub risk: Option<RiskComputation>,
    pub subject_digest: Option<String>,
    pub approvals: Option<ApprovalEvaluation>,
    pub findings: Vec<String>,
    pub ok: bool,
    /// Shadow 标记：本报告永不表示 live 授权
    pub live_authorization: bool,
}

#[derive(Debug, Default)]
pub struct AuthorityCheckArgs {
    pub registry_only: bool,
    pub changed_paths: Vec<String>,
    pub declared_risk_tier: Option<String>,
    pub subject: Option<SubjectInput>,
    pub reviews: Vec<ReviewRecord>,
    pub risk_reviews: Vec<ReviewRecord>,
    pub pr_author: Option<String>,
    pub allow_live_registry: bool,
}

pub fn run(root: &Path, args: AuthorityCheckArgs) -> Result<AuthorityCheckReport> {
    let mut findings = Vec::new();

    // Refuse live registry path unless allow_live (RFC Effective 后由人类打开)
    let live_path = root.join(LIVE_REGISTRY_REL);
    if live_path.exists() && !args.allow_live_registry {
        findings.push(format!(
            "live-registry-present-but-not-activated:{}",
            LIVE_REGISTRY_REL
        ));
    }

    let target_path = root.join(TARGET_REGISTRY_REL);
    if !target_path.exists() {
        findings.push(format!("target-registry-missing:{TARGET_REGISTRY_REL}"));
        return Ok(AuthorityCheckReport {
            mode: "fail-closed".into(),
            registry: None,
            risk: None,
            subject_digest: None,
            approvals: None,
            ok: false,
            findings,
            live_authorization: false,
        });
    }

    let registry = validate_registry_at(root, &target_path, args.allow_live_registry)?;
    findings.extend(registry.findings.iter().cloned());

    if args.registry_only {
        let ok = registry.ok && findings.is_empty();
        return Ok(AuthorityCheckReport {
            mode: "registry-only".into(),
            ok,
            registry: Some(registry),
            risk: None,
            subject_digest: None,
            approvals: None,
            findings,
            live_authorization: false,
        });
    }

    let rules = load_risk_rules(root)?;
    if rules.status != "shadow" && rules.status != "active" {
        findings.push(format!("risk-rules-status-unexpected:{}", rules.status));
    }

    let risk = compute_risk(
        &rules,
        &args.changed_paths,
        args.declared_risk_tier.as_deref(),
    )?;
    findings.extend(risk.findings.iter().cloned());

    let primary_class = risk.primary_operation_class.clone();

    let mut subject_digest_out = None;
    let mut approvals_out = None;

    if let Some(mut subject) = args.subject {
        if subject.operation_class.is_empty() {
            subject.operation_class = primary_class.clone();
        }
        if subject.changed_files_digest.is_empty() {
            subject.changed_files_digest = changed_files_digest(&args.changed_paths);
        }
        if subject.authority_registry_sha.is_empty() {
            let bytes = fs::read(&target_path)?;
            subject.authority_registry_sha = sha256_hex_of_bytes(&bytes);
        }
        let digest = subject_digest(&subject);
        subject_digest_out = Some(digest.clone());

        let eval = evaluate_approvals(
            &subject.head_sha,
            &digest,
            args.pr_author.as_deref(),
            &args.reviews,
            &args.risk_reviews,
            risk.required_human_approvals,
            risk.require_independent_risk_review,
        );
        findings.extend(eval.findings.iter().cloned());
        approvals_out = Some(eval);
    }

    let ok = findings.is_empty()
        && registry.ok
        && approvals_out
            .as_ref()
            .map(|a| a.meets_human_quorum && a.meets_risk_review && a.subject_bound_ok)
            .unwrap_or(true);

    Ok(AuthorityCheckReport {
        mode: "full-shadow".into(),
        registry: Some(registry),
        risk: Some(risk),
        subject_digest: subject_digest_out,
        approvals: approvals_out,
        findings,
        ok,
        live_authorization: false,
    })
}

pub fn workspace_root() -> PathBuf {
    // xtask 通常从仓库根或 tools/xtask 运行；与其他子命令一致，从 CARGO_MANIFEST_DIR 上溯
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent() // tools
        .and_then(|p| p.parent()) // repo
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_REGISTRY: &str = r#"
schema_version = 1
status = "proposed_shadow_target"
live_ssot = false
rfc = "RFC-0003"

[[authority]]
id = "constitution.core"
class = "governance"
canonical = "docs/governance/CONSTITUTION.md"
owner_role = "Governance Maintainer"
validator = "scripts/ci/validate-governance.mjs"

[[authority]]
id = "agent.spec"
class = "spec"
canonical_roots = [".agent/SSOT/"]
owner_role = "Spec Owner"
"#;

    #[test]
    fn registry_unique_and_valid() {
        let report =
            validate_registry_text("authority-registry.target.toml", SAMPLE_REGISTRY, false)
                .unwrap();
        assert!(report.ok, "{:?}", report.findings);
        assert_eq!(report.entry_count, 2);
        assert!(!report.live_ssot);
    }

    #[test]
    fn registry_rejects_duplicate_ids() {
        let bad = SAMPLE_REGISTRY.to_string()
            + r#"
[[authority]]
id = "constitution.core"
class = "governance"
canonical = "other.md"
owner_role = "Governance Maintainer"
"#;
        let report = validate_registry_text("x.target.toml", &bad, false).unwrap();
        assert!(!report.ok);
        assert!(report
            .findings
            .iter()
            .any(|f| f.contains("authority-id-duplicate")));
    }

    #[test]
    fn registry_rejects_live_ssot_on_target() {
        let bad = SAMPLE_REGISTRY.replace("live_ssot = false", "live_ssot = true");
        let report = validate_registry_text("authority-registry.target.toml", &bad, false).unwrap();
        assert!(!report.ok);
        assert!(report.findings.iter().any(|f| f.contains("live_ssot")));
    }

    #[test]
    fn risk_tier_escalates_for_constitution() {
        let rules: RiskRulesFile = serde_json::from_str(include_str!(
            "../../../.github/ai-native/authority-risk-rules.json"
        ))
        .unwrap();
        let comp = compute_risk(
            &rules,
            &["docs/governance/CONSTITUTION.md".into()],
            Some("T2"),
        )
        .unwrap();
        assert_eq!(comp.computed_risk_tier, "T4");
        assert_eq!(comp.primary_operation_class, "governance.amendment");
        assert!(comp
            .findings
            .iter()
            .any(|f| f.contains("declared-risk-tier-below-computed")));
        assert!(comp.require_independent_risk_review);
        assert_eq!(comp.required_human_approvals, 2);
    }

    #[test]
    fn risk_tier_evidence_is_t1() {
        let rules: RiskRulesFile = serde_json::from_str(include_str!(
            "../../../.github/ai-native/authority-risk-rules.json"
        ))
        .unwrap();
        let comp = compute_risk(&rules, &["evidence/changes/x.md".into()], None).unwrap();
        assert_eq!(comp.computed_risk_tier, "T1");
        assert_eq!(comp.primary_operation_class, "evidence.raw.append");
        assert!(comp
            .operation_classes
            .iter()
            .any(|c| c == "evidence.raw.append"));
    }

    #[test]
    fn subject_digest_stable_and_sensitive() {
        let base = SubjectInput {
            repository_id: "1297557216".into(),
            base_sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            head_sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            head_tree_sha: "cccccccccccccccccccccccccccccccccccccccc".into(),
            changed_files_digest: "sha256:ddd".into(),
            contract_blob_sha: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
            policy_snapshot_sha: "sha256:fff".into(),
            authority_registry_sha: "sha256:000".into(),
            operation_class: "code.fix".into(),
        };
        let d1 = subject_digest(&base);
        let d2 = subject_digest(&base);
        assert_eq!(d1, d2);
        assert!(d1.starts_with("sha256:"));
        assert_eq!(d1.len(), "sha256:".len() + 64);

        let mut changed = base.clone();
        changed.head_sha = "dddddddddddddddddddddddddddddddddddddddd".into();
        assert_ne!(subject_digest(&changed), d1);
    }

    #[test]
    fn bot_approval_not_counted() {
        let head = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let subject = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let reviews = vec![
            ReviewRecord {
                login: "dependabot[bot]".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("Bot".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
            ReviewRecord {
                login: "copilot".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
            ReviewRecord {
                login: "alice".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
        ];
        let eval = evaluate_approvals(head, subject, Some("author"), &reviews, &[], 1, false);
        assert_eq!(eval.human_approvals_counted, 1);
        assert!(eval.meets_human_quorum);
        assert!(eval.rejected.iter().any(|r| r.contains("non-human")));
        assert!(eval.counted_logins.contains(&"alice".to_string()));
    }

    #[test]
    fn stale_head_and_subject_mismatch_rejected() {
        let head = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let subject = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let reviews = vec![
            ReviewRecord {
                login: "alice".into(),
                state: "APPROVED".into(),
                commit_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
            ReviewRecord {
                login: "bob".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(
                    "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                        .into(),
                ),
            },
        ];
        let eval = evaluate_approvals(head, subject, None, &reviews, &[], 1, false);
        assert_eq!(eval.human_approvals_counted, 0);
        assert!(!eval.meets_human_quorum);
    }

    #[test]
    fn t4_requires_risk_review_independent() {
        let head = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let subject = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let reviews = vec![
            ReviewRecord {
                login: "m1".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
            ReviewRecord {
                login: "m2".into(),
                state: "APPROVED".into(),
                commit_id: head.into(),
                user_type: Some("User".into()),
                author_association: None,
                role: None,
                subject_digest: Some(subject.into()),
            },
        ];
        // Risk reviewer same as m1 → rejected
        let risk = vec![ReviewRecord {
            login: "m1".into(),
            state: "APPROVED".into(),
            commit_id: head.into(),
            user_type: Some("User".into()),
            author_association: None,
            role: Some("Risk Owner".into()),
            subject_digest: Some(subject.into()),
        }];
        let eval = evaluate_approvals(head, subject, None, &reviews, &risk, 2, true);
        assert!(eval.meets_human_quorum);
        assert!(!eval.meets_risk_review);
        assert!(eval
            .findings
            .iter()
            .any(|f| f.contains("risk-review-not-independent")
                || f.contains("independent-risk-review-missing")));

        let risk_ok = vec![ReviewRecord {
            login: "risk-owner".into(),
            state: "APPROVED".into(),
            commit_id: head.into(),
            user_type: Some("User".into()),
            author_association: None,
            role: Some("Risk Owner".into()),
            subject_digest: Some(subject.into()),
        }];
        let eval2 = evaluate_approvals(head, subject, None, &reviews, &risk_ok, 2, true);
        assert!(eval2.meets_risk_review);
        assert_eq!(eval2.risk_reviews_counted, 1);
    }

    #[test]
    fn path_glob_basics() {
        assert!(path_matches_glob(
            "docs/governance/x.md",
            "docs/governance/**"
        ));
        assert!(path_matches_glob("evidence/a/b.md", "evidence/**"));
        assert!(path_matches_glob("crates/foo/tests/bar.rs", "**/tests/**"));
        assert!(!path_matches_glob("crates/foo/src/lib.rs", "evidence/**"));
    }
}
