//! `test-graph-check`：测试支持平面生产图隔离（SPEC-TESTKIT-002 §14）。
//!
//! - TESTKIT-GRAPH-001：`testkit` 只能作为 dev-dependency  
//! - TESTKIT-GRAPH-002：`contract-testkit` 只能作为 dev-dependency  
//! - TESTKIT-GRAPH-003：test-support 包不得作为 build-dependency  
//! - TESTKIT-GRAPH-004：apps/services 的 normal resolve 图不得包含 test-support  
//! - TESTKIT-GRAPH-005：默认 feature 解析路径下仍不得泄漏 test-support  
//!
//! 输出列：test_support_package · consumer · dependency_kind · target · feature_path · verdict

use crate::classify::{classify_layer, Layer};
use anyhow::{bail, Result};
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

const TEST_SUPPORT_PACKAGES: &[&str] = &[
    "testkit",
    "testkitx",
    "contract-testkit",
    "contract_testkit",
    "xhyper-testkit",
    "xhyper-contract-testkit",
];

#[derive(Debug, Clone, Serialize)]
struct Row {
    test_support_package: String,
    consumer: String,
    dependency_kind: String,
    target: String,
    feature_path: String,
    verdict: &'static str,
    rule: &'static str,
    message: String,
}

#[derive(Serialize)]
struct Report {
    ok: bool,
    rows: Vec<Row>,
}

pub fn run(json: bool) -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let members: HashSet<&PackageId> = metadata.workspace_members.iter().collect();
    let by_id: HashMap<&PackageId, &Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();

    let mut rows = Vec::new();
    rows.extend(scan_declared_deps(&metadata, &members, &by_id));
    rows.extend(scan_resolve_graph(&metadata, &members, &by_id));

    let fail_count = rows.iter().filter(|r| r.verdict == "FAIL").count();
    let ok = fail_count == 0;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&Report {
                ok,
                rows: rows.clone(),
            })?
        );
    } else {
        println!("test-graph-check: SPEC-TESTKIT-002 §14");
        println!(
            "{:<22} {:<28} {:<10} {:<12} {:<14} {:<6} rule",
            "test_support", "consumer", "kind", "target", "feature_path", "verdict"
        );
        for r in &rows {
            println!(
                "{:<22} {:<28} {:<10} {:<12} {:<14} {:<6} {} — {}",
                trunc(&r.test_support_package, 22),
                trunc(&r.consumer, 28),
                trunc(&r.dependency_kind, 10),
                trunc(&r.target, 12),
                trunc(&r.feature_path, 14),
                r.verdict,
                r.rule,
                r.message
            );
        }
        if ok {
            println!("test-graph-check: PASS ({} rows, 0 FAIL)", rows.len());
        } else {
            println!(
                "test-graph-check: FAIL ({fail_count} FAIL / {} rows)",
                rows.len()
            );
        }
    }

    if !ok {
        bail!("test-graph-check found production graph pollution by test-support packages");
    }
    Ok(())
}

fn is_test_support_name(name: &str) -> bool {
    if TEST_SUPPORT_PACKAGES.contains(&name) {
        return true;
    }
    let short = name.strip_prefix("xhyper-").unwrap_or(name);
    matches!(
        short.replace('-', "_").as_str(),
        "testkit" | "testkitx" | "contract_testkit"
    )
}

fn kind_str(k: DependencyKind) -> &'static str {
    match k {
        DependencyKind::Normal => "normal",
        DependencyKind::Development => "dev",
        DependencyKind::Build => "build",
        _ => "unknown",
    }
}

