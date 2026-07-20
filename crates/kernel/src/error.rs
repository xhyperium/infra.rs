//! 全系统统一的错误分类与响应语义。
//!
//! # 设计原则
//!
//! 错误按"调用方应该如何反应"分类，而非按"错误来自哪个模块"分类。
//! 调用方不得通过字符串匹配决定是否重试、降级或 fail-fast。
//!
//! ## 下游编译负向合同（SPEC-KERNEL-002 §11.4）
//!
//! `XError` 字段保持私有，下游只能通过构造器和查询方法使用：
//!
//! ```compile_fail
//! use kernel::{ErrorKind, XError};
//!
//! let _ = XError {
//!     kind: ErrorKind::Internal,
//!     context: "not opaque".into(),
//!     retry_after: None,
//!     source: None,
//! };
//! ```
//!
//! kernel 不公开通用 `Component` trait：
//!
//! ```compile_fail
//! use kernel::Component;
//! ```
//!
//! 时间类型不提供 `Default`，避免用零值冒充有效时间：
//!
//! ```compile_fail
//! use kernel::Timestamp;
//!
//! let _ = Timestamp::default();
//! ```
//!
//! ```compile_fail
//! use kernel::MonotonicInstant;
//!
//! let _ = MonotonicInstant::default();
//! ```
//!
//! `ShutdownGuard` 不可克隆，唯一触发权不能被复制：
//!
//! ```compile_fail
//! use kernel::ShutdownSignal;
//!
//! let (guard, _signal) = ShutdownSignal::new();
//! let _second_guard = guard.clone();
//! ```

use std::borrow::Cow;
use std::fmt;

use crate::clock::ClockError;

// ---------------------------------------------------------------------------
// 公开类型别名
// ---------------------------------------------------------------------------

/// 可跨线程传递的装箱错误类型别名。
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// `kernel` crate 专用的 `Result` 别名，错误侧固定为 [`XError`]。
pub type XResult<T> = Result<T, XError>;

// ---------------------------------------------------------------------------
// ErrorKind
// ---------------------------------------------------------------------------

/// 错误的语义分类，按"调用方应如何反应"划分。
///
/// 禁止通过字符串匹配或类型断言替代对本枚举的匹配。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// 请求、参数或输入本身非法。
    ///
    /// 反应：不自动重试；修正输入后可重新提交；不表示系统故障。
    Invalid,
    /// 请求的实体、资源或已声明依赖不存在。
    ///
    /// 反应：不立即自动重试；调用方可选择 fallback。
    Missing,
    /// 输入本身合法，但与当前状态冲突。
    ///
    /// 反应：不按瞬时故障自动重试；只有状态变化后重试才有意义。
    Conflict,
    /// 暂时性失败，保持相同语义的重试可能成功。
    ///
    /// 反应：可使用退避和抖动重试；`retry_after` 仅为提示。
    Transient,
    /// 下层依赖或必要基础能力不可用。
    ///
    /// 反应：默认传播；由 lifecycle / composition 决定降级或 fail-fast。
    Unavailable,
    /// 操作被调用方或系统取消。
    ///
    /// 反应：不自动重试；不记录为内部故障；上层可将其视为正常终止路径。
    Cancelled,
    /// 操作未在调用方给定的 deadline 内完成。
    ///
    /// 反应：本次操作终止；是否重试由上层策略裁定。
    DeadlineExceeded,
    /// 内部不变量、前置条件或不可发生状态被破坏。
    ///
    /// 反应：不重试；视为 bug；必须进入错误预算、告警或受控 fail-fast。
    Invariant,
    /// 暂时无法归入以上类别的内部错误。
    ///
    /// 反应：不自动重试；必须进入使用量棘轮；长期目标是趋近于零。
    Internal,
}

// ---------------------------------------------------------------------------
// XError
// ---------------------------------------------------------------------------

/// `kernel` 中唯一的错误类型。
///
/// 所有字段均为私有，调用方只能通过构造器和查询方法使用错误语义。
pub struct XError {
    kind: ErrorKind,
    context: Cow<'static, str>,
    retry_after: Option<std::time::Duration>,
    source: Option<BoxError>,
}

impl XError {
    // -- 构造器 ------------------------------------------------------------

