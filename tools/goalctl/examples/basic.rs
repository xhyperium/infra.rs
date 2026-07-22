//! goalctl 最小示例：编译 fixture 风格 Goal 字符串。
use goalctl::compile_goal_str;

fn main() {
    let raw = r#"
id: GOAL-EX-001
outcome: example compiles
risk: R1
acceptance:
  - id: AC-1
    statement: digest non-empty
invariants: []
forbidden: []
not_in_scope: []
touches: [tools/goalctl]
"#;
    let out = compile_goal_str(raw, Some("yaml")).expect("compile");
    assert_eq!(out.contract.digest.len(), 64);
    println!("goalctl example ok digest={}", &out.contract.digest[..12]);
}
