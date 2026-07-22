//! 本地证据策略说明（非远程合规）。

/// 是否允许将 InMemory 用于生产合规审计。
#[must_use]
pub const fn allows_in_memory_for_compliance() -> bool {
    false
}

/// 文件追加器是否满足最小持久化合同。
#[must_use]
pub const fn file_appender_is_min_durable() -> bool {
    true
}

/// 策略摘要。
#[must_use]
pub fn policy_summary() -> &'static str {
    "in-memory=dev-only; file=min-durable; remote=transport-injected; sign=hmac-sha256"
}

/// 评估后端是否可用于给定用途。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendClass {
    /// 仅开发/测试。
    DevOnly,
    /// 本地最小持久化。
    LocalMinDurable,
    /// 未交付。
    Deferred,
}

/// 分类内存后端。
#[must_use]
pub fn classify_in_memory() -> BackendClass {
    BackendClass::DevOnly
}

/// 分类文件后端。
#[must_use]
pub fn classify_file() -> BackendClass {
    BackendClass::LocalMinDurable
}

/// 分类远程后端。
///
/// 可注入 [`crate::EvidenceTransport`] 的远程追加为 **DevOnly**（非合规唯一落盘）。
#[must_use]
pub fn classify_remote() -> BackendClass {
    BackendClass::DevOnly
}

/// 是否允许作为合规唯一落盘。
#[must_use]
pub fn allows_as_sole_compliance_store(class: BackendClass) -> bool {
    matches!(class, BackendClass::LocalMinDurable)
        && file_appender_is_min_durable()
        && !allows_in_memory_for_compliance()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_matrix() {
        assert!(!allows_in_memory_for_compliance());
        assert!(file_appender_is_min_durable());
        assert!(policy_summary().contains("dev-only"));
        assert_eq!(classify_in_memory(), BackendClass::DevOnly);
        assert_eq!(classify_file(), BackendClass::LocalMinDurable);
        assert_eq!(classify_remote(), BackendClass::DevOnly);
        assert!(!allows_as_sole_compliance_store(BackendClass::DevOnly));
        assert!(allows_as_sole_compliance_store(BackendClass::LocalMinDurable));
        assert!(!allows_as_sole_compliance_store(BackendClass::Deferred));
        let _ = format!("{:?}", BackendClass::DevOnly);
        for _ in 0..30 {
            assert_eq!(classify_remote(), BackendClass::DevOnly);
        }
    }
}
