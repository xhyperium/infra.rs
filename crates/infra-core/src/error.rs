use serde::{de, ser};
use std::fmt;
use std::io;

/// `infra-core` 的基础错误类型。
///
/// 涵盖基础设施层面最常见的四类错误：I/O、配置、参数和内部错误。
/// 所有变体均通过 `thiserror` 自动实现 [`std::fmt::Display`] 和 [`std::error::Error`]。
/// 同时手动实现 [`serde::Serialize`] 和 [`serde::Deserialize`]。
///
/// # 序列化格式
///
/// 所有变体序列化为 `{"kind": "<变体名>", "message": "<错误信息>"}`：
///
/// | 变体 | `kind` 值 | `message` 内容 |
/// |------|-----------|---------------|
/// | `Io` | `"Io"` | `io::Error` 的 `Display` 字符串 |
/// | `Config` | `"Config"` | 错误描述 |
/// | `InvalidArgument` | `"InvalidArgument"` | 错误描述 |
/// | `Internal` | `"Internal"` | 错误描述 |
///
/// 反序列化时 `Io` 重建为 `io::Error::other(...)`，其余变体原样还原。
///
/// # 使用示例
///
/// ```
/// # use infra_core::{Error, Result};
/// fn load_config(path: &str) -> Result<String> {
///     std::fs::read_to_string(path).map_err(Error::from)
/// }
///
/// fn validate_port(port: u16) -> Result<u16> {
///     if port == 0 {
///         Err(Error::InvalidArgument("port cannot be zero".into()))
///     } else {
///         Ok(port)
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O 操作错误。
    ///
    /// 用于文件读写、网络 I/O 等系统级操作失败的场景。
    /// 实现了 `From<io::Error>`，可以直接使用 `?` 传播。
    ///
    /// `Display` 格式: `I/O error: <底层 io::Error 的 Display>`
    ///
    /// # 示例
    ///
    /// ```
    /// # use infra_core::Error;
    /// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    /// let err: Error = io_err.into();
    /// assert!(matches!(err, Error::Io(_)));
    /// assert_eq!(err.to_string(), "I/O error: file not found");
    /// ```
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// 配置错误。
    ///
    /// 用于配置加载、解析、校验失败等场景。
    ///
    /// `Display` 格式: `Config error: <具体说明>`
    ///
    /// # 示例
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::Config("missing required field 'host'".into());
    /// assert_eq!(err.to_string(), "Config error: missing required field 'host'");
    /// ```
    #[error("Config error: {0}")]
    Config(String),

    /// 参数校验错误。
    ///
    /// 用于函数入参不合法，如空字符串、越界索引、非法端口号等。
    ///
    /// `Display` 格式: `Invalid argument: <具体说明>`
    ///
    /// # 示例
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::InvalidArgument("port must be in range 1..=65535".into());
    /// assert_eq!(
    ///     err.to_string(),
    ///     "Invalid argument: port must be in range 1..=65535"
    /// );
    /// ```
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// 内部错误。
    ///
    /// 用于不应暴露给最终用户的未预期错误，例如不变量被破坏、状态机异常等。
    /// 上层可将其映射为通用错误响应，避免泄漏内部实现细节。
    ///
    /// `Display` 格式: `Internal error: <具体说明>`
    ///
    /// # 示例
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::Internal("connection pool exhausted beyond limit".into());
    /// assert_eq!(
    ///     err.to_string(),
    ///     "Internal error: connection pool exhausted beyond limit"
    /// );
    /// ```
    #[error("Internal error: {0}")]
    Internal(String),
}

// ── serde::Serialize ──────────────────────────────────────────────

impl ser::Serialize for Error {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeStruct;

        let mut s = serializer.serialize_struct("Error", 2)?;
        let (kind, message) = match self {
            Error::Io(e) => ("Io", e.to_string()),
            Error::Config(m) => ("Config", m.clone()),
            Error::InvalidArgument(m) => ("InvalidArgument", m.clone()),
            Error::Internal(m) => ("Internal", m.clone()),
        };
        s.serialize_field("kind", kind)?;
        s.serialize_field("message", &message)?;
        s.end()
    }
}

// ── serde::Deserialize ────────────────────────────────────────────

impl<'de> de::Deserialize<'de> for Error {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Kind,
            Message,
        }

        struct ErrorVisitor;

        impl<'de> de::Visitor<'de> for ErrorVisitor {
            type Value = Error;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a struct with `kind` and `message` fields")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Error, A::Error> {
                let mut kind: Option<String> = None;
                let mut message: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Kind => {
                            if kind.is_some() {
                                return Err(de::Error::duplicate_field("kind"));
                            }
                            kind = Some(map.next_value()?);
                        }
                        Field::Message => {
                            if message.is_some() {
                                return Err(de::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                    }
                }
                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let message = message.ok_or_else(|| de::Error::missing_field("message"))?;
                match kind.as_str() {
                    "Io" => Ok(Error::Io(io::Error::other(message))),
                    "Config" => Ok(Error::Config(message)),
                    "InvalidArgument" | "InvalidArgument" => Ok(Error::InvalidArgument(message)),
                    "Internal" => Ok(Error::Internal(message)),
                    other => Err(de::Error::unknown_variant(other, &["Io", "Config", "InvalidArgument", "Internal"])),
                }
            }
        }

        deserializer.deserialize_struct("Error", &["kind", "message"], ErrorVisitor)
    }
}

/// `infra-core` 的标准 [`Result`] 类型别名。
///
/// 等价于 `std::result::Result<T, infra_core::Error>`。
///
/// 使用此别名可避免每处都写 `<T, Error>`，保持代码简洁。
///
/// # 示例
///
/// ```
/// # use infra_core::Result;
/// fn do_work() -> Result<i32> {
///     Ok(42)
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_config_error() {
        let err = Error::Config("timeout missing".into());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"Config""#));
        assert!(json.contains(r#""message":"timeout missing""#));
    }

    #[test]
    fn deserialize_config_error() {
        let json = r#"{"kind":"Config","message":"timeout missing"}"#;
        let err: Error = serde_json::from_str(json).unwrap();
        assert!(matches!(err, Error::Config(m) if m == "timeout missing"));
    }

    #[test]
    fn serialize_iov_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"Io""#));
        assert!(json.contains("file not found"));
    }

    #[test]
    fn deserialize_iov_error() {
        let json = r#"{"kind":"Io","message":"I/O error: file not found"}"#;
        let err: Error = serde_json::from_str(json).unwrap();
        assert!(matches!(err, Error::Io(_)));
        assert_eq!(err.to_string(), "I/O error: I/O error: file not found");
    }

    #[test]
    fn roundtrip_all_variants() {
        let errors = vec![
            Error::Io(io::Error::other("disk full")),
            Error::Config("bad config".into()),
            Error::InvalidArgument("port 0".into()),
            Error::Internal("panic recovery".into()),
        ];
        for original in errors {
            let json = serde_json::to_string(&original).unwrap();
            let restored: Error = serde_json::from_str(&json).unwrap();
            assert_eq!(original.to_string(), restored.to_string());
        }
    }
}
