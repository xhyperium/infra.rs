//! 执行上下文：run_id / token / 取消 / 配置（规范 §5.2、§7）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::config::PostgresSelfCheckConfig;

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
    pub config: Arc<PostgresSelfCheckConfig>,
    pub cancel: CancelFlag,
}

impl ValidationContext {
    /// 新建运行上下文；token 默认由 run_id 派生。
    #[must_use]
    pub fn new(config: PostgresSelfCheckConfig) -> Self {
        let run_id = new_run_id();
        let token = sanitize_token(&run_id);
        Self { run_id, token, config: Arc::new(config), cancel: CancelFlag::new() }
    }

    /// 显式 token / run_id（测试用）。
    #[must_use]
    pub fn with_ids(
        config: PostgresSelfCheckConfig,
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

    /// 自检表名：`_self_check_{token}`（可选 suffix 保证多表隔离）。
    #[must_use]
    pub fn table(&self, suffix: &str) -> String {
        if suffix.is_empty() {
            format!("_self_check_{}", self.token)
        } else {
            format!("_self_check_{}_{suffix}", self.token)
        }
    }

    /// LISTEN/NOTIFY 频道名（合法标识符）。
    #[must_use]
    pub fn channel(&self) -> String {
        format!("sc_{}", self.token)
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

/// 仅保留 SQL 标识符安全字符，最长 40。
fn sanitize_token(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .take(40)
        .collect();
    if s.is_empty() { "token".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_namespace_follows_spec() {
        let ctx = ValidationContext::with_ids(PostgresSelfCheckConfig::default(), "run1", "abc_01");
        assert_eq!(ctx.table(""), "_self_check_abc_01");
        assert_eq!(ctx.table("crud"), "_self_check_abc_01_crud");
        assert!(ctx.table("").starts_with("_self_check_"));
        assert_eq!(ctx.channel(), "sc_abc_01");
    }
}
