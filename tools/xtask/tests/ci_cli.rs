use std::path::PathBuf;
use std::process::Command;

#[test]
fn ci_dry_run_commands_succeed() {
    let all_pass =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ci_negative/fixtures/all_pass.json");
    for args in [
        vec!["ci", "run", "--dry-run", "true"],
        vec![
            "ci",
            "aggregate",
            "--decisions-file",
            all_pass.to_str().unwrap(),
            "--expected",
            "fast,build_test",
        ],
        vec!["ci", "reconcile"],
        vec!["ci", "metrics"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(&args)
            .output()
            .expect("run xtask CI dry-run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "ci {:?} must succeed in shadow dry-run; stdout={stdout:?} stderr={stderr:?}",
            args
        );
        assert!(
            !stdout.contains("NOT_IMPLEMENTED") && !stderr.contains("NOT_IMPLEMENTED"),
            "dry-run surface must not claim NOT_IMPLEMENTED; stdout={stdout:?} stderr={stderr:?}"
        );
    }
}

#[test]
fn ci_aggregate_without_decisions_file_fails_closed() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "aggregate", "--json"])
        .output()
        .expect("aggregate bare");
    assert!(
        !output.status.success(),
        "bare aggregate must fail-closed (no default all-green)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("FAIL")
            || combined.contains("decisions_file")
            || combined.contains("FAIL-CLOSED"),
        "must mention fail-closed / FAIL; got {combined}"
    );
}

/// 构造 shipped `ci locks --root` 所需的隔离仓库形状；绝不修改 tracked 文件。
fn isolated_locks_root(mutator: impl FnOnce(&str) -> String) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let temp = tempfile::tempdir().expect("temp lock root");
    for relative in [
        ".github/ci/baseline.toml",
        ".github/ci/tools.lock.toml",
        ".github/actions/install-cargo-tool/action.yml",
    ] {
        let from = source.join(relative);
        let to = temp.path().join(relative);
        std::fs::create_dir_all(to.parent().expect("fixture parent")).expect("mkdir fixture");
        std::fs::copy(&from, &to)
            .unwrap_or_else(|error| panic!("copy {}: {error}", from.display()));
    }
    let toolchains = std::fs::read_to_string(source.join(".github/ci/toolchains.lock.toml"))
        .expect("read toolchains.lock");
    let path = temp.path().join(".github/ci/toolchains.lock.toml");
    std::fs::write(path, mutator(&toolchains)).expect("write fixture toolchains.lock");
    temp
}

fn run_ci_locks(root: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "locks",
            "--root",
            root.to_str().expect("UTF-8 root"),
            "--json",
        ])
        .output()
        .expect("ci locks")
}

#[test]
fn ci_locks_complete_pins_pass_with_root_override() {
    let fixture = isolated_locks_root(str::to_owned);
    let output = run_ci_locks(fixture.path());
    assert!(
        output.status.success(),
        "complete fixture must pass; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ci_locks_empty_msrv_fails_closed_cli() {
    let fixture = isolated_locks_root(|orig| {
        // 去掉 msrv 行，保留 primary → shipped check_locks 必须 FAIL
        orig.lines()
            .filter(|line| !line.trim().starts_with("msrv"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    });
    let output = run_ci_locks(fixture.path());
    assert!(
        !output.status.success(),
        "empty msrv must FAIL ci locks CLI"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("missing_msrv_pin"),
        "CLI must surface missing_msrv_pin; got {combined}"
    );
}

#[test]
fn ci_locks_empty_primary_fails_closed_cli() {
    let fixture = isolated_locks_root(|orig| {
        orig.lines()
            .filter(|line| !line.trim().starts_with("primary"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    });
    let output = run_ci_locks(fixture.path());
    assert!(
        !output.status.success(),
        "empty primary must FAIL ci locks CLI"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("missing_primary_pin"),
        "CLI must surface missing_primary_pin; got {combined}"
    );
}

#[test]
fn ci_aggregate_synthetic_smoke_ok() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "aggregate", "--synthetic-smoke", "--json"])
        .output()
        .expect("aggregate synthetic");
    assert!(
        output.status.success(),
        "synthetic-smoke must PASS; stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SYNTHETIC_SMOKE") || stdout.contains("PASS"),
        "{stdout}"
    );
}

#[test]
fn ci_run_non_dry_run_fails_closed() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "run", "--dry-run", "false"])
        .output()
        .expect("run xtask CI non-dry-run");
    assert!(
        !output.status.success(),
        "non-dry-run execution must remain unauthorized in shadow mode"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("not authorized") || combined.contains("dry-run"),
        "error must mention dry-run/authorization; got {combined}"
    );
}

