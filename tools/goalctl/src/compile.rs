//! Goal 编译为 Contract + digest。

use std::fs;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::model::{GoalContract, GoalDocument};
use crate::validate::{ValidateError, validate_goal};

/// 编译错误。
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(String),
    #[error("validate: {0}")]
    Validate(#[from] ValidateError),
    #[error("serialize: {0}")]
    Serialize(String),
}

/// 编译产物。
#[derive(Debug, Clone)]
pub struct CompileOutput {
    /// Contract 对象。
    pub contract: GoalContract,
    /// 漂亮打印 JSON。
    pub json: String,
}

/// 从文件路径编译（按扩展名选 YAML/JSON；未知则先 YAML 再 JSON）。
pub fn compile_goal(path: &Path) -> Result<CompileOutput, CompileError> {
    let raw = fs::read_to_string(path)?;
    compile_goal_str(&raw, path.extension().and_then(|e| e.to_str()))
}

/// 从字符串编译。`hint`：`Some("yaml"|"yml"|"json")` 或 `None` 自动。
pub fn compile_goal_str(raw: &str, hint: Option<&str>) -> Result<CompileOutput, CompileError> {
    let goal = parse_goal(raw, hint)?;
    validate_goal(&goal)?;
    let contract = to_contract(goal)?;
    let json = serde_json::to_string_pretty(&contract)
        .map_err(|e| CompileError::Serialize(e.to_string()))?;
    Ok(CompileOutput { contract, json })
}

fn parse_goal(raw: &str, hint: Option<&str>) -> Result<GoalDocument, CompileError> {
    match hint.map(|s| s.to_ascii_lowercase()) {
        Some(ref e) if e == "json" => {
            serde_json::from_str(raw).map_err(|e| CompileError::Parse(format!("json: {e}")))
        }
        Some(ref e) if e == "yaml" || e == "yml" => {
            serde_yaml::from_str(raw).map_err(|e| CompileError::Parse(format!("yaml: {e}")))
        }
        _ => {
            // 自动：先 YAML，再 JSON
            if let Ok(g) = serde_yaml::from_str::<GoalDocument>(raw) {
                return Ok(g);
            }
            serde_json::from_str(raw).map_err(|e| CompileError::Parse(format!("auto: {e}")))
        }
    }
}

fn to_contract(goal: GoalDocument) -> Result<GoalContract, CompileError> {
    // 先构造无 digest 的稳定结构
    #[derive(Serialize)]
    struct Body<'a> {
        schema: &'a str,
        id: &'a str,
        outcome: &'a str,
        risk: &'a crate::model::RiskLevel,
        acceptance: &'a [crate::model::AcceptanceItem],
        invariants: &'a [String],
        forbidden: &'a [String],
        not_in_scope: &'a [String],
        touches: &'a [String],
    }
    let body = Body {
        schema: GoalContract::SCHEMA,
        id: &goal.id,
        outcome: &goal.outcome,
        risk: &goal.risk,
        acceptance: &goal.acceptance,
        invariants: &goal.invariants,
        forbidden: &goal.forbidden,
        not_in_scope: &goal.not_in_scope,
        touches: &goal.touches,
    };
    let canonical = to_canonical_json(&body).map_err(|e| CompileError::Serialize(e.to_string()))?;
    let digest = sha256_hex(canonical.as_bytes());
    Ok(GoalContract {
        schema: GoalContract::SCHEMA.into(),
        id: goal.id,
        outcome: goal.outcome,
        risk: goal.risk,
        acceptance: goal.acceptance,
        invariants: goal.invariants,
        forbidden: goal.forbidden,
        not_in_scope: goal.not_in_scope,
        touches: goal.touches,
        digest,
    })
}

/// RFC 级简化：serde_json Value 递归排序 key 后序列化。
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let sorted = sort_value(v);
    serde_json::to_string(&sorted)
}

fn sort_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                if let Some(val) = map.get(&k) {
                    out.insert(k, sort_value(val.clone()));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_value).collect())
        }
        other => other,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_stable() {
        let raw = r#"
id: GOAL-DEMO
outcome: "all unit tests pass"
risk: R1
acceptance:
  - id: AC-1
    statement: "cargo test -p goalctl passes"
invariants:
  - "no network in unit tests"
forbidden:
  - "mock live pass"
not_in_scope:
  - "ui"
touches:
  - "tools/goalctl"
"#;
        let a = compile_goal_str(raw, Some("yaml")).unwrap();
        let b = compile_goal_str(raw, Some("yaml")).unwrap();
        assert_eq!(a.contract.digest, b.contract.digest);
        assert_eq!(a.contract.digest.len(), 64);
    }
}