fn scan_declared_deps(
    metadata: &Metadata,
    members: &HashSet<&PackageId>,
    by_id: &HashMap<&PackageId, &Package>,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for id in members {
        let Some(pkg) = by_id.get(id) else { continue };
        let from_layer = classify_layer(pkg.manifest_path.as_str());
        for dep in &pkg.dependencies {
            let Some(dep_pkg) = metadata
                .packages
                .iter()
                .find(|p| p.name.as_str() == dep.name.as_str() && members.contains(&p.id))
            else {
                continue;
            };
            if !is_test_support_name(dep_pkg.name.as_str()) {
                continue;
            }

            let kind = dep.kind;
            let target = dep
                .target
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "all".into());
            let features = if dep.features.is_empty() {
                "default".into()
            } else {
                dep.features.join(",")
            };

            let (verdict, rule, message) = match kind {
                DependencyKind::Development => (
                    "PASS",
                    "TESTKIT-GRAPH-001/002",
                    format!("{} 以 dev-dependency 使用 {}", pkg.name, dep_pkg.name),
                ),
                DependencyKind::Build => (
                    "FAIL",
                    "TESTKIT-GRAPH-003",
                    format!(
                        "{} 以 build-dependency 引用 test-support 包 {}",
                        pkg.name, dep_pkg.name
                    ),
                ),
                DependencyKind::Normal => {
                    if matches!(from_layer, Layer::TestSupport | Layer::XTask) {
                        (
                            "PASS",
                            "TESTKIT-GRAPH-001/002",
                            format!(
                                "test-support/tool 内部 normal 引用 {}（非生产污染）",
                                dep_pkg.name
                            ),
                        )
                    } else {
                        (
                            "FAIL",
                            if dep_pkg.name == "contract-testkit" {
                                "TESTKIT-GRAPH-002"
                            } else {
                                "TESTKIT-GRAPH-001"
                            },
                            format!(
                                "{} 以 normal 依赖引用测试设施 {}（仅允许 dev-dependency）",
                                pkg.name, dep_pkg.name
                            ),
                        )
                    }
                }
                _ => (
                    "PASS",
                    "TESTKIT-GRAPH-001",
                    format!("未分类 dependency kind for {}", dep_pkg.name),
                ),
            };

            rows.push(Row {
                test_support_package: dep_pkg.name.to_string(),
                consumer: pkg.name.to_string(),
                dependency_kind: kind_str(kind).into(),
                target,
                feature_path: features,
                verdict,
                rule,
                message,
            });
        }
    }
    rows
}

fn scan_resolve_graph(
    metadata: &Metadata,
    members: &HashSet<&PackageId>,
    by_id: &HashMap<&PackageId, &Package>,
) -> Vec<Row> {
    let mut rows = Vec::new();
    let Some(resolve) = metadata.resolve.as_ref() else {
        rows.push(Row {
            test_support_package: "*".into(),
            consumer: "*".into(),
            dependency_kind: "resolve".into(),
            target: "all".into(),
            feature_path: "default".into(),
            verdict: "FAIL",
            rule: "TESTKIT-GRAPH-005",
            message: "cargo metadata 无 resolve 图，无法验证 feature 路径隔离".into(),
        });
        return rows;
    };

    let mut name_by_id: HashMap<&PackageId, &str> = HashMap::new();
    for (id, pkg) in by_id {
        if members.contains(id) {
            name_by_id.insert(id, pkg.name.as_str());
        }
    }

    let mut edges: HashMap<&PackageId, Vec<&PackageId>> = HashMap::new();
    for node in &resolve.nodes {
        let deps: Vec<&PackageId> = node.deps.iter().map(|d| &d.pkg).collect();
        edges.insert(&node.id, deps);
    }

    let ts_ids: HashSet<&PackageId> = by_id
        .iter()
        .filter(|(id, p)| members.contains(*id) && is_test_support_name(p.name.as_str()))
        .map(|(id, _)| *id)
        .collect();

    for id in members {
        let Some(pkg) = by_id.get(id) else { continue };
        let layer = classify_layer(pkg.manifest_path.as_str());
        if !matches!(layer, Layer::Apps | Layer::Services) {
            continue;
        }

        let mut polluted = false;
        let mut seen = HashSet::new();
        let mut stack = vec![*id];
        seen.insert(*id);
        while let Some(cur) = stack.pop() {
            if cur != *id && ts_ids.contains(cur) {
                polluted = true;
                let ts_name = name_by_id.get(cur).copied().unwrap_or("?");
                rows.push(Row {
                    test_support_package: ts_name.into(),
                    consumer: pkg.name.to_string(),
                    dependency_kind: "resolve".into(),
                    target: "normal".into(),
                    feature_path: "default+".into(),
                    verdict: "FAIL",
                    rule: "TESTKIT-GRAPH-004/005",
                    message: format!(
                        "apps/services 包 {} 的 normal resolve 图包含 test-support {}",
                        pkg.name, ts_name
                    ),
                });
            }
            if let Some(deps) = edges.get(cur) {
                for d in deps {
                    if seen.insert(*d) {
                        stack.push(*d);
                    }
                }
            }
        }

        if !polluted {
            rows.push(Row {
                test_support_package: "(none)".into(),
                consumer: pkg.name.to_string(),
                dependency_kind: "resolve".into(),
                target: "normal".into(),
                feature_path: "default+".into(),
                verdict: "PASS",
                rule: "TESTKIT-GRAPH-004/005",
                message: format!(
                    "{} normal resolve 图未包含 testkit/contract-testkit",
                    pkg.name
                ),
            });
        }
    }
    rows
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(n.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_test_support_names() {
        assert!(is_test_support_name("testkit"));
        assert!(is_test_support_name("contract-testkit"));
        assert!(!is_test_support_name("kernel"));
    }
}
