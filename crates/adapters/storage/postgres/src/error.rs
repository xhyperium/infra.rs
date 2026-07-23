//! Postgres / deadpool 错误 → [`kernel::XError`] 映射。
//!
//! SQLSTATE 分类基于 PostgreSQL 官方错误类；未知码回落为 `Internal`。
//!
//! ## 映射锚点（与 draft 表对照）
//!
//! | SQLSTATE | 含义 | [`ErrorKind`] |
//! |----------|------|---------------|
//! | `23505` | unique_violation | `Conflict` |
//! | `23503` / `23502` / `23514` | FK / not-null / check | **`Invalid`**（本仓选型：约束输入非法；非乐观并发 `Conflict`） |
//! | `40P01` / `40001` | deadlock / serialization | `Transient` |
//! | `42P01` | undefined_table | `Missing` |
//! | `08*` | connection exception | `Unavailable` |
//! | `57014` | query_canceled | `Cancelled` |
//!
//! draft 中 FK 可倾向 `Conflict`；本仓固定 **`Invalid`**，调用方应按码语义处理。

use kernel::{ErrorKind, XError};
use std::fmt;

/// 事务业务失败且回滚也失败时的结构化复合 source。
///
/// [`std::error::Error::source`] 沿主业务错误继续；[`Self::rollback`] 保留独立回滚
/// 错误及其完整 source chain，调用方可从外层 [`XError`] source downcast 后分别处理。
#[derive(Debug)]
pub struct TransactionRollbackFailure {
    original: XError,
    rollback: XError,
}

impl TransactionRollbackFailure {
    /// 构造双失败 source。
    #[must_use]
    pub fn new(original: XError, rollback: XError) -> Self {
        Self { original, rollback }
    }

    /// 原始业务/操作错误。
    #[must_use]
    pub fn original(&self) -> &XError {
        &self.original
    }

    /// 回滚错误。
    #[must_use]
    pub fn rollback(&self) -> &XError {
        &self.rollback
    }
}

impl fmt::Display for TransactionRollbackFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "事务原操作失败且回滚失败")
    }
}

impl std::error::Error for TransactionRollbackFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.original)
    }
}

/// 将 SQLSTATE 五字符码映射为 [`ErrorKind`]。
///
/// 仅依赖码本身，便于单测；消息上下文由调用方附加。
#[must_use]
pub fn error_kind_from_sqlstate(code: &str) -> ErrorKind {
    // 类（前两位）+ 常见精确码
    match code {
        // Class 08 — Connection Exception
        c if c.starts_with("08") => ErrorKind::Unavailable,

        // Class 28 — Invalid Authorization Specification
        "28P01" | "28000" => ErrorKind::Invalid,

        // Class 3D — Invalid Catalog Name
        "3D000" => ErrorKind::Invalid,

        // Class 22 — Data Exception
        c if c.starts_with("22") => ErrorKind::Invalid,

        // Class 23 — Integrity Constraint Violation
        "23505" => ErrorKind::Conflict, // unique_violation
        "23503" => ErrorKind::Invalid,  // foreign_key_violation
        "23502" => ErrorKind::Invalid,  // not_null_violation
        "23514" => ErrorKind::Invalid,  // check_violation
        c if c.starts_with("23") => ErrorKind::Conflict,

        // Class 25 — Invalid Transaction State
        c if c.starts_with("25") => ErrorKind::Invariant,

        "40P01" => ErrorKind::Transient, // deadlock_detected
        "40001" => ErrorKind::Transient, // serialization_failure
        c if c.starts_with("40") => ErrorKind::Transient,

        // Class 42 — Syntax Error or Access Rule Violation
        "42P01" => ErrorKind::Missing, // undefined_table
        "42703" => ErrorKind::Invalid, // undefined_column
        "42704" => ErrorKind::Missing, // undefined_object
        "42601" => ErrorKind::Invalid, // syntax_error
        "42501" => ErrorKind::Invalid, // insufficient_privilege
        c if c.starts_with("42") => ErrorKind::Invalid,

        // Class 53 — Insufficient Resources
        "53300" => ErrorKind::Transient, // too_many_connections
        "53200" => ErrorKind::Transient, // out_of_memory
        c if c.starts_with("53") => ErrorKind::Transient,

        // Class 55 — Object Not In Prerequisite State
        "55P03" => ErrorKind::Transient, // lock_not_available
        c if c.starts_with("55") => ErrorKind::Transient,

        // Class 57 — Operator Intervention
        "57014" => ErrorKind::Cancelled, // query_canceled
        "57P01" | "57P02" | "57P03" => ErrorKind::Unavailable,
        c if c.starts_with("57") => ErrorKind::Unavailable,

        // Class 58 — System Error
        c if c.starts_with("58") => ErrorKind::Unavailable,

        // Class XX — Internal Error
        c if c.starts_with("XX") => ErrorKind::Internal,

        // Class 53 already handled; P0001 raise_exception → Invalid-ish
        "P0001" => ErrorKind::Invalid,

        _ => ErrorKind::Internal,
    }
}

