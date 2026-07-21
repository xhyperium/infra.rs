//! 本 crate 本地错误类型（scaffold；非 contracts 公共面）。

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("not connected")]
    NotConnected,
    #[error("already connected")]
    AlreadyConnected,
    #[error("internal: {0}")]
    Internal(String),
}
