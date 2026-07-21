//! Determinism / No-Lookahead / 领域 Gate 升级路径（PHASE-3-05..07）
//!
//! **Transitional**：提供可跑负测骨架与升级路径声明；**≠** 生产语义证明完成。

use anyhow::{bail, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct DeterminismReport {
    pub ok: bool,
    pub mode: &'static str,
    pub status: String,
    pub digest_a: String,
    pub digest_b: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct NoLookaheadReport {
    pub ok: bool,
    pub mode: &'static str,
    pub status: String,
    pub violations: Vec<String>,
    pub layers_checked: Vec<String>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct DomainGateReport {
    pub ok: bool,
    pub mode: &'static str,
    pub transitional_docs: Vec<String>,
    pub upgrade_path_present: bool,
    pub negative_fixture_present: bool,
    pub note: String,
}

/// 对规范化业务载荷做两次 digest 比较（AC-11 升级路径：比 exit code 更进一步）。
/// `payload` 为 canonical JSON 字符串；相同输入必须相同 digest。
pub fn determinism_digest_of(payload: &str) -> String {
    // strip trailing whitespace per line + collapse trailing empty lines
    let mut lines: Vec<&str> = payload.lines().map(str::trim_end).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let canonical = lines.join("\n");
    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

pub fn check_determinism_twice(payload: &str) -> DeterminismReport {
    let a = determinism_digest_of(payload);
    let b = determinism_digest_of(payload);
    let ok = a == b && !payload.is_empty();
    DeterminismReport {
        ok,
        mode: "shadow",
        status: if ok { "PASS".into() } else { "FAIL".into() },
        digest_a: a,
        digest_b: b,
        note: "Transitional: digest of canonical payload (not production domain replay)".into(),
    }
}

/// 负向：同一语义不同规范化空白不应被当成不同业务结果时，
/// 我们要求 trim 后稳定；若原始字节不同但 trim 后相同 → digest 相同。
#[cfg(test)]
pub fn check_determinism_normalized_pair(raw_a: &str, raw_b: &str) -> DeterminismReport {
    let a = determinism_digest_of(raw_a);
    let b = determinism_digest_of(raw_b);
    let ok = a == b;
    DeterminismReport {
        ok,
        mode: "shadow",
        status: if ok { "PASS".into() } else { "FAIL".into() },
        digest_a: a,
        digest_b: b,
        note: "normalized trailing whitespace must not change business digest".into(),
    }
}

#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub observed_at: i64,
    pub available_at: i64,
    pub effective_at: i64,
    pub decision_as_of: i64,
}

/// 属性层（骨架）：AvailableAt/EffectiveAt 不得超前 decision as-of（no-lookahead）。
pub fn check_no_lookahead(events: &[TimelineEvent]) -> NoLookaheadReport {
    let mut violations = Vec::new();
    for (i, e) in events.iter().enumerate() {
        if e.available_at > e.decision_as_of {
            violations.push(format!(
                "event[{i}]: available_at {} > decision_as_of {}",
                e.available_at, e.decision_as_of
            ));
        }
        if e.effective_at > e.decision_as_of {
            violations.push(format!(
                "event[{i}]: effective_at {} > decision_as_of {}",
                e.effective_at, e.decision_as_of
            ));
        }
        if e.observed_at > e.decision_as_of {
            // observed 可晚于 decision 的 wall-clock，但 as-of 决策不得使用未来 available
            // 仅记录信息性：不作为 hard fail（与 available/effective 区分）
        }
    }
    let ok = violations.is_empty();
    NoLookaheadReport {
        ok,
        mode: "shadow",
        status: if ok { "PASS".into() } else { "FAIL".into() },
        violations,
        layers_checked: vec![
            "property_cutoff".into(),
            "timeline_ordering".into(),
            // Replay digest layer still deferred
        ],
        note:
            "Transitional: property layer only; type constraints + replay digest still upgrade path"
                .into(),
    }
}

pub fn domain_gate_upgrade_check(root: &Path) -> DomainGateReport {
    let det = root.join(".agent/gates/determinism-gate.md");
    let nl = root.join(".agent/gates/no-lookahead-gate.md");
    let upgrade = root.join(".agent/SSOT/cicd/domain-gate-upgrade-path.md");
    let neg = root.join("tools/xtask/tests/ci_negative/fixtures/no_lookahead_violation.json");
    let mut docs = Vec::new();
    for p in [&det, &nl, &upgrade] {
        if p.is_file() {
            docs.push(p.display().to_string());
        }
    }
    let transitional_ok = det.is_file()
        && nl.is_file()
        && fs::read_to_string(&det)
            .map(|s| s.contains("Transitional"))
            .unwrap_or(false)
        && fs::read_to_string(&nl)
            .map(|s| s.contains("Transitional"))
            .unwrap_or(false);
    let upgrade_path_present = upgrade.is_file();
    let negative_fixture_present = neg.is_file();
    let ok = transitional_ok && upgrade_path_present && negative_fixture_present;
    DomainGateReport {
        ok,
        mode: "shadow",
        transitional_docs: docs,
        upgrade_path_present,
        negative_fixture_present,
        note: "upgrade path + transitional markers + negative fixture; not production domain proof"
            .into(),
    }
}

pub fn domain_gate_or_bail(root: &Path) -> Result<DomainGateReport> {
    let r = domain_gate_upgrade_check(root);
    if !r.ok {
        bail!("ci domain-gates: incomplete upgrade path skeleton");
    }
    Ok(r)
}

pub fn no_lookahead_from_fixture(path: &Path) -> Result<NoLookaheadReport> {
    let raw = fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let arr = v
        .as_array()
        .or_else(|| v.get("events").and_then(|x| x.as_array()))
        .ok_or_else(|| anyhow::anyhow!("fixture must be array or {{events:[]}}"))?;
    let mut events = Vec::new();
    for item in arr {
        events.push(TimelineEvent {
            observed_at: item["observed_at"].as_i64().unwrap_or(0),
            available_at: item["available_at"].as_i64().unwrap_or(0),
            effective_at: item["effective_at"].as_i64().unwrap_or(0),
            decision_as_of: item["decision_as_of"].as_i64().unwrap_or(0),
        });
    }
    Ok(check_no_lookahead(&events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;

    #[test]
    fn determinism_stable_for_same_payload() {
        let r = check_determinism_twice(r#"{"qty":"1.00","px":"2"}"#);
        assert!(r.ok);
        assert_eq!(r.digest_a, r.digest_b);
        assert!(r.digest_a.starts_with("sha256:"));
    }

    #[test]
    fn determinism_trims_trailing_ws() {
        let r = check_determinism_normalized_pair("{\"a\":1}\n", "{\"a\":1}\n  \n");
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn no_lookahead_pass_when_asof_covers() {
        let events = vec![TimelineEvent {
            observed_at: 10,
            available_at: 10,
            effective_at: 9,
            decision_as_of: 10,
        }];
        let r = check_no_lookahead(&events);
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn no_lookahead_fails_future_available() {
        let events = vec![TimelineEvent {
            observed_at: 5,
            available_at: 20,
            effective_at: 5,
            decision_as_of: 10,
        }];
        let r = check_no_lookahead(&events);
        assert!(!r.ok);
        assert!(r.violations.iter().any(|v| v.contains("available_at")));
    }

    #[test]
    fn domain_gate_upgrade_skeleton_present() {
        let root = repo_root_from_manifest();
        let r = domain_gate_upgrade_check(&root);
        assert!(r.ok, "{r:?}");
    }

    #[test]
    fn no_lookahead_fixture_fails() {
        let root = repo_root_from_manifest();
        let f = root.join("tools/xtask/tests/ci_negative/fixtures/no_lookahead_violation.json");
        let r = no_lookahead_from_fixture(&f).unwrap();
        assert!(!r.ok, "violation fixture must fail");
    }
}
