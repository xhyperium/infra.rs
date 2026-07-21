//! verify-runner 合同检查（PHASE-2-02）。
//!
//! 生产观测固定读取 root-owned runner attestation，并现场采集资源、toolchain、
//! tool binary digest 与 native dependency；workflow 常量不能替代观测。

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct VerifyRunnerReport {
    pub ok: bool,
    pub mode: &'static str,
    pub status: String,
    pub failures: Vec<String>,
    pub observation_source: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunnerAttestation {
    pub schema_version: u64,
    pub runner_class: String,
    pub image_digest: String,
    pub trust_domain: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedTool {
    pub name: String,
    pub version: String,
    pub binary_sha256: String,
}

#[derive(Debug, Clone)]
pub struct RunnerObserved {
    pub runner_class: String,
    pub image_digest: String,
    pub trust_domain: String,
    pub labels: Vec<String>,
    pub cpu_count: u64,
    pub memory_gib: u64,
    pub disk_free_gib: u64,
    pub toolchain_version: String,
    pub tools: Vec<ObservedTool>,
    pub native_dependencies: Vec<String>,
    pub observation_source: String,
}

#[derive(Debug)]
struct DesiredTool {
    name: String,
    version: String,
    binary_sha256: String,
}

type SimpleSection = BTreeMap<String, String>;
type SimpleToml = BTreeMap<String, SimpleSection>;

/// 对照 `.github/ci/runners.toml` / `tools.lock.toml` Desired 与 Observed。
pub fn verify_runner(root: &Path, observed: &RunnerObserved) -> Result<VerifyRunnerReport> {
    let raw = fs::read_to_string(root.join(".github/ci/runners.toml"))
        .context("read .github/ci/runners.toml")?;
    let section_name = format!("runner.{}", observed.runner_class);
    let parsed = match parse_simple_toml(&raw) {
        Ok(parsed) => parsed,
        Err(error) => return Ok(infra_report(observed, vec![error])),
    };
    let Some(body) = parsed.get(&section_name) else {
        return Ok(infra_report(
            observed,
            vec![format!("unknown_runner_class:{}", observed.runner_class)],
        ));
    };

    let mut failures = Vec::new();
    let wanted_digest = required_str(body, "image_digest", &mut failures);
    let wanted_trust = required_str(body, "trust_domain", &mut failures);
    let wanted_toolchain = required_str(body, "toolchain", &mut failures);
    let wanted_labels = required_array(body, "labels", &mut failures);
    let forbidden_labels = required_array(body, "forbidden_labels", &mut failures);
    let wanted_native = required_array(body, "native_dependencies", &mut failures);
    let attestation_path = required_str(body, "attestation_path", &mut failures);
    let tool_binary_root = required_str(body, "tool_binary_root", &mut failures);
    let cpu_min = required_u64(body, "cpu_min", &mut failures);
    let memory_min = required_u64(body, "memory_gib_min", &mut failures);
    let disk_min = required_u64(body, "disk_free_gib_min", &mut failures);

    for (name, values) in [
        ("labels", &wanted_labels),
        ("forbidden_labels", &forbidden_labels),
        ("native_dependencies", &wanted_native),
    ] {
        if BTreeSet::from_iter(values.iter()).len() != values.len() {
            failures.push(format!("contract_duplicate:{name}"));
        }
    }
    if wanted_labels
        .iter()
        .any(|label| forbidden_labels.contains(label))
    {
        failures.push("contract_overlap:labels_and_forbidden_labels".into());
    }
    if !matches!(attestation_path.as_deref(), Some(path) if path.starts_with("/etc/xhyper/")) {
        failures.push("contract_invalid:attestation_path".into());
    }
    if !matches!(tool_binary_root.as_deref(), Some(path) if path.starts_with("/opt/xhyper/")) {
        failures.push("contract_invalid:tool_binary_root".into());
    }

    if let Some(want) = wanted_digest {
        if is_placeholder(&want) {
            failures.push("contract_placeholder:image_digest".into());
        } else if !valid_sha256(&want) {
            failures.push("contract_invalid:image_digest".into());
        } else if want != observed.image_digest {
            failures.push(format!(
                "digest_mismatch:want={want}:got={}",
                observed.image_digest
            ));
        }
    }
    if is_placeholder(&observed.image_digest) || !valid_sha256(&observed.image_digest) {
        failures.push("observed_invalid:image_digest".into());
    }
    if let Some(want) = wanted_trust {
        if want != observed.trust_domain {
            failures.push(format!(
                "trust_mismatch:want={want}:got={}",
                observed.trust_domain
            ));
        }
    }
    if let Some(want) = wanted_toolchain {
        if want != observed.toolchain_version {
            failures.push(format!(
                "toolchain_mismatch:want={want}:got={}",
                observed.toolchain_version
            ));
        }
    }

    let observed_labels: BTreeSet<_> = observed.labels.iter().cloned().collect();
    if observed_labels.len() != observed.labels.len() {
        failures.push("observed_duplicate:labels".into());
    }
    for label in wanted_labels {
        if !observed_labels.contains(&label) {
            failures.push(format!("label_missing:{label}"));
        }
    }
    for label in forbidden_labels {
        if observed_labels.contains(&label) {
            failures.push(format!("forbidden_label:{label}"));
        }
    }

    check_min("cpu", cpu_min, observed.cpu_count, &mut failures);
    check_min("memory_gib", memory_min, observed.memory_gib, &mut failures);
    check_min(
        "disk_free_gib",
        disk_min,
        observed.disk_free_gib,
        &mut failures,
    );

    if wanted_native.is_empty() {
        failures.push("contract_empty:native_dependencies".into());
    }
    let observed_native: BTreeSet<_> = observed.native_dependencies.iter().cloned().collect();
    for dependency in wanted_native {
        if !observed_native.contains(&dependency) {
            failures.push(format!("native_dependency_missing:{dependency}"));
        }
    }

    let desired_tools = desired_tools(root, &mut failures)?;
    let observed_tools: BTreeMap<_, _> = observed
        .tools
        .iter()
        .map(|tool| (tool.name.as_str(), tool))
        .collect();
    if observed_tools.len() != observed.tools.len() {
        failures.push("observed_duplicate:tools".into());
    }
    if desired_tools.is_empty() {
        failures.push("contract_empty:tools".into());
    }
    for tool in desired_tools {
        if is_placeholder(&tool.binary_sha256) {
            failures.push(format!(
                "contract_placeholder:tool:{}:binary_sha256",
                tool.name
            ));
        } else if !valid_sha256(&tool.binary_sha256) {
            failures.push(format!("contract_invalid:tool:{}:binary_sha256", tool.name));
        }
        match observed_tools.get(tool.name.as_str()) {
            None => failures.push(format!("tool_missing:{}", tool.name)),
            Some(observed_tool) => {
                if observed_tool.version != tool.version {
                    failures.push(format!(
                        "tool_version_mismatch:{}:want={}:got={}",
                        tool.name, tool.version, observed_tool.version
                    ));
                }
                if observed_tool.binary_sha256 != tool.binary_sha256 {
                    failures.push(format!(
                        "tool_digest_mismatch:{}:want={}:got={}",
                        tool.name, tool.binary_sha256, observed_tool.binary_sha256
                    ));
                }
            }
        }
    }

    let ok = failures.is_empty();
    Ok(VerifyRunnerReport {
        ok,
        mode: "shadow",
        status: if ok { "PASS" } else { "INFRA_FAILURE" }.into(),
        failures,
        observation_source: observed.observation_source.clone(),
        note: "root-owned attestation + live resource/tool observation; not external runner isolation approval".into(),
    })
}

pub fn verify_runner_or_bail(root: &Path, observed: &RunnerObserved) -> Result<VerifyRunnerReport> {
    let report = verify_runner(root, observed)?;
    if !report.ok {
        bail!("ci verify-runner: INFRA_FAILURE {:?}", report.failures);
    }
    Ok(report)
}

/// 生产观测：attestation path 来自 runner contract，但必须位于 `/etc/xhyper/`，
/// 是 root-owned regular file，且 group/other 不可写。
pub fn observe_current(root: &Path, runner_class: &str) -> Result<RunnerObserved> {
    let runners = fs::read_to_string(root.join(".github/ci/runners.toml"))
        .context("read .github/ci/runners.toml")?;
    let parsed = parse_simple_toml(&runners).map_err(|error| anyhow::anyhow!(error))?;
    let body = parsed
        .get(&format!("runner.{runner_class}"))
        .with_context(|| format!("INFRA_FAILURE unknown_runner_class:{runner_class}"))?;
    let attestation_path = key_str(body, "attestation_path")
        .context("INFRA_FAILURE missing contract key attestation_path")?;
    let attestation_path = PathBuf::from(attestation_path);
    verify_attestation_path(&attestation_path)?;
    let bytes = fs::read(&attestation_path)
        .with_context(|| format!("INFRA_FAILURE read {}", attestation_path.display()))?;
    let attestation: RunnerAttestation =
        serde_json::from_slice(&bytes).context("INFRA_FAILURE parse runner attestation")?;
    if attestation.schema_version != 1 || attestation.runner_class != runner_class {
        bail!("INFRA_FAILURE attestation schema/class mismatch");
    }

    let cpu_count = std::thread::available_parallelism()
        .context("INFRA_FAILURE observe cpu")?
        .get() as u64;
    let memory_gib = observe_memory_gib()?;
    let disk_probe = env::var_os("CARGO_TARGET_DIR")
        .or_else(|| env::var_os("RUNNER_TEMP"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.to_path_buf());
    let disk_free_gib = observe_disk_gib(&disk_probe)?;
    let toolchain_version = command_semver("rustc")?;

    let tool_binary_root = key_str(body, "tool_binary_root")
        .context("INFRA_FAILURE missing contract key tool_binary_root")?;
    let tool_binary_root = PathBuf::from(tool_binary_root);
    verify_tool_binary_root(&tool_binary_root)?;

    let mut contract_failures = Vec::new();
    let desired_tools = desired_tools(root, &mut contract_failures)?;
    if !contract_failures.is_empty() {
        bail!("INFRA_FAILURE invalid tools contract {contract_failures:?}");
    }
    let mut tools = Vec::new();
    for desired in desired_tools {
        let executable = tool_executable(&desired.name);
        let path = tool_binary_root.join(executable);
        verify_root_owned_executable(&path, false)?;
        let digest_before = sha256_file(&path)?;
        let version = command_semver_path(&path)?;
        let digest_after = sha256_file(&path)?;
        if digest_before != digest_after {
            bail!("INFRA_FAILURE tool changed while observed: {executable}");
        }
        tools.push(ObservedTool {
            name: desired.name,
            version,
            binary_sha256: digest_after,
        });
    }

    let mut native_failures = Vec::new();
    let native_contract = required_array(body, "native_dependencies", &mut native_failures);
    if !native_failures.is_empty() || native_contract.is_empty() {
        bail!("INFRA_FAILURE invalid native dependency contract");
    }
    let native_dependencies = native_contract
        .into_iter()
        .filter(|name| find_trusted_native_executable(name).is_some())
        .collect();

    Ok(RunnerObserved {
        runner_class: runner_class.into(),
        image_digest: attestation.image_digest,
        trust_domain: attestation.trust_domain,
        labels: attestation.labels,
        cpu_count,
        memory_gib,
        disk_free_gib,
        toolchain_version,
        tools,
        native_dependencies,
        observation_source: attestation_path.display().to_string(),
    })
}

fn infra_report(observed: &RunnerObserved, failures: Vec<String>) -> VerifyRunnerReport {
    VerifyRunnerReport {
        ok: false,
        mode: "shadow",
        status: "INFRA_FAILURE".into(),
        failures,
        observation_source: observed.observation_source.clone(),
        note: "runner contract failure; no runtime repair permitted".into(),
    }
}

fn required_str(section: &SimpleSection, key: &str, failures: &mut Vec<String>) -> Option<String> {
    let value = key_str(section, key);
    if value.is_none() {
        failures.push(format!("contract_missing:{key}"));
    }
    value
}

fn required_u64(section: &SimpleSection, key: &str, failures: &mut Vec<String>) -> Option<u64> {
    let value = key_u64(section, key);
    if value.is_none() {
        failures.push(format!("contract_missing_or_invalid:{key}"));
    }
    value
}

fn required_array(section: &SimpleSection, key: &str, failures: &mut Vec<String>) -> Vec<String> {
    match key_array(section, key) {
        Some(values) if !values.is_empty() => values,
        _ => {
            failures.push(format!("contract_missing_or_empty:{key}"));
            Vec::new()
        }
    }
}

fn check_min(name: &str, wanted: Option<u64>, observed: u64, failures: &mut Vec<String>) {
    if let Some(wanted) = wanted {
        if observed < wanted {
            failures.push(format!("{name}_insufficient:need>={wanted}:got={observed}"));
        }
    }
}

fn desired_tools(root: &Path, failures: &mut Vec<String>) -> Result<Vec<DesiredTool>> {
    let raw = fs::read_to_string(root.join(".github/ci/tools.lock.toml"))
        .context("read .github/ci/tools.lock.toml")?;
    let parsed = parse_simple_toml(&raw).map_err(|error| anyhow::anyhow!(error))?;
    if parsed
        .get("")
        .and_then(|global| key_u64(global, "schema_version"))
        != Some(1)
    {
        failures.push("contract_missing_or_invalid:tools.schema_version".into());
    }
    let mut tools = Vec::new();
    for (section_name, body) in &parsed {
        let Some(name) = section_name.strip_prefix("tool.") else {
            continue;
        };
        let version = key_str(body, "version");
        let binary_sha256 = key_str(body, "binary_sha256");
        match (version, binary_sha256) {
            (Some(version), Some(binary_sha256)) => tools.push(DesiredTool {
                name: name.into(),
                version,
                binary_sha256,
            }),
            _ => failures.push(format!(
                "contract_missing:tool:{name}:version_or_binary_sha256"
            )),
        }
    }
    Ok(tools)
}

/// runners/tools 合同只使用单行 scalar/array TOML 子集。这里严格拒绝重复 section、
/// 重复 key、未闭合引号和未知语法，避免“取第一项”掩盖恶意第二项。
fn parse_simple_toml(raw: &str) -> std::result::Result<SimpleToml, String> {
    let mut sections = SimpleToml::new();
    sections.insert(String::new(), SimpleSection::new());
    let mut current = String::new();
    for (index, source) in raw.lines().enumerate() {
        let line = strip_toml_comment(source)
            .map_err(|error| format!("INFRA_FAILURE contract parse line {}: {error}", index + 1))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let name = line
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
                .filter(|value| !value.is_empty() && !value.contains('[') && !value.contains(']'))
                .ok_or_else(|| {
                    format!(
                        "INFRA_FAILURE malformed contract section at line {}",
                        index + 1
                    )
                })?;
            if sections.contains_key(name) {
                return Err(format!("INFRA_FAILURE duplicate contract section:{name}"));
            }
            current = name.to_string();
            sections.insert(current.clone(), SimpleSection::new());
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            format!(
                "INFRA_FAILURE malformed contract entry at line {}",
                index + 1
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(format!(
                "INFRA_FAILURE malformed contract key/value at line {}",
                index + 1
            ));
        }
        let section = sections
            .get_mut(&current)
            .expect("current contract section must exist");
        if section.insert(key.to_string(), value.to_string()).is_some() {
            return Err(format!(
                "INFRA_FAILURE duplicate contract key:{current}.{key}"
            ));
        }
    }
    Ok(sections)
}

fn strip_toml_comment(line: &str) -> std::result::Result<String, &'static str> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            continue;
        }
        if character == '#' && !quoted {
            return Ok(line[..index].to_string());
        }
    }
    if quoted || escaped {
        return Err("unterminated quoted value");
    }
    Ok(line.to_string())
}

