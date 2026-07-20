//! `infra-core` — 核心基础设施库。
//!
//! 提供可复用的错误类型与基础工具函数，作为 workspace 的起点 crate。

// ── 宪章强制 lint (CONSTITUTION.md §4) ────────

#![deny(missing_docs, unsafe_code)]
#![warn(clippy::todo)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

// ── 模块 ──────────────────────────────────────

/// 错误类型与序列化支持。
pub mod error;

pub use error::{Error, Result};

/// 返回库的问候语。
///
/// 主要用于冒烟测试与脚手架验证。
///
/// ```
/// assert_eq!(infra_core::hello(), "你好，infra-core");
/// ```
pub fn hello() -> &'static str {
    "你好，infra-core"
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "你好，infra-core");
    }

    #[test]
    fn test_error_from_io() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let infra_err: Error = err.into();
        assert!(matches!(infra_err, Error::Io(_)));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(Error::InvalidArgument("缺少字段".into()).to_string(), "参数无效: 缺少字段");
    }

    #[test]
    fn test_result_alias() {
        fn returns_result() -> Result<i32> {
            Err(Error::Config("测试".into()))
        }
        assert!(returns_result().is_err());
    }
}