    /// 构造一个 [`ErrorKind::Invalid`] 错误。
    pub fn invalid(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Invalid,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Missing`] 错误。
    pub fn missing(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Missing,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Conflict`] 错误。
    pub fn conflict(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Conflict,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Transient`] 错误，无 `retry_after`。
    pub fn transient(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Transient,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Transient`] 错误，附带 `retry_after` 提示。
    pub fn transient_after(context: impl Into<String>, retry_after: std::time::Duration) -> Self {
        Self {
            kind: ErrorKind::Transient,
            context: Cow::Owned(context.into()),
            retry_after: Some(retry_after),
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Unavailable`] 错误。
    pub fn unavailable(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Unavailable,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Cancelled`] 错误。
    pub fn cancelled(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Cancelled,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::DeadlineExceeded`] 错误。
    pub fn deadline_exceeded(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::DeadlineExceeded,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Invariant`] 错误。
    pub fn invariant(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Invariant,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    /// 构造一个 [`ErrorKind::Internal`] 错误。
    ///
    /// 使用此构造器的调用点必须受棘轮约束；非紧急情况下应选择更精确的
    /// `ErrorKind`。
    pub fn internal(context: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            context: Cow::Owned(context.into()),
            retry_after: None,
            source: None,
        }
    }

    // -- 构建器 ------------------------------------------------------------

    /// 附加底层 error source，保持原有 [`ErrorKind`] 不变。
    pub fn with_source(mut self, source: impl Into<BoxError>) -> Self {
        self.source = Some(source.into());
        self
    }

    // -- 查询方法 ----------------------------------------------------------

    /// 返回错误的语义分类。
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// 返回人类可读的上下文描述。
    pub fn context(&self) -> &str {
        &self.context
    }

    /// 返回建议的重试等待时间（仅 [`ErrorKind::Transient`] 可能非 None）。
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        self.retry_after
    }

    /// 仅当 [`ErrorKind::Transient`] 时返回 `true`。
    pub fn is_retryable(&self) -> bool {
        self.kind == ErrorKind::Transient
    }

    /// 仅当 [`ErrorKind::Invariant`] 时返回 `true`。
    pub fn is_bug(&self) -> bool {
        self.kind == ErrorKind::Invariant
    }
}

// ---------------------------------------------------------------------------
// Trait 实现
// ---------------------------------------------------------------------------

impl fmt::Display for XError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.context)
    }
}

impl fmt::Debug for XError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug 与 Display 一致，不展开 source 细节
        f.debug_struct("XError")
            .field("kind", &self.kind)
            .field("context", &self.context)
            .field("retry_after", &self.retry_after)
            .field("source", &self.source.as_ref().map(|_| "..."))
            .finish()
    }
}

impl std::error::Error for XError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

// ---------------------------------------------------------------------------
// ClockError → XError 映射
// ---------------------------------------------------------------------------

impl From<ClockError> for XError {
    fn from(err: ClockError) -> Self {
        Self {
            kind: ErrorKind::Unavailable,
            context: Cow::Owned(err.to_string()),
            retry_after: None,
            source: Some(Box::new(err)),
        }
    }
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // -- 每个 ErrorKind 的构造器 -------------------------------------------