fn key_str(section: &SimpleSection, key: &str) -> Option<String> {
    serde_json::from_str(section.get(key)?).ok()
}

fn key_u64(section: &SimpleSection, key: &str) -> Option<u64> {
    section.get(key)?.parse().ok()
}

fn key_array(section: &SimpleSection, key: &str) -> Option<Vec<String>> {
    serde_json::from_str(section.get(key)?).ok()
}

fn is_placeholder(value: &str) -> bool {
    value.to_ascii_uppercase().contains("PLACEHOLDER")
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn verify_attestation_path(path: &Path) -> Result<()> {
    if !path.is_absolute() || !path.starts_with("/etc/xhyper/") {
        bail!("INFRA_FAILURE attestation path outside /etc/xhyper");
    }
    verify_root_owned_directory(
        path.parent()
            .context("INFRA_FAILURE attestation path has no parent")?,
        "/etc/xhyper",
    )?;
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("INFRA_FAILURE stat {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("INFRA_FAILURE attestation must be a regular non-symlink file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != 0 || metadata.permissions().mode() & 0o022 != 0 {
            bail!("INFRA_FAILURE attestation must be root-owned and not group/other writable");
        }
    }
    #[cfg(not(unix))]
    bail!("INFRA_FAILURE root-owned attestation validation requires unix");
    Ok(())
}

fn observe_memory_gib() -> Result<u64> {
    let raw = fs::read_to_string("/proc/meminfo").context("INFRA_FAILURE read /proc/meminfo")?;
    let kib = raw
        .lines()
        .find_map(|line| {
            line.strip_prefix("MemTotal:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()
        })
        .context("INFRA_FAILURE parse MemTotal")?;
    Ok(kib / 1024 / 1024)
}

fn observe_disk_gib(root: &Path) -> Result<u64> {
    let output = Command::new("df")
        .arg("-Pk")
        .arg(root)
        .output()
        .context("INFRA_FAILURE execute df")?;
    if !output.status.success() {
        bail!("INFRA_FAILURE df failed");
    }
    let text = String::from_utf8(output.stdout).context("INFRA_FAILURE df output utf8")?;
    let available_kib: u64 = text
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(3))
        .and_then(|value| value.parse().ok())
        .context("INFRA_FAILURE parse df available blocks")?;
    Ok(available_kib / 1024 / 1024)
}

fn tool_executable(name: &str) -> &str {
    match name {
        "nextest" => "cargo-nextest",
        other => other,
    }
}

fn find_executable(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let path = PathBuf::from(name);
        return is_executable(&path).then_some(path);
    }
    env::split_paths(&env::var_os("PATH")?).find_map(|directory| {
        let candidate = directory.join(name);
        is_executable(&candidate).then_some(candidate)
    })
}

