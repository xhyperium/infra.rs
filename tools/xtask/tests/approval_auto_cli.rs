//! xhyper-4do：approval-auto apply 冻结的负向 CLI 测试。
//!
//! 覆盖 AC：
//! 1. 未经 Approved 裁定不得恢复 apply（缺 --authorized-by / env → fail-closed）
//! 2. 不得由命令自授 standing_authorization / owner / 策略（policy gate bail）
//! 3. commit / time / identity 失败须 fail-closed（无 fallback）
//! 4. 写入有锁（pre-existing lock → 第二 apply fail-closed）
//!
//! 这些测试只走 apply 早失败的路径，不验证 dry-run 完整流（dry-run 仍依赖
//! git/date 等外部命令，超出 agent-safe subset 边界）。

use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn canonical_registry() -> Value {
    serde_json::from_slice(
        &fs::read(repo_root().join("docs/plans/infra-ig1-decisions.json")).unwrap(),
    )
    .unwrap()
}

fn copy_subjects(registry: &Value, root: &Path) {
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

/// 构造临时 workspace：minimal Cargo + canonical registry (+ 修改)。
fn make_workspace(mutate: impl FnOnce(&mut Value)) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
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
    let mut registry = canonical_registry();
    copy_subjects(&registry, root);
    mutate(&mut registry);
    let registry_path = root.join("docs/plans/infra-ig1-decisions.json");
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(registry_path, serde_json::to_vec_pretty(&registry).unwrap()).unwrap();
    dir
}

fn run_approval_auto(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn stderr_string(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// AC1: apply 路径未提供授权（CLI / env） → fail-closed。
#[test]
fn apply_without_authorization_fails_closed() {
    let dir = make_workspace(|_| {});
    let out = run_approval_auto(
        dir.path(),
        &["approval-auto", "--apply", "--owner", "alice"],
    );
    assert!(!out.status.success(), "expected failure");
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("apply 冻结中") && stderr.contains("xhyper-4do"),
        "missing apply-gate error in stderr: {stderr}"
    );
}

/// AC1: env 路径可触发授权（与 CLI 等价）。
#[test]
fn apply_authorization_env_path_recognized() {
    let dir = make_workspace(|_| {});
    // 用 env 但指向不存在的 decision id — 应在第二道（authorization invalid）失败，
    // 而非第一道（missing authorization），证明 env 被读取。
    let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .args(["approval-auto", "--apply", "--owner", "alice"])
        .env("XHYPER_APPROVAL_AUTO_APPROVED", "D-NOPE")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("授权无效") && stderr.contains("D-NOPE"),
        "env path not recognized or wrong error: {stderr}"
    );
}

/// AC1: --authorized-by 引用不存在的 decision → fail-closed。
#[test]
fn apply_with_unknown_authorization_fails_closed() {
    let dir = make_workspace(|_| {});
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-NOPE",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("授权无效") && stderr.contains("D-NOPE"),
        "missing invalid-auth error: {stderr}"
    );
}

/// AC1: --authorized-by 引用存在但非 APPROVED 的 decision → fail-closed。
#[test]
fn apply_with_unapproved_authorization_fails_closed() {
    let dir = make_workspace(|registry| {
        // 把 D-01 改成非 APPROVED；其它 policy 保留
        for decision in registry["decisions"].as_array_mut().unwrap() {
            if decision["id"] == "D-01" {
                decision["status"] = serde_json::json!("DRAFT");
            }
        }
    });
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("授权无效") && stderr.contains("D-01"),
        "missing unapproved-auth error: {stderr}"
    );
}

/// AC2: 禁止 mode 自授（strict_raci → bail）。
#[test]
fn policy_gate_rejects_strict_raci_self_grant() {
    let dir = make_workspace(|registry| {
        registry["approval_automation"]["mode"] = serde_json::json!("strict_raci");
    });
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("single_accountable_owner") && stderr.contains("xhyper-4do"),
        "missing mode policy error: {stderr}"
    );
}

/// AC2: 禁止 standing_authorization 自授。
#[test]
fn policy_gate_rejects_no_standing_authorization() {
    let dir = make_workspace(|registry| {
        registry["approval_automation"]["standing_authorization"] = serde_json::json!(false);
    });
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("standing_authorization") && stderr.contains("xhyper-4do"),
        "missing standing_authorization policy error: {stderr}"
    );
}

/// AC2: 禁止 ai_may_invoke_auto 自授。
#[test]
fn policy_gate_rejects_no_ai_may_invoke() {
    let dir = make_workspace(|registry| {
        registry["approval_automation"]["ai_may_invoke_auto"] = serde_json::json!(false);
    });
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("ai_may_invoke_auto") && stderr.contains("xhyper-4do"),
        "missing ai_may_invoke policy error: {stderr}"
    );
}

/// AC3: 缺 owner（无 CLI / registry / gh）→ fail-closed；不再 fallback `ZoneCNH`。
/// 测试用 `/dev/null` PATH 隔离 gh 与其它外部命令，模拟全部源失败的运行时。
#[test]
fn apply_fails_closed_when_owner_missing() {
    let dir = make_workspace(|registry| {
        registry["approval_automation"]["accountable_owner_handle"] =
            serde_json::json!("UNASSIGNED");
    });
    let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir.path())
        .args(["approval-auto", "--apply", "--authorized-by", "D-01"])
        .env(
            "PATH",
            "/dev/null:/usr/bin/env", // gh/git/date 都不可发现
        )
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("owner") || stderr.contains("gh"),
        "missing owner-fail error: {stderr}"
    );
    // 关键：fallback `ZoneCNH` 已移除；命令不应再静默使用虚构 handle
    assert!(
        !stderr.contains("ZoneCNH"),
        "ZoneCNH fallback still present: {stderr}"
    );
}

/// AC3: AI/reserved handle 拒绝。
#[test]
fn apply_rejects_ai_handle_owner() {
    let dir = make_workspace(|_| {});
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "claude-bot",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("non-human") || stderr.contains("reserved"),
        "missing reserved-handle error: {stderr}"
    );
}

/// AC4: 已有 lock 时第二个 apply 直接 fail-closed（即使 gate 通过）。
#[test]
fn apply_with_preexisting_lock_fails_closed() {
    let dir = make_workspace(|_| {});
    let lock_dir = dir.path().join(".approval-auto.lock.d");
    fs::create_dir_all(&lock_dir).unwrap();
    fs::write(lock_dir.join("approval-auto.apply.lock"), b"held-by-other").unwrap();
    let out = run_approval_auto(
        dir.path(),
        &[
            "approval-auto",
            "--apply",
            "--owner",
            "alice",
            "--authorized-by",
            "D-01",
        ],
    );
    assert!(!out.status.success());
    let stderr = stderr_string(&out);
    assert!(
        stderr.contains("AppLock") || stderr.contains("lock"),
        "missing lock error: {stderr}"
    );
}

/// AC1 边界：dry-run 路径不触发 apply 授权 gate（即便 --authorized-by 缺失）。
/// 注意：dry-run 仍可能因 git_head / date 失败，但不应在 apply gate 处失败。
#[test]
fn dry_run_skips_apply_authorization_gate() {
    let dir = make_workspace(|_| {});
    let out = run_approval_auto(dir.path(), &["approval-auto", "--owner", "alice"]);
    let stderr = stderr_string(&out);
    // 不论 dry-run 后续是否成功，apply gate 不应触发
    assert!(
        !stderr.contains("apply 冻结中"),
        "dry-run triggered apply gate: {stderr}"
    );
    assert!(
        !stderr.contains("授权无效"),
        "dry-run triggered authorization check: {stderr}"
    );
}
