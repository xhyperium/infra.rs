//! `.architecture/workspace.toml` 与 `policies/dependency.toml` 的 TOML 反序列化。
//!
//! - 真 TOML 解析（字段顺序无关）
//! - `schema_version` 校验
//! - `[defaults]` 合并到 `[[unit]]` 缺省字段

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

/// 当前 archgate 支持的 registry schema。
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

const VALID_STATUS_TIERS: &[&str] = &[
    "experimental",
    "incubating",
    "stable",
    "deprecated",
    "legacy",
    "quarantined",
];

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
    layer: String,
    /// 缺省时回退 `[defaults].status`，再回退 `"incubating"`。
    status: Option<String>,
    /// 缺省时回退 `[defaults].publish`，再回退 `false`。
    publish: Option<bool>,
}

/// 合并 defaults 后的已解析 unit。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub path: String,
    pub layer: String,
    pub status: String,
    pub publish: bool,
}

/// 已加载并校验的 workspace registry。
#[derive(Debug, Clone)]
pub struct WorkspaceRegistry {
    pub schema_version: u32,
    pub units: Vec<Unit>,
}

impl WorkspaceRegistry {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".architecture/workspace.toml");
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let raw: WorkspaceToml = toml::from_str(text).context("workspace.toml TOML deserialize")?;
        if raw.schema_version != SUPPORTED_SCHEMA_VERSION {
            bail!(
                "unsupported workspace.toml schema_version={} (supported={SUPPORTED_SCHEMA_VERSION})",
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

        if !VALID_STATUS_TIERS.contains(&default_status.as_str()) {
            bail!("invalid defaults.status={default_status:?}; allowed={VALID_STATUS_TIERS:?}");
        }

        let mut units = Vec::with_capacity(raw.unit.len());
        let mut seen_paths = HashSet::new();
        for u in raw.unit {
            if u.path.is_empty() {
                bail!("workspace unit has empty path");
            }
            if u.layer.is_empty() {
                bail!("workspace unit {} has empty layer", u.path);
            }
            if !seen_paths.insert(u.path.clone()) {
                bail!("duplicate workspace unit path: {}", u.path);
            }
            let status = u.status.unwrap_or_else(|| default_status.clone());
            if !VALID_STATUS_TIERS.contains(&status.as_str()) {
                bail!(
                    "invalid status={status:?} for unit {}; allowed={VALID_STATUS_TIERS:?}",
                    u.path
                );
            }
            let publish = u.publish.unwrap_or(default_publish);
            units.push(Unit {
                path: u.path,
                layer: u.layer,
                status,
                publish,
            });
        }

        Ok(Self {
            schema_version: raw.schema_version,
            units,
        })
    }

    pub fn path_set(&self) -> HashSet<String> {
        self.units.iter().map(|u| u.path.clone()).collect()
    }

    pub fn layers(&self) -> HashMap<String, String> {
        self.units
            .iter()
            .map(|u| (u.path.clone(), u.layer.clone()))
            .collect()
    }

    pub fn statuses(&self) -> HashMap<String, String> {
        self.units
            .iter()
            .map(|u| (u.path.clone(), u.status.clone()))
            .collect()
    }

    pub fn publish_flags(&self) -> HashMap<String, bool> {
        self.units
            .iter()
            .map(|u| (u.path.clone(), u.publish))
            .collect()
    }

    /// 非法 status（解析阶段已校验 unit；此接口保留给 JSON 字段兼容，正常为空）。
    pub fn invalid_statuses(&self) -> Vec<String> {
        self.units
            .iter()
            .filter(|u| !VALID_STATUS_TIERS.contains(&u.status.as_str()))
            .map(|u| u.status.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// policies/dependency.toml
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyToml {
    /// 显式 schema；缺失或未知版本均 fail closed。
    schema_version: u32,
    /// 仅允许 "forbidden"；其它值 fail。
    #[serde(default = "default_cycles_policy")]
    cycles: String,
    #[serde(default)]
    layers: Vec<LayerToml>,
    #[serde(default)]
    forbidden: Vec<ForbiddenToml>,
}

fn default_cycles_policy() -> String {
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

/// 依赖策略：layer → 允许依赖的 layer 集合，以及 forbidden 边。
#[derive(Debug, Clone)]
pub struct DependencyPolicy {
    /// 解析期已校验；供诊断/JSON 扩展读取。
    #[allow(dead_code)]
    pub schema_version: u32,
    /// 解析期已强制为 `"forbidden"`。
    #[allow(dead_code)]
    pub cycles: String,
    pub allowances: HashMap<String, HashSet<String>>,
    pub forbidden: Vec<(String, String)>,
}

impl DependencyPolicy {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".architecture/policies/dependency.toml");
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let raw: DependencyToml =
            toml::from_str(text).context("dependency.toml TOML deserialize")?;
        if raw.schema_version != SUPPORTED_SCHEMA_VERSION {
            bail!(
                "unsupported dependency.toml schema_version={} (supported={SUPPORTED_SCHEMA_VERSION})",
                raw.schema_version
            );
        }
        if raw.cycles != "forbidden" {
            bail!(
                "dependency.toml cycles={:?} unsupported; only \"forbidden\" is enforced",
                raw.cycles
            );
        }
        let mut allowances = HashMap::new();
        for layer in raw.layers {
            if layer.name.is_empty() {
                bail!("dependency layer has empty name");
            }
            if allowances.contains_key(&layer.name) {
                bail!("duplicate dependency layer name: {}", layer.name);
            }
            allowances.insert(
                layer.name,
                layer.may_depend_on.into_iter().collect::<HashSet<_>>(),
            );
        }
        let layer_names: HashSet<_> = allowances.keys().cloned().collect();
        for (name, deps) in &allowances {
            for dep in deps {
                if !layer_names.contains(dep) {
                    bail!("layer {name} may_depend_on unknown layer {dep}");
                }
            }
        }
        let mut forbidden = Vec::new();
        let mut seen_forbidden = HashSet::new();
        for f in raw.forbidden {
            if f.from.is_empty() || f.to.is_empty() {
                bail!("forbidden edge has empty from/to");
            }
            if !layer_names.contains(&f.from) {
                bail!("forbidden.from unknown layer: {}", f.from);
            }
            if !layer_names.contains(&f.to) {
                bail!("forbidden.to unknown layer: {}", f.to);
            }
            let key = (f.from.clone(), f.to.clone());
            if !seen_forbidden.insert(key.clone()) {
                bail!("duplicate forbidden edge: {} -> {}", f.from, f.to);
            }
            let _ = f.reason; // 说明性；裁决只用 from/to
            forbidden.push((f.from, f.to));
        }
        Ok(Self {
            schema_version: raw.schema_version,
            cycles: raw.cycles,
            allowances,
            forbidden,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_merges_defaults_and_is_order_independent() {
        let text = r#"
schema_version = 1

[defaults]
status = "incubating"
publish = false

[[unit]]
publish = true
layer = "kernel"
status = "stable"
path = "crates/kernel"

[[unit]]
path = "crates/types/decimal"
layer = "types"
"#;
        let reg = WorkspaceRegistry::parse(text).expect("parse");
        assert_eq!(reg.schema_version, 1);
        assert_eq!(reg.units.len(), 2);
        let kernel = reg
            .units
            .iter()
            .find(|u| u.path == "crates/kernel")
            .unwrap();
        assert_eq!(kernel.layer, "kernel");
        assert_eq!(kernel.status, "stable");
        assert!(kernel.publish);
        let decimal = reg
            .units
            .iter()
            .find(|u| u.path == "crates/types/decimal")
            .unwrap();
        assert_eq!(decimal.status, "incubating");
        assert!(!decimal.publish);
    }

    #[test]
    fn workspace_rejects_bad_schema_and_status() {
        assert!(WorkspaceRegistry::parse("schema_version = 99\nunit = []\n").is_err());
        let bad = r#"
schema_version = 1
[[unit]]
path = "crates/kernel"
layer = "kernel"
status = "not-a-tier"
"#;
        assert!(WorkspaceRegistry::parse(bad).is_err());
    }

    #[test]
    fn workspace_rejects_duplicate_paths() {
        let text = r#"
schema_version = 1
[[unit]]
path = "crates/kernel"
layer = "kernel"
[[unit]]
path = "crates/kernel"
layer = "kernel"
"#;
        assert!(WorkspaceRegistry::parse(text).is_err());
    }

    #[test]
    fn dependency_policy_parses_multiline_may_depend_on() {
        let text = r#"
schema_version = 1
cycles = "forbidden"

[[layers]]
name = "kernel"
may_depend_on = ["kernel"]

[[layers]]
name = "types"
may_depend_on = ["kernel", "types"]

[[layers]]
name = "domain"
may_depend_on = ["kernel", "types", "domain"]

[[layers]]
name = "adapters"
may_depend_on = ["kernel", "types"]

[[layers]]
name = "tools"
may_depend_on = [
    "kernel",
    "types",
    "domain",
]

[[forbidden]]
from = "adapters"
to = "domain"
reason = "R2.1"
"#;
        let pol = DependencyPolicy::parse(text).expect("parse");
        let tools = pol.allowances.get("tools").expect("tools layer");
        assert!(tools.contains("kernel"));
        assert!(tools.contains("types"));
        assert!(tools.contains("domain"));
        assert_eq!(pol.forbidden, vec![("adapters".into(), "domain".into())]);
        assert_eq!(pol.schema_version, 1);
        assert_eq!(pol.cycles, "forbidden");
    }

    #[test]
    fn workspace_rejects_unknown_fields() {
        let text = r#"
schema_version = 1
extra_typo = true
[[unit]]
path = "crates/kernel"
layer = "kernel"
"#;
        assert!(WorkspaceRegistry::parse(text).is_err());
    }

    #[test]
    fn dependency_rejects_unknown_fields_and_duplicate_layers() {
        let missing_schema = r#"
cycles = "forbidden"
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
"#;
        assert!(DependencyPolicy::parse(missing_schema).is_err());

        let unknown = r#"
schema_version = 1
cycles = "forbidden"
typo_field = 1
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
"#;
        assert!(DependencyPolicy::parse(unknown).is_err());

        let dup = r#"
schema_version = 1
cycles = "forbidden"
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
[[layers]]
name = "kernel"
may_depend_on = ["kernel"]
"#;
        assert!(DependencyPolicy::parse(dup).is_err());

        let bad_ref = r#"
schema_version = 1
cycles = "forbidden"
[[layers]]
name = "kernel"
may_depend_on = ["nope"]
"#;
        assert!(DependencyPolicy::parse(bad_ref).is_err());
    }
}
