//! 集成：plan（dry）+ execute `true`。

use std::path::Path;

use verifyctl::{PlanOptions, RunStatus, build_plan, execute_plan};

#[test]
fn plan_and_execute_dry_true() {
    let opts = PlanOptions { dry: true };
    let contract = r#"{"digest":"deadbeef","touches":["tools/verifyctl"]}"#;
    let plan = build_plan(contract, &["tools/verifyctl/src/lib.rs".into()], &opts).expect("plan");
    assert!(!plan.checks.is_empty());
    assert_eq!(plan.contract_digest, "deadbeef");

    let run = execute_plan(&plan, Path::new(".")).expect("execute");
    assert_eq!(run.status, RunStatus::Pass, "all dry checks should pass");
    assert_eq!(run.plan_digest, plan.plan_digest);
    for c in &run.checks {
        assert_eq!(c.exit_code, 0, "check {} failed", c.id);
        assert_eq!(c.output_digest.len(), 64);
    }
}
