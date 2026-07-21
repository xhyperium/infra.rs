//! crate 分层分类（spec §3 路径约定）。

/// workspace crate 的分层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Kernel,      // crates/kernel/ (L0)
    Types,       // crates/types/
    Contract,    // crates/contracts/
    Infra,       // crates/infra/  (L1，含 transportx，ADR-007)
    Storage,     // crates/adapters/storage/
    Exchange,    // crates/adapters/exchange/
    Domain,      // crates/domain/ (L2.5)
    Services,    // crates/services/ (capabilities)
    Apps,        // apps/ (composition roots)
    XTask,       // 工具链自身，不受业务规则约束
    Legacy,      // 历史 `legacy/` 路径分类（目录已 sunset；保留枚举以兼容路径规则）
    TestSupport, // crates/test-support/ + testkit plane (SPEC-TESTKIT-002)
    Unknown,
}

/// 是否为 workspace 内 `tools/<member>/…` 清单路径。
///
/// 避免父目录名含 `tools` 的误匹配（如 `/home/me/tools/xhyper.rs/crates/kernel/…`）。
fn is_workspace_tools_manifest(path: &str) -> bool {
    let p = path.replace('\\', "/");
    for (idx, _) in p.match_indices("/tools/") {
        let rest = &p[idx + "/tools/".len()..];
        let mut parts = rest.split('/');
        let Some(member) = parts.next() else {
            continue;
        };
        if member.is_empty() {
            continue;
        }
        // 父目录假阳性：tools/<clone-name>/crates|apps|…
        match parts.next() {
            Some("crates" | "apps" | "docs" | ".git" | ".github") => continue,
            Some(_) | None => return true,
        }
    }
    // 相对路径 `tools/<member>/Cargo.toml`
    p.starts_with("tools/") && p.matches('/').count() >= 1
}

/// 按 manifest 路径前缀判定分层（spec §3 路径约定）。
pub fn classify_layer(manifest_path: &str) -> Layer {
    let p = manifest_path.replace('\\', "/");
    // tools/*（archgate / schema_codegen / xtask / evidence-cli / goalctl …）
    // 对齐 .architecture/workspace.toml layer = "tools"，不按业务 R1–R6 约束。
    if is_workspace_tools_manifest(&p) {
        Layer::XTask
    } else if p.contains("/crates/evidence/") {
        // evidence core 为 L0（runtime 不得再在 tools/）
        // runtime gate crate 已退役删除（PLAN-GATE-RETIRE-001）；勿再将 infra/gate 归 Kernel
        Layer::Kernel
    } else if p.contains("/legacy/") {
        Layer::Legacy
    } else if p.contains("/crates/test-support/") || p.contains("/crates/testkit/") {
        // SPEC-TESTKIT-002：T0 test-support 平面
        Layer::TestSupport
    } else if p.contains("/crates/kernel/") {
        Layer::Kernel
    } else if p.contains("/crates/types/") {
        Layer::Types
    } else if p.contains("/crates/contracts/") {
        Layer::Contract
    } else if p.contains("/crates/infra/") {
        Layer::Infra
    } else if p.contains("/crates/adapters/storage/") || p.contains("/crates/adapters/evidence/") {
        // evidence adapters（memory/file/postgres/signer）与存储适配器同属 Adapter 层矩阵
        Layer::Storage
    } else if p.contains("/crates/adapters/exchange/") {
        Layer::Exchange
    } else if p.contains("/crates/domain/") {
        Layer::Domain
    } else if p.contains("/crates/services/") {
        Layer::Services
    } else if p.contains("/apps/") {
        Layer::Apps
    } else {
        classify_by_name(crate_name_from_path(&p)).unwrap_or(Layer::Unknown)
    }
}

/// 名称兜底：路径异常时按 crate 名识别（兼容 `xhyper-` 前缀 package 名）。
pub fn classify_by_name(name: &str) -> Option<Layer> {
    let short = name.strip_prefix("xhyper-").unwrap_or(name);
    let underscored = short.replace('-', "_");
    match short {
        "xtask" | "evidence-cli" | "goalctl" | "archgate" | "schema-codegen" => Some(Layer::XTask),
        "kernel" | "gate" | "evidence" => Some(Layer::Kernel),
        "testkit" | "contract-testkit" => Some(Layer::TestSupport),
        "stdio" | "quant" => Some(Layer::Legacy),
        _ => match underscored.as_str() {
            "evidence_memory" | "evidence_file" | "evidence_postgres" | "evidence_signer" => {
                Some(Layer::Storage)
            }
            "contract_testkit" => Some(Layer::TestSupport),
            "schema_codegen" => Some(Layer::XTask),
            _ => None,
        },
    }
}

fn crate_name_from_path(path: &str) -> &str {
    let parts: Vec<&str> = path.split('/').collect();
    for (i, p) in parts.iter().enumerate() {
        if (*p == "crates" || *p == "tools") && i + 1 < parts.len() {
            return parts[i + 1];
        }
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_paths_classify_as_xtask() {
        for path in [
            "/repo/tools/xtask/Cargo.toml",
            "/repo/tools/archgate/Cargo.toml",
            "/repo/tools/schema_codegen/Cargo.toml",
            "/repo/tools/evidence-cli/Cargo.toml",
            "/repo/tools/goalctl/Cargo.toml",
            "tools/archgate/Cargo.toml",
        ] {
            assert_eq!(
                classify_layer(path),
                Layer::XTask,
                "expected XTask for {path}"
            );
        }
    }

    #[test]
    fn parent_dir_named_tools_does_not_classify_crates_as_xtask() {
        // Codex review: clone under …/tools/<repo>/ 时不得把业务 crate 标成 XTask
        assert_eq!(
            classify_layer("/home/me/tools/xhyper.rs/crates/kernel/Cargo.toml"),
            Layer::Kernel
        );
        assert_eq!(
            classify_layer("/home/me/tools/xhyper.rs/crates/domain/core/Cargo.toml"),
            Layer::Domain
        );
    }

    #[test]
    fn tools_package_names_classify_as_xtask() {
        for name in [
            "xhyper-archgate",
            "archgate",
            "xhyper-schema-codegen",
            "schema-codegen",
            "xhyper-xtask",
        ] {
            assert_eq!(
                classify_by_name(name),
                Some(Layer::XTask),
                "expected XTask for {name}"
            );
        }
    }

    #[test]
    fn evidence_adapters_are_storage_not_kernel() {
        assert_eq!(
            classify_layer("/repo/crates/adapters/evidence/memory/Cargo.toml"),
            Layer::Storage
        );
        assert_eq!(
            classify_layer("/repo/crates/evidence/Cargo.toml"),
            Layer::Kernel
        );
    }
}
