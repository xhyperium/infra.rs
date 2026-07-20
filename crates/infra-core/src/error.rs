use std::io;

/// 基础错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// 配置错误
    #[error("Config error: {0}")]
    Config(String),

    /// 不合法的参数
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// 内部错误（不应暴露给用户的未预期错误）
    #[error("Internal error: {0}")]
    Internal(String),
}

/// `infra-core` 的标准 `Result` 别名
pub type Result<T> = std::result::Result<T, Error>;
