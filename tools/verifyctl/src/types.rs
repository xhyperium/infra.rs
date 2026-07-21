//! 计划与运行结果类型。

use serde::{Deserialize, Serialize};

/// 检查类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    Fmt,
    Clippy,
    Test,
    Docs,
    /// 自定义 / dry 命令（如 `true`）。
    Custom,
}

/// 单条检查规格。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckSpec {
    /// 稳定 id。
    pub id: String,
    /// 类别。
    pub kind: CheckKind,
    /// argv：`[program, args...]`。
    pub argv: Vec<String>,
    /// 超时秒（默认 120）。
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    120
}

/// 验证计划。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPlan {
    /// schema。
    pub schema: String,
    /// 关联 contract digest（可空）。
    pub contract_digest: String,
    /// 变更路径（输入镜像）。
    pub changed_paths: Vec<String>,
    /// 检查列表。
    pub checks: Vec<CheckSpec>,
    /// 计划自身 digest（canonical JSON without this field）。
    pub plan_digest: String,
}

impl VerificationPlan {
    pub const SCHEMA: &'static str = "verification-plan/v1";
}

/// 单次检查结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub kind: CheckKind,
    pub exit_code: i32,
    /// stdout+stderr 的 sha256 hex（避免落盘巨大日志）。
    pub output_digest: String,
    pub duration_ms: u64,
}

/// 总状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RunStatus {
    Pass,
    Fail,
}

/// 运行汇总。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResult {
    pub schema: String,
    pub status: RunStatus,
    pub plan_digest: String,
    /// git HEAD（尽力；失败则 empty）。
    pub commit: String,
    pub checks: Vec<CheckResult>,
}

impl RunResult {
    pub const SCHEMA: &'static str = "verification-run/v1";
}
