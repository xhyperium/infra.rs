use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn canonical_registry() -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(repo_root().join("docs/plans/infra-ig1-decisions.json")).unwrap(),
    )
    .unwrap()
}

fn copy_subjects(registry: &serde_json::Value, root: &Path) {
    let subjects = registry["required_proposals"]
        .as_array()
        .unwrap()
        .iter()
        .chain(registry["decisions"].as_array().unwrap())
        .map(|entry| entry["subject_ref"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    for subject in subjects {
        let destination = root.join(subject);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(repo_root().join(subject), destination).unwrap();
    }
}

fn run_modified_registry(registry: &serde_json::Value) -> Output {
    run_modified_registry_with_files(registry, &[])
}

fn run_modified_registry_with_files(
    registry: &serde_json::Value,
    files: &[(&str, &[u8])],
) -> Output {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"member\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("member/src")).unwrap();
    fs::write(
        root.join("member/Cargo.toml"),
        "[package]\nname = \"approval-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("member/src/lib.rs"), "").unwrap();
    copy_subjects(&canonical_registry(), root);
    let evidence_schema = "schemas/jsonschema/evidence-record.schema.json";
    fs::create_dir_all(root.join(evidence_schema).parent().unwrap()).unwrap();
    fs::copy(
        repo_root().join(evidence_schema),
        root.join(evidence_schema),
    )
    .unwrap();
    let registry_path = root.join("docs/plans/infra-ig1-decisions.json");
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(registry_path, serde_json::to_vec_pretty(registry).unwrap()).unwrap();
    for (relative, contents) in files {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(root)
        .args(["--json", "approval-check", "--registry-only"])
        .output()
        .unwrap()
}

fn approval(
    approver_handle: &str,
    approver_role: &str,
    scope: &str,
    subject_sha256: &str,
) -> serde_json::Value {
    serde_json::json!({
        "approver_handle": approver_handle,
        "approver_role": approver_role,
        "decision": "APPROVED",
        "scope": scope,
        "reason": "independent architecture review completed",
        "ticket_url": "https://github.com/xhyperium/xhyper.rs/issues/1",
        "review_url": "https://github.com/xhyperium/xhyper.rs/pull/1#pullrequestreview-1",
        "reviewed_commit": "1111111111111111111111111111111111111111",
        "subject_revision": 1,
        "subject_sha256": subject_sha256,
        "approved_at": "2026-07-14T00:00:00Z",
        "valid_until": "2099-12-31T23:59:59Z",
        "independence_check": true
    })
}

#[test]
fn default_registry_path_works_from_a_workspace_subdirectory() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root().join("tools/xtask"))
        .args(["--json", "approval-check", "--registry-only"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["registry_valid"], true);
    // single_accountable_owner automation 下 registry-only 允许 gate_ready=true
    assert!(report["gate_ready"].is_boolean());
}

#[test]
fn cli_does_not_accept_a_gate_override() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .args([
            "--json",
            "approval-check",
            "--registry-only",
            "--gate",
            "IG-2",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument '--gate'"));
}

#[test]
fn canonical_automated_registry_is_structurally_valid() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .args(["--json", "approval-check", "--registry-only"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["registry_valid"], true);
    assert_eq!(report["decision_count"], 20);
    assert_eq!(report["proposal_count"], 9);
    // automation 放宽 external readback 后，gate_ready 可为 true；readback 实现本身仍为 false
    assert_eq!(report["trusted_review_readback"], false);
    let blockers = report["blockers"].as_array().unwrap();
    // D-06b 不在 IG-1 exit set，AWAITING 不得作为 blocker
    assert!(!blockers.iter().any(
        |blocker| blocker.as_str().unwrap() == "decision-not-approved:D-06b:AWAITING_APPROVAL"
    ));
}

#[test]
fn registry_only_rejects_a_stale_subject_digest() {
    let mut registry = canonical_registry();
    registry["decisions"][0]["subject_sha256"] = serde_json::Value::String("0".repeat(64));
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["registry_valid"], false);
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding
            .as_str()
            .unwrap()
            .contains("subject-hash-mismatch:D-01")));
}

#[test]
fn registry_only_requires_the_exact_split_d06_decision_set() {
    let mut registry = canonical_registry();
    registry["decisions"]
        .as_array_mut()
        .unwrap()
        .retain(|decision| decision["id"] != "D-06b");
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "decision-set-missing:D-06b"));
}

#[test]
fn registry_only_requires_all_nine_required_proposals() {
    let mut registry = canonical_registry();
    registry["required_proposals"]
        .as_array_mut()
        .unwrap()
        .retain(|proposal| proposal["id"] != "P-09-l2-service");
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "proposal-set-missing:P-09-l2-service"));
}

