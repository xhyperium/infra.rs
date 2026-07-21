//! Validation for the frozen P2 physical-migration registry.

use crate::architecture_toml::{MigrationRegistry, WorkspaceRegistry};
use anyhow::{bail, Context, Result};
use std::path::{Component, Path};

const VALID_PHASES: &[&str] = &["P2"];
const VALID_STATUSES: &[&str] = &["completed", "quarantined"];

pub fn run(check: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let workspace = WorkspaceRegistry::load(&root).context("load .architecture/workspace.toml")?;
    let declared = workspace.path_set();
    let migration = MigrationRegistry::load(&root).context("load .architecture/migration.toml")?;
    let seen = migration.source_paths();

    for entry in &migration.entries {
        validate_entry(&root, entry)?;
    }

    // F-10：migration 为冻结历史 Evidence，不再要求每个 workspace member 镜像一条 from==to。
    // 仍拒绝 stale：registry 中 from 不在当前 workspace 登记 → fail（防止幽灵路径）。
    // missing（workspace 有、migration 无）允许，表示 post-freeze 新成员。
    let _declared = declared; // 保留 load 校验；missing 不再阻断
    let stale: Vec<_> = seen.difference(&_declared).cloned().collect();
    if !stale.is_empty() {
        bail!("migration --check: stale registry paths not in workspace: {stale:?}");
    }
    if check {
        println!(
            "migration --check: PASS ({} historical entries, schema_version={}, frozen; new workspace members need not mirror)",
            migration.entries.len(),
            migration.schema_version
        );
    }
    Ok(())
}

fn validate_entry(root: &Path, entry: &crate::architecture_toml::MigrationEntry) -> Result<()> {
    if entry.from.is_empty() || entry.to.is_empty() {
        bail!("migration --check: from/to must not be empty");
    }
    if !VALID_PHASES.contains(&entry.phase.as_str()) {
        bail!(
            "migration --check: invalid phase {:?} for {} (allowed: {:?})",
            entry.phase,
            entry.from,
            VALID_PHASES
        );
    }
    if !VALID_STATUSES.contains(&entry.status.as_str()) {
        bail!(
            "migration --check: invalid status {:?} for {} (allowed: {:?})",
            entry.status,
            entry.from,
            VALID_STATUSES
        );
    }
    let target = Path::new(&entry.to);
    if target.is_absolute()
        || target
            .components()
            .any(|component| component == Component::ParentDir)
    {
        bail!(
            "migration --check: target path must stay inside the repository: {}",
            entry.to
        );
    }
    let target = root.join(target);
    if !target.is_dir() {
        bail!(
            "migration --check: target directory does not exist: {}",
            entry.to
        );
    }
    if !target.join("Cargo.toml").is_file() {
        bail!(
            "migration --check: target manifest does not exist: {}/Cargo.toml",
            entry.to
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architecture_toml::MigrationEntry;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "xhyper-xtask-migration-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before unix epoch")
                .as_nanos()
        ))
    }

    #[test]
    fn validate_entry_requires_target_manifest_and_known_state() {
        let root = temp_root();
        fs::create_dir_all(root.join("crates/example")).expect("create target");
        fs::write(root.join("crates/example/Cargo.toml"), "[package]\n").expect("write manifest");
        let entry = MigrationEntry {
            from: "old/example".into(),
            to: "crates/example".into(),
            phase: "P2".into(),
            status: "completed".into(),
        };
        validate_entry(&root, &entry).expect("valid target");

        let mut bad = entry.clone();
        bad.status = "done".into();
        assert!(validate_entry(&root, &bad).is_err());
        bad.status = "completed".into();
        bad.to = "crates/missing".into();
        assert!(validate_entry(&root, &bad).is_err());
        fs::remove_dir_all(root).expect("remove temp root");
    }
}
