//! verifyctl 最小示例：dry plan。
use verifyctl::{PlanOptions, build_plan};

fn main() {
    let contract = r#"{"schema":"goal-contract/v1","digest":"abc","touches":["tools/verifyctl"]}"#;
    let changed = vec!["tools/verifyctl".to_string()];
    let plan = build_plan(contract, &changed, &PlanOptions { dry: true }).expect("plan");
    assert_eq!(plan.schema, "verification-plan/v1");
    assert!(!plan.plan_digest.is_empty());
    println!("verifyctl example ok digest={}", &plan.plan_digest[..12]);
}
