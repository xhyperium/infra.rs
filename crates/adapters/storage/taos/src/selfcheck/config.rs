//! taos 模块自检配置（规范 §9 中 `[selfcheck.modules.taos]` 子集）。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// taos 自检模块参数与检查项开关。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaosSelfCheckConfig {
    /// 期望库精度（`ms` / `us` / `ns`）；`None` 时 `db_config` 只报告当前生效精度。
    pub expected_precision: Option<String>,
    /// 自动子表批量行数（`auto_subtable`，默认 300）。
    pub auto_subtable_rows: usize,
    /// 按 ID 禁用检查项。
    pub skip: HashSet<String>,
    /// 基线覆盖（毫秒）。
    pub baseline_override_ms: HashMap<String, u64>,
}

impl Default for TaosSelfCheckConfig {
    fn default() -> Self {
        Self {
            expected_precision: None,
            auto_subtable_rows: 300,
            skip: HashSet::new(),
            baseline_override_ms: HashMap::new(),
        }
    }
}

impl TaosSelfCheckConfig {
    #[must_use]
    pub fn is_skipped(&self, id: &str) -> bool {
        self.skip.contains(id)
    }

    /// 解析基线：覆盖优先，否则使用 catalog 默认。
    #[must_use]
    pub fn baseline_ms(&self, id: &str, default: Option<u64>) -> Option<u128> {
        self.baseline_override_ms.get(id).copied().or(default).map(u128::from)
    }
}
