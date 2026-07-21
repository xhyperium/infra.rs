//! Goal 结构 / 语义校验（fail-closed）。

use thiserror::Error;

use crate::lint::lint_subjective;
use crate::model::GoalDocument;

/// 校验错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidateError {
    #[error("goal id is empty")]
    EmptyId,
    #[error("goal outcome is empty")]
    EmptyOutcome,
    #[error("acceptance[{0}] missing id")]
    MissingAcceptanceId(usize),
    #[error("acceptance id duplicated: {0}")]
    DuplicateAcceptanceId(String),
    #[error("subjective language: {0}")]
    Subjective(String),
}

/// 校验 Goal；成功返回 `Ok(())`。
pub fn validate_goal(goal: &GoalDocument) -> Result<(), ValidateError> {
    if goal.id.trim().is_empty() {
        return Err(ValidateError::EmptyId);
    }
    if goal.outcome.trim().is_empty() {
        return Err(ValidateError::EmptyOutcome);
    }

    let mut seen = std::collections::BTreeSet::new();
    for (i, ac) in goal.acceptance.iter().enumerate() {
        if ac.id.trim().is_empty() {
            return Err(ValidateError::MissingAcceptanceId(i));
        }
        if !seen.insert(ac.id.clone()) {
            return Err(ValidateError::DuplicateAcceptanceId(ac.id.clone()));
        }
    }

    // 主观词：扫描 outcome + acceptance statements + invariants
    let mut blob = goal.outcome.clone();
    for ac in &goal.acceptance {
        blob.push('\n');
        blob.push_str(&ac.statement);
    }
    for inv in &goal.invariants {
        blob.push('\n');
        blob.push_str(inv);
    }
    let hits = lint_subjective(&blob);
    if !hits.is_empty() {
        return Err(ValidateError::Subjective(hits.join(", ")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AcceptanceItem, RiskLevel};

    fn sample() -> GoalDocument {
        GoalDocument {
            id: "GOAL-1".into(),
            outcome: "cargo test passes".into(),
            risk: RiskLevel::R1,
            acceptance: vec![AcceptanceItem {
                id: "AC-1".into(),
                statement: "unit tests green".into(),
            }],
            invariants: vec![],
            forbidden: vec![],
            not_in_scope: vec![],
            touches: vec![],
        }
    }

    #[test]
    fn ok_goal() {
        assert!(validate_goal(&sample()).is_ok());
    }

    #[test]
    fn empty_outcome() {
        let mut g = sample();
        g.outcome = "  ".into();
        assert_eq!(validate_goal(&g), Err(ValidateError::EmptyOutcome));
    }

    #[test]
    fn missing_ac_id() {
        let mut g = sample();
        g.acceptance[0].id = "".into();
        assert_eq!(validate_goal(&g), Err(ValidateError::MissingAcceptanceId(0)));
    }
}
