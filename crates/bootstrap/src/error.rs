//! Bootstrap 组合错误（PLAN-GATE-RETIRE-001 §3.5 / DEFER-ERR-MAP）。

use kernel::{BoxError, ErrorKind, XError, XResult};
use std::fmt;

/// 启动期组装错误（非 exhaustive）。
///
/// 映射：
/// - [`BootstrapError::MissingDependency`] → [`ErrorKind::Missing`]
/// - [`BootstrapError::InvalidConfiguration`] → [`ErrorKind::Invalid`]
/// - [`BootstrapError::DependencyUnavailable`] → [`ErrorKind::Unavailable`]
#[non_exhaustive]
#[derive(Debug)]
pub enum BootstrapError {
    /// 必需 typed 依赖未注入。
    MissingDependency {
        /// 依赖逻辑名（如 `"evidence"`）。
        name: &'static str,
    },
    /// 配置非法。
    InvalidConfiguration {
        /// 配置项名。
        name: &'static str,
    },
    /// 依赖存在但不可用。
    DependencyUnavailable {
        /// 依赖逻辑名。
        name: &'static str,
        /// 下层原因。
        source: BoxError,
    },
}

impl fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDependency { name } => {
                write!(f, "missing required dependency: {name}")
            }
            Self::InvalidConfiguration { name } => {
                write!(f, "invalid bootstrap configuration: {name}")
            }
            Self::DependencyUnavailable { name, source } => {
                write!(f, "dependency unavailable: {name}: {source}")
            }
        }
    }
}

impl std::error::Error for BootstrapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DependencyUnavailable { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl BootstrapError {
    /// 映射为 [`XError`]（保留反应分类）。
    pub fn into_xerror(self) -> XError {
        match self {
            Self::MissingDependency { name } => XError::missing(format!("dependency:{name}")),
            Self::InvalidConfiguration { name } => {
                XError::invalid(format!("bootstrap config:{name}"))
            }
            Self::DependencyUnavailable { name, source } => {
                XError::unavailable(format!("dependency:{name}")).with_source(source)
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
            source: Box::new(io::Error::other("down")),
        };
        assert_eq!(e.kind(), ErrorKind::Unavailable);
        let x: XError = e.into();
        assert_eq!(x.kind(), ErrorKind::Unavailable);
    }

    #[test]
    fn display_and_source_chain() {
        let missing = BootstrapError::MissingDependency { name: "evidence" };
        assert_eq!(missing.to_string(), "missing required dependency: evidence");
        assert!(std::error::Error::source(&missing).is_none());

        let invalid = BootstrapError::InvalidConfiguration { name: "timeout" };
        assert_eq!(invalid.to_string(), "invalid bootstrap configuration: timeout");
        assert!(std::error::Error::source(&invalid).is_none());

        let unavail = BootstrapError::DependencyUnavailable {
            name: "redis",
            source: Box::new(io::Error::other("down")),
        };
        assert!(unavail.to_string().contains("dependency unavailable: redis"));
        assert!(std::error::Error::source(&unavail).is_some());
    }

    #[test]
    fn into_xerror_and_into_xresult() {
        let missing = BootstrapError::MissingDependency { name: "evidence" };
        let x = missing.into_xerror();
        assert_eq!(x.kind(), ErrorKind::Missing);

        let invalid = BootstrapError::InvalidConfiguration { name: "x" };
        let x2: XError = invalid.into();
        assert_eq!(x2.kind(), ErrorKind::Invalid);

        let ok: XResult<u8> = into_xresult(Ok(7));
        assert_eq!(ok.unwrap(), 7);

        let err: XResult<u8> =
            into_xresult(Err(BootstrapError::MissingDependency { name: "evidence" }));
        assert_eq!(err.unwrap_err().kind(), ErrorKind::Missing);

        let unavail = BootstrapError::DependencyUnavailable {
            name: "db",
            source: Box::new(io::Error::other("timeout")),
        };
        assert_eq!(unavail.into_xerror().kind(), ErrorKind::Unavailable);
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