fn find_trusted_native_executable(name: &str) -> Option<PathBuf> {
    [Path::new("/usr/bin"), Path::new("/bin")]
        .into_iter()
        .map(|root| root.join(name))
        .find(|candidate| verify_root_owned_executable(candidate, true).is_ok())
}

fn verify_tool_binary_root(path: &Path) -> Result<()> {
    if !path.is_absolute() || !path.starts_with("/opt/xhyper/") {
        bail!("INFRA_FAILURE tool_binary_root outside /opt/xhyper");
    }
    verify_root_owned_directory(path, "/opt/xhyper")
}

fn verify_root_owned_directory(path: &Path, allowed_prefix: &str) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("INFRA_FAILURE canonicalize {}", path.display()))?;
    if !canonical.starts_with(allowed_prefix) {
        bail!("INFRA_FAILURE trusted directory escapes {allowed_prefix}");
    }
    let mut current = Some(canonical.as_path());
    while let Some(directory) = current {
        let metadata = fs::metadata(directory)
            .with_context(|| format!("INFRA_FAILURE stat {}", directory.display()))?;
        if !metadata.is_dir() {
            bail!("INFRA_FAILURE trusted path is not a directory");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            if metadata.uid() != 0 || metadata.permissions().mode() & 0o022 != 0 {
                bail!("INFRA_FAILURE trusted directory chain must be root-owned and not group/other writable");
            }
        }
        current = directory.parent();
    }
    Ok(())
}

