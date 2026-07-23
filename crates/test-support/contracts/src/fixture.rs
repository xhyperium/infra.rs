//! 确定性的隔离 fixture 命名空间。

use crate::failure::ContractFailure;

const MAX_NAMESPACE_LEN: usize = 32;
const MAX_RESOURCE_LEN: usize = 63;

fn is_portable_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// 跨后端可复用的确定性 fixture 前缀。
///
/// 为兼容对象 key、消息 channel 与未引用的 SQL identifier，名称必须以 ASCII
/// 小写字母开头，其余字符仅允许 ASCII 小写字母、数字和下划线。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureNamespace(String);

impl FixtureNamespace {
    /// 校验并构造命名空间。
    ///
    /// # Errors
    ///
    /// 名称为空、超过 32 字节或不满足可移植标识符规则时返回 [`ContractFailure`]。
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
        if !is_portable_identifier(&value) {
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
    ///
    /// # Errors
    ///
    /// 后缀不满足可移植标识符规则或最终资源名超过 63 字节时返回 [`ContractFailure`]。
    pub fn resource(&self, suffix: &str) -> Result<String, ContractFailure> {
        if !is_portable_identifier(suffix) {
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
        let fixture = FixtureNamespace::new("ctk_run_42").expect("fixture 应合法");
        assert_eq!(fixture.as_str(), "ctk_run_42");
        assert_eq!(fixture.resource("object").expect("资源名应合法"), "ctk_run_42__object");

        let boundary = "a".repeat(MAX_NAMESPACE_LEN);
        assert_eq!(FixtureNamespace::new(&boundary).expect("最大长度应合法").as_str(), boundary,);
    }

    #[test]
    fn rejects_empty_long_or_nonportable_namespace() {
        for value in [
            "",
            &"a".repeat(MAX_NAMESPACE_LEN + 1),
            "Upper",
            "1name",
            "_name",
            "aUpper",
            "with-dash",
            "a\n",
            "aé",
        ] {
            let failure = FixtureNamespace::new(value).expect_err("非法 fixture 必须被拒绝");
            assert_eq!(failure.contract, "FixtureNamespace");
        }
    }

    #[test]
    fn rejects_nonportable_or_oversized_resource_suffix() {
        let fixture = FixtureNamespace::new("a".repeat(MAX_NAMESPACE_LEN)).expect("fixture 应合法");
        for suffix in ["", "Upper", "1suffix", "_suffix", "aUpper", "with-dash", "aé"] {
            let failure = fixture.resource(suffix).expect_err("非法后缀必须被拒绝");
            assert_eq!(failure.contract, "FixtureNamespace");
            assert_eq!(failure.case, "portable_suffix");
        }
        let max_suffix_len = MAX_RESOURCE_LEN - MAX_NAMESPACE_LEN - 2;
        let boundary = fixture.resource(&"a".repeat(max_suffix_len)).expect("63 字节应合法");
        assert_eq!(boundary.len(), MAX_RESOURCE_LEN);
        let failure =
            fixture.resource(&"a".repeat(max_suffix_len + 1)).expect_err("64 字节必须被拒绝");
        assert_eq!(failure.case, "resource_max_len");
    }
}
