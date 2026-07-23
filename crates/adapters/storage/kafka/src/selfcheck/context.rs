//! 执行上下文：run_id / token / 取消 / 配置（规范 §5.2、§7）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::config::KafkaSelfCheckConfig;

/// 协作式取消标志（等价于 CancellationToken 的最小子集，避免新增依赖）。
#[derive(Debug, Clone, Default)]
pub struct CancelFlag {
    inner: Arc<AtomicBool>,
}

impl CancelFlag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

/// 单次验证运行上下文。
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub run_id: String,
    /// 资源命名 token：`_self_check_{token}`（C-07）。
    pub token: String,
    pub config: Arc<KafkaSelfCheckConfig>,
    pub cancel: CancelFlag,
}

impl ValidationContext {
    /// 新建运行上下文；token 默认由 run_id 派生。
    #[must_use]
    pub fn new(config: KafkaSelfCheckConfig) -> Self {
        let run_id = new_run_id();
        let token = sanitize_token(&run_id);
        Self { run_id, token, config: Arc::new(config), cancel: CancelFlag::new() }
    }

    /// 显式 token / run_id（测试用）。
    #[must_use]
    pub fn with_ids(
        config: KafkaSelfCheckConfig,
        run_id: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            token: sanitize_token(&token.into()),
            config: Arc::new(config),
            cancel: CancelFlag::new(),
        }
    }

    /// 自检 topic：`{prefix}_{token}` 或带 suffix。
    #[must_use]
    pub fn topic(&self, suffix: &str) -> String {
        let prefix = self.config.check_topic_prefix.trim();
        let prefix = if prefix.is_empty() { "_self_check" } else { prefix };
        if suffix.is_empty() {
            format!("{prefix}_{}", self.token)
        } else {
            format!("{prefix}_{}_{suffix}", self.token)
        }
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

fn new_run_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("sc-{nanos:x}-{:x}", std::process::id())
}

fn sanitize_token(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .take(40)
        .collect();
    if s.is_empty() { "token".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_namespace_follows_spec() {
        let ctx = ValidationContext::with_ids(KafkaSelfCheckConfig::default(), "run1", "abc_01");
        assert_eq!(ctx.topic(""), "_self_check_abc_01");
        assert_eq!(ctx.topic("rw"), "_self_check_abc_01_rw");
    }
}
