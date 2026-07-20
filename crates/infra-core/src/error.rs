

use serde::{de, ser};
use std::error::Error as StdError;
use std::fmt;
use std::io;

/// 序列化/反序列化时保留 `io::Error` 的错误链。
///
/// 实现 `Error + Send + Sync + 'static`，使 `io::Error::other()` 可将其作为 source 包裹。
#[derive(Debug)]
struct ChainNode {
    message: String,
    source: Option<Box<ChainNode>>,
}

impl fmt::Display for ChainNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for ChainNode {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|n| n as &(dyn StdError + 'static))
    }
}

/// 将错误链展开为消息列表（从最深层到最顶层）。
fn collect_chain(err: &(dyn StdError + 'static)) -> Vec<String> {
    let mut messages = vec![err.to_string()];
    let mut current = err.source();
    while let Some(src) = current {
        messages.push(src.to_string());
        current = src.source();
    }
    messages
}

/// 从消息列表和顶层 kind 重建错误链（列表从最深层到最顶层）。
fn rebuild_io_error(kind: io::ErrorKind, chain: Vec<String>) -> io::Error {
    // 自底向上构建 ChainNode 链
    let mut source: Option<Box<ChainNode>> = None;
    // chain[0] 是最深层，chain[last] 是最顶层
    for msg in chain.iter() {
        source = Some(Box::new(ChainNode {
            message: msg.clone(),
            source,
        }));
    }

    if let Some(node) = source {
        io::Error::new(kind, io::Error::other(node))
    } else {
        io::Error::other("deserialized io::Error (empty chain)")
    }
}

/// `infra-core` 的基础错误类型。
///
/// 涵盖基础设施层面最常见的四类错误：I/O、配置、参数和内部错误。
/// `thiserror` 自动实现 [`fmt::Display`] 和 [`StdError`]。
/// 手动实现 [`serde::Serialize`] / [`serde::Deserialize`]。
///
/// # 序列化格式
///
/// `Io` 变体保留完整错误链（`ErrorKind` + 所有 `source()` 消息）：
/// ```json
/// {
///   "kind": "Io",
///   "error_kind": "NotFound",
///   "chain": ["deepest source", ..., "top-level message"]
/// }
/// ```
///
/// 其余变体：`{"kind": "<变体名>", "message": "<描述>"}`
///
/// # 使用示例
///
/// ```
/// # use infra_core::{Error, Result};
/// fn load(path: &str) -> Result<String> {
///     Ok(std::fs::read_to_string(path)?)
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O 操作错误。
    ///
    /// 文件读写、网络 I/O 等系统操作失败。实现 `From<io::Error>`，可直接 `?` 传播。
    /// 序列化时保留完整错误链（`ErrorKind` + 所有 `source()` 消息）。
    ///
    /// `Display`：`I/O 错误: <底层 io::Error 的 Display>`
    ///
    /// ```
    /// # use infra_core::Error;
    /// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    /// let err: Error = io_err.into();
    /// assert!(matches!(err, Error::Io(_)));
    /// ```
    #[error("I/O 错误: {0}")]
    Io(#[from] io::Error),

    /// 配置错误。
    ///
    /// 配置加载、解析、校验失败。
    ///
    /// `Display`：`配置错误: <说明>`
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::Config("missing field 'host'".into());
    /// assert_eq!(err.to_string(), "配置错误: missing field 'host'");
    /// ```
    #[error("配置错误: {0}")]
    Config(String),

    /// 参数校验错误。
    ///
    /// 函数入参不合法：空字符串、越界索引、非法端口号等。
    ///
    /// `Display`：`参数无效: <说明>`
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::InvalidArgument("port must be 1..=65535".into());
    /// assert_eq!(err.to_string(), "参数无效: port must be 1..=65535");
    /// ```
    #[error("参数无效: {0}")]
    InvalidArgument(String),

    /// 内部错误。
    ///
    /// 不应暴露给用户的未预期错误：不变量破坏、状态机异常等。
    ///
    /// `Display`：`内部错误: <说明>`
    ///
    /// ```
    /// # use infra_core::Error;
    /// let err = Error::Internal("pool exhausted".into());
    /// assert_eq!(err.to_string(), "内部错误: pool exhausted");
    /// ```
    #[error("内部错误: {0}")]
    Internal(String),
}

// ── serde::Serialize ──────────────────────────────────────────────

impl ser::Serialize for Error {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use ser::SerializeStruct;

        match self {
            Error::Io(e) => {
                let chain = collect_chain(e);
                let mut s = serializer.serialize_struct("Error", 3)?;
                s.serialize_field("kind", "Io")?;
                s.serialize_field("error_kind", &format!("{:?}", e.kind()))?;
                s.serialize_field("chain", &chain)?;
                s.end()
            }
            _ => {
                let (kind, msg) = match self {
                    Error::Config(m) => ("Config", m.as_str()),
                    Error::InvalidArgument(m) => ("InvalidArgument", m.as_str()),
                    Error::Internal(m) => ("Internal", m.as_str()),
                    Error::Io(_) => unreachable!(),
                };
                let mut s = serializer.serialize_struct("Error", 2)?;
                s.serialize_field("kind", kind)?;
                s.serialize_field("message", msg)?;
                s.end()
            }
        }
    }
}