fn verify_root_owned_executable(path: &Path, allow_symlink: bool) -> Result<()> {
    let link_metadata = fs::symlink_metadata(path)
        .with_context(|| format!("INFRA_FAILURE stat executable {}", path.display()))?;
    if link_metadata.file_type().is_symlink() && !allow_symlink {
        bail!("INFRA_FAILURE tool executable cannot be a symlink");
    }
    let canonical = path
        .canonicalize()
        .with_context(|| format!("INFRA_FAILURE canonicalize executable {}", path.display()))?;
    let metadata = fs::metadata(&canonical)
        .with_context(|| format!("INFRA_FAILURE stat executable {}", canonical.display()))?;
    if !metadata.is_file() {
        bail!("INFRA_FAILURE executable is not a regular file");
    }
    verify_root_owned_directory(
        canonical
            .parent()
            .context("INFRA_FAILURE executable has no parent directory")?,
        "/",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != 0
            || metadata.permissions().mode() & 0o022 != 0
            || metadata.permissions().mode() & 0o111 == 0
        {
            bail!("INFRA_FAILURE executable must be root-owned, immutable to group/other, and executable");
        }
    }
    Ok(())
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    true
}

fn command_semver(command: &str) -> Result<String> {
    let path = find_executable(command)
        .with_context(|| format!("INFRA_FAILURE executable missing: {command}"))?;
    command_semver_path(&path)
}

