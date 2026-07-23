//! postgres 模块自检配置（规范 §9 中 `[selfcheck.modules.postgres]` 子集）。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// postgres 自检模块参数与检查项开关。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresSelfCheckConfig {
    /// 最低主版本号（`server_version_num / 10000`），默认 13。
    pub min_version: u32,
    /// 是否执行 `postgres.full.replication_lag`（默认 false → Skipped）。
    pub replica_check: bool,
    /// 副本延迟阈值（毫秒）；仅 `replica_check=true` 时生效。
    pub max_replication_lag_ms: u64,
    /// 按 ID 禁用检查项。
    pub skip: HashSet<String>,
    /// 基线覆盖（毫秒）。
    pub baseline_override_ms: HashMap<String, u64>,
    /// LISTEN/NOTIFY 等待超时（毫秒）。
    pub notify_wait_ms: u64,
    /// 池饱和 acquire 超时（毫秒）。
    pub pool_acquire_probe_ms: u64,
}

impl Default for PostgresSelfCheckConfig {
    fn default() -> Self {
        Self {
            min_version: 13,
            replica_check: false,
            max_replication_lag_ms: 1_000,
            skip: HashSet::new(),
            baseline_override_ms: HashMap::new(),
            notify_wait_ms: 2_000,
            pool_acquire_probe_ms: 300,
        }
    }
}

impl PostgresSelfCheckConfig {
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