// ── serde::Deserialize ────────────────────────────────────────────

impl<'de> de::Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ErrorVisitor;

        impl<'de> de::Visitor<'de> for ErrorVisitor {
            type Value = Error;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("包含 kind 字段的 Error 结构体")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> std::result::Result<Error, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                #[derive(serde::Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                enum Field {
                    Kind,
                    Message,
                    #[serde(rename = "error_kind")]
                    ErrorKind,
                    Chain,
                }

                let mut kind: Option<String> = None;
                let mut message: Option<String> = None;
                let mut error_kind: Option<String> = None;
                let mut chain: Option<Vec<String>> = None;

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
                        Field::ErrorKind => {
                            if error_kind.is_some() {
                                return Err(de::Error::duplicate_field("error_kind"));
                            }
                            error_kind = Some(map.next_value()?);
                        }
                        Field::Chain => {
                            if chain.is_some() {
                                return Err(de::Error::duplicate_field("chain"));
                            }
                            chain = Some(map.next_value()?);
                        }
                    }
                }

                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                match kind.as_str() {
                    "Io" => {
                        let chain =
                            chain.ok_or_else(|| de::Error::missing_field("chain"))?;
                        let err_kind = match error_kind.as_deref() {
                            Some("NotFound") => io::ErrorKind::NotFound,
                            Some("PermissionDenied") => io::ErrorKind::PermissionDenied,
                            Some("ConnectionRefused") => io::ErrorKind::ConnectionRefused,
                            Some("ConnectionReset") => io::ErrorKind::ConnectionReset,
                            Some("AddrInUse") => io::ErrorKind::AddrInUse,
                            Some("AddrNotAvailable") => io::ErrorKind::AddrNotAvailable,
                            Some("BrokenPipe") => io::ErrorKind::BrokenPipe,
                            Some("AlreadyExists") => io::ErrorKind::AlreadyExists,
                            Some("WouldBlock") => io::ErrorKind::WouldBlock,
                            Some("NotADirectory") => io::ErrorKind::NotADirectory,
                            Some("IsADirectory") => io::ErrorKind::IsADirectory,
                            Some("DirectoryNotEmpty") => io::ErrorKind::DirectoryNotEmpty,
                            Some("ReadOnlyFilesystem") => io::ErrorKind::ReadOnlyFilesystem,
                            Some("InvalidInput") => io::ErrorKind::InvalidInput,
                            Some("InvalidData") => io::ErrorKind::InvalidData,
                            Some("TimedOut") => io::ErrorKind::TimedOut,
                            Some("WriteZero") => io::ErrorKind::WriteZero,
                            Some("Interrupted") => io::ErrorKind::Interrupted,
                            Some("Unsupported") => io::ErrorKind::Unsupported,
                            Some("UnexpectedEof") => io::ErrorKind::UnexpectedEof,
                            Some("OutOfMemory") => io::ErrorKind::OutOfMemory,
                            _ => io::ErrorKind::Other,
                        };
                        Ok(Error::Io(rebuild_io_error(err_kind, chain)))
                    }
                    "Config" => {
                        let msg =
                            message.ok_or_else(|| de::Error::missing_field("message"))?;
                        Ok(Error::Config(msg))
                    }
                    "InvalidArgument" => {
                        let msg =
                            message.ok_or_else(|| de::Error::missing_field("message"))?;
                        Ok(Error::InvalidArgument(msg))
                    }
                    "Internal" => {
                        let msg =
                            message.ok_or_else(|| de::Error::missing_field("message"))?;
                        Ok(Error::Internal(msg))
                    }
                    other => Err(de::Error::unknown_variant(
                        other,
                        &["Io", "Config", "InvalidArgument", "Internal"],
                    )),
                }
            }
        }

        deserializer.deserialize_struct(
            "Error",
            &["kind", "message", "error_kind", "chain"],
            ErrorVisitor,
        )
    }
}