fn command_semver_path(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .with_context(|| format!("INFRA_FAILURE execute {} --version", path.display()))?;
    if !output.status.success() {
        bail!("INFRA_FAILURE version command failed: {}", path.display());
    }
    let text = String::from_utf8(output.stdout).context("INFRA_FAILURE version output utf8")?;
    text.split_whitespace()
        .find(|token| {
            let mut parts = token.split('.');
            matches!(parts.next(), Some(first) if first.bytes().all(|b| b.is_ascii_digit()))
                && matches!(parts.next(), Some(second) if second.bytes().all(|b| b.is_ascii_digit()))
                && matches!(parts.next(), Some(third) if third.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|token| token.trim_end_matches(|c: char| !c.is_ascii_digit()).to_string())
        .context("INFRA_FAILURE version semver missing")
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("INFRA_FAILURE read tool binary {}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_contract(root: &Path, digest: &str, tool_digest: &str) {
        fs::create_dir_all(root.join(".github/ci")).unwrap();
        fs::write(
            root.join(".github/ci/runners.toml"),
            format!(
                r#"[runner.fast]
labels = ["self-hosted", "xhyper-ci-fast-v1"]
forbidden_labels = ["linux", "x64", "xhyper-ci-release-v1"]
trust_domain = "pr-untrusted"
cpu_min = 4
memory_gib_min = 8
disk_free_gib_min = 20
image_digest = "{digest}"
toolchain = "1.94.1"
native_dependencies = ["bash", "python3"]
tool_binary_root = "/opt/xhyper/bin"
attestation_path = "/etc/xhyper/runner-attestation.json"
"#,
            ),
        )
        .unwrap();
        fs::write(
            root.join(".github/ci/tools.lock.toml"),
            format!(
                r#"schema_version = 1
[tool.nextest]
version = "0.9.140"
sha256 = "sha256:{}"
binary_sha256 = "{tool_digest}"
"#,
                "a".repeat(64)
            ),
        )
        .unwrap();
    }

    fn observed(digest: &str, tool_digest: &str) -> RunnerObserved {
        RunnerObserved {
            runner_class: "fast".into(),
            image_digest: digest.into(),
            trust_domain: "pr-untrusted".into(),
            labels: vec!["self-hosted".into(), "xhyper-ci-fast-v1".into()],
            cpu_count: 8,
            memory_gib: 16,
            disk_free_gib: 40,
            toolchain_version: "1.94.1".into(),
            tools: vec![ObservedTool {
                name: "nextest".into(),
                version: "0.9.140".into(),
                binary_sha256: tool_digest.into(),
            }],
            native_dependencies: vec!["bash".into(), "python3".into()],
            observation_source: "/etc/xhyper/runner-attestation.json".into(),
        }
    }

    #[test]
    fn complete_observation_matches_non_placeholder_contract() {
        let temp = TempDir::new().unwrap();
        let digest = format!("sha256:{}", "b".repeat(64));
        let tool_digest = format!("sha256:{}", "c".repeat(64));
        write_contract(temp.path(), &digest, &tool_digest);
        let report = verify_runner(temp.path(), &observed(&digest, &tool_digest)).unwrap();
        assert!(report.ok, "{report:?}");
    }

    #[test]
    fn placeholders_empty_tools_and_forbidden_labels_fail_closed() {
        let temp = TempDir::new().unwrap();
        let placeholder = "sha256:PLACEHOLDER_FAST_IMAGE_DIGEST";
        let tool_placeholder = "sha256:PLACEHOLDER_NEXTEST_BINARY";
        write_contract(temp.path(), placeholder, tool_placeholder);
        let mut observation = observed(placeholder, tool_placeholder);
        observation.labels.push("linux".into());
        observation.tools.clear();
        let report = verify_runner(temp.path(), &observation).unwrap();
        assert!(!report.ok);
        assert!(report
            .failures
            .iter()
            .any(|failure| failure.contains("contract_placeholder:image_digest")));
        assert!(report
            .failures
            .iter()
            .any(|failure| failure == "forbidden_label:linux"));
        assert!(report
            .failures
            .iter()
            .any(|failure| failure == "tool_missing:nextest"));
    }

    #[test]
    fn resource_toolchain_native_and_tool_digest_mismatches_fail_closed() {
        let temp = TempDir::new().unwrap();
        let digest = format!("sha256:{}", "b".repeat(64));
        let tool_digest = format!("sha256:{}", "c".repeat(64));
        write_contract(temp.path(), &digest, &tool_digest);
        let mut observation = observed(&digest, &format!("sha256:{}", "d".repeat(64)));
        observation.cpu_count = 1;
        observation.memory_gib = 1;
        observation.disk_free_gib = 1;
        observation.toolchain_version = "stable".into();
        observation.native_dependencies.clear();
        let report = verify_runner(temp.path(), &observation).unwrap();
        assert!(!report.ok);
        let failures = report.failures.join("\n");
        assert!(failures.contains("cpu_insufficient"));
        assert!(failures.contains("memory_gib_insufficient"));
        assert!(failures.contains("disk_free_gib_insufficient"));
        assert!(failures.contains("toolchain_mismatch"));
        assert!(failures.contains("native_dependency_missing"));
        assert!(failures.contains("tool_digest_mismatch"));
    }

    #[test]
    fn key_parser_does_not_accept_prefix_confusion() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "image_digest_suffix = \"bad\"").unwrap();
        writeln!(file, "image_digest = \"good\"").unwrap();
        let raw = fs::read_to_string(file.path()).unwrap();
        let parsed = parse_simple_toml(&raw).unwrap();
        assert_eq!(
            key_str(parsed.get("").unwrap(), "image_digest").as_deref(),
            Some("good")
        );
    }

    #[test]
    fn duplicate_runner_sections_and_keys_fail_closed() {
        for raw in [
            "[runner.fast]\ntrust_domain = \"a\"\n[runner.fast]\ntrust_domain = \"b\"\n",
            "[runner.fast]\ntrust_domain = \"a\"\ntrust_domain = \"b\"\n",
            "[tool.nextest]\nversion = \"1\"\nversion = \"2\"\n",
        ] {
            let error = parse_simple_toml(raw).unwrap_err();
            assert!(error.contains("duplicate contract"), "{error}");
        }
    }

    #[test]
    fn attestation_schema_rejects_unknown_fields_and_external_paths() {
        let invalid = serde_json::json!({
            "schema_version": 1,
            "runner_class": "fast",
            "image_digest": format!("sha256:{}", "a".repeat(64)),
            "trust_domain": "pr-untrusted",
            "labels": ["self-hosted", "xhyper-ci-fast-v1"],
            "workflow_claim": "forbidden"
        });
        assert!(serde_json::from_value::<RunnerAttestation>(invalid).is_err());

        let temp = TempDir::new().unwrap();
        let outside = temp.path().join("runner-attestation.json");
        fs::write(&outside, "{}").unwrap();
        assert!(verify_attestation_path(&outside).is_err());
    }
}
