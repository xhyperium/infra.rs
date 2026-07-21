//! Affected package graph（PHASE-1-03）— 基于 cargo metadata 的 reverse dependents。
//! Shadow only：解析失败 → Full 安全回退（HC-03）。

use anyhow::Result;
use cargo_metadata::{Metadata, MetadataCommand};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct AffectedGraphReport {
    pub ok: bool,
    pub mode: &'static str,
    pub full_fallback: bool,
    pub fallback_reason: Option<String>,
    pub seeds: Vec<String>,
    pub affected_packages: Vec<String>,
    pub reverse_edges_sample: BTreeMap<String, Vec<String>>,
    pub note: String,
}

/// 从变更路径推导受影响 package（含 reverse dependents）。
pub fn affected_from_paths(root: &Path, paths: &[String]) -> Result<AffectedGraphReport> {
    if paths.is_empty() {
        return Ok(AffectedGraphReport {
            ok: true,
            mode: "shadow",
            full_fallback: true,
            fallback_reason: Some("empty_paths".into()),
            seeds: vec![],
            affected_packages: vec!["*".into()],
            reverse_edges_sample: BTreeMap::new(),
            note: "no paths → Full fallback (safe over-run)".into(),
        });
    }

    let meta = match MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
    {
        Ok(m) => m,
        Err(e) => {
            return Ok(AffectedGraphReport {
                ok: true,
                mode: "shadow",
                full_fallback: true,
                fallback_reason: Some(format!("cargo_metadata_failed: {e}")),
                seeds: vec![],
                affected_packages: vec!["*".into()],
                reverse_edges_sample: BTreeMap::new(),
                note: "metadata failure → Full fallback".into(),
            });
        }
    };

    // 需要完整 resolve 图做 reverse deps
    let meta_full = match MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
    {
        Ok(m) => m,
        Err(_) => meta,
    };

    let seeds = seed_packages(&meta_full, root, paths);
    if seeds.is_empty() {
        // 无法映射到 package（如纯 docs）— 不强制 full packages，交由 planner class
        return Ok(AffectedGraphReport {
            ok: true,
            mode: "shadow",
            full_fallback: false,
            fallback_reason: None,
            seeds: vec![],
            affected_packages: vec![],
            reverse_edges_sample: BTreeMap::new(),
            note: "paths map to zero workspace packages (e.g. docs-only)".into(),
        });
    }

    let rev = reverse_dep_map(&meta_full);
    let mut affected: BTreeSet<String> = BTreeSet::new();
    let mut q: VecDeque<String> = VecDeque::new();
    for s in &seeds {
        if affected.insert(s.clone()) {
            q.push_back(s.clone());
        }
    }
    while let Some(cur) = q.pop_front() {
        if let Some(deps) = rev.get(&cur) {
            for d in deps {
                if affected.insert(d.clone()) {
                    q.push_back(d.clone());
                }
            }
        }
    }

    // sample reverse edges for seeds
    let mut sample = BTreeMap::new();
    for s in seeds.iter().take(8) {
        if let Some(v) = rev.get(s) {
            sample.insert(s.clone(), v.iter().take(12).cloned().collect());
        }
    }

    Ok(AffectedGraphReport {
        ok: true,
        mode: "shadow",
        full_fallback: false,
        fallback_reason: None,
        seeds,
        affected_packages: affected.into_iter().collect(),
        reverse_edges_sample: sample,
        note: "affected packages include reverse dependents via cargo metadata".into(),
    })
}

