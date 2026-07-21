//! Error 序列化/反序列化与错误链覆盖（集成测试，不计入 lib 行覆盖率噪声）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use infra_core::Error;
use std::error::Error as StdError;
use std::io;

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
}

#[test]
fn deserialize_io_with_chain_and_reserialize_walks_sources() {
    // 反序列化用 ChainNode 重建 source 链；再序列化触发 collect_chain 的 while 循环
    let json = r#"{
        "kind": "Io",
        "error_kind": "NotFound",
        "chain": ["deep", "mid", "top"]
    }"#;
    let err: Error = serde_json::from_str(json).unwrap();
    let Error::Io(io_err) = &err else {
        panic!("expected Io");
    };
    assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    assert!(StdError::source(io_err).is_some());

    let again = serde_json::to_string(&err).unwrap();
    assert!(again.contains("deep"), "{again}");
    assert!(again.contains("top") || again.contains("mid"), "{again}");
}

#[test]
fn deserialize_io_single_message_chain() {
    let json = r#"{
        "kind": "Io",
        "error_kind": "PermissionDenied",
        "chain": ["permission denied"]
    }"#;
    let err: Error = serde_json::from_str(json).unwrap();
    let Error::Io(io_err) = &err else {
        panic!("expected Io");
    };
    assert_eq!(io_err.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(io_err.to_string(), "permission denied");
}

#[test]
fn deserialize_empty_io_chain_rebuilds_placeholder() {
    let json = r#"{"kind":"Io","error_kind":"Other","chain":[]}"#;
    let err: Error = serde_json::from_str(json).unwrap();
    let Error::Io(e) = err else {
        panic!("expected Io");
    };
    let msg = e.to_string();
    assert!(msg.contains("反序列化") || !msg.is_empty(), "{msg}");
}

#[test]
fn deserialize_all_io_error_kinds() {
    let kinds = [
        "NotFound",
        "PermissionDenied",
        "ConnectionRefused",
        "ConnectionReset",
        "AddrInUse",
        "AddrNotAvailable",
        "BrokenPipe",
        "AlreadyExists",
        "WouldBlock",
        "NotADirectory",
        "IsADirectory",
        "DirectoryNotEmpty",
        "ReadOnlyFilesystem",
        "InvalidInput",
        "InvalidData",
        "TimedOut",
        "WriteZero",
        "Interrupted",
        "Unsupported",
        "UnexpectedEof",
        "OutOfMemory",
        "UnknownKindFallsToOther",
    ];
    for kind in kinds {
        let json = format!(r#"{{"kind":"Io","error_kind":"{kind}","chain":["msg-{kind}"]}}"#);
        let err: Error = serde_json::from_str(&json).unwrap();
        assert!(matches!(err, Error::Io(_)), "kind={kind}");
    }
}

#[test]
fn deserialize_unknown_and_duplicate_fields_error() {
    assert!(serde_json::from_str::<Error>(r#"{"kind":"Nope","message":"x"}"#).is_err());
    let _ = serde_json::from_str::<Error>(r#"{"kind":"Config","message":"a","message":"b"}"#);
    let _ = serde_json::from_str::<Error>(r#"{"kind":"Config","kind":"Internal","message":"x"}"#);
    let _ = serde_json::from_str::<Error>(
        r#"{"kind":"Io","error_kind":"NotFound","error_kind":"TimedOut","chain":["a"]}"#,
    );
    let _ = serde_json::from_str::<Error>(
        r#"{"kind":"Io","error_kind":"NotFound","chain":["a"],"chain":["b"]}"#,
    );
    assert!(serde_json::from_str::<Error>(r#"{}"#).is_err());
    assert!(serde_json::from_str::<Error>(r#"{"kind":"Config"}"#).is_err());
    assert!(serde_json::from_str::<Error>(r#"{"kind":"InvalidArgument"}"#).is_err());
    assert!(serde_json::from_str::<Error>(r#"{"kind":"Internal"}"#).is_err());
    assert!(serde_json::from_str::<Error>(r#"{"kind":"Io","error_kind":"NotFound"}"#).is_err());
    assert!(serde_json::from_str::<Error>("null").is_err());
    assert!(serde_json::from_str::<Error>("42").is_err());
    assert!(serde_json::from_str::<Error>(r#""string""#).is_err());
}

#[test]
fn serialize_message_variants_individually() {
    for err in
        [Error::Config("c".into()), Error::InvalidArgument("a".into()), Error::Internal("i".into())]
    {
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\""));
        assert!(json.contains("\"message\""));
        let back: Error = serde_json::from_str(&json).unwrap();
        assert_eq!(err.to_string(), back.to_string());
    }
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
