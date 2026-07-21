//! lint-deps：校验 workspace 依赖图是否符合 spec §2 R1-R5，并通过 [`check_r6`] 对
//! Rust 源码做 `pub use` 静态扫描以覆盖 R6。`check_r6` 是基于逐行文本匹配的最小实现，
//! 已知局限（不处理 glob/别名导入、`pub use foo::{A, B}` 多项导入、`pub(crate) use`、
//! 经中间模块转发的重导出）见 ADR-009。

use crate::allowed_matrix::allowed_targets;
use crate::classify::{classify_layer, Layer};
use anyhow::{bail, Context, Result};
use cargo_metadata::{DependencyKind, MetadataCommand};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// 单条违规。
#[derive(serde::Serialize)]
struct Violation {
    rule: &'static str,
    message: String,
}

/// package 身份归一：去 `xhyper-` 前缀并把 kebab 规范为 underscore，便于规则表匹配。
fn pkg_canon(name: &str) -> String {
    name.strip_prefix("xhyper-")
        .unwrap_or(name)
        .replace('-', "_")
}

pub fn run(json: bool) -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let members: HashSet<&cargo_metadata::PackageId> = metadata.workspace_members.iter().collect();
    let by_id: HashMap<&cargo_metadata::PackageId, &cargo_metadata::Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();
    let by_name: HashMap<&str, &cargo_metadata::Package> = metadata
        .packages
        .iter()
        .map(|p| (p.name.as_str(), p))
        .collect();

    let mut violations = Vec::new();
    violations.extend(check_deps(&metadata, &members, &by_id));
    violations.extend(check_r6(&members, &by_id, &by_name)?);

    report(&violations, json)
}

/// R1–R5 + ADR-007：Cargo.toml 级别的依赖方向校验。
fn check_deps(
    metadata: &cargo_metadata::Metadata,
    members: &HashSet<&cargo_metadata::PackageId>,
    by_id: &HashMap<&cargo_metadata::PackageId, &cargo_metadata::Package>,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let domain_peers = ["domain_market", "domain_macro", "domain_exchange"];

    for id in members {
        let Some(pkg) = by_id.get(id) else { continue };
        let from = classify_layer(pkg.manifest_path.as_str());
        let from_canon = pkg_canon(pkg.name.as_str());
        for dep in &pkg.dependencies {
            // path/package 依赖：优先按 package 名匹配 workspace member
            let Some(dep_pkg) = metadata.packages.iter().find(|p| {
                members.contains(&p.id)
                    && (p.name == dep.name
                        || dep.rename.as_deref() == Some(p.name.as_str())
                        || pkg_canon(p.name.as_str()) == pkg_canon(dep.name.as_str()))
            }) else {
                continue;
            };
            let to = classify_layer(dep_pkg.manifest_path.as_str());
            let to_canon = pkg_canon(dep_pkg.name.as_str());

            // R1: 测试设施仅允许 dev-dependency
            if matches!(
                to_canon.as_str(),
                "testkit" | "testkitx" | "contract_testkit"
            ) && dep.kind == DependencyKind::Normal
                && !matches!(
                    from,
                    Layer::XTask | Layer::Kernel | Layer::Infra | Layer::TestSupport
                )
            {
                violations.push(Violation {
                    rule: "R1",
                    message: format!(
                        "{} 以 normal 依赖引用测试设施 {}（仅允许 dev-dependency）",
                        pkg.name, dep_pkg.name
                    ),
                });
            }

            if dep.kind != DependencyKind::Normal {
                continue;
            }

            // R1/R2/R4: 跨层允许矩阵
            if !allowed_targets(from).contains(&to) {
                violations.push(Violation {
                    rule: "R1/R2/R4",
                    message: format!(
                        "{} ({:?}) -> {} ({:?}) 不在允许的跨层依赖矩阵内",
                        pkg.name, from, dep_pkg.name, to
                    ),
                });
                continue;
            }

            // R3: L1 互依禁止（bootstrap 豁免，ADR-005）
            if from == Layer::Infra
                && to == Layer::Infra
                && pkg.name != dep_pkg.name
                && from_canon != "bootstrap"
            {
                violations.push(Violation {
                    rule: "R3",
                    message: format!(
                        "L1 模块 {} 禁止直接依赖同层 {}，应通过 contracts trait 解耦",
                        pkg.name, dep_pkg.name
                    ),
                });
            }

            // R2/R2.1: 适配器同层互依禁止
            if (from == Layer::Storage || from == Layer::Exchange)
                && to == from
                && pkg.name != dep_pkg.name
            {
                violations.push(Violation {
                    rule: "R2/R2.1",
                    message: format!(
                        "适配器 {} 禁止直接依赖同层适配器 {}",
                        pkg.name, dep_pkg.name
                    ),
                });
            }

            // ADR-007: decimalx 不依赖 canonical
            if from_canon == "decimalx" && to_canon == "canonical" {
                violations.push(Violation {
                    rule: "ADR-007",
                    message: "decimalx 是基础数值层，禁止依赖 canonical".into(),
                });
            }

            // R5: domain 三平级互斥
            if from == Layer::Domain
                && to == Layer::Domain
                && domain_peers.contains(&from_canon.as_str())
                && domain_peers.contains(&to_canon.as_str())
            {
                violations.push(Violation {
                    rule: "R5",
                    message: format!(
                        "{} 禁止依赖 {}（三领域模块互相独立）",
                        pkg.name, dep_pkg.name
                    ),
                });
            }

            // R1.2: domainx 单向链
            if from_canon == "domainx" && domain_peers.contains(&to_canon.as_str()) {
                violations.push(Violation {
                    rule: "R1.2",
                    message: format!("domainx 禁止反向依赖 {}", dep_pkg.name),
                });
            }
        }
    }
    violations
}

