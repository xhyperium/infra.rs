//! `.architecture/*.toml` 真 TOML 反序列化（与 archgate `registry` 同 schema 语义）。
//!
//! 供 `migration` / `inventory-ssot` 消费，避免行扫描解析与字段顺序脆弱性。

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::Path,
};

pub const SUPPORTED_WORKSPACE_SCHEMA: u32 = 1;
pub const SUPPORTED_MIGRATION_SCHEMA: u32 = 1;

// ---------------------------------------------------------------------------
// workspace.toml
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceToml {
    schema_version: u32,
    #[serde(default)]
    defaults: WorkspaceDefaults,
    #[serde(default)]
    unit: Vec<UnitToml>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceDefaults {
    status: Option<String>,
    publish: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitToml {
    path: String,
    #[serde(default)]
    layer: Option<String>,
    /// 可选显式 id；缺省用 `path:{path}`。
    id: Option<String>,
    status: Option<String>,
    publish: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceUnit {
    pub path: String,
    /// 供 archgate 对齐；inventory 仅强制 path。
    #[allow(dead_code)]
    pub layer: Option<String>,
    pub id: String,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub publish: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRegistry {
    #[allow(dead_code)]
    pub schema_version: u32,
    pub units: Vec<WorkspaceUnit>,
}

impl WorkspaceRegistry {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".architecture/workspace.toml");
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let raw: WorkspaceToml = toml::from_str(text).context("workspace.toml TOML deserialize")?;
        if raw.schema_version != SUPPORTED_WORKSPACE_SCHEMA {
            bail!(
                "unsupported workspace.toml schema_version={} (supported={SUPPORTED_WORKSPACE_SCHEMA})",
                raw.schema_version
            );
        }
        let default_status = raw
            .defaults
            .status
            .as_deref()
            .unwrap_or("incubating")
            .to_owned();
        let default_publish = raw.defaults.publish.unwrap_or(false);

        let mut units = Vec::with_capacity(raw.unit.len());
        let mut seen_paths = HashSet::new();
        let mut seen_ids = HashSet::new();
        for u in raw.unit {
            if u.path.is_empty() {
                bail!("workspace unit has empty path");
            }
            if !seen_paths.insert(u.path.clone()) {
                bail!("duplicate workspace unit path: {}", u.path);
            }
            let id = match u.id {
                None => format!("path:{}", u.path),
                Some(ref s) if s.is_empty() => {
                    bail!("workspace unit {} has empty explicit id", u.path);
                }
                Some(s) => s,
            };
            if !seen_ids.insert(id.clone()) {
                bail!("duplicate workspace unit id: {id}");
            }
            units.push(WorkspaceUnit {
                path: u.path,
                layer: u.layer.filter(|s| !s.is_empty()),
                id,
                status: u.status.unwrap_or_else(|| default_status.clone()),
                publish: u.publish.unwrap_or(default_publish),
            });
        }
        Ok(Self {
            schema_version: raw.schema_version,
            units,
        })
    }

    pub fn path_set(&self) -> BTreeSet<String> {
        self.units.iter().map(|u| u.path.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// migration.toml
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MigrationToml {
    schema_version: u32,
    #[serde(default)]
    migration: Vec<MigrationEntryToml>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MigrationEntryToml {
    from: String,
    to: String,
    phase: String,
    status: String,
}

#[derive(Debug, Clone)]
pub struct MigrationEntry {
    pub from: String,
    pub to: String,
    pub phase: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MigrationRegistry {
    pub schema_version: u32,
    pub entries: Vec<MigrationEntry>,
}

impl MigrationRegistry {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".architecture/migration.toml");
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let raw: MigrationToml = toml::from_str(text).context("migration.toml TOML deserialize")?;
        if raw.schema_version != SUPPORTED_MIGRATION_SCHEMA {
            bail!(
                "unsupported migration.toml schema_version={} (supported={SUPPORTED_MIGRATION_SCHEMA})",
                raw.schema_version
            );
        }
        let mut entries = Vec::with_capacity(raw.migration.len());
        let mut seen = HashSet::new();
        for e in raw.migration {
            if e.from.is_empty() || e.to.is_empty() {
                bail!("migration entry has empty from/to");
            }
            if !seen.insert(e.from.clone()) {
                bail!("duplicate migration from path: {}", e.from);
            }
            entries.push(MigrationEntry {
                from: e.from,
                to: e.to,
                phase: e.phase,
                status: e.status,
            });
        }
        Ok(Self {
            schema_version: raw.schema_version,
            entries,
        })
    }

    pub fn source_paths(&self) -> BTreeSet<String> {
        self.entries.iter().map(|e| e.from.clone()).collect()
    }
}

// ---------------------------------------------------------------------------
// policies/dependency.toml（inventory 严格度检查用；与 archgate schema 对齐）
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyToml {
    schema_version: u32,
    #[serde(default = "default_cycles")]
    cycles: String,
    #[serde(default)]
    layers: Vec<LayerToml>,
    #[serde(default)]
    forbidden: Vec<ForbiddenToml>,
}

fn default_cycles() -> String {
    "forbidden".into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerToml {
    name: String,
    #[serde(default)]
    may_depend_on: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForbiddenToml {
    from: String,
    to: String,
    #[serde(default)]
    reason: String,
}

/// layer name → may_depend_on 集合。
pub fn load_dependency_allowances(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let path = root.join(".architecture/policies/dependency.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    parse_dependency_allowances(&text)
}

pub fn parse_dependency_allowances(text: &str) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let raw: DependencyToml = toml::from_str(text).context("dependency.toml TOML deserialize")?;
    if raw.schema_version != SUPPORTED_WORKSPACE_SCHEMA {
        bail!(
            "unsupported dependency.toml schema_version={} (supported={SUPPORTED_WORKSPACE_SCHEMA})",
            raw.schema_version
        );
    }
    if raw.cycles != "forbidden" {
        bail!(
            "dependency.toml cycles={:?} unsupported; only \"forbidden\"",
            raw.cycles
        );
    }
    let mut out = BTreeMap::new();
    for layer in raw.layers {
        if layer.name.is_empty() {
            bail!("dependency layer has empty name");
        }
        if out.contains_key(&layer.name) {
            bail!("duplicate dependency layer name: {}", layer.name);
        }
        out.insert(
            layer.name,
            layer.may_depend_on.into_iter().collect::<BTreeSet<_>>(),
        );
    }
    // 消费 forbidden 字段避免 inert：至少校验 from/to 非空（裁决仍由 archgate/lint-deps）
    for f in &raw.forbidden {
        if f.from.is_empty() || f.to.is_empty() {
            bail!("forbidden edge has empty from/to");
        }
        let _ = &f.reason;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_defaults_and_duplicate_detection() {
        let text = r#"
schema_version = 1
[defaults]
status = "incubating"
publish = false
[[unit]]
path = "crates/kernel"
layer = "kernel"
status = "stable"
publish = true
[[unit]]
path = "crates/types/decimal"
layer = "types"
"#;
        let reg = WorkspaceRegistry::parse(text).unwrap();
        assert_eq!(reg.units.len(), 2);
        assert!(reg.units[0].publish);
        assert!(!reg.units[1].publish);
        assert_eq!(reg.units[1].status, "incubating");
    }

    #[test]
    fn workspace_rejects_empty_explicit_id() {
        let text = r#"
schema_version = 1
[[unit]]
path = "crates/kernel"
layer = "kernel"
id = ""
"#;
        assert!(WorkspaceRegistry::parse(text).is_err());
    }

    #[test]
    fn migration_parses_schema_and_entries() {
        let text = r#"
schema_version = 1
[[migration]]
from = "crates/kernel"
to = "crates/kernel"
phase = "P2"
status = "completed"
"#;
        let reg = MigrationRegistry::parse(text).unwrap();
        assert_eq!(reg.entries.len(), 1);
        assert_eq!(reg.entries[0].phase, "P2");
    }

    #[test]
    fn dependency_multiline_may_depend_on() {
        let text = r#"
schema_version = 1
cycles = "forbidden"
[[layers]]
name = "infra"
may_depend_on = [
  "kernel",
  "types",
  "contracts",
]
"#;
        let map = parse_dependency_allowances(text).unwrap();
        let infra = map.get("infra").unwrap();
        assert!(infra.contains("kernel"));
        assert!(!infra.contains("infra"));
    }

    #[test]
    fn dependency_rejects_unknown_fields() {
        let missing_schema = r#"
cycles = "forbidden"
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
"#;
        assert!(parse_dependency_allowances(missing_schema).is_err());

        let text = r#"
schema_version = 1
cycles = "forbidden"
typo = 1
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
"#;
        assert!(parse_dependency_allowances(text).is_err());
    }
}