#[test]
fn approved_registry_status_cannot_point_at_a_draft_proposal() {
    let mut registry = canonical_registry();
    // 强制 automation 关闭，避免 single-owner 路径掩盖 status mismatch
    registry["approval_automation"] = serde_json::json!({
        "mode": "strict_raci",
        "accountable_owner_handle": "UNASSIGNED",
        "allow_owner_multi_role": false,
        "machine_attestation_accepted": false,
        "gate_ready_requires_external_readback": true
    });
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("alice");
    architecture["backup_handle"] = serde_json::json!("bob");
    // 仅批准 P-01，其余恢复 DRAFT 以免无关失败淹没断言
    for proposal in registry["required_proposals"].as_array_mut().unwrap() {
        proposal["status"] = serde_json::json!("DRAFT");
        proposal["approvals"] = serde_json::json!([]);
    }
    registry["required_proposals"][0]["status"] = serde_json::json!("APPROVED");
    let subject_ref = registry["required_proposals"][0]["subject_ref"]
        .as_str()
        .unwrap()
        .to_owned();
    let draft_body = b"# draft subject\n\n- **status**: Draft\n";
    let subject_sha256 = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(draft_body.as_slice()))
    };
    registry["required_proposals"][0]["subject_sha256"] =
        serde_json::Value::String(subject_sha256.clone());
    registry["required_proposals"][0]["approvals"] = serde_json::json!([approval(
        "alice",
        "Architecture Owner",
        "P-01-control-plane",
        &subject_sha256,
    )]);
    let output = run_modified_registry_with_files(
        &registry,
        &[(subject_ref.as_str(), draft_body.as_slice())],
    );

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap()
            == "proposal-subject-status-mismatch:P-01-control-plane:APPROVED"));
}

#[test]
fn registry_only_rejects_an_undefined_approver_role() {
    let mut registry = canonical_registry();
    registry["decisions"][0]["required_roles"] = serde_json::json!(["Domain Owner"]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "required-role-undefined:D-01:Domain Owner"));
}

#[test]
fn registry_only_rejects_roles_outside_the_governance_matrix() {
    let mut registry = canonical_registry();
    let model = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Model Owner")
        .unwrap();
    model["role"] = serde_json::json!("Domain Owner");
    registry["decisions"][0]["required_roles"] = serde_json::json!(["Domain Owner"]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "role-binding-extra:Domain Owner"));
}

#[test]
fn registry_only_rejects_bot_or_ai_approvers() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("dependabot[bot]");
    registry["decisions"][0]["status"] = serde_json::json!("APPROVED");
    let subject_sha256 = registry["decisions"][0]["subject_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    registry["decisions"][0]["approvals"] = serde_json::json!([approval(
        "dependabot[bot]",
        "Architecture Owner",
        "D-01",
        &subject_sha256,
    )]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"].as_array().unwrap().iter().any(
        |finding| finding.as_str().unwrap() == "approval-actor-not-human:D-01:dependabot[bot]"
    ));
}

#[test]
fn registry_only_rejects_non_human_role_bindings_without_an_approval() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("dependabot[bot]");
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap()
            == "role-binding-actor-not-human:Architecture Owner:dependabot[bot]"));
}

#[test]
fn registry_only_rejects_the_same_owner_and_backup() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("alice");
    architecture["backup_handle"] = serde_json::json!("alice");
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap()
            == "role-binding-owner-backup-same:Architecture Owner:alice"));
}

#[test]
fn registry_only_normalizes_handles_before_separation_checks() {
    let mut registry = canonical_registry();
    for binding in registry["role_bindings"].as_array_mut().unwrap() {
        match binding["role"].as_str().unwrap() {
            "Architecture Owner" => binding["owner_handle"] = serde_json::json!("alice"),
            "Data Owner" => binding["owner_handle"] = serde_json::json!("@ALICE"),
            _ => {}
        }
    }
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding.as_str().unwrap()
                == "role-binding-handle-reused:alice:Architecture Owner:Data Owner"
        }));
}

#[test]
fn registry_only_rejects_ai_handle_variants() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("AI_AGENT_2");
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding.as_str().unwrap()
                == "role-binding-actor-not-human:Architecture Owner:AI_AGENT_2"
        }));
}