/// `infra-core` 的标准 [`Result`] 类型别名。
///
/// 等价于 `std::result::Result<T, Error>`。避免每处都写 `<T, Error>`。
///
/// ```
/// # use infra_core::Result;
/// fn work() -> Result<i32> { Ok(42) }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    // 仅为 .source() 方法解析导入 trait，避免遮蔽 crate Error 类型
    use std::error::Error as StdError;

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
    fn serialize_io_with_chain() {
        let inner = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let outer = io::Error::new(io::ErrorKind::NotFound, inner);
        let err = Error::Io(outer);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"Io""#));
        assert!(json.contains(r#""error_kind":"NotFound""#));
        assert!(json.contains("connection refused"));
    }

    #[test]
    fn deserialize_io_with_chain() {
        let json = r#"{
            "kind": "Io",
            "error_kind": "NotFound",
            "chain": ["connection refused", "file not found"]
        }"#;
        let err: Error = serde_json::from_str(json).unwrap();
        let io_err = match &err {
            Error::Io(e) => e,
            _ => unreachable!(),
        };
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
        assert!(io_err.source().is_some());
        assert_eq!(io_err.source().unwrap().to_string(), "connection refused");
    }

    #[test]
    fn deserialize_io_no_chain() {
        let json = r#"{
            "kind": "Io",
            "error_kind": "PermissionDenied",
            "chain": ["permission denied"]
        }"#;
        let err: Error = serde_json::from_str(json).unwrap();
        let io_err = match &err {
            Error::Io(e) => e,
            _ => unreachable!(),
        };
        assert_eq!(io_err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(io_err.to_string(), "permission denied");
    }

    #[test]
    fn io_chain_deep_roundtrip() {
        let inner = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let mid = io::Error::new(io::ErrorKind::ConnectionReset, inner);
        let outer = io::Error::new(io::ErrorKind::NotFound, mid);
        let original = Error::Io(outer);

        let json = serde_json::to_string(&original).unwrap();
        let restored: Error = serde_json::from_str(&json).unwrap();

        assert_eq!(original.to_string(), restored.to_string());

        fn count_sources(err: &dyn StdError) -> usize {
            let mut count = 0;
            let mut cur = err.source();
            while let Some(s) = cur {
                count += 1;
                cur = s.source();
            }
            count
        }

        let orig_count = match &original {
            Error::Io(e) => count_sources(e),
            _ => 0,
        };
        let rest_count = match &restored {
            Error::Io(e) => count_sources(e),
            _ => 0,
        };
        assert_eq!(orig_count, rest_count);
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
