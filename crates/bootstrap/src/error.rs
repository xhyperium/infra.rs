//! Bootstrap 组合错误（PLAN-GATE-RETIRE-001 §3.5 / DEFER-ERR-MAP）。

use kernel::{BoxError, ErrorKind, XError, XResult};

/// 启动期组装错误（非 exhaustive）。
///
/// 映射：
/// - [`BootstrapError::MissingDependency`] → [`ErrorKind::Missing`]
/// - [`BootstrapError::InvalidConfiguration`] → [`ErrorKind::Invalid`]
/// - [`BootstrapError::DependencyUnavailable`] → [`ErrorKind::Unavailable`]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    /// 必需 typed 依赖或生命周期所有权不存在。
    #[error("缺少必需依赖：{name}")]
    MissingDependency {
        /// 依赖逻辑名（如 `"evidence"`）。
        name: &'static str,
    },
    /// 配置非法。
    #[error("bootstrap 配置无效：{name}")]
    InvalidConfiguration {
        /// 配置项名。
        name: &'static str,
    },
    /// 依赖存在但不可用。
    #[error("依赖不可用：{name}")]
    DependencyUnavailable {
        /// 依赖逻辑名。
        name: &'static str,
        /// 下层原因。
        #[source]
        source: BoxError,
    },
}

impl BootstrapError {
    /// 映射为 [`XError`]（保留反应分类）。
    pub fn into_xerror(self) -> XError {
        match self {
            Self::MissingDependency { name } => XError::missing(format!("缺少必需依赖：{name}")),
            Self::InvalidConfiguration { name } => {
                XError::invalid(format!("bootstrap 配置无效：{name}"))
            }
            Self::DependencyUnavailable { name, source } => {
                XError::unavailable(format!("依赖不可用：{name}")).with_source(source)
            }
        }
    }

    /// 反应分类（与 into_xerror 一致）。
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::MissingDependency { .. } => ErrorKind::Missing,
            Self::InvalidConfiguration { .. } => ErrorKind::Invalid,
            Self::DependencyUnavailable { .. } => ErrorKind::Unavailable,
        }
    }
}

impl From<BootstrapError> for XError {
    fn from(value: BootstrapError) -> Self {
        value.into_xerror()
    }
}

/// 将 [`BootstrapError`] 结果提升为 [`XResult`]。
pub fn into_xresult<T>(r: Result<T, BootstrapError>) -> XResult<T> {
    r.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn maps_to_xerror_kinds() {
        assert_eq!(
            BootstrapError::MissingDependency { name: "evidence" }.kind(),
            ErrorKind::Missing
        );
        assert_eq!(
            BootstrapError::InvalidConfiguration { name: "timeout" }.kind(),
            ErrorKind::Invalid
        );
        let e = BootstrapError::DependencyUnavailable {
            name: "redis",
            source: Box::new(io::Error::other("下层不可用")),
        };
        assert_eq!(e.kind(), ErrorKind::Unavailable);
        let x: XError = e.into();
        assert_eq!(x.kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn display_and_source_chain() {
        let missing = BootstrapError::MissingDependency { name: "evidence" };
        assert_eq!(missing.to_string(), "缺少必需依赖：evidence");
        assert!(std::error::Error::source(&missing).is_none());

        let invalid = BootstrapError::InvalidConfiguration { name: "timeout" };
        assert_eq!(invalid.to_string(), "bootstrap 配置无效：timeout");
        assert!(std::error::Error::source(&invalid).is_none());

        let unavail = BootstrapError::DependencyUnavailable {
            name: "redis",
            source: Box::new(io::Error::other("下层不可用")),
        };
        assert_eq!(unavail.to_string(), "依赖不可用：redis");
        assert!(std::error::Error::source(&unavail).is_some());
    }

    #[test]
    fn into_xerror_and_into_xresult() {
        let missing = BootstrapError::MissingDependency { name: "evidence" };
        let x = missing.into_xerror();
        assert_eq!(x.kind(), ErrorKind::Missing);
        assert_eq!(x.context(), "缺少必需依赖：evidence");

        let invalid = BootstrapError::InvalidConfiguration { name: "x" };
        let x2: XError = invalid.into();
        assert_eq!(x2.kind(), ErrorKind::Invalid);
        assert_eq!(x2.context(), "bootstrap 配置无效：x");

        let ok: XResult<u8> = into_xresult(Ok(7));
        assert_eq!(ok.unwrap(), 7);

        let err: XResult<u8> =
            into_xresult(Err(BootstrapError::MissingDependency { name: "evidence" }));
        assert_eq!(err.unwrap_err().kind(), ErrorKind::Missing);

        let unavail = BootstrapError::DependencyUnavailable {
            name: "db",
            source: Box::new(io::Error::other("连接超时")),
        };
        let x3 = unavail.into_xerror();
        assert_eq!(x3.kind(), ErrorKind::Unavailable);
        assert_eq!(x3.context(), "依赖不可用：db");
        assert_eq!(
            std::error::Error::source(&x3).expect("必须保留下层原因").to_string(),
            "连接超时"
        );
    }

    #[test]
    fn debug_fmt_covers_variants() {
        let m = BootstrapError::MissingDependency { name: "a" };
        let i = BootstrapError::InvalidConfiguration { name: "b" };
        let u = BootstrapError::DependencyUnavailable {
            name: "c",
            source: Box::new(io::Error::other("e")),
        };
        assert!(format!("{m:?}").contains("MissingDependency"));
        assert!(format!("{i:?}").contains("InvalidConfiguration"));
        assert!(format!("{u:?}").contains("DependencyUnavailable"));
    }
}