fn seed_packages(meta: &Metadata, root: &Path, paths: &[String]) -> Vec<String> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut seeds = BTreeSet::new();
    for raw in paths {
        let p = raw.replace('\\', "/");
        // workspace root manifests force full-ish seeds: all members
        if p == "Cargo.toml" || p == "Cargo.lock" {
            for pkg in meta.workspace_packages() {
                seeds.insert(pkg.name.to_string());
            }
            continue;
        }
        let abs = if Path::new(&p).is_absolute() {
            PathBuf::from(&p)
        } else {
            root.join(&p)
        };
        // find package whose manifest dir is longest prefix of path
        let mut best: Option<(usize, String)> = None;
        for pkg in meta.workspace_packages() {
            let manifest = PathBuf::from(&pkg.manifest_path);
            let dir = manifest.parent().unwrap_or(Path::new("."));
            let dir_c = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
            if abs.starts_with(&dir_c)
                || abs.to_string_lossy().contains(&pkg.name.replace('-', "_"))
            {
                // also match crates/<name>/ path segment
                let name = pkg.name.to_string();
                let score = dir_c.components().count();
                if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                    best = Some((score, name));
                }
            }
            // path contains package dir name
            let dir_s = dir.to_string_lossy().replace('\\', "/");
            if p.contains(&format!("/{}/", pkg.name))
                || p.starts_with(&format!("{}/", pkg.name))
                || p.contains(&pkg.name.replace("xhyper-", ""))
            {
                // weaker match via path string
                let name = pkg.name.to_string();
                let score = 1;
                if best.as_ref().map(|(s, _)| score >= *s).unwrap_or(true) && best.is_none() {
                    best = Some((score, name));
                }
            }
            let _ = dir_s;
        }
        // path-based heuristics for crates/
        if let Some(rest) = p.strip_prefix("crates/") {
            let seg = rest.split('/').next().unwrap_or("");
            for pkg in meta.workspace_packages() {
                let m = pkg.manifest_path.to_string().replace('\\', "/");
                if m.contains(&format!("/crates/{seg}/")) || m.contains(&format!("/crates/{seg}/"))
                {
                    seeds.insert(pkg.name.to_string());
                }
            }
        }
        if let Some(rest) = p.strip_prefix("tools/") {
            let seg = rest.split('/').next().unwrap_or("");
            for pkg in meta.workspace_packages() {
                let m = pkg.manifest_path.to_string().replace('\\', "/");
                if m.contains(&format!("/tools/{seg}/")) {
                    seeds.insert(pkg.name.to_string());
                }
            }
        }
        if let Some(rest) = p.strip_prefix("apps/") {
            let seg = rest.split('/').next().unwrap_or("");
            for pkg in meta.workspace_packages() {
                let m = pkg.manifest_path.to_string().replace('\\', "/");
                if m.contains(&format!("/apps/{seg}/")) {
                    seeds.insert(pkg.name.to_string());
                }
            }
        }
        if let Some((_, name)) = best {
            seeds.insert(name);
        }
    }
    seeds.into_iter().collect()
}

/// package name → packages that depend on it (workspace members only)
fn reverse_dep_map(meta: &Metadata) -> BTreeMap<String, Vec<String>> {
    let ws: BTreeSet<String> = meta
        .workspace_packages()
        .into_iter()
        .map(|p| p.name.to_string())
        .collect();
    let mut rev: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for pkg in meta.workspace_packages() {
        let from = pkg.name.to_string();
        for dep in &pkg.dependencies {
            let to = dep.name.clone();
            if ws.contains(to.as_str()) {
                rev.entry(to).or_default().insert(from.clone());
            }
        }
    }
    rev.into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;

    #[test]
    fn empty_paths_full_fallback() {
        let root = repo_root_from_manifest();
        let r = affected_from_paths(&root, &[]).unwrap();
        assert!(r.full_fallback);
        assert_eq!(r.affected_packages, vec!["*"]);
    }

    #[test]
    fn docs_paths_zero_or_empty_packages() {
        let root = repo_root_from_manifest();
        let r = affected_from_paths(&root, &["docs/architecture/spec.md".into()]).unwrap();
        assert!(!r.full_fallback || r.affected_packages == vec!["*"]);
        // docs-only should not require * unless fallback
        if !r.full_fallback {
            assert!(r.affected_packages.is_empty() || r.seeds.is_empty());
        }
    }

    #[test]
    fn xtask_path_seeds_xhyper_xtask() {
        let root = repo_root_from_manifest();
        let r = affected_from_paths(&root, &["tools/xtask/src/ci/mod.rs".into()]).unwrap();
        assert!(
            r.seeds.iter().any(|s| s.contains("xtask"))
                || r.affected_packages.iter().any(|s| s.contains("xtask"))
                || r.full_fallback,
            "expected xtask seed or full fallback; got {:?}",
            r
        );
    }
}
