//! 从 contract + 变更路径生成 VerificationPlan。

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::types::{CheckKind, CheckSpec, VerificationPlan};

/// 计划错误。
#[derive(Debug, Error)]
pub enum PlanError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse contract: {0}")]
    Parse(String),
}

/// 计划选项。
#[derive(Debug, Clone, Default)]
pub struct PlanOptions {
    /// 全部检查替换为 `true`（集成测 / 无 cargo 环境）。
    pub dry: bool,
}

impl PlanOptions {
    /// 从环境变量读取：`VERIFYCTL_DRY=1` → dry。
    #[must_use]
    pub fn from_env() -> Self {
        let dry = std::env::var("VERIFYCTL_DRY").ok().as_deref() == Some("1");
        Self { dry }
    }
}

#[derive(Debug, Deserialize)]
struct ContractLite {
    #[serde(default)]
    digest: String,
    #[serde(default)]
    touches: Vec<String>,
}

/// 构建验证计划（选项见 [`PlanOptions`]）。
///
/// 策略（最小可交付）：
/// - 始终包含 `fmt` / `clippy` / `test`
/// - 若变更路径含 `docs/` 或 `*.md`，追加 `docs` 检查
/// - `opts.dry` 时全部替换为 `true`
pub fn build_plan(
    contract_json: &str,
    changed_paths: &[String],
    opts: &PlanOptions,
) -> Result<VerificationPlan, PlanError> {
    let lite: ContractLite = if contract_json.trim().is_empty() {
        ContractLite { digest: String::new(), touches: vec![] }
    } else {
        serde_json::from_str(contract_json).map_err(|e| PlanError::Parse(e.to_string()))?
    };

    let dry = opts.dry;

    let mut checks = vec![
        check("fmt", CheckKind::Fmt, cargo_like(dry, &["fmt", "--all", "--", "--check"])),
        check(
            "clippy",
            CheckKind::Clippy,
            cargo_like(dry, &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
        ),
        check("test", CheckKind::Test, cargo_like(dry, &["test", "--workspace", "--all-features"])),
    ];

    let need_docs = changed_paths.iter().any(|p| {
        p.starts_with("docs/")
            || p.ends_with(".md")
            || lite.touches.iter().any(|t| t.contains("docs"))
    });
    if need_docs {
        checks.push(check(
            "docs",
            CheckKind::Docs,
            if dry {
                vec!["true".into()]
            } else {
                vec!["cargo".into(), "doc".into(), "--workspace".into(), "--no-deps".into()]
            },
        ));
    }

    if dry {
        checks.push(CheckSpec {
            id: "dry-true".into(),
            kind: CheckKind::Custom,
            argv: vec!["true".into()],
            timeout_secs: 10,
        });
    }

    let mut plan = VerificationPlan {
        schema: VerificationPlan::SCHEMA.into(),
        contract_digest: lite.digest,
        changed_paths: changed_paths.to_vec(),
        checks,
        plan_digest: String::new(),
    };
    plan.plan_digest = plan_digest_of(&plan);
    Ok(plan)
}

fn check(id: &str, kind: CheckKind, argv: Vec<String>) -> CheckSpec {
    CheckSpec { id: id.into(), kind, argv, timeout_secs: 300 }
}

fn cargo_like(dry: bool, args: &[&str]) -> Vec<String> {
    if dry {
        return vec!["true".into()];
    }
    let mut v = vec!["cargo".into()];
    v.extend(args.iter().map(|s| (*s).to_string()));
    v
}

fn plan_digest_of(plan: &VerificationPlan) -> String {
    #[derive(serde::Serialize)]
    struct Body<'a> {
        schema: &'a str,
        contract_digest: &'a str,
        changed_paths: &'a [String],
        checks: &'a [CheckSpec],
    }
    let body = Body {
        schema: &plan.schema,
        contract_digest: &plan.contract_digest,
        changed_paths: &plan.changed_paths,
        checks: &plan.checks,
    };
    let json = serde_json::to_string(&body).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(json.as_bytes());
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_includes_docs_when_md_changed() {
        let opts = PlanOptions { dry: true };
        let plan =
            build_plan("{\"digest\":\"abc\",\"touches\":[]}", &["docs/ssot/x.md".into()], &opts)
                .unwrap();
        assert!(plan.checks.iter().any(|c| c.id == "docs"));
        assert_eq!(plan.plan_digest.len(), 64);
    }

    #[test]
    fn dry_plan_uses_true() {
        let opts = PlanOptions { dry: true };
        let plan = build_plan("", &[], &opts).unwrap();
        assert!(
            plan.checks.iter().all(|c| {
                c.argv.first().map(String::as_str) == Some("true") || c.id == "dry-true"
            })
        );
    }
}