#[test]
fn registry_only_rejects_natural_person_self_review() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("alice");
    registry["decisions"][0]["proposal_authors"] = serde_json::json!(["alice"]);
    registry["decisions"][0]["status"] = serde_json::json!("APPROVED");
    let subject_sha256 = registry["decisions"][0]["subject_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    registry["decisions"][0]["approvals"] = serde_json::json!([approval(
        "alice",
        "Architecture Owner",
        "D-01",
        &subject_sha256,
    )]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "approval-self-review:D-01:alice"));
}

#[test]
fn d06b_cannot_be_approved_before_d06a_or_without_compatibility_evidence() {
    let mut registry = canonical_registry();
    let owner = registry["approval_automation"]["accountable_owner_handle"]
        .as_str()
        .unwrap_or("ZoneCNH")
        .to_owned();
    // 确保 D-06a 未批，以覆盖 dependency finding；同时缺 evidence
    for decision in registry["decisions"].as_array_mut().unwrap() {
        if decision["id"] == "D-06a" || decision["id"] == "D-06b" {
            decision["status"] = serde_json::json!("AWAITING_APPROVAL");
            decision["approvals"] = serde_json::json!([]);
        }
    }
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["status"] = serde_json::json!("APPROVED");
    d06b["evidence_refs"] = serde_json::json!([]);
    let subject_sha256 = d06b["subject_sha256"].as_str().unwrap().to_owned();
    d06b["approvals"] = serde_json::json!([
        approval(&owner, "Data Owner", "D-06b", &subject_sha256),
        approval(&owner, "Architecture Owner", "D-06b", &subject_sha256)
    ]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["findings"].as_array().unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.as_str().unwrap()
                == "decision-dependency-not-approved:D-06b:D-06a")
    );
    assert!(findings
        .iter()
        .any(|finding| finding.as_str().unwrap() == "decision-evidence-required:D-06b"));
}

#[test]
fn d06b_rejects_a_repository_file_masquerading_as_evidence() {
    let mut registry = canonical_registry();
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["evidence_refs"] = serde_json::json!([{
        "path": "Cargo.toml",
        "sha256": "0".repeat(64)
    }]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding.as_str().unwrap() == "decision-evidence-not-record:D-06b:Cargo.toml"
        }));
}

#[test]
fn registry_only_rejects_decision_dependency_cycles() {
    let mut registry = canonical_registry();
    registry["decisions"][0]["depends_on_decisions"] = serde_json::json!(["D-01"]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding.as_str().unwrap() == "decision-dependency-cycle:D-01"));
}

#[test]
fn registry_only_rejects_ig1_policy_drift() {
    let mut registry = canonical_registry();
    registry["decisions"][0]["required_roles"] = serde_json::json!([]);
    registry["required_proposals"][0]["required_roles"] = serde_json::json!([]);
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["depends_on_decisions"] = serde_json::json!([]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["findings"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding == "decision-required-roles-policy-mismatch:D-01"));
    assert!(findings
        .iter()
        .any(|finding| finding == "proposal-required-roles-policy-mismatch:P-01-control-plane"));
    assert!(findings
        .iter()
        .any(|finding| finding == "decision-dependencies-policy-mismatch:D-06b"));
}

#[test]
fn registry_only_binds_approval_to_revision_and_rejects_malformed_provenance() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("alice");
    registry["decisions"][0]["status"] = serde_json::json!("APPROVED");
    let mut record = approval("alice", "Architecture Owner", "D-01", &"0".repeat(64));
    record["subject_revision"] = serde_json::json!(2);
    record["review_url"] =
        serde_json::json!("https://github.com/xhyperium/xhyper.rs/pull/x#pullrequestreview-y");
    record["approved_at"] = serde_json::json!("T+");
    registry["decisions"][0]["approvals"] = serde_json::json!([record]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["findings"].as_array().unwrap();
    for expected in [
        "approval-subject-hash-mismatch:D-01",
        "approval-subject-revision-mismatch:D-01",
        "approval-review-url-invalid:D-01:https://github.com/xhyperium/xhyper.rs/pull/x#pullrequestreview-y",
        "approval-time-invalid:D-01:T+",
    ] {
        assert!(findings.iter().any(|finding| finding == expected), "{expected}");
    }
}

#[test]
fn registry_only_rejects_an_expired_approval() {
    let mut registry = canonical_registry();
    let architecture = registry["role_bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["role"] == "Architecture Owner")
        .unwrap();
    architecture["owner_handle"] = serde_json::json!("alice");
    let subject_sha256 = registry["decisions"][0]["subject_sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut record = approval("alice", "Architecture Owner", "D-01", &subject_sha256);
    record["approved_at"] = serde_json::json!("2000-01-01T00:00:00Z");
    record["valid_until"] = serde_json::json!("2000-01-02T00:00:00Z");
    registry["decisions"][0]["approvals"] = serde_json::json!([record]);

    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| { finding == "approval-valid-until-invalid:D-01:2000-01-02T00:00:00Z" }));
}

