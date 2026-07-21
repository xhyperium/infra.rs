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
    #[test]
    fn version_is_nonempty() {
        assert!(!crate::VERSION.is_empty());
        assert!(crate::VERSION.chars().next().unwrap().is_ascii_digit());
    }
}