#[test]
fn ci_plan_classifies_docs_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "plan", "--paths", "docs/x.md,README.md", "--json"])
        .output()
        .expect("plan");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("docs_only"),
        "expected docs_only classification; stdout={stdout}"
    );
    assert!(
        stdout.contains("NOT_APPLICABLE"),
        "docs-only should N/A build_test; stdout={stdout}"
    );
}

#[test]
fn ci_aggregate_missing_lane_fails() {
    let dir = tempfile::tempdir().expect("tmp");
    let dec = dir.path().join("dec.json");
    std::fs::write(&dec, r#"{"fast":"RUN_PASS"}"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "aggregate",
            "--decisions-file",
            dec.to_str().unwrap(),
            "--expected",
            "fast,build_test",
        ])
        .output()
        .expect("aggregate");
    assert!(
        !output.status.success(),
        "missing lane must fail-closed aggregate"
    );
}

#[test]
fn ci_aggregate_invalid_state_fails_with_normalized_schema_output() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/ci_negative/fixtures/aggregate_invalid_state.json");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "aggregate",
            "--decisions-file",
            fixture.to_str().unwrap(),
            "--expected",
            "fast,build_test",
            "--json",
        ])
        .output()
        .expect("aggregate invalid state");
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schema-valid report JSON");
    assert_eq!(report["decisions"]["fast"]["decision"], "UNKNOWN");
    assert_eq!(report["final_decision"], "FAIL");
    assert_eq!(report["schema_version"], 2);
}

#[test]
fn ci_aggregate_reused_nonexistent_attestation_fails() {
    let temp = tempfile::tempdir().expect("temp attestation root");
    let decisions = temp.path().join("decisions.json");
    std::fs::write(
        &decisions,
        r#"{"fast":{"decision":"REUSED","attestation":"missing.json"},"build_test":"RUN_PASS"}"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "aggregate",
            "--decisions-file",
            decisions.to_str().unwrap(),
            "--expected",
            "fast,build_test",
            "--attestation-root",
            temp.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("aggregate reused");
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schema-valid report JSON");
    assert_eq!(report["decisions"]["fast"]["decision"], "REUSED");
    assert_eq!(
        report["decisions"]["fast"]["attestation"],
        serde_json::Value::Null
    );
    assert_eq!(report["final_decision"], "FAIL");
}

#[test]
fn ci_wave2_5_surface_smoke() {
    for args in [
        vec!["ci", "graph", "--paths", "tools/xtask/src/ci/mod.rs"],
        vec!["ci", "drift"],
        vec!["ci", "locks"],
        vec!["ci", "taxonomy"],
        vec!["ci", "evidence-root"],
        vec!["ci", "reuse"],
        vec!["ci", "flake"],
        vec!["ci", "metrics", "--weekly-template"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(&args)
            .output()
            .expect("run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "ci {:?} must succeed; stdout={stdout:?} stderr={stderr:?}",
            args
        );
    }
}

#[test]
fn ci_fingerprint_requires_complete_typed_input() {
    for args in [
        vec!["ci", "fingerprint"],
        vec!["ci", "fingerprint", "--lane", "fast"],
        vec!["ci", "fingerprint", "--plan-digest", "sha256:legacy"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(&args)
            .output()
            .expect("fingerprint incomplete input");
        assert!(!output.status.success(), "legacy args must fail: {args:?}");
    }
}

#[test]
fn ci_fingerprint_typed_candidate_is_not_reusable_or_proven() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/ci_negative/fixtures/fingerprint_valid.json");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "fingerprint",
            "--input",
            fixture.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("typed fingerprint");
    assert!(output.status.success(), "{:?}", output);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["contract_version"], 1);
    assert_eq!(report["reusable"], false);
    assert_eq!(report["provenance_unverified"], true);
    assert!(report["fingerprint"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(!rendered.contains("RUSTFLAGS"));
    assert!(!rendered.contains("cargo test"));
}

#[test]
fn ci_reuse_structural_candidate_always_falls_back_to_run() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ci_negative/fixtures");
    let context = fixture_root.join("reuse_context_valid.json");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "reuse",
            "--want-reused",
            "--attestation-root",
            fixture_root.to_str().unwrap(),
            "--attestation",
            "reuse_attestation_valid.json",
            "--context",
            context.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("reuse structural candidate");
    assert!(output.status.success(), "{:?}", output);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["decision"], "RUN");
    assert_eq!(report["reuse_enabled"], false);
    assert_eq!(report["validation"]["production_activation"], false);
    assert_eq!(report["validation"]["trusted_attestation_verified"], false);
    assert_eq!(report["validation"]["runner_trust_verified"], false);
    assert_eq!(
        report["validation"]["structural_conditions"]["baseline_permits"],
        false
    );
}

#[test]
fn ci_reuse_missing_context_is_diagnostic_run_not_reused() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ci_negative/fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "reuse",
            "--want-reused",
            "--attestation-root",
            fixture_root.to_str().unwrap(),
            "--attestation",
            "reuse_attestation_valid.json",
            "--json",
        ])
        .output()
        .expect("reuse without context");
    assert!(output.status.success(), "{:?}", output);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["decision"], "RUN");
    assert_eq!(
        report["validation"]["all_structural_predicates_satisfied"],
        false
    );
    assert!(report["validation"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason.as_str().unwrap().contains("context_missing")));
}

