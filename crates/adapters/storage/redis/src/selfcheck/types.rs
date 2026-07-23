//! 自验证结果模型（对齐 `.cargo/draft/verifyctl.md` / LIB-SELFCHECK-SPEC §5.1）。
//!
//! 本 crate 先落地 **redisx 模块**实现；跨模块调度器（`SelfValidator`）不在本包范围。

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 检查级别：高级别包含低级别（`Full ⊇ ReadWrite ⊇ Basic`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CheckLevel {
    /// 连通性 / liveness（≤ 3s）。
    Basic,
    /// 数据闭环 / readiness（≤ 15s）。
    ReadWrite,
    /// 功能完整性 / 巡检与 CD（≤ 120s）。
    Full,
}

impl CheckLevel {
    /// 级别最大墙钟时长（规范 §4）。
    #[must_use]
    pub fn max_duration(self) -> Duration {
        match self {
            Self::Basic => Duration::from_secs(3),
            Self::ReadWrite => Duration::from_secs(15),
            Self::Full => Duration::from_secs(120),
        }
    }

    /// 本级别需要执行的最低检查 category 集合（含自身）。
    #[must_use]
    pub fn includes(self, item_level: CheckLevel) -> bool {
        item_level <= self
    }

    /// 稳定 wire / 配置字符串。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::ReadWrite => "read_write",
            Self::Full => "full",
        }
    }
}

impl fmt::Display for CheckLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 检查状态 —— 四态模型（规范 §5.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CheckStatus {
    /// 功能正常且延迟在基线内。
    Passed,
    /// 功能正常但延迟超基线 —— 不计入失败，应告警。
    Degraded,
    /// 功能异常。
    Failed,
    /// 前置失败短路，或配置禁用 / 拓扑不适用。
    Skipped,
}

impl CheckStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// 单项检查结果。
#[derive(Debug, Clone, Serialize)]
pub struct CheckItem {
    /// 全局唯一 ID：`redisx.<category>.<name>`。
    pub id: String,
    pub status: CheckStatus,
    pub latency_ms: u128,
    pub baseline_ms: Option<u128>,
    /// Failed/Skipped 时建议非空；**不得**含密码 / 完整连接串。
    pub detail: Option<String>,
    /// RFC 3339 UTC（秒精度 + 毫秒小数）。
    pub started_at: String,
}

impl CheckItem {
    /// 构造结果；若有基线且功能成功但超时 → Degraded。
    #[must_use]
    pub fn finish(
        id: impl Into<String>,
        mut status: CheckStatus,
        latency: Duration,
        baseline_ms: Option<u128>,
        detail: Option<String>,
        started_at: String,
    ) -> Self {
        let latency_ms = latency.as_millis();
        if matches!(status, CheckStatus::Passed) {
            if let Some(base) = baseline_ms {
                if latency_ms > base {
                    status = CheckStatus::Degraded;
                    let over = latency_ms.saturating_sub(base);
                    let pct =
                        over.checked_mul(100).and_then(|n| n.checked_div(base)).unwrap_or(100);
                    let deg = format!("延迟超基线 {pct}%（{latency_ms}ms > {base}ms）");
                    return Self {
                        id: id.into(),
                        status,
                        latency_ms,
                        baseline_ms,
                        detail: Some(match detail {
                            Some(d) if !d.is_empty() => format!("{d}; {deg}"),
                            _ => deg,
                        }),
                        started_at,
                    };
                }
            }
        }
        Self { id: id.into(), status, latency_ms, baseline_ms, detail, started_at }
    }

    #[must_use]
    pub fn skipped(id: impl Into<String>, reason: impl Into<String>, started_at: String) -> Self {
        Self {
            id: id.into(),
            status: CheckStatus::Skipped,
            latency_ms: 0,
            baseline_ms: None,
            detail: Some(reason.into()),
            started_at,
        }
    }
}

/// 模块级验证报告。
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub module: String,
    pub level: CheckLevel,
    /// 无 Failed 即为 true（Degraded 不影响）。
    pub passed: bool,
    pub degraded: bool,
    pub total_ms: u128,
    pub items: Vec<CheckItem>,
}

impl ValidationReport {
    #[must_use]
    pub fn from_items(module: impl Into<String>, level: CheckLevel, items: Vec<CheckItem>) -> Self {
        let passed = items.iter().all(|i| i.status != CheckStatus::Failed);
        let degraded = items.iter().any(|i| i.status == CheckStatus::Degraded);
        let total_ms = items.iter().map(|i| i.latency_ms).sum();
        Self { module: module.into(), level, passed, degraded, total_ms, items }
    }

    /// 机器可读 JSON 报告（规范 §10.1 模块子集）。
    ///
    /// # Errors
    ///
    /// 序列化失败时返回错误字符串（极少发生）。
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("ValidationReport JSON: {e}"))
    }
}

/// 检查项元描述（catalog / 配置校验）。
#[derive(Debug, Clone, Serialize)]
pub struct CheckDescriptor {
    pub id: String,
    pub level: CheckLevel,
    pub default_baseline_ms: Option<u64>,
    pub description: String,
    /// 是否含 DDL/Admin 等重操作（redisx 目录当前均为 false）。
    pub destructive: bool,
}

/// 当前 UTC 时间戳字符串。
#[must_use]
pub fn now_rfc3339() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    // 简易 UTC：不依赖 chrono；足够用于报告关联
    let days = secs / 86_400;
    let day_secs = secs % 86_400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    // 1970-01-01 + days（足够自检报告，非完整日历库）
    let (y, mo, da) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{da:02}T{hour:02}:{min:02}:{sec:02}.{millis:03}Z")
}

/// Howard Hinnant civil_from_days（公历）。
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_includes_is_nested() {
        assert!(CheckLevel::Full.includes(CheckLevel::Basic));
        assert!(CheckLevel::Full.includes(CheckLevel::ReadWrite));
        assert!(!CheckLevel::Basic.includes(CheckLevel::Full));
        assert!(CheckLevel::ReadWrite.includes(CheckLevel::Basic));
    }

    #[test]
    fn degraded_when_over_baseline() {
        let item = CheckItem::finish(
            "redisx.basic.ping",
            CheckStatus::Passed,
            Duration::from_millis(50),
            Some(20),
            None,
            now_rfc3339(),
        );
        assert_eq!(item.status, CheckStatus::Degraded);
        assert!(item.detail.as_ref().is_some_and(|d| d.contains("超基线")));
    }

    #[test]
    fn report_passed_ignores_degraded() {
        let items = vec![
            CheckItem::finish(
                "a",
                CheckStatus::Passed,
                Duration::from_millis(1),
                Some(10),
                None,
                now_rfc3339(),
            ),
            CheckItem::finish(
                "b",
                CheckStatus::Passed,
                Duration::from_millis(100),
                Some(10),
                None,
                now_rfc3339(),
            ),
        ];
        let r = ValidationReport::from_items("redisx", CheckLevel::Basic, items);
        assert!(r.passed);
        assert!(r.degraded);
    }
}
