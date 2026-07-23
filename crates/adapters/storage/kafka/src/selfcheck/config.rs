//! kafka 模块自检配置（规范 §9 中 `[selfcheck.modules.kafka]` 子集）。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// kafka 自检模块参数与检查项开关。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaSelfCheckConfig {
    /// broker 数量下限（bootstrap 列表计数；默认 1，适配单节点隔离）。
    pub min_brokers: u32,
    /// 自检 topic 前缀（规范默认 `_self_check`）。
    pub check_topic_prefix: String,
    /// Full 大消息字节数（默认 256 KiB；可配置 skip）。
    pub large_message_bytes: usize,
    /// 收发等待超时（毫秒）。
    pub consume_wait_ms: u64,
    /// 按 ID 禁用检查项。
    pub skip: HashSet<String>,
    /// 基线覆盖（毫秒）。
    pub baseline_override_ms: HashMap<String, u64>,
    /// topic 副本因子（单节点集群用 1）。
    pub replication: i16,
}

impl Default for KafkaSelfCheckConfig {
    fn default() -> Self {
        Self {
            min_brokers: 1,
            check_topic_prefix: "_self_check".into(),
            large_message_bytes: 256 * 1024,
            consume_wait_ms: 8_000,
            skip: HashSet::new(),
            baseline_override_ms: HashMap::new(),
            replication: 1,
        }
    }
}

impl KafkaSelfCheckConfig {
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
