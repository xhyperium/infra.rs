//! 验证数据类型。

use serde::{Deserialize, Serialize};

/// 检查类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    /// 配置验证
    Config,
    /// 连接检测
    Connectivity,
    /// 签名验证
    Signature,
    /// 基本对象操作
    ObjectOps,
    /// 流式操作
    Streaming,
    /// 高级功能
    Advanced,
    /// 安全验证
    Security,
    /// 并发验证
    Concurrency,
    /// 自定义
    Custom,
}

/// 单条检查规格。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckSpec {
    pub id: String,
    pub kind: CheckKind,
    pub description: String,
    pub layer: u8,
    pub timeout_secs: u64,
}

/// 验证计划。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub schema: String,
    pub module: String,
    pub version: String,
    pub layers: Vec<u8>,
    pub checks: Vec<CheckSpec>,
    pub plan_digest: String,
}

impl VerificationPlan {
    pub const SCHEMA: &'static str = "oss-verification-plan/v1";
}

/// 单次检查结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub kind: CheckKind,
    pub passed: bool,
    pub duration_ms: u64,
    pub message: String,
    pub detail: Option<String>,
}

/// 总状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RunStatus {
    Pass,
    Fail,
    Partial,
}

/// 运行汇总。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResult {
    pub schema: String,
    pub status: RunStatus,
    pub module: String,
    pub version: String,
    pub plan_digest: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub checks: Vec<CheckResult>,
    pub summary: String,
}

impl RunResult {
    pub const SCHEMA: &'static str = "oss-verification-run/v1";
}
