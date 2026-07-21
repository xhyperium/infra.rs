//! 契约 suite 失败类型（SPEC-TESTKIT-002 §9.6）。

use std::fmt;

/// 单条合同用例失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFailure {
    /// 契约名（trait / suite 标识）。
    pub contract: &'static str,
    /// 用例名。
    pub case: &'static str,
    /// 失败细节（中文优先）。
    pub detail: String,
}

impl ContractFailure {
    /// 构造失败。
    pub fn new(contract: &'static str, case: &'static str, detail: impl Into<String>) -> Self {
        Self { contract, case, detail: detail.into() }
    }
}

impl fmt::Display for ContractFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "合同失败 contract={} case={}: {}", self.contract, self.case, self.detail)
    }
}

impl std::error::Error for ContractFailure {}

/// Suite 结果别名。
pub type ContractResult = Result<(), ContractFailure>;

/// 将布尔条件转为 [`ContractResult`]。
pub fn ensure(
    contract: &'static str,
    case: &'static str,
    ok: bool,
    detail: impl Into<String>,
) -> ContractResult {
    if ok { Ok(()) } else { Err(ContractFailure::new(contract, case, detail)) }
}
