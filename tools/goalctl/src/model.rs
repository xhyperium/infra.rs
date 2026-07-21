//! Goal / Contract 数据模型。

use serde::{Deserialize, Serialize};

/// 风险等级。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    R0,
    R1,
    R2,
    R3,
    R4,
    #[serde(other)]
    Unknown,
}

/// 验收项。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceItem {
    /// 稳定 id（必填）。
    pub id: String,
    /// 可观测陈述。
    pub statement: String,
}

/// Goal 文档（YAML/JSON 输入）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalDocument {
    /// Goal id。
    pub id: String,
    /// 可观测 outcome（必填、非空）。
    pub outcome: String,
    /// 风险。
    #[serde(default = "default_risk")]
    pub risk: RiskLevel,
    /// 验收列表。
    #[serde(default)]
    pub acceptance: Vec<AcceptanceItem>,
    /// 不变量。
    #[serde(default)]
    pub invariants: Vec<String>,
    /// 禁止项。
    #[serde(default)]
    pub forbidden: Vec<String>,
    /// 不在范围内。
    #[serde(default)]
    pub not_in_scope: Vec<String>,
    /// 触及路径。
    #[serde(default)]
    pub touches: Vec<String>,
}

fn default_risk() -> RiskLevel {
    RiskLevel::R2
}

/// 编译后的 Contract（确定性字段顺序由 serde 定义 + canonical JSON 保证）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalContract {
    /// schema 版本。
    pub schema: String,
    /// Goal id。
    pub id: String,
    /// outcome。
    pub outcome: String,
    /// 风险。
    pub risk: RiskLevel,
    /// 验收。
    pub acceptance: Vec<AcceptanceItem>,
    /// 不变量。
    pub invariants: Vec<String>,
    /// 禁止。
    pub forbidden: Vec<String>,
    /// 范围外。
    pub not_in_scope: Vec<String>,
    /// 触及路径。
    pub touches: Vec<String>,
    /// canonical JSON 的 sha256 hex（不含本字段时计算，再写回）。
    pub digest: String,
}

impl GoalContract {
    pub const SCHEMA: &'static str = "goal-contract/v1";
}