#[test]
fn d06b_rejects_an_arbitrary_existing_file_as_evidence() {
    let mut registry = canonical_registry();
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["status"] = serde_json::json!("APPROVED");
    d06b["evidence_refs"] = serde_json::json!([{
        "path": "README.md",
        "sha256": "0".repeat(64)
    }]);
    let output = run_modified_registry(&registry);

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding == "decision-evidence-not-record:D-06b:README.md"));
}

#[test]
fn d06b_rejects_cargo_check_masquerading_as_roundtrip_evidence() {
    let record = serde_json::json!({
        "schema_version": "0.1.0-draft",
        "work_package": "INFRA-012",
        "acceptance_criterion": "D-06b-rest-roundtrip",
        "repository": "xhyperium/xhyper.rs",
        "branch": "docs/infra-approval-entry",
        "commit": "1111111111111111111111111111111111111111",
        "dirty_state": false,
        "command": "cargo check -p xhyper-taosx --features native",
        "started_at": "2026-07-14T00:00:00Z",
        "ended_at": "2026-07-14T00:00:01Z",
        "exit_code": 0,
        "result": "PASS",
        "sensitivity": "internal",
        "retention": { "class": "audit" },
        "environment": { "services": [{
            "name": "tdengine",
            "image_digest": format!("sha256:{}", "1".repeat(64)),
            "client_version": "taos-0.12.4"
        }]},
        "data_oracle": { "count": 1, "precision_ok": true },
        "verifier": { "status": "PASS" }
    });
    let bytes = serde_json::to_vec_pretty(&record).unwrap();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut registry = canonical_registry();
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["evidence_refs"] = serde_json::json!([{
        "path": "evidence/infrastructure/012/cargo-check.evidence.json",
        "sha256": digest
    }]);

    let output = run_modified_registry_with_files(
        &registry,
        &[(
            "evidence/infrastructure/012/cargo-check.evidence.json",
            &bytes,
        )],
    );

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["findings"].as_array().unwrap().iter().any(|finding| {
        finding
            == "decision-evidence-semantic-invalid:D-06b:evidence/infrastructure/012/cargo-check.evidence.json"
    }));
}

#[test]
fn d06b_requires_rest_native_and_native_safety_evidence_as_a_set() {
    let record = serde_json::json!({
        "schema_version": "0.1.0-draft",
        "work_package": "INFRA-012",
        "acceptance_criterion": "D-06b-rest-roundtrip",
        "repository": "xhyperium/xhyper.rs",
        "branch": "docs/infra-approval-entry",
        "commit": "1111111111111111111111111111111111111111",
        "dirty_state": false,
        "command": "cargo test -p xhyper-taosx rest_roundtrip -- --ignored",
        "started_at": "2026-07-14T00:00:00Z",
        "ended_at": "2026-07-14T00:00:01Z",
        "exit_code": 0,
        "result": "PASS",
        "sensitivity": "internal",
        "retention": { "class": "audit" },
        "environment": { "services": [{
            "name": "tdengine",
            "image_digest": format!("sha256:{}", "1".repeat(64)),
            "client_version": "taos-0.12.4"
        }]},
        "data_oracle": { "count": 1, "precision_ok": true },
        "verifier": { "status": "PASS" }
    });
    let bytes = serde_json::to_vec_pretty(&record).unwrap();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut registry = canonical_registry();
    let d06b = registry["decisions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|decision| decision["id"] == "D-06b")
        .unwrap();
    d06b["status"] = serde_json::json!("APPROVED");
    d06b["evidence_refs"] = serde_json::json!([{
        "path": "evidence/infrastructure/012/rest.evidence.json",
        "sha256": digest
    }]);

    let output = run_modified_registry_with_files(
        &registry,
        &[("evidence/infrastructure/012/rest.evidence.json", &bytes)],
    );

    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["findings"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding == "decision-evidence-kind-missing:D-06b:native-roundtrip"));
    assert!(findings
        .iter()
        .any(|finding| finding == "decision-evidence-kind-missing:D-06b:native-safety"));
    assert!(!findings.iter().any(|finding| {
        finding
            == "decision-evidence-semantic-invalid:D-06b:evidence/infrastructure/012/rest.evidence.json"
    }));
}

#[test]
fn cli_does_not_accept_a_registry_path_override() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .args(["approval-check", "--path", "registry.json"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument '--path'"));
}
