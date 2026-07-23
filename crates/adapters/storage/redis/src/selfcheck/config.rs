//! redisx 模块自检配置（规范 §9 中 `[selfcheck.modules.redisx]` 子集）。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// redisx 自检模块参数与检查项开关。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSelfCheckConfig {
    /// 是否按集群语义执行 `cluster_slots`（也可由连接配置 mode 推断）。
    pub cluster_mode: bool,
    /// `used_memory / maxmemory` 上限（`maxmemory=0` 时跳过比例断言）。
    pub max_memory_ratio: f64,
    /// 按 ID 禁用检查项。
    pub skip: HashSet<String>,
    /// 基线覆盖（毫秒）。
    pub baseline_override_ms: HashMap<String, u64>,
    /// Pub/Sub 检查等待消息超时（毫秒）。
    pub pubsub_wait_ms: u64,
    /// TTL 语义检查：键存活时间（毫秒），应明显小于 wait。
    pub ttl_expire_ms: u64,
    /// TTL 语义检查：等待后读取的间隔（毫秒）。
    pub ttl_wait_ms: u64,
}

impl Default for RedisSelfCheckConfig {
    fn default() -> Self {
        Self {
            cluster_mode: false,
            max_memory_ratio: 0.9,
            skip: HashSet::new(),
            baseline_override_ms: HashMap::new(),
            pubsub_wait_ms: 2_000,
            ttl_expire_ms: 200,
            ttl_wait_ms: 350,
        }
    }
}

impl RedisSelfCheckConfig {
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
