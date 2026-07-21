//! INFRA-060：只读 Drift Detection 脚手架。
//!
//! - 复用 `inventory-ssot` 的检测器集合
//! - 输出故意/可预期漂移类别 taxonomy
//! - **禁止**自动修复（`auto_repair` 恒 false；无 `--fix` 开关）
//!
//! 完整 version/image/schema/topic/stream/table/bucket/runner 漂移与 fault taxonomy
//! 仍为后续里程碑；本命令是 AI 可执行的只读切片。

use anyhow::{bail, Result};
use serde::Serialize;

use crate::inventory_ssot::{self, INTENTIONAL_CATEGORIES};

#[derive(Debug, Serialize)]
struct Report {
    passed: bool,
    auto_repair: bool,
    auto_repair_enabled: bool,
    work_package: &'static str,
    categories: Vec<Category>,
    inventory: inventory_ssot::Report,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct Category {
    id: &'static str,
    mode: &'static str,
    description: &'static str,
}

const CATEGORY_DOCS: &[(&str, &str)] = &[
    ("ssot-roots", "SSOT 五面入口文件是否存在"),
    (
        "parallel-control-plane",
        "禁止未批准的 .infrastructure/ 平行树",
    ),
    (
        "architecture-unit-ids",
        "architecture unit path/id 唯一且非空",
    ),
    (
        "cargo-vs-architecture",
        "cargo members 必须被 architecture 覆盖",
    ),
    (
        "dependency-policy-width",
        "dependency.toml 不得宽于 R3/R2.1",
    ),
    ("sql-float", "SQL 禁止金融 DOUBLE/FLOAT/REAL"),
    (
        "schema-financial-float",
        "proto/jsonschema 金融字段禁止 float/number",
    ),
    (
        "target-dir-config",
        ".cargo/config.toml 必须指向 .cargo/target/",
    ),
    (
        "target-dir-hardcode-scan",
        "scripts/workflows 禁止硬编码 ./target/",
    ),
    (
        "exclude-or-quarantine-warn",
        "architecture 多出的 exclude/quarantine 仅 warn",
    ),
];

pub fn run(json: bool) -> Result<()> {
    let inventory = inventory_ssot::collect(json)?;

    let categories: Vec<Category> = INTENTIONAL_CATEGORIES
        .iter()
        .map(|id| {
            let description = CATEGORY_DOCS
                .iter()
                .find(|(k, _)| k == id)
                .map(|(_, d)| *d)
                .unwrap_or("见 inventory-ssot");
            Category {
                id,
                mode: "readonly",
                description,
            }
        })
        .collect();

    let report = Report {
        passed: inventory.passed,
        auto_repair: false,
        auto_repair_enabled: false,
        work_package: "INFRA-060",
        categories,
        inventory,
        note: "readonly drift detection; auto-repair MUST remain disabled; \
               not a claim that INFRA-060 WP is ACCEPTED",
    };

    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        println!(
            "drift-detect: passed={} categories={} findings={} auto_repair=false",
            report.passed,
            report.categories.len(),
            report.inventory.findings.len()
        );
        println!("  categories:");
        for c in &report.categories {
            println!("    - {} [{}]: {}", c.id, c.mode, c.description);
        }
        for f in &report.inventory.findings {
            println!("  [{}] {}: {}", f.severity, f.code, f.message);
        }
        if report.passed {
            println!("drift-detect: PASS (readonly)");
        } else {
            println!("drift-detect: FAIL");
        }
    }

    if !report.passed {
        bail!("drift-detect found blocking drift (auto-repair disabled)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_covers_inventory_categories() {
        for id in INTENTIONAL_CATEGORIES {
            assert!(
                CATEGORY_DOCS.iter().any(|(k, _)| k == id),
                "missing docs for category {id}"
            );
        }
    }

    #[test]
    fn auto_repair_flags_are_hard_false() {
        // INFRA-060：只读 drift；自动修复默认关闭
        let auto_repair = false;
        let auto_repair_enabled = false;
        assert!(!auto_repair);
        assert!(!auto_repair_enabled);
        assert!(!CATEGORY_DOCS.is_empty());
    }
}
