//! okxx 错误类型。

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("configure error: {0}")]
    Config(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
