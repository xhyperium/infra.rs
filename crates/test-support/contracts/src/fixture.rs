//! 确定性的隔离 fixture 命名空间。

use crate::failure::ContractFailure;

const MAX_NAMESPACE_LEN: usize = 32;
const MAX_RESOURCE_LEN: usize = 63;

/// 跨后端可复用的确定性 fixture 前缀。
///
/// 为兼容对象 key、消息 channel 与未引用的 SQL identifier，名称必须以 ASCII
/// 小写字母开头，其余字符仅允许 ASCII 小写字母、数字和下划线。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureNamespace(String);

impl FixtureNamespace {
    /// 校验并构造命名空间。
    pub fn new(value: impl Into<String>) -> Result<Self, ContractFailure> {
        let value = value.into();
        if value.is_empty() {
            return Err(ContractFailure::new(
                "FixtureNamespace",
                "nonempty",
                "fixture 命名空间不能为空",
            ));
        }
        if value.len() > MAX_NAMESPACE_LEN {
            return Err(ContractFailure::new(
                "FixtureNamespace",
                "max_len",
                format!("fixture 命名空间长度不得超过 {MAX_NAMESPACE_LEN}"),
            ));
        }
        let bytes = value.as_bytes();
        if !bytes[0].is_ascii_lowercase()
            || bytes[1..]
                .iter()
                .any(|byte| !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_'))
        {
            return Err(ContractFailure::new(
                "FixtureNamespace",
                "portable_identifier",
                "fixture 命名空间必须以 ASCII 小写字母开头，且仅含小写字母、数字、下划线",
            ));
        }
        Ok(Self(value))
    }

    /// 返回已校验的命名空间。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 派生一个不超过 63 字节的可移植资源名。
    pub fn resource(&self, suffix: &str) -> Result<String, ContractFailure> {
        let bytes = suffix.as_bytes();
        if bytes.is_empty()
            || !bytes[0].is_ascii_lowercase()
            || !bytes[1..]
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
        {
            return Err(ContractFailure::new(
                "FixtureNamespace",
                "portable_suffix",
                "fixture 资源后缀必须以 ASCII 小写字母开头，且仅含小写字母、数字、下划线",
            ));
        }
        let resource = format!("{}__{suffix}", self.0);
        if resource.len() > MAX_RESOURCE_LEN {
            return Err(ContractFailure::new(
                "FixtureNamespace",
                "resource_max_len",
                format!("fixture 资源名长度不得超过 {MAX_RESOURCE_LEN}"),
            ));
        }
        Ok(resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_portable_namespace_and_derives_isolated_resource() {
        let fixture = FixtureNamespace::new("ctk_run_42").expect("valid fixture");
        assert_eq!(fixture.as_str(), "ctk_run_42");
        assert_eq!(fixture.resource("object").expect("resource"), "ctk_run_42__object");

        let boundary = "a".repeat(MAX_NAMESPACE_LEN);
        assert_eq!(
            FixtureNamespace::new(&boundary).expect("max length is valid").as_str(),
            boundary,
        );
    }

    #[test]
    fn rejects_empty_long_or_nonportable_namespace() {
        for value in ["", &"a".repeat(MAX_NAMESPACE_LEN + 1), "Upper", "with-dash", "a\n"] {
            let failure = FixtureNamespace::new(value).expect_err("invalid fixture");
            assert_eq!(failure.contract, "FixtureNamespace");
        }
    }

    #[test]
    fn rejects_nonportable_or_oversized_resource_suffix() {
        let fixture = FixtureNamespace::new("a".repeat(MAX_NAMESPACE_LEN)).expect("fixture");
        for suffix in ["", "Upper", "with-dash"] {
            let failure = fixture.resource(suffix).expect_err("invalid suffix");
            assert_eq!(failure.contract, "FixtureNamespace");
            assert_eq!(failure.case, "portable_suffix");
        }
        let failure = fixture.resource(&"a".repeat(MAX_RESOURCE_LEN)).expect_err("too long");
        assert_eq!(failure.case, "resource_max_len");
    }
}
