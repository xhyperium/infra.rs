//! 夹具集成：good 编译；bad fail-closed。

use std::path::PathBuf;

use goalctl::{CompileError, ValidateError, compile_goal};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn good_goal_compiles_with_digest() {
    let out = compile_goal(&fixture("good_goal.yaml")).expect("compile good");
    assert_eq!(out.contract.id, "GOAL-2026-DEMO-001");
    assert_eq!(out.contract.digest.len(), 64);
    assert!(out.json.contains("goal-contract/v1"));
    // 再编一次 digest 稳定
    let out2 = compile_goal(&fixture("good_goal.yaml")).unwrap();
    assert_eq!(out.contract.digest, out2.contract.digest);
}

#[test]
fn empty_outcome_fails() {
    let err = compile_goal(&fixture("bad_empty_outcome.yaml")).unwrap_err();
    match err {
        CompileError::Validate(ValidateError::EmptyOutcome) => {}
        other => panic!("unexpected: {other}"),
    }
}

#[test]
fn missing_ac_id_fails() {
    let err = compile_goal(&fixture("bad_missing_ac_id.yaml")).unwrap_err();
    match err {
        CompileError::Validate(ValidateError::MissingAcceptanceId(0)) => {}
        other => panic!("unexpected: {other}"),
    }
}
