//! CLI 集成：authority-check registry-only / risk / human filter。

use std::process::Command;

fn xtask() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_xtask"));
    c.current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..");
    c
}

#[test]
fn registry_only_passes_on_target_shadow() {
    let out = xtask()
        .args(["authority-check", "--registry-only", "--json"])
        .output()
        .expect("run authority-check");
    assert!(
        out.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["live_authorization"], false);
    assert_eq!(v["registry"]["live_ssot"], false);
    assert!(v["registry"]["entry_count"].as_u64().unwrap() >= 1);
}

#[test]
fn constitution_path_is_t4() {
    let out = xtask()
        .args([
            "authority-check",
            "--path",
            "docs/governance/CONSTITUTION.md",
            "--json",
        ])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["risk"]["computed_risk_tier"], "T4");
    assert_eq!(v["risk"]["primary_operation_class"], "governance.amendment");
    assert_eq!(v["risk"]["require_independent_risk_review"], true);
    assert_eq!(v["live_authorization"], false);
}

#[test]
fn evidence_path_is_t1() {
    let out = xtask()
        .args([
            "authority-check",
            "--path",
            "evidence/changes/demo.md",
            "--json",
        ])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["risk"]["computed_risk_tier"], "T1");
    assert_eq!(v["risk"]["primary_operation_class"], "evidence.raw.append");
}
