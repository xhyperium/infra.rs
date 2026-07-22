//! 通用 wire envelope：仅携带 `schema_version` + `payload`，不含业务校验。
//!
//! 调用方负责选择版本常量并解释 `payload`；本模块只提供形状与版本辅助。

use serde::{Deserialize, Serialize};

/// Envelope 自身 wire schema（字段：`schema_version` + `payload`）。
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// 当前推荐的 payload schema 版本起点（与 [`ENVELOPE_SCHEMA_VERSION`] 独立）。
///
/// 具体 DTO 的 committed wire 版本见 [`crate::wire`]；本常量供 envelope 包裹方
/// 作为默认起点，不绑定某一业务类型。
pub const CURRENT_PAYLOAD_SCHEMA_VERSION: u32 = 1;

/// 带 schema 版本的通用信封。
///
/// - `schema_version`：payload 的协议版本（由生产者声明）
/// - `payload`：任意可序列化载荷
///
/// 未知字段拒绝（`deny_unknown_fields`）；**不做**业务字段校验。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope<T> {
    /// payload 协议版本。
    pub schema_version: u32,
    /// 业务载荷（形状由 `T` 定义）。
    pub payload: T,
}

impl<T> Envelope<T> {
    /// 用给定版本包装 payload。
    pub fn wrap(schema_version: u32, payload: T) -> Self {
        Self { schema_version, payload }
    }

    /// 用 [`CURRENT_PAYLOAD_SCHEMA_VERSION`] 包装 payload。
    pub fn wrap_current(payload: T) -> Self {
        Self::wrap(CURRENT_PAYLOAD_SCHEMA_VERSION, payload)
    }

    /// 校验 `schema_version` 是否等于期望值。
    ///
    /// 成功时返回 `&payload`；不匹配返回实际版本。
    pub fn validate_version(&self, expected: u32) -> Result<&T, EnvelopeVersionError> {
        if self.schema_version == expected {
            Ok(&self.payload)
        } else {
            Err(EnvelopeVersionError { expected, actual: self.schema_version })
        }
    }

    /// 消耗 self，校验版本后返回 payload。
    pub fn into_payload_if_version(self, expected: u32) -> Result<T, EnvelopeVersionError> {
        if self.schema_version == expected {
            Ok(self.payload)
        } else {
            Err(EnvelopeVersionError { expected, actual: self.schema_version })
        }
    }
}

/// schema_version 不匹配。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeVersionError {
    /// 调用方期望的版本。
    pub expected: u32,
    /// 信封中的实际版本。
    pub actual: u32,
}

impl std::fmt::Display for EnvelopeVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "envelope schema_version 不匹配: expected={}, actual={}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for EnvelopeVersionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SamplePayload {
        id: String,
        n: i64,
    }

    #[test]
    fn wrap_and_roundtrip() {
        let payload = SamplePayload { id: "a".into(), n: 7 };
        let env = Envelope::wrap(CURRENT_PAYLOAD_SCHEMA_VERSION, payload.clone());
        assert_eq!(env.schema_version, 1);
        let json = serde_json::to_string(&env).expect("serialize");
        assert_eq!(json, r#"{"schema_version":1,"payload":{"id":"a","n":7}}"#);
        let back: Envelope<SamplePayload> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, env);
        assert_eq!(back.validate_version(1).unwrap(), &payload);
    }

    #[test]
    fn wrap_current_uses_constant() {
        let env = Envelope::wrap_current(42u32);
        assert_eq!(env.schema_version, CURRENT_PAYLOAD_SCHEMA_VERSION);
        assert_eq!(env.payload, 42);
    }

    #[test]
    fn reject_missing_schema_version() {
        let j = r#"{"payload":{"id":"a","n":1}}"#;
        let err = serde_json::from_str::<Envelope<SamplePayload>>(j).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("schema_version") || msg.contains("missing"),
            "must reject missing schema_version: {msg}"
        );
    }

    #[test]
    fn reject_unknown_fields() {
        let j = r#"{"schema_version":1,"payload":{"id":"a","n":1},"extra":true}"#;
        let err = serde_json::from_str::<Envelope<SamplePayload>>(j).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unknown") || msg.contains("extra"),
            "must deny unknown fields: {msg}"
        );
    }

    #[test]
    fn validate_version_mismatch() {
        let env = Envelope::wrap(2, SamplePayload { id: "x".into(), n: 0 });
        let err = env.validate_version(1).unwrap_err();
        assert_eq!(err.expected, 1);
        assert_eq!(err.actual, 2);
        assert!(err.to_string().contains("不匹配"));

        let err2 = env.into_payload_if_version(1).unwrap_err();
        assert_eq!(err2.actual, 2);
    }

    #[test]
    fn into_payload_if_version_ok() {
        let env = Envelope::wrap(1, SamplePayload { id: "ok".into(), n: 3 });
        let p = env.into_payload_if_version(1).unwrap();
        assert_eq!(p.id, "ok");
        assert_eq!(p.n, 3);
    }

    #[test]
    fn envelope_schema_constants_stable() {
        assert_eq!(ENVELOPE_SCHEMA_VERSION, 1);
        assert_eq!(CURRENT_PAYLOAD_SCHEMA_VERSION, 1);
        // 字段名冻结：golden JSON keys
        let v = json!({"schema_version": 1, "payload": 0});
        let env: Envelope<i32> = serde_json::from_value(v).unwrap();
        assert_eq!(env.payload, 0);
    }
}
