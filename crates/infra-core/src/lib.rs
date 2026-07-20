//! `infra-core` — 核心基础设施库。
//!
//! 提供可复用的错误类型与基础工具函数，作为 workspace 的起点 crate。

pub mod error;

pub use error::{Error, Result};

/// 返回库的问候语。
///
/// 主要用于冒烟测试与脚手架验证。
///
/// # 示例
///
/// ```
/// assert_eq!(infra_core::hello(), "你好，infra-core");
/// ```
pub fn hello() -> &'static str {
    "你好，infra-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 问候语() {
        assert_eq!(hello(), "你好，infra-core");
    }

    #[test]
    fn 错误可从_io_转换() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let infra_err: Error = err.into();
        assert!(matches!(infra_err, Error::Io(_)));
    }

    #[test]
    fn 错误显示() {
        assert_eq!(
            Error::InvalidArgument("缺少字段".into()).to_string(),
            "参数无效: 缺少字段"
        );
    }

    #[test]
    fn result_别名() {
        fn returns_result() -> Result<i32> {
            Err(Error::Config("测试".into()))
        }
        assert!(returns_result().is_err());
    }
}
