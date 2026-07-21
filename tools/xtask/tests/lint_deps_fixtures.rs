//! xhyper-gmd agent-safe subset：lint-deps 负向 fixture 集合。
//!
//! AC 对照（与 evidence/xtask/lint-deps-fixtures/2026-07-18-implementation.md 一致）：
//! 1. R1/R3/R5/R1.2/ADR-007/R6 各覆盖至少 1 个负向 fixture；R1/R3/R6 同时含正向 fixture。
//!    **不**覆盖 R2/R4 独立 fixture（这两条已通过 R1/R2/R4 合并矩阵的既有测试间接覆盖；
//!    R2 同层互依、R4 contracts 出口未单独抽测，留 follow-up）。
//! 2. Unknown / 不可读输入 fail-closed（配合 lint_deps.rs 的 check_r6 / collect_rs_files 修复）。
//!    注意：本 PR 仅覆盖「不可读 .rs 文件」与「空/损坏 workspace metadata」；
//!    **不**覆盖 Unknown 零边 member（DEFERRED 项，由 RFC 裁定后另立 fixture）。
//! 3. 现有 main 的 27 unit + 25 + 11 CLI 测试不退化（CI test-stable PASS 为证据）。
//!
//! **DEFERRED**（不在本 PR 修复；跟踪 xhyper-gmd）：
//! - R6 按 package name 匹配 Rust ident（rename 后漏报）
//! - build-dependency R1-R5 跳过（line `if dep.kind != Normal { continue; }`）
//! - Unknown 零边 member 静默放过
//! - /crates/domain/ 路径过宽（domainx / domain_* 同标 Domain）
//! - R2 Types 粗粒度
//! - R1.2 箭头语义裁定（DEFERRED 给 Approved RFC）

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// 极简 member crate：package_name + 子目录 + 空 lib.rs。
struct MemberSpec {
    /// 相对 workspace root 的子目录（如 "crates/kernel"）；layer 由该路径推导。
    rel_dir: &'static str,
    package_name: &'static str,
}

fn make_workspace(members: &[MemberSpec], edges: &[(usize, usize)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // 注意：cargo metadata 解析 [workspace].members 时，**目录路径优先**；
    // 带 `/Cargo.toml` 后缀在某些 cargo 版本下会得到空 workspace。
    let member_paths: Vec<String> = members.iter().map(|m| m.rel_dir.to_string()).collect();
    let members_toml = member_paths
        .iter()
        .map(|p| format!("\"{p}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nresolver = \"2\"\nmembers = [{members_toml}]\n"),
    )
    .unwrap();

    // 创建每个 member
    let mut manifest_paths: Vec<PathBuf> = Vec::new();
    for m in members {
        let member_root = root.join(m.rel_dir);
        fs::create_dir_all(member_root.join("src")).unwrap();
        fs::write(member_root.join("src/lib.rs"), "").unwrap();
        manifest_paths.push(member_root.join("Cargo.toml"));
    }

    // 写每个 member 的 Cargo.toml（含 [package] + 依赖）
    for (idx, m) in members.iter().enumerate() {
        let mut deps_lines = Vec::new();
        for (from, to) in edges {
            if *from == idx {
                let target = &members[*to];
                // path 相对 member_root
                let from_path = root.join(m.rel_dir);
                let to_abs = root.join(target.rel_dir);
                let rel = pathdiff(&from_path, &to_abs);
                deps_lines.push(format!(
                    "{} = {{ package = \"{}\", path = \"{}\" }}",
                    dep_key(target.package_name),
                    target.package_name,
                    rel.display()
                ));
            }
        }
        let deps_block = if deps_lines.is_empty() {
            String::new()
        } else {
            let inner = deps_lines.join("\n");
            format!("\n[dependencies]\n{inner}")
        };
        let manifest = format!(
            "[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[lib]\nname = \"{}\"\npath = \"src/lib.rs\"\n{deps_block}\n",
            m.package_name,
            lib_name(m.package_name),
        );
        fs::write(&manifest_paths[idx], manifest).unwrap();
    }

    dir
}

/// 把 package_name (`xhyper-foo-bar`) 转成 dep key (`foo_bar`，与 lib name 同步)。
fn dep_key(package_name: &str) -> String {
    package_name
        .strip_prefix("xhyper-")
        .unwrap_or(package_name)
        .replace('-', "_")
}

fn lib_name(package_name: &str) -> String {
    dep_key(package_name)
}

fn pathdiff(from: &Path, to: &Path) -> PathBuf {
    // 极简实现：从 `from` 出发，先 ".." 到共同祖先，再到 `to`。
    // 对 fixture（结构简单）足够。
    let mut from_comps: Vec<_> = from.components().collect();
    let mut to_comps: Vec<_> = to.components().collect();
    while !from_comps.is_empty() && !to_comps.is_empty() && from_comps[0] == to_comps[0] {
        from_comps.remove(0);
        to_comps.remove(0);
    }
    let mut out = PathBuf::new();
    for _ in &from_comps {
        out.push("..");
    }
    for c in to_comps {
        if let std::path::Component::Normal(s) = c {
            out.push(s);
        }
    }
    out
}

fn run_lint_deps(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(root)
        .args(["lint-deps"])
        .output()
        .unwrap()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// ============================================================
// R1：testkit 仅允许 dev-dependency（非 XTask/Kernel/Infra/TestSupport）
// ============================================================

#[test]
fn r1_positive_testkit_as_dev_dep_passes() {
    // domain → testkit 仅在 dev-deps，不影响 normal dep 校验
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/domain/foo",
                package_name: "xhyper-domain-foo",
            },
            MemberSpec {
                rel_dir: "crates/testkit",
                package_name: "xhyper-testkit",
            },
        ],
        &[], // 无 normal edge
    );
    // 手动追加 dev-deps
    let member_a = dir.path().join("crates/domain/foo/Cargo.toml");
    let mut content = fs::read_to_string(&member_a).unwrap();
    content.push_str("\n[dev-dependencies]\nxhyper_testkit = { package = \"xhyper-testkit\", path = \"../../testkit\" }\n");
    fs::write(&member_a, content).unwrap();

    let out = run_lint_deps(dir.path());
    assert!(out.status.success(), "expected PASS: {}", stderr(&out));
}

