//! `goalctl` — 最小 Goal → Contract 编译器。
//!
//! - 输入：Goal YAML/JSON（id / outcome / risk / acceptance / invariants / …）
//! - 输出：Contract JSON + 稳定 digest（canonical JSON 的 sha256）
//! - fail-closed：空 outcome、缺失 AC id、主观词 lint

#![forbid(unsafe_code)]

mod compile;
mod lint;
mod model;
mod validate;

pub use compile::{CompileError, CompileOutput, compile_goal, compile_goal_str};
pub use lint::{SUBJECTIVE_WORDS, lint_subjective};
pub use model::{AcceptanceItem, GoalContract, GoalDocument, RiskLevel};
pub use validate::{ValidateError, validate_goal};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.chars().next().unwrap().is_ascii_digit());
    }

    /// 默认 crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        assert!(!SUBJECTIVE_WORDS.is_empty());
        let hits = lint_subjective("make it better somehow");
        assert!(!hits.is_empty());

        let raw = r#"
id: GOAL-SURFACE
outcome: "all unit tests pass"
risk: R1
acceptance:
  - id: AC-1
    statement: "cargo test -p goalctl passes"
invariants: []
forbidden: []
not_in_scope: []
touches:
  - "tools/goalctl"
"#;
        let out: CompileOutput = compile_goal_str(raw, Some("yaml")).expect("compile");
        let contract: &GoalContract = &out.contract;
        assert_eq!(contract.digest.len(), 64);
        assert_eq!(contract.risk, RiskLevel::R1);
        let ac = AcceptanceItem { id: "AC-1".into(), statement: "x".into() };
        assert_eq!(ac.id, "AC-1");
        let _ = validate_goal;
        let _ = compile_goal;
        fn assert_type<T: ?Sized>() {}
        assert_type::<CompileError>();
        assert_type::<ValidateError>();
        assert_type::<GoalDocument>();
        assert_type::<CompileOutput>();
        assert_type::<GoalContract>();
    }
}
