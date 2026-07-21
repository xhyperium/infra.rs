//! `clickhousex` — clickhouse storage adapter。
//!
//! 实现 contracts trait，连接 clickhouse。

pub use self::error::{Error, Result};

mod error;