#[test]
fn r1_negative_testkit_as_normal_dep_fails() {
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/domain/foo",
                package_name: "xhyper-domain-foo",
            },
            MemberSpec {
                rel_dir: "crates/testkit",
                package_name: "xhyper-testkit",
            },
        ],
        &[(0, 1)], // domain → testkit (normal)
    );
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(s.contains("[R1]"), "missing R1 violation: {s}");
}

// ============================================================
// R3：L1 互依禁止（bootstrap 豁免）
// ============================================================

#[test]
fn r3_positive_l1_to_contract_passes() {
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/infra/foo",
                package_name: "xhyper-foo",
            },
            MemberSpec {
                rel_dir: "crates/contracts",
                package_name: "xhyper-contracts",
            },
        ],
        &[(0, 1)],
    );
    let out = run_lint_deps(dir.path());
    assert!(out.status.success(), "expected PASS: {}", stderr(&out));
}

#[test]
fn r3_negative_l1_to_l1_fails() {
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/infra/foo",
                package_name: "xhyper-foo",
            },
            MemberSpec {
                rel_dir: "crates/infra/bar",
                package_name: "xhyper-bar",
            },
        ],
        &[(0, 1)],
    );
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(s.contains("[R3]"), "missing R3 violation: {s}");
}

// ============================================================
// R5：domain 三平级互斥（domain_market / domain_macro / domain_exchange）
// ============================================================

#[test]
fn r5_negative_domain_peer_to_peer_fails() {
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/domain/market",
                package_name: "xhyper-domain-market",
            },
            MemberSpec {
                rel_dir: "crates/domain/macro",
                package_name: "xhyper-domain-macro",
            },
        ],
        &[(0, 1)],
    );
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(s.contains("[R5]"), "missing R5 violation: {s}");
}

// ============================================================
// R1.2：domainx 单向链（domainx → domain_* 反向依赖禁止）
// ============================================================

#[test]
fn r1_2_negative_domainx_to_domain_peer_fails() {
    // 注意：classify_layer 对 `/crates/domain/` 全部归 Domain（已知 DEFERRED 的过宽问题），
    // 但 R1.2 单独按 canonical name (`domainx` → `domain_market` 等) 判定。
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/domain/core",
                package_name: "xhyper-domainx",
            },
            MemberSpec {
                rel_dir: "crates/domain/market",
                package_name: "xhyper-domain-market",
            },
        ],
        &[(0, 1)],
    );
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(s.contains("[R1.2]"), "missing R1.2 violation: {s}");
}

// ============================================================
// ADR-007：decimalx 不依赖 canonical
// ============================================================

#[test]
fn adr_007_negative_decimalx_to_canonical_fails() {
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/types/decimalx",
                package_name: "xhyper-decimalx",
            },
            MemberSpec {
                rel_dir: "crates/types/canonical",
                package_name: "xhyper-canonical",
            },
        ],
        &[(0, 1)],
    );
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(s.contains("[ADR-007]"), "missing ADR-007 violation: {s}");
}