/// R6：禁止跨层 `pub use` L1/适配器具体实现类型（/types/ 豁免）。
///
/// **已知局限（ADR-009）**：不追踪"先在本 crate 内部 `use`，再从本 crate
/// 重导出"这种间接转发。这一遗漏会造成假阴性（漏报违规），不会造成假阳性；本函数的
/// 通过结果仍不能解释为"完整证明 R6 合规"。
///
/// xhyper-4do / xhyper-gmd agent-safe subset：不可读目录 / 文件一律 fail-closed
/// （原实现 `if let Ok(...)` / `let Ok(...) else continue` 静默跳过，是假阴性源）。
fn check_r6(
    members: &HashSet<&cargo_metadata::PackageId>,
    by_id: &HashMap<&cargo_metadata::PackageId, &cargo_metadata::Package>,
    by_name: &HashMap<&str, &cargo_metadata::Package>,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for id in members {
        let Some(pkg) = by_id.get(id) else { continue };
        let from = classify_layer(pkg.manifest_path.as_str());
        // /types/ 层豁免（纯数据类型）；Legacy 层豁免（ADR-008 过渡期）
        if matches!(from, Layer::Types | Layer::Legacy) {
            continue;
        }
        let src_dir = pkg.manifest_path.parent().unwrap().join("src");
        if !src_dir.exists() {
            continue;
        }
        let mut rs_files = Vec::new();
        collect_rs_files(src_dir.as_std_path(), &mut rs_files)
            .with_context(|| format!("R6 收集 {src_dir:?} 下的 .rs 文件"))?;
        for file_path in rs_files {
            let content =
                fs::read_to_string(&file_path).with_context(|| format!("R6 读取 {file_path:?}"))?;
            // Collect complete statements so a grouped or multi-line re-export is
            // checked rather than silently skipped by a line-based scan.
            let mut statement = String::new();
            for line in content.lines() {
                let line = line.split("//").next().unwrap_or_default().trim();
                if line.is_empty() {
                    continue;
                }
                statement.push(' ');
                statement.push_str(line);
                if !line.contains(';') {
                    continue;
                }
                let candidate = std::mem::take(&mut statement);
                let candidate = candidate.trim();
                let rest = candidate
                    .strip_prefix("pub use ")
                    .or_else(|| candidate.strip_prefix("pub(crate) use "));
                let Some(rest) = rest else {
                    continue;
                };
                let Some(colon_pos) = rest.find("::") else {
                    continue;
                };
                let crate_name = rest[..colon_pos].trim();
                if matches!(crate_name, "crate" | "self" | "super") {
                    continue;
                }
                if let Some(dep_pkg) = by_name.get(crate_name) {
                    let to = classify_layer(dep_pkg.manifest_path.as_str());
                    if matches!(to, Layer::Infra | Layer::Storage | Layer::Exchange) && to != from {
                        violations.push(Violation {
                            rule: "R6",
                            message: format!(
                                "{} 跨层 pub use {} ({:?}) 的具体实现类型，应面向 contracts trait",
                                pkg.name, crate_name, to
                            ),
                        });
                    }
                }
            }
        }
    }
    Ok(violations)
}

/// 输出校验结果，有违规时返回 Err。
fn report(violations: &[Violation], json: bool) -> Result<()> {
    if !violations.is_empty() {
        if json {
            let output = serde_json::json!({
                "passed": false,
                "violations": violations,
            });
            println!("{output}");
        } else {
            eprintln!("依赖规则校验失败，共 {} 处违规：", violations.len());
            for v in violations {
                eprintln!("  - [{}] {}", v.rule, v.message);
            }
        }
        bail!("lint-deps failed");
    }
    if json {
        println!(r#"{{"passed":true}}"#);
    } else {
        println!("依赖图校验通过，符合 spec §2 R1–R6");
    }
    Ok(())
}

/// 递归收集目录下所有 .rs 文件。
///
/// xhyper-gmd agent-safe subset：原实现 `if let Ok(entries) = fs::read_dir(dir)`
/// 在目录不可读时**静默跳过**，是 R6 假阴性源。现在改为 fail-closed：返回 Err。
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("read_dir {dir:?}"))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("readdir entry in {dir:?}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}