/// 由 SQLSTATE 构造带上下文的 [`XError`]。
#[must_use]
pub fn xerror_from_sqlstate(code: &str, message: impl Into<String>) -> XError {
    let kind = error_kind_from_sqlstate(code);
    let msg = message.into();
    let context = format!("postgres sqlstate={code}: {msg}");
    match kind {
        ErrorKind::Invalid => XError::invalid(context),
        ErrorKind::Missing => XError::missing(context),
        ErrorKind::Conflict => XError::conflict(context),
        ErrorKind::Transient => XError::transient(context),
        ErrorKind::Unavailable => XError::unavailable(context),
        ErrorKind::Cancelled => XError::cancelled(context),
        ErrorKind::DeadlineExceeded => XError::deadline_exceeded(context),
        ErrorKind::Invariant => XError::invariant(context),
        ErrorKind::Internal => XError::internal(context),
        // non_exhaustive：未来变体
        _ => XError::internal(context),
    }
}

/// 映射 `tokio_postgres::Error`。
pub fn map_tokio_error(err: tokio_postgres::Error) -> XError {
    if err.is_closed() {
        return XError::unavailable(format!("postgres 连接已关闭: {err}")).with_source(err);
    }
    if let Some(db) = err.as_db_error() {
        let code = db.code().code();
        let message = db.message().to_string();
        return xerror_from_sqlstate(code, message).with_source(err);
    }
    // 连接期 / IO / 超时等
    let s = err.to_string();
    if s.contains("timed out") || s.contains("timeout") {
        return XError::deadline_exceeded(format!("postgres: {s}")).with_source(err);
    }
    if s.contains("connect") || s.contains("Connection") || s.contains("connection") {
        return XError::unavailable(format!("postgres: {s}")).with_source(err);
    }
    XError::internal(format!("postgres: {s}")).with_source(err)
}

/// 映射 deadpool 获取连接错误。
pub fn map_pool_error(err: deadpool_postgres::PoolError) -> XError {
    match err {
        deadpool_postgres::PoolError::Timeout(_) => {
            XError::deadline_exceeded("postgres 连接池超时".to_string())
        }
        deadpool_postgres::PoolError::Backend(e) => map_tokio_error(e),
        deadpool_postgres::PoolError::Closed => {
            XError::unavailable("postgres 连接池已关闭".to_string())
        }
        deadpool_postgres::PoolError::NoRuntimeSpecified => {
            XError::invariant("postgres 连接池未配置 runtime".to_string())
        }
        deadpool_postgres::PoolError::PostCreateHook(e) => {
            XError::unavailable(format!("postgres 连接创建后钩子失败: {e}"))
        }
    }
}

/// 映射建池错误。
pub fn map_create_pool_error(err: deadpool_postgres::CreatePoolError) -> XError {
    XError::unavailable(format!("postgres 创建连接池失败: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;
    use std::error::Error;
    use std::io;

    #[test]
    fn sqlstate_unique_violation_is_conflict() {
        assert_eq!(error_kind_from_sqlstate("23505"), ErrorKind::Conflict);
    }

    #[test]
    fn sqlstate_fk_is_invalid() {
        assert_eq!(error_kind_from_sqlstate("23503"), ErrorKind::Invalid);
    }

    #[test]
    fn sqlstate_undefined_table_is_missing() {
        assert_eq!(error_kind_from_sqlstate("42P01"), ErrorKind::Missing);
    }

    #[test]
    fn sqlstate_deadlock_is_transient() {
        assert_eq!(error_kind_from_sqlstate("40P01"), ErrorKind::Transient);
        assert_eq!(error_kind_from_sqlstate("40001"), ErrorKind::Transient);
    }

    #[test]
    fn sqlstate_query_canceled_is_cancelled() {
        assert_eq!(error_kind_from_sqlstate("57014"), ErrorKind::Cancelled);
    }

    #[test]
    fn sqlstate_connection_class_is_unavailable() {
        assert_eq!(error_kind_from_sqlstate("08006"), ErrorKind::Unavailable);
        assert_eq!(error_kind_from_sqlstate("08000"), ErrorKind::Unavailable);
    }

    #[test]
    fn sqlstate_auth_is_invalid() {
        assert_eq!(error_kind_from_sqlstate("28P01"), ErrorKind::Invalid);
    }

    #[test]
    fn sqlstate_too_many_connections_is_transient() {
        assert_eq!(error_kind_from_sqlstate("53300"), ErrorKind::Transient);
    }

    #[test]
    fn sqlstate_unknown_is_internal() {
        assert_eq!(error_kind_from_sqlstate("99999"), ErrorKind::Internal);
    }

    #[test]
    fn xerror_preserves_kind() {
        let e = xerror_from_sqlstate("23505", "duplicate key");
        assert_eq!(e.kind(), ErrorKind::Conflict);
        assert!(e.context().contains("23505"));
    }

    #[test]
    fn transaction_rollback_failure_preserves_both_source_branches() {
        let original = XError::deadline_exceeded("业务超时")
            .with_source(io::Error::new(io::ErrorKind::TimedOut, "business-source"));
        let rollback = XError::unavailable("回滚断连")
            .with_source(io::Error::new(io::ErrorKind::ConnectionReset, "rollback-source"));
        let composite = TransactionRollbackFailure::new(original, rollback);

        assert_eq!(composite.original().kind(), ErrorKind::DeadlineExceeded);
        assert_eq!(composite.rollback().kind(), ErrorKind::Unavailable);
        assert!(Error::source(composite.original()).is_some());
        assert!(Error::source(composite.rollback()).is_some());
        assert!(Error::source(&composite).is_some());
    }
}