// ============================================================
// R6：跨层 pub use 具体实现类型禁止
//
// 注意：当前实现 `check_r6` 用 `by_name.get(crate_name)` 全字符串匹配
// —— 对真实 workspace（package = `xhyper-foo`，lib = `foo`，源码 `pub use foo::*`）
// 会**漏报**。这是 xhyper-gmd DEFERRED 的「R6 package-vs-ident 假阴性」，
// 留 Approved RFC 后统一修复。
//
// 本 fixture 选用 package name == lib ident 的命名，让现有实现可触发 R6；
// 同时为后续修复提供 pass-fail 对照（修复后应继续通过）。
// ============================================================

#[test]
fn r6_negative_pub_use_l1_concrete_fails() {
    // 用「package 名 == lib 名」的 fixture，绕过 DEFERRED 假阴性。
    // 构造 Apps → Infra 边（allowed_targets 允许 Apps → Infra），让 check_deps 不报
    // [R1/R2/R4]，从而隔离 check_r6 的纯 [R6] 触发路径（避免 R6 测试被前置规则短路）。
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "apps/foo",
                package_name: "xhyper-app-foo",
            },
            MemberSpec {
                rel_dir: "crates/infra/myconfigx",
                package_name: "myconfigx", // package = lib ident（非 xhyper- 前缀）
            },
        ],
        &[(0, 1)],
    );
    let lib = dir.path().join("apps/foo/src/lib.rs");
    fs::write(&lib, "pub use myconfigx::Foo;\n").unwrap();

    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL");
    let s = stderr(&out);
    assert!(
        s.contains("[R6]"),
        "missing R6 violation (may indicate regression in check_r6): {s}"
    );
    assert!(
        !s.contains("[R1/R2/R4]"),
        "R1/R2/R4 fired unexpectedly; Apps->Infra should be allowed: {s}"
    );
}

#[test]
fn r6_positive_pub_use_kernel_passes() {
    // Apps crate 的源码 `pub use mykernel::Foo;`（kernel 类，允许重导出）。
    // 用 Apps 而非 Domain 让本测试与 r6_negative_pub_use_l1_concrete_fails 对称，
    // 避免 R6 测试在 Domain→L1 上意外触发 R1/R2/R4。
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "apps/foo",
                package_name: "xhyper-app-foo",
            },
            MemberSpec {
                rel_dir: "crates/kernel",
                package_name: "mykernel", // package = lib ident；让 by_name 能匹配
            },
        ],
        &[(0, 1)],
    );
    let lib = dir.path().join("apps/foo/src/lib.rs");
    fs::write(&lib, "pub use mykernel::Foo;\n").unwrap();

    let out = run_lint_deps(dir.path());
    assert!(out.status.success(), "expected PASS: {}", stderr(&out));
}

// ============================================================
// AC #2：不可读 / Unknown 输入 fail-closed
// ============================================================

/// 不可读 .rs 文件触发 check_r6 fail-closed（修复后的行为）。
/// 注：在 root 权限下 chmod 000 仍可读；本测试需要非 root runner。
#[test]
fn r6_unreadable_rs_file_fails_closed() {
    // SAFETY：CI 通常非 root；若 root 跑测试则 skip。
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("[skip] running as root, chmod no-op");
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    let dir = make_workspace(
        &[
            MemberSpec {
                rel_dir: "crates/domain/foo",
                package_name: "xhyper-domain-foo",
            },
            MemberSpec {
                rel_dir: "crates/kernel",
                package_name: "xhyper-kernel",
            },
        ],
        &[(0, 1)],
    );
    let lib = dir.path().join("crates/domain/foo/src/lib.rs");
    fs::write(&lib, "pub use kernel::Foo;\n").unwrap();
    // chmod 000
    let perms = fs::Permissions::from_mode(0o000);
    fs::set_permissions(&lib, perms).unwrap();

    let out = run_lint_deps(dir.path());
    assert!(
        !out.status.success(),
        "expected FAIL (unreadable file): {}",
        stderr(&out)
    );
    let s = stderr(&out);
    // 失败原因应该是 R6 read 失败（非 lint-deps 规则违规）
    assert!(
        s.contains("R6") && (s.contains("read") || s.contains("读取")),
        "expected read failure context in stderr: {s}"
    );
}

/// 空 workspace（`members = []`）— cargo metadata 自身对 virtual manifest
/// 且无 members 时 **fail-closed**（"workspace has no members"）。
/// 这正是 agent-safe subset 期望的「Unknown 输入 fail-closed」行为。
#[test]
fn empty_workspace_fails_closed_at_metadata() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = []\n",
    )
    .unwrap();
    let out = run_lint_deps(dir.path());
    assert!(
        !out.status.success(),
        "expected FAIL for empty workspace: {}",
        stderr(&out)
    );
}

/// 损坏的 Cargo.toml — cargo metadata fail-closed。
#[test]
fn malformed_cargo_toml_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace\nthis is not valid toml\n",
    )
    .unwrap();
    let out = run_lint_deps(dir.path());
    assert!(!out.status.success(), "expected FAIL for malformed toml");
}