#[test]
fn ci_verify_runner_workflow_declared_observation_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "verify-runner",
            "--class",
            "fast",
            "--digest",
            "sha256:PLACEHOLDER_FAST_IMAGE_DIGEST",
            "--trust",
            "production-forbidden",
            "--disk-gib",
            "64",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("unexpected argument '--digest'"),
        "{combined}"
    );
}

/// 隔离 generated 负测根：拷贝 baseline + 已渲染 contract/policy，绝不改写 tracked 文件。
fn isolated_drift_root() -> tempfile::TempDir {
    use std::fs;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let temp = tempfile::tempdir().expect("temp drift root");
    for relative in [
        ".github/ci/baseline.toml",
        ".github/ci/generated/workflow-contract.json",
        ".github/ci/generated/policy-table.md",
    ] {
        let from = source.join(relative);
        let to = temp.path().join(relative);
        fs::create_dir_all(to.parent().expect("parent")).expect("mkdir");
        fs::copy(&from, &to).unwrap_or_else(|e| panic!("copy {}: {e}", from.display()));
    }
    temp
}

#[test]
fn ci_verify_runner_observe_current_fails_closed_without_external_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "verify-runner",
            "--class",
            "fast",
            "--observe-current",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("INFRA_FAILURE"), "{combined}");
}

#[test]
fn ci_drift_detects_hand_edit() {
    use std::fs;
    let fixture = isolated_drift_root();
    let root = fixture.path();
    let contract = root.join(".github/ci/generated/workflow-contract.json");
    let original = fs::read_to_string(&contract).expect("contract");
    // hand-edit only the isolated copy
    fs::write(&contract, original + "\n").expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "drift", "--root", root.to_str().expect("UTF-8 root")])
        .output()
        .expect("drift");
    assert!(
        !output.status.success(),
        "hand-edited generated must fail drift; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("DRIFT") || combined.contains("drift"),
        "{combined}"
    );
}

#[test]
fn ci_flake_expired_blocks() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/ci_negative/fixtures/flake_expired.toml");
    assert!(fixture.is_file(), "fixture {}", fixture.display());
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "ci",
            "flake",
            "--today",
            "2026-07-16",
            "--registry-file",
            fixture.to_str().unwrap(),
        ])
        .output()
        .expect("flake");
    assert!(
        !output.status.success(),
        "expired open flake must fail-closed"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("expired") || combined.contains("flake"),
        "{combined}"
    );
}

#[test]
fn ci_flake_uses_utc_today_without_hardcoded_default() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "flake"])
        .output()
        .expect("flake with system UTC date");
    assert!(output.status.success(), "{:?}", output);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("open=0"), "{combined}");
}

#[test]
fn ci_flake_rejects_invalid_today() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "flake", "--today", "2026-02-31"])
        .output()
        .expect("flake with invalid date");
    assert!(!output.status.success(), "invalid calendar date must fail");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("invalid --today"), "{combined}");
}

#[test]
fn ci_determinism_ok() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "determinism", "--payload", r#"{"ok":true}"#])
        .output()
        .expect("det");
    assert!(output.status.success(), "{:?}", output);
}

#[test]
fn ci_no_lookahead_violation_fails() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/ci_negative/fixtures/no_lookahead_violation.json");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "no-lookahead", "--fixture", fixture.to_str().unwrap()])
        .output()
        .expect("nl");
    assert!(!output.status.success());
}

#[test]
fn ci_domain_gates_ok() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "domain-gates"])
        .output()
        .expect("dg");
    assert!(
        output.status.success(),
        "stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ci_chaos_subset_ok() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "chaos", "--json"])
        .output()
        .expect("chaos");
    assert!(
        output.status.success(),
        "stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["gate_ok"], true);
    assert_eq!(report["coverage_complete"], false);
    assert_eq!(report["executable_count"], 11);
    assert_eq!(report["stub_count"], 9);
    assert!(report["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["status"] == "STUB")
        .all(|case| case["executed"] == false && case["ok_expected_fail"] == false));
}

#[test]
fn ci_autoresearch_shadow_ok() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["ci", "autoresearch"])
        .output()
        .expect("autoresearch");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("shadow") || stdout.contains("Shadow") || stdout.contains("recs="),
        "{stdout}"
    );
}