    #[test]
    fn test_invalid_constructor() {
        let err = XError::invalid("bad input");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert_eq!(err.context(), "bad input");
        assert_eq!(err.retry_after(), None);
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_missing_constructor() {
        let err = XError::missing("not found");
        assert_eq!(err.kind(), ErrorKind::Missing);
        assert_eq!(err.context(), "not found");
        assert_eq!(err.retry_after(), None);
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_conflict_constructor() {
        let err = XError::conflict("duplicate");
        assert_eq!(err.kind(), ErrorKind::Conflict);
        assert_eq!(err.context(), "duplicate");
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_transient_constructor() {
        let err = XError::transient("network blip");
        assert_eq!(err.kind(), ErrorKind::Transient);
        assert_eq!(err.context(), "network blip");
        assert_eq!(err.retry_after(), None);
        assert!(err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_transient_after_constructor() {
        let d = std::time::Duration::from_secs(5);
        let err = XError::transient_after("rate limited", d);
        assert_eq!(err.kind(), ErrorKind::Transient);
        assert_eq!(err.context(), "rate limited");
        assert_eq!(err.retry_after(), Some(d));
        assert!(err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_unavailable_constructor() {
        let err = XError::unavailable("storage offline");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        assert_eq!(err.context(), "storage offline");
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_cancelled_constructor() {
        let err = XError::cancelled("user cancelled");
        assert_eq!(err.kind(), ErrorKind::Cancelled);
        assert_eq!(err.context(), "user cancelled");
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_deadline_exceeded_constructor() {
        let err = XError::deadline_exceeded("timeout");
        assert_eq!(err.kind(), ErrorKind::DeadlineExceeded);
        assert_eq!(err.context(), "timeout");
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    #[test]
    fn test_invariant_constructor() {
        let err = XError::invariant("broken invariant");
        assert_eq!(err.kind(), ErrorKind::Invariant);
        assert_eq!(err.context(), "broken invariant");
        assert!(!err.is_retryable());
        assert!(err.is_bug());
    }

    #[test]
    fn test_internal_constructor() {
        let err = XError::internal("unknown error");
        assert_eq!(err.kind(), ErrorKind::Internal);
        assert_eq!(err.context(), "unknown error");
        assert!(!err.is_retryable());
        assert!(!err.is_bug());
    }

    // -- with_source -------------------------------------------------------

    #[test]
    fn test_with_source_preserves_kind() {
        let source = std::io::Error::other("io fail");
        let err = XError::transient("retry io").with_source(source);
        assert_eq!(err.kind(), ErrorKind::Transient);
        assert!(err.is_retryable());
        assert!(err.source().is_some());
    }

    #[test]
    fn test_with_source_chain_accessible() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = XError::missing("config not found").with_source(source);
        assert_eq!(err.kind(), ErrorKind::Missing);
        let src = err.source().unwrap();
        assert!(src.to_string().contains("file missing"));
    }

    // -- Display / Debug / Error chain -----------------------------------

    #[test]
    fn test_display_does_not_contain_source_details() {
        let source = std::io::Error::other("secret");
        let err = XError::internal("wrapped").with_source(source);
        let display = err.to_string();
        assert!(!display.contains("secret"));
    }

    #[test]
    fn test_debug_does_not_expand_source() {
        let source = std::io::Error::other("sensitive");
        let err = XError::internal("wrapped").with_source(source);
        let debug = format!("{:?}", err);
        assert!(!debug.contains("sensitive"));
    }

    #[test]
    fn test_error_source_chain() {
        let source = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = XError::invalid("bad permissions").with_source(source);
        assert!(err.source().is_some());
        assert!(err.source().unwrap().to_string().contains("access denied"));
    }

    // -- is_retryable / is_bug 精确性 ------------------------------------

    #[test]
    fn test_is_retryable_only_transient() {
        assert!(XError::transient("t").is_retryable());
        assert!(XError::transient_after("t", std::time::Duration::from_secs(1)).is_retryable());
        assert!(!XError::invalid("i").is_retryable());
        assert!(!XError::missing("m").is_retryable());
        assert!(!XError::conflict("c").is_retryable());
        assert!(!XError::unavailable("u").is_retryable());
        assert!(!XError::cancelled("ca").is_retryable());
        assert!(!XError::deadline_exceeded("d").is_retryable());
        assert!(!XError::invariant("inv").is_retryable());
        assert!(!XError::internal("int").is_retryable());
    }

    #[test]
    fn test_is_bug_only_invariant() {
        assert!(XError::invariant("inv").is_bug());
        assert!(!XError::invalid("i").is_bug());
        assert!(!XError::missing("m").is_bug());
        assert!(!XError::conflict("c").is_bug());
        assert!(!XError::transient("t").is_bug());
        assert!(!XError::unavailable("u").is_bug());
        assert!(!XError::cancelled("ca").is_bug());
        assert!(!XError::deadline_exceeded("d").is_bug());
        assert!(!XError::internal("int").is_bug());
    }

    // -- ClockError → XError（§5.7） --------------------------------------

    #[test]
    fn test_clock_error_maps_all_variants_to_unavailable() {
        for clock_err in
            [ClockError::BeforeUnixEpoch, ClockError::Overflow, ClockError::Unavailable]
        {
            let e: XError = clock_err.into();
            assert_eq!(e.kind(), ErrorKind::Unavailable);
            assert!(!e.is_retryable());
            assert!(!e.is_bug());
            assert!(Error::source(&e).is_some());
            // Display 不含 source 细节；kind 前缀固定
            let display = e.to_string();
            assert!(display.starts_with("Unavailable:"), "display={display}");
            assert!(!display.contains("secret"));
        }
    }
}
