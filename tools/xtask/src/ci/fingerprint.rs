//! Lane fingerprint typed contract（PHASE-3-01）与尚未启用的 reuse 表面。

use anyhow::{bail, Context, Result};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const FINGERPRINT_CONTRACT_VERSION: u32 = 1;
const FINGERPRINT_DOMAIN_SEPARATOR: &[u8] = b"xhyper-ci-lane-fingerprint\0v1\0";
const MAX_FINGERPRINT_INPUT_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct Sha256Digest(String);

impl Sha256Digest {
    fn parse(value: String) -> Result<Self> {
        let Some(hex) = value.strip_prefix("sha256:") else {
            bail!("digest must start with sha256:");
        };
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            bail!("digest must contain exactly 64 lowercase hexadecimal characters");
        }
        if hex.bytes().all(|byte| byte == b'0') {
            bail!("all-zero digest is forbidden");
        }
        Ok(Self(value))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CommandInputV1 {
    argv: Vec<String>,
    cwd: CommandCwd,
    program: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CommandCwd {
    RepoRoot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceFileDigestV1 {
    digest: Sha256Digest,
    repo_relative_posix_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CargoPackageIdentityV1 {
    package: String,
    version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EnvironmentDigestV1 {
    name: String,
    value_digest: Sha256Digest,
}

/// Spec §9.1 的 12 类输入；`contract_version` 是仓库编码协议的额外 domain 字段。
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FingerprintInputV1 {
    contract_version: u32,
    lane_id: String,
    command: CommandInputV1,
    source_file_digests: Vec<SourceFileDigestV1>,
    affected_dependency_closure: Vec<CargoPackageIdentityV1>,
    cargo_lock_digest: Sha256Digest,
    toolchain_digest: Sha256Digest,
    tool_lock_digest: Sha256Digest,
    runner_image_digest: Sha256Digest,
    relevant_environment: Vec<EnvironmentDigestV1>,
    baseline_digest: Sha256Digest,
    plan_version: u32,
    evidence_schema_version: u32,
}

/// 仓库定义的 V1 canonical encoding。字段按此声明顺序写出；禁止 map。
#[derive(Serialize)]
struct CanonicalFingerprintInputV1<'a> {
    affected_dependency_closure: &'a [CargoPackageIdentityV1],
    baseline_digest: &'a Sha256Digest,
    cargo_lock_digest: &'a Sha256Digest,
    command: &'a CommandInputV1,
    contract_version: u32,
    evidence_schema_version: u32,
    lane_id: &'a str,
    plan_version: u32,
    relevant_environment: &'a [EnvironmentDigestV1],
    runner_image_digest: &'a Sha256Digest,
    source_file_digests: &'a [SourceFileDigestV1],
    tool_lock_digest: &'a Sha256Digest,
    toolchain_digest: &'a Sha256Digest,
}

#[derive(Debug, Serialize)]
pub struct FingerprintReport {
    pub ok: bool,
    pub mode: &'static str,
    pub contract_version: u32,
    pub lane: String,
    pub fingerprint: String,
    pub reusable: bool,
    pub provenance_unverified: bool,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct ReuseReport {
    pub ok: bool,
    pub mode: &'static str,
    pub reuse_enabled: bool,
    pub decision: String,
    pub attestation_ok: Option<bool>,
    pub validation: Option<ReuseValidationReport>,
    pub note: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ReuseStructuralConditions {
    pub lane_reusable: bool,
    pub fingerprint_exact: bool,
    pub source_run_pass: bool,
    pub attestation_contract_and_source_head_match: bool,
    pub ttl_fresh: bool,
    pub runner_class_contract_match: bool,
    pub revocation_observation_match: bool,
    pub baseline_permits: bool,
    pub dynamic_input_unchanged: bool,
}

impl ReuseStructuralConditions {
    fn all_satisfied(&self) -> bool {
        self.lane_reusable
            && self.fingerprint_exact
            && self.source_run_pass
            && self.attestation_contract_and_source_head_match
            && self.ttl_fresh
            && self.runner_class_contract_match
            && self.revocation_observation_match
            && self.baseline_permits
            && self.dynamic_input_unchanged
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReuseValidationReport {
    pub reference_valid: bool,
    pub source_path: Option<String>,
    pub source_digest: Option<String>,
    pub structural_conditions: ReuseStructuralConditions,
    pub all_structural_predicates_satisfied: bool,
    pub reasons: Vec<String>,
    pub candidate_observation_unverified: bool,
    pub trusted_attestation_verified: bool,
    pub runner_trust_verified: bool,
    pub production_activation: bool,
}

impl ReuseValidationReport {
    fn invalid(reason: impl Into<String>) -> Self {
        Self {
            reference_valid: false,
            source_path: None,
            source_digest: None,
            structural_conditions: ReuseStructuralConditions::default(),
            all_structural_predicates_satisfied: false,
            reasons: vec![reason.into()],
            candidate_observation_unverified: true,
            trusted_attestation_verified: false,
            runner_trust_verified: false,
            production_activation: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct LaneAttestationV1 {
    schema_version: u32,
    #[serde(rename = "type")]
    kind: String,
    run_id: String,
    lane: String,
    decision: String,
    base_sha: String,
    head_sha: String,
    plan_digest: Sha256Digest,
    fingerprint: Sha256Digest,
    runner_class: String,
    runner_image_digest: Sha256Digest,
    toolchain_digest: Sha256Digest,
    started_at: String,
    finished_at: String,
    result_digest: Sha256Digest,
    reuse_inputs: Option<ReuseInputsV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReuseInputsV1 {
    baseline_raw_sha256: Sha256Digest,
    dynamic_input_digest: Sha256Digest,
    revocation_registry_digest: Sha256Digest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReuseObservationV1 {
    schema_version: u32,
    expected_lane: String,
    expected_fingerprint: Sha256Digest,
    expected_source_head_sha: String,
    now: String,
    current_dynamic_input_digest: Sha256Digest,
    current_revocation_registry_digest: Sha256Digest,
    revoked_fingerprints: Vec<Sha256Digest>,
}

#[derive(Debug, Deserialize)]
struct ReuseBaselineV1 {
    reuse: ReuseBaselinePolicyV1,
    lanes: BTreeMap<String, ReuseBaselineLaneV1>,
}

#[derive(Debug, Deserialize)]
struct ReuseBaselinePolicyV1 {
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ReuseBaselineLaneV1 {
    reusable: bool,
    runner_class: String,
    ttl_seconds: Option<u64>,
}

/// 唯一公开入口：parse → validate → normalize → canonicalize → hash。
pub fn fingerprint_from_file(path: &Path) -> Result<FingerprintReport> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if !metadata.is_file() {
        bail!("fingerprint input must be a regular file");
    }
    if metadata.len() > MAX_FINGERPRINT_INPUT_BYTES {
        bail!("fingerprint input exceeds 1 MiB limit");
    }
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    fingerprint_from_json(&bytes)
}

fn fingerprint_from_json(bytes: &[u8]) -> Result<FingerprintReport> {
    if bytes.len() as u64 > MAX_FINGERPRINT_INPUT_BYTES {
        bail!("fingerprint input exceeds 1 MiB limit");
    }
    let input: FingerprintInputV1 =
        serde_json::from_slice(bytes).context("parse typed FingerprintInputV1 JSON")?;
    fingerprint(input)
}

fn fingerprint(mut input: FingerprintInputV1) -> Result<FingerprintReport> {
    validate_and_normalize(&mut input)?;
    let canonical = canonical_bytes(&input)?;
    let mut preimage = Vec::with_capacity(FINGERPRINT_DOMAIN_SEPARATOR.len() + canonical.len());
    preimage.extend_from_slice(FINGERPRINT_DOMAIN_SEPARATOR);
    preimage.extend_from_slice(&canonical);
    let fingerprint = format!("sha256:{:x}", Sha256::digest(&preimage));
    Ok(FingerprintReport {
        ok: true,
        mode: "shadow",
        contract_version: FINGERPRINT_CONTRACT_VERSION,
        lane: input.lane_id,
        fingerprint,
        reusable: false,
        provenance_unverified: true,
        note: "typed deterministic candidate fingerprint; observation provenance and reuse eligibility are not verified".into(),
    })
}

fn canonical_bytes(input: &FingerprintInputV1) -> Result<Vec<u8>> {
    serde_json::to_vec(&CanonicalFingerprintInputV1 {
        affected_dependency_closure: &input.affected_dependency_closure,
        baseline_digest: &input.baseline_digest,
        cargo_lock_digest: &input.cargo_lock_digest,
        command: &input.command,
        contract_version: input.contract_version,
        evidence_schema_version: input.evidence_schema_version,
        lane_id: &input.lane_id,
        plan_version: input.plan_version,
        relevant_environment: &input.relevant_environment,
        runner_image_digest: &input.runner_image_digest,
        source_file_digests: &input.source_file_digests,
        tool_lock_digest: &input.tool_lock_digest,
        toolchain_digest: &input.toolchain_digest,
    })
    .context("serialize canonical FingerprintInputV1")
}

fn validate_and_normalize(input: &mut FingerprintInputV1) -> Result<()> {
    if input.contract_version != FINGERPRINT_CONTRACT_VERSION {
        bail!(
            "unsupported fingerprint contract_version {}; expected {FINGERPRINT_CONTRACT_VERSION}",
            input.contract_version
        );
    }
    validate_ascii_identifier(&input.lane_id, "lane_id", 64, |byte, index| {
        if index == 0 {
            byte.is_ascii_lowercase()
        } else {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        }
    })?;
    validate_ascii_text(&input.command.program, "command.program")?;
    if input.command.argv.is_empty() {
        bail!("command.argv must not be empty");
    }
    for (index, argument) in input.command.argv.iter().enumerate() {
        validate_utf8_text(argument, &format!("command.argv[{index}]"))?;
    }

    if input.source_file_digests.is_empty() {
        bail!("source_file_digests must not be empty");
    }
    for source in &input.source_file_digests {
        validate_repo_path(&source.repo_relative_posix_path)?;
    }
    input.source_file_digests.sort_by(|left, right| {
        left.repo_relative_posix_path
            .cmp(&right.repo_relative_posix_path)
    });
    reject_adjacent_duplicates(
        input
            .source_file_digests
            .iter()
            .map(|source| source.repo_relative_posix_path.as_str()),
        "source path",
    )?;

    if input.affected_dependency_closure.is_empty() {
        bail!("affected_dependency_closure must not be empty");
    }
    for package in &input.affected_dependency_closure {
        validate_package(&package.package)?;
        validate_version(&package.version, "affected_dependency_closure.version")?;
    }
    input.affected_dependency_closure.sort_by(|left, right| {
        (&left.package, &left.version).cmp(&(&right.package, &right.version))
    });
    reject_adjacent_duplicates(
        input
            .affected_dependency_closure
            .iter()
            .map(|package| format!("{}@{}", package.package, package.version)),
        "Cargo package identity",
    )?;

    if input.relevant_environment.is_empty() {
        bail!("relevant_environment must not be empty");
    }
    for environment in &input.relevant_environment {
        validate_env_name(&environment.name)?;
    }
    input
        .relevant_environment
        .sort_by(|left, right| left.name.cmp(&right.name));
    reject_adjacent_duplicates(
        input
            .relevant_environment
            .iter()
            .map(|environment| environment.name.as_str()),
        "environment name",
    )?;

    if input.plan_version == 0 || input.evidence_schema_version == 0 {
        bail!("plan_version and evidence_schema_version must be positive");
    }
    Ok(())
}

fn reject_adjacent_duplicates<I, T>(values: I, label: &str) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Ord + std::fmt::Display,
{
    let mut previous: Option<T> = None;
    for value in values {
        if previous.as_ref() == Some(&value) {
            bail!("duplicate {label}: {value}");
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_ascii_text(value: &str, field: &str) -> Result<()> {
    validate_utf8_text(value, field)?;
    if !value.is_ascii() {
        bail!("{field} must be ASCII");
    }
    Ok(())
}

/// V1 对 argv/path 保留原始 UTF-8 bytes，不做 Unicode normalization。
fn validate_utf8_text(value: &str, field: &str) -> Result<()> {
    if value.is_empty() || value != value.trim() {
        bail!("{field} must be non-empty without edge whitespace");
    }
    if value.chars().any(char::is_control) || contains_placeholder(value) {
        bail!("{field} contains a forbidden control or placeholder value");
    }
    Ok(())
}

fn validate_ascii_identifier<F>(value: &str, field: &str, max_len: usize, allowed: F) -> Result<()>
where
    F: Fn(u8, usize) -> bool,
{
    if value.is_empty() || value.len() > max_len || !value.is_ascii() {
        bail!("{field} has invalid length or encoding");
    }
    if contains_placeholder(value)
        || !value
            .bytes()
            .enumerate()
            .all(|(index, byte)| allowed(byte, index))
    {
        bail!("{field} has invalid characters");
    }
    Ok(())
}

fn validate_repo_path(path: &str) -> Result<()> {
    validate_utf8_text(path, "source repo_relative_posix_path")?;
    if path.starts_with('/') || path.ends_with('/') || path.contains('\\') {
        bail!("source path must be repo-relative POSIX");
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        bail!("source path contains an empty, dot, or dot-dot segment");
    }
    Ok(())
}

fn validate_package(package: &str) -> Result<()> {
    validate_ascii_identifier(
        package,
        "affected_dependency_closure.package",
        128,
        |byte, index| {
            if index == 0 {
                byte.is_ascii_alphanumeric()
            } else {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
            }
        },
    )
}

fn validate_version(version: &str, field: &str) -> Result<()> {
    validate_ascii_identifier(version, field, 64, |byte, index| {
        if index == 0 {
            byte.is_ascii_digit()
        } else {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+')
        }
    })
}

fn validate_env_name(name: &str) -> Result<()> {
    validate_ascii_identifier(name, "relevant_environment.name", 128, |byte, index| {
        if index == 0 {
            byte.is_ascii_uppercase() || byte == b'_'
        } else {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
        }
    })
}

fn contains_placeholder(value: &str) -> bool {
    value.to_ascii_uppercase().contains("PLACEHOLDER")
}

/// `ci reuse` 始终回退 RUN；validator 仅报告未验证的结构 predicate。
pub fn evaluate_reuse(
    root: &Path,
    want_reused: bool,
    attestation_root: Option<&Path>,
    reference: Option<&str>,
    context_file: Option<&Path>,
) -> Result<ReuseReport> {
    if !want_reused {
        return Ok(ReuseReport {
            ok: true,
            mode: "shadow",
            reuse_enabled: false,
            decision: "RUN".into(),
            attestation_ok: None,
            validation: None,
            note: "reuse not requested".into(),
        });
    }
    let validation = match load_observation(context_file) {
        Ok(observation) => validate_reuse_reference(
            root,
            attestation_root,
            reference,
            &observation.expected_lane,
            context_file,
        ),
        Err(_) => ReuseValidationReport::invalid("reuse_observation_context_missing_or_invalid"),
    };
    Ok(ReuseReport {
        ok: true,
        mode: "shadow",
        reuse_enabled: false,
        decision: "RUN".into(),
        attestation_ok: Some(false),
        validation: Some(validation),
        note: "production reuse activation is disabled; structural candidate can only fall back to RUN".into(),
    })
}

pub fn validate_reuse_reference(
    repo_root: &Path,
    attestation_root: Option<&Path>,
    reference: Option<&str>,
    expected_lane: &str,
    context_file: Option<&Path>,
) -> ReuseValidationReport {
    validate_reuse_reference_inner(
        repo_root,
        attestation_root,
        reference,
        expected_lane,
        context_file,
    )
    .unwrap_or_else(ReuseValidationReport::invalid)
}

fn validate_reuse_reference_inner(
    repo_root: &Path,
    attestation_root: Option<&Path>,
    reference: Option<&str>,
    expected_lane: &str,
    context_file: Option<&Path>,
) -> std::result::Result<ReuseValidationReport, String> {
    let reference = reference
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "missing_reused_attestation".to_string())?;
    validate_reference_path(reference)?;
    let root = attestation_root.ok_or_else(|| "missing_attestation_root".to_string())?;
    let trusted_root = root
        .canonicalize()
        .map_err(|_| "invalid_attestation_root".to_string())?;
    let candidate = trusted_root
        .join(reference)
        .canonicalize()
        .map_err(|_| "reused_attestation_not_found".to_string())?;
    if !candidate.starts_with(&trusted_root) || !candidate.is_file() {
        return Err("attestation_path_escape".into());
    }
    let bytes = fs::read(&candidate).map_err(|_| "reused_attestation_read_failed".to_string())?;
    if bytes.len() as u64 > MAX_FINGERPRINT_INPUT_BYTES {
        return Err("reused_attestation_too_large".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "reused_attestation_invalid_json".to_string())?;
    let schema_bytes = fs::read(repo_root.join("evidence/schemas/ci/lane-attestation.schema.json"))
        .map_err(|_| "lane_attestation_schema_missing".to_string())?;
    let schema: serde_json::Value = serde_json::from_slice(&schema_bytes)
        .map_err(|_| "lane_attestation_schema_invalid".to_string())?;
    let schema_valid = crate::schema_lite::json_schema_matches(&value, &schema);
    let attestation: LaneAttestationV1 = serde_json::from_slice(&bytes)
        .map_err(|_| "reused_attestation_typed_contract_invalid".to_string())?;
    let baseline_raw = fs::read(repo_root.join(".github/ci/baseline.toml"))
        .map_err(|_| "reuse_baseline_missing".to_string())?;
    let baseline: ReuseBaselineV1 = toml::from_str(
        std::str::from_utf8(&baseline_raw).map_err(|_| "reuse_baseline_not_utf8".to_string())?,
    )
    .map_err(|_| "reuse_baseline_invalid".to_string())?;
    let lane_contract = baseline
        .lanes
        .get(expected_lane)
        .ok_or_else(|| "reuse_lane_missing_from_baseline".to_string())?;
    let lane_ttl_seconds = lane_contract
        .ttl_seconds
        .filter(|seconds| *seconds > 0)
        .ok_or_else(|| "reuse_lane_ttl_missing_or_zero".to_string())?;
    if lane_contract.runner_class.trim().is_empty() {
        return Err("reuse_lane_contract_invalid".into());
    }

    let context = context_file.and_then(|path| load_observation(Some(path)).ok());
    let context_present = context.is_some();
    let reuse_inputs = attestation.reuse_inputs.as_ref();
    let source_times = parse_attestation_times(&attestation);
    let strict_core = validate_attestation_core(&attestation).is_ok();
    let expected_lane_matches = context
        .as_ref()
        .is_some_and(|observation| observation.expected_lane == expected_lane);
    let source_head_matches = context
        .as_ref()
        .is_some_and(|observation| observation.expected_source_head_sha == attestation.head_sha);
    let fingerprint_exact = context
        .as_ref()
        .is_some_and(|observation| observation.expected_fingerprint == attestation.fingerprint);
    let ttl_fresh = context.as_ref().is_some_and(|observation| {
        source_times.is_some_and(|(started, finished)| {
            parse_rfc3339_epoch(&observation.now).is_some_and(|now| {
                started <= finished
                    && finished <= now
                    && finished
                        .checked_add(lane_ttl_seconds)
                        .is_some_and(|expires| now <= expires)
            })
        })
    });
    let baseline_raw_sha256 =
        Sha256Digest::parse(format!("sha256:{:x}", Sha256::digest(&baseline_raw)))
            .map_err(|_| "reuse_baseline_digest_failed".to_string())?;
    let revocation_observation_match = context.as_ref().is_some_and(|observation| {
        reuse_inputs.is_some_and(|inputs| {
            inputs.revocation_registry_digest == observation.current_revocation_registry_digest
                && !observation
                    .revoked_fingerprints
                    .contains(&attestation.fingerprint)
        })
    });
    let baseline_permits = reuse_inputs.is_some_and(|inputs| {
        baseline.reuse.enabled && inputs.baseline_raw_sha256 == baseline_raw_sha256
    });
    let dynamic_input_unchanged = context.as_ref().is_some_and(|observation| {
        reuse_inputs.is_some_and(|inputs| {
            inputs.dynamic_input_digest == observation.current_dynamic_input_digest
        })
    });
    let conditions = ReuseStructuralConditions {
        lane_reusable: lane_contract.reusable,
        fingerprint_exact,
        source_run_pass: attestation.decision == "RUN_PASS",
        attestation_contract_and_source_head_match: schema_valid
            && strict_core
            && reuse_inputs.is_some()
            && expected_lane_matches
            && attestation.lane == expected_lane
            && source_head_matches,
        ttl_fresh,
        runner_class_contract_match: attestation.runner_class == lane_contract.runner_class,
        revocation_observation_match,
        baseline_permits,
        dynamic_input_unchanged,
    };
    let all_satisfied = conditions.all_satisfied();
    let mut reasons = Vec::new();
    for (satisfied, reason) in [
        (conditions.lane_reusable, "lane_not_reusable"),
        (
            conditions.fingerprint_exact,
            "fingerprint_mismatch_or_context_missing",
        ),
        (conditions.source_run_pass, "source_not_run_pass"),
        (
            conditions.attestation_contract_and_source_head_match,
            "attestation_contract_lane_or_source_head_mismatch",
        ),
        (
            conditions.ttl_fresh,
            "ttl_expired_invalid_or_context_missing",
        ),
        (
            conditions.runner_class_contract_match,
            "runner_class_contract_mismatch",
        ),
        (
            conditions.revocation_observation_match,
            "revocation_observation_mismatch_or_context_missing",
        ),
        (
            conditions.baseline_permits,
            "baseline_reuse_disabled_or_digest_mismatch",
        ),
        (
            conditions.dynamic_input_unchanged,
            "dynamic_input_changed_or_context_missing",
        ),
    ] {
        if !satisfied {
            reasons.push(reason.to_string());
        }
    }
    if !context_present {
        reasons.push("reuse_observation_context_missing_or_invalid".into());
    }
    Ok(ReuseValidationReport {
        reference_valid: schema_valid
            && strict_core
            && attestation.lane == expected_lane
            && attestation.decision == "RUN_PASS",
        source_path: Some(reference.to_string()),
        source_digest: Some(format!("sha256:{:x}", Sha256::digest(&bytes))),
        structural_conditions: conditions,
        all_structural_predicates_satisfied: all_satisfied,
        reasons,
        candidate_observation_unverified: true,
        trusted_attestation_verified: false,
        runner_trust_verified: false,
        production_activation: false,
    })
}

fn load_observation(path: Option<&Path>) -> Result<ReuseObservationV1> {
    let path = path.context("reuse observation context is required")?;
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_FINGERPRINT_INPUT_BYTES {
        bail!("reuse observation must be a regular file no larger than 1 MiB");
    }
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut observation: ReuseObservationV1 =
        serde_json::from_slice(&bytes).context("parse ReuseObservationV1")?;
    if observation.schema_version != 1
        || observation.expected_lane.trim().is_empty()
        || !is_git_sha(&observation.expected_source_head_sha)
        || parse_rfc3339_epoch(&observation.now).is_none()
    {
        bail!("invalid ReuseObservationV1 identity/time fields");
    }
    observation
        .revoked_fingerprints
        .sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let unique = observation
        .revoked_fingerprints
        .iter()
        .map(Sha256Digest::as_str)
        .collect::<BTreeSet<_>>();
    if unique.len() != observation.revoked_fingerprints.len() {
        bail!("duplicate revoked fingerprint");
    }
    Ok(observation)
}

fn validate_reference_path(reference: &str) -> std::result::Result<(), String> {
    validate_utf8_text(reference, "reuse attestation reference")
        .map_err(|_| "untrusted_attestation_path".to_string())?;
    if reference.starts_with('/')
        || reference.ends_with('/')
        || reference.contains('\\')
        || reference
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err("untrusted_attestation_path".into());
    }
    Ok(())
}

fn validate_attestation_core(attestation: &LaneAttestationV1) -> Result<()> {
    if attestation.schema_version != 1
        || attestation.kind != "ci-lane-attestation"
        || attestation.run_id.trim().is_empty()
        || attestation.lane.trim().is_empty()
        || attestation.runner_class.trim().is_empty()
        || !is_git_sha(&attestation.base_sha)
        || !is_git_sha(&attestation.head_sha)
        || parse_attestation_times(attestation).is_none()
    {
        bail!("invalid lane attestation core");
    }
    for digest in [
        &attestation.plan_digest,
        &attestation.fingerprint,
        &attestation.runner_image_digest,
        &attestation.toolchain_digest,
        &attestation.result_digest,
    ] {
        Sha256Digest::parse(digest.as_str().to_string())?;
    }
    Ok(())
}

fn parse_attestation_times(attestation: &LaneAttestationV1) -> Option<(u64, u64)> {
    Some((
        parse_rfc3339_epoch(&attestation.started_at)?,
        parse_rfc3339_epoch(&attestation.finished_at)?,
    ))
}

fn parse_rfc3339_epoch(value: &str) -> Option<u64> {
    if !crate::schema_lite::is_rfc3339_utc(value) {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era
        .checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)?;
    let seconds = days_since_epoch
        .checked_mul(86_400)?
        .checked_add(hour.checked_mul(3_600)?)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)?;
    u64::try_from(seconds).ok()
}

fn is_git_sha(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;

    fn digest(character: char) -> Sha256Digest {
        Sha256Digest::parse(format!("sha256:{}", character.to_string().repeat(64))).unwrap()
    }

    fn valid_input() -> FingerprintInputV1 {
        FingerprintInputV1 {
            contract_version: 1,
            lane_id: "fast".into(),
            command: CommandInputV1 {
                program: "cargo".into(),
                argv: vec!["test".into(), "-p".into(), "xhyper-xtask".into()],
                cwd: CommandCwd::RepoRoot,
            },
            source_file_digests: vec![
                SourceFileDigestV1 {
                    repo_relative_posix_path: "tools/xtask/src/main.rs".into(),
                    digest: digest('a'),
                },
                SourceFileDigestV1 {
                    repo_relative_posix_path: "tools/xtask/src/ci/mod.rs".into(),
                    digest: digest('b'),
                },
            ],
            affected_dependency_closure: vec![
                CargoPackageIdentityV1 {
                    package: "xhyper-kernel".into(),
                    version: "0.1.0".into(),
                },
                CargoPackageIdentityV1 {
                    package: "xhyper-xtask".into(),
                    version: "0.1.0".into(),
                },
            ],
            cargo_lock_digest: digest('c'),
            toolchain_digest: digest('d'),
            tool_lock_digest: digest('e'),
            runner_image_digest: digest('f'),
            relevant_environment: vec![
                EnvironmentDigestV1 {
                    name: "CARGO_TARGET_DIR".into(),
                    value_digest: digest('1'),
                },
                EnvironmentDigestV1 {
                    name: "RUSTFLAGS".into(),
                    value_digest: digest('2'),
                },
            ],
            baseline_digest: digest('3'),
            plan_version: 1,
            evidence_schema_version: 1,
        }
    }

    fn report(input: FingerprintInputV1) -> FingerprintReport {
        fingerprint(input).unwrap()
    }

    #[test]
    fn canonical_bytes_and_digest_golden() {
        let mut input = valid_input();
        validate_and_normalize(&mut input).unwrap();
        let canonical = String::from_utf8(canonical_bytes(&input).unwrap()).unwrap();
        let expected = r#"{"affected_dependency_closure":[{"package":"xhyper-kernel","version":"0.1.0"},{"package":"xhyper-xtask","version":"0.1.0"}],"baseline_digest":"sha256:3333333333333333333333333333333333333333333333333333333333333333","cargo_lock_digest":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","command":{"argv":["test","-p","xhyper-xtask"],"cwd":"repo_root","program":"cargo"},"contract_version":1,"evidence_schema_version":1,"lane_id":"fast","plan_version":1,"relevant_environment":[{"name":"CARGO_TARGET_DIR","value_digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},{"name":"RUSTFLAGS","value_digest":"sha256:2222222222222222222222222222222222222222222222222222222222222222"}],"runner_image_digest":"sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff","source_file_digests":[{"digest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","repo_relative_posix_path":"tools/xtask/src/ci/mod.rs"},{"digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","repo_relative_posix_path":"tools/xtask/src/main.rs"}],"tool_lock_digest":"sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","toolchain_digest":"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#;
        assert_eq!(canonical, expected);
        assert_eq!(
            report(valid_input()).fingerprint,
            "sha256:573d2906be63e5520e7163cbfab51f7e897bce549fb4a5824a0289e36b11c33c"
        );
    }

    #[test]
    fn input_object_and_set_order_do_not_change_fingerprint() {
        let input = valid_input();
        let mut reordered = input.clone();
        reordered.source_file_digests.reverse();
        reordered.affected_dependency_closure.reverse();
        reordered.relevant_environment.reverse();
        assert_eq!(report(input).fingerprint, report(reordered).fingerprint);

        let mut value = serde_json::to_value(valid_input()).unwrap();
        let object = value.as_object_mut().unwrap();
        let lane = object.remove("lane_id").unwrap();
        object.insert("lane_id".into(), lane);
        let from_json = fingerprint_from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(from_json.fingerprint, report(valid_input()).fingerprint);
    }

    #[test]
    fn every_required_field_and_unknown_fields_fail_closed() {
        let fields = [
            "contract_version",
            "lane_id",
            "command",
            "source_file_digests",
            "affected_dependency_closure",
            "cargo_lock_digest",
            "toolchain_digest",
            "tool_lock_digest",
            "runner_image_digest",
            "relevant_environment",
            "baseline_digest",
            "plan_version",
            "evidence_schema_version",
        ];
        for field in fields {
            let mut value = serde_json::to_value(valid_input()).unwrap();
            value.as_object_mut().unwrap().remove(field);
            assert!(
                fingerprint_from_json(&serde_json::to_vec(&value).unwrap()).is_err(),
                "missing {field} must fail"
            );
        }
        let mut value = serde_json::to_value(valid_input()).unwrap();
        value["unknown"] = serde_json::json!(true);
        assert!(fingerprint_from_json(&serde_json::to_vec(&value).unwrap()).is_err());
        value = serde_json::to_value(valid_input()).unwrap();
        value["command"]["unknown"] = serde_json::json!(true);
        assert!(fingerprint_from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn duplicate_json_fields_fail_closed() {
        let raw = serde_json::to_string(&valid_input()).unwrap();
        let duplicate_top = raw.replacen(
            "\"lane_id\":\"fast\"",
            "\"lane_id\":\"fast\",\"lane_id\":\"other\"",
            1,
        );
        let duplicate_nested = raw.replacen(
            "\"program\":\"cargo\"",
            "\"program\":\"cargo\",\"program\":\"rustc\"",
            1,
        );
        assert!(fingerprint_from_json(duplicate_top.as_bytes()).is_err());
        assert!(fingerprint_from_json(duplicate_nested.as_bytes()).is_err());
    }

    #[test]
    fn empty_placeholder_and_version_values_fail_closed() {
        let mut cases = Vec::new();
        let mut input = valid_input();
        input.lane_id.clear();
        cases.push(input);
        let mut input = valid_input();
        input.command.program = "PLACEHOLDER".into();
        cases.push(input);
        let mut input = valid_input();
        input.command.argv.clear();
        cases.push(input);
        let mut input = valid_input();
        input.source_file_digests.clear();
        cases.push(input);
        let mut input = valid_input();
        input.affected_dependency_closure.clear();
        cases.push(input);
        let mut input = valid_input();
        input.relevant_environment.clear();
        cases.push(input);
        let mut input = valid_input();
        input.plan_version = 0;
        cases.push(input);
        let mut input = valid_input();
        input.evidence_schema_version = 0;
        cases.push(input);
        let mut input = valid_input();
        input.contract_version = 2;
        cases.push(input);
        for input in cases {
            assert!(fingerprint(input).is_err());
        }
    }

    #[test]
    fn digest_contract_is_strict() {
        for invalid in [
            "",
            "PLACEHOLDER",
            "sha1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sha256:ABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCD",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256:short",
        ] {
            assert!(Sha256Digest::parse(invalid.into()).is_err(), "{invalid}");
        }
    }

    #[test]
    fn invalid_paths_identities_and_logical_duplicates_fail() {
        for path in [
            "/absolute",
            "../escape",
            "a/./b",
            "a//b",
            "a\\b",
            "a/",
            "a/PLACEHOLDER",
        ] {
            let mut input = valid_input();
            input.source_file_digests[0].repo_relative_posix_path = path.into();
            assert!(fingerprint(input).is_err(), "{path}");
        }
        let mut input = valid_input();
        input.source_file_digests[1].repo_relative_posix_path = input.source_file_digests[0]
            .repo_relative_posix_path
            .clone();
        assert!(fingerprint(input).is_err());
        let mut input = valid_input();
        input.affected_dependency_closure[1] = input.affected_dependency_closure[0].clone();
        assert!(fingerprint(input).is_err());
        let mut input = valid_input();
        input.relevant_environment[1].name = input.relevant_environment[0].name.clone();
        assert!(fingerprint(input).is_err());
        let mut input = valid_input();
        input.relevant_environment[0].name = "lowercase".into();
        assert!(fingerprint(input).is_err());
    }

    #[test]
    fn every_spec_input_class_changes_the_digest() {
        let baseline = report(valid_input()).fingerprint;
        let mut mutations = Vec::new();
        let mut input = valid_input();
        input.lane_id = "build_test".into();
        mutations.push(input);
        let mut input = valid_input();
        input.command.argv[0] = "check".into();
        mutations.push(input);
        let mut input = valid_input();
        input.source_file_digests[0].digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.affected_dependency_closure[0].version = "0.1.1".into();
        mutations.push(input);
        let mut input = valid_input();
        input.cargo_lock_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.toolchain_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.tool_lock_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.runner_image_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.relevant_environment[0].value_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.baseline_digest = digest('4');
        mutations.push(input);
        let mut input = valid_input();
        input.plan_version = 2;
        mutations.push(input);
        let mut input = valid_input();
        input.evidence_schema_version = 2;
        mutations.push(input);
        for input in mutations {
            assert_ne!(report(input).fingerprint, baseline);
        }
    }

    #[test]
    fn argv_boundaries_are_part_of_the_fingerprint() {
        let mut joined = valid_input();
        joined.command.argv = vec!["test -p".into(), "xhyper-xtask".into()];
        assert_ne!(
            report(joined).fingerprint,
            report(valid_input()).fingerprint
        );
    }

    #[test]
    fn repository_utf8_paths_and_argv_are_preserved_without_normalization() {
        let mut utf8 = valid_input();
        utf8.source_file_digests[0].repo_relative_posix_path =
            ".agent/SSOT/tools/goalctl/goal/Goal-终态目标.md".into();
        utf8.command.argv.push("终态规范".into());
        let utf8_fingerprint = report(utf8).fingerprint;

        let mut composed = valid_input();
        composed.command.argv.push("ca".to_owned() + "f\u{e9}");
        let mut decomposed = valid_input();
        decomposed.command.argv.push("ca".to_owned() + "fe\u{301}");
        assert_ne!(report(composed).fingerprint, report(decomposed).fingerprint);
        assert_ne!(utf8_fingerprint, report(valid_input()).fingerprint);
    }

    #[test]
    fn report_does_not_echo_sensitive_inputs() {
        let mut input = valid_input();
        input.command.argv.push("SECRET_TOKEN".into());
        input.relevant_environment[0].name = "SECRET_TOKEN".into();
        let serialized = serde_json::to_string(&report(input)).unwrap();
        assert!(!serialized.contains("SECRET_TOKEN"));
        assert!(!serialized.contains("xhyper-xtask"));
        assert!(serialized.contains("provenance_unverified"));
        assert!(serialized.contains("\"reusable\":false"));
    }

    #[test]
    fn oversized_input_fails_before_json_parse() {
        let bytes = vec![b' '; MAX_FINGERPRINT_INPUT_BYTES as usize + 1];
        assert!(fingerprint_from_json(&bytes).is_err());
    }

    struct ReuseFixture {
        dir: tempfile::TempDir,
        attestation: serde_json::Value,
        observation: serde_json::Value,
        baseline_enabled: bool,
        lane_reusable: bool,
        runner_class: String,
        ttl_seconds: u64,
    }

    impl ReuseFixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            fs::create_dir_all(dir.path().join(".github/ci")).unwrap();
            fs::create_dir_all(dir.path().join("evidence/schemas/ci")).unwrap();
            fs::create_dir_all(dir.path().join("attestations")).unwrap();
            fs::copy(
                repo_root_from_manifest().join("evidence/schemas/ci/lane-attestation.schema.json"),
                dir.path()
                    .join("evidence/schemas/ci/lane-attestation.schema.json"),
            )
            .unwrap();
            let mut fixture = Self {
                dir,
                attestation: serde_json::json!({
                    "schema_version": 1,
                    "type": "ci-lane-attestation",
                    "run_id": "source-run",
                    "lane": "fast",
                    "decision": "RUN_PASS",
                    "base_sha": "a".repeat(40),
                    "head_sha": "b".repeat(40),
                    "plan_digest": format!("sha256:{}", "c".repeat(64)),
                    "fingerprint": format!("sha256:{}", "d".repeat(64)),
                    "runner_class": "fast",
                    "runner_image_digest": format!("sha256:{}", "e".repeat(64)),
                    "toolchain_digest": format!("sha256:{}", "f".repeat(64)),
                    "started_at": "2026-07-17T00:00:00Z",
                    "finished_at": "2026-07-17T00:01:00Z",
                    "result_digest": format!("sha256:{}", "1".repeat(64)),
                    "reuse_inputs": {
                        "baseline_raw_sha256": format!("sha256:{}", "2".repeat(64)),
                        "dynamic_input_digest": format!("sha256:{}", "3".repeat(64)),
                        "revocation_registry_digest": format!("sha256:{}", "4".repeat(64))
                    }
                }),
                observation: serde_json::json!({
                    "schema_version": 1,
                    "expected_lane": "fast",
                    "expected_fingerprint": format!("sha256:{}", "d".repeat(64)),
                    "expected_source_head_sha": "b".repeat(40),
                    "now": "2026-07-17T00:01:50Z",
                    "current_dynamic_input_digest": format!("sha256:{}", "3".repeat(64)),
                    "current_revocation_registry_digest": format!("sha256:{}", "4".repeat(64)),
                    "revoked_fingerprints": []
                }),
                baseline_enabled: true,
                lane_reusable: true,
                runner_class: "fast".into(),
                ttl_seconds: 100,
            };
            fixture.write();
            fixture
        }

        fn write(&mut self) {
            let baseline = format!(
                "[reuse]\nenabled = {}\n\n[lanes.fast]\nreusable = {}\nrunner_class = \"{}\"\nttl_seconds = {}\n",
                self.baseline_enabled,
                self.lane_reusable,
                self.runner_class,
                self.ttl_seconds
            );
            fs::write(self.dir.path().join(".github/ci/baseline.toml"), &baseline).unwrap();
            if let Some(reuse_inputs) = self.attestation.get_mut("reuse_inputs") {
                reuse_inputs["baseline_raw_sha256"] =
                    serde_json::json!(format!("sha256:{:x}", Sha256::digest(baseline.as_bytes())));
            }
            fs::write(
                self.dir.path().join("attestations/source.json"),
                serde_json::to_vec_pretty(&self.attestation).unwrap(),
            )
            .unwrap();
            fs::write(
                self.dir.path().join("context.json"),
                serde_json::to_vec_pretty(&self.observation).unwrap(),
            )
            .unwrap();
        }

        fn validate(&mut self) -> ReuseValidationReport {
            self.write();
            validate_reuse_reference(
                self.dir.path(),
                Some(&self.dir.path().join("attestations")),
                Some("source.json"),
                "fast",
                Some(&self.dir.path().join("context.json")),
            )
        }
    }

    #[test]
    fn all_structural_reuse_predicates_can_match_without_activating_reuse() {
        let report = ReuseFixture::new().validate();
        assert!(report.reference_valid, "{:?}", report.reasons);
        assert!(report.all_structural_predicates_satisfied, "{report:?}");
        assert!(report.candidate_observation_unverified);
        assert!(!report.trusted_attestation_verified);
        assert!(!report.runner_trust_verified);
        assert!(!report.production_activation);
    }

    #[test]
    fn each_spec_reuse_predicate_fails_closed_independently() {
        let mut fixture = ReuseFixture::new();
        fixture.lane_reusable = false;
        assert!(!fixture.validate().structural_conditions.lane_reusable);

        let mut fixture = ReuseFixture::new();
        fixture.observation["expected_fingerprint"] =
            serde_json::json!(format!("sha256:{}", "5".repeat(64)));
        assert!(!fixture.validate().structural_conditions.fingerprint_exact);

        let mut fixture = ReuseFixture::new();
        fixture.attestation["decision"] = serde_json::json!("RUN_FAIL");
        assert!(!fixture.validate().structural_conditions.source_run_pass);

        let mut fixture = ReuseFixture::new();
        fixture.observation["expected_source_head_sha"] = serde_json::json!("c".repeat(40));
        assert!(
            !fixture
                .validate()
                .structural_conditions
                .attestation_contract_and_source_head_match
        );

        let mut fixture = ReuseFixture::new();
        fixture.observation["now"] = serde_json::json!("2026-07-17T00:03:00Z");
        assert!(!fixture.validate().structural_conditions.ttl_fresh);

        let mut fixture = ReuseFixture::new();
        fixture.attestation["runner_class"] = serde_json::json!("build");
        assert!(
            !fixture
                .validate()
                .structural_conditions
                .runner_class_contract_match
        );

        let mut fixture = ReuseFixture::new();
        fixture.observation["current_revocation_registry_digest"] =
            serde_json::json!(format!("sha256:{}", "5".repeat(64)));
        assert!(
            !fixture
                .validate()
                .structural_conditions
                .revocation_observation_match
        );

        let mut fixture = ReuseFixture::new();
        fixture.baseline_enabled = false;
        assert!(!fixture.validate().structural_conditions.baseline_permits);

        let mut fixture = ReuseFixture::new();
        fixture.observation["current_dynamic_input_digest"] =
            serde_json::json!(format!("sha256:{}", "5".repeat(64)));
        assert!(
            !fixture
                .validate()
                .structural_conditions
                .dynamic_input_unchanged
        );
    }

    #[test]
    fn reuse_ttl_boundaries_invalid_time_and_overflow_fail_closed() {
        let mut boundary = ReuseFixture::new();
        boundary.observation["now"] = serde_json::json!("2026-07-17T00:02:40Z");
        assert!(boundary.validate().structural_conditions.ttl_fresh);

        let mut expired = ReuseFixture::new();
        expired.observation["now"] = serde_json::json!("2026-07-17T00:02:41Z");
        assert!(!expired.validate().structural_conditions.ttl_fresh);

        let mut future = ReuseFixture::new();
        future.observation["now"] = serde_json::json!("2026-07-17T00:00:59Z");
        assert!(!future.validate().structural_conditions.ttl_fresh);

        let mut invalid = ReuseFixture::new();
        invalid.observation["now"] = serde_json::json!("2026-02-31T00:00:00Z");
        assert!(!invalid.validate().structural_conditions.ttl_fresh);

        let mut overflow = ReuseFixture::new();
        overflow.ttl_seconds = u64::MAX;
        assert!(!overflow.validate().structural_conditions.ttl_fresh);
    }

    #[test]
    fn optional_reuse_inputs_preserve_v1_but_cannot_match_reuse_contract() {
        let mut fixture = ReuseFixture::new();
        fixture
            .attestation
            .as_object_mut()
            .unwrap()
            .remove("reuse_inputs");
        let report = fixture.validate();
        assert!(report.reference_valid);
        assert!(
            !report
                .structural_conditions
                .attestation_contract_and_source_head_match
        );
        assert!(!report.all_structural_predicates_satisfied);
    }

    #[test]
    fn malformed_reuse_extension_and_context_fail_closed() {
        let mut unknown = ReuseFixture::new();
        unknown.attestation["reuse_inputs"]["unknown"] = serde_json::json!(true);
        assert!(!unknown.validate().reference_valid);

        let mut placeholder = ReuseFixture::new();
        placeholder.attestation["reuse_inputs"]["dynamic_input_digest"] =
            serde_json::json!("sha256:PLACEHOLDER_DYNAMIC");
        assert!(!placeholder.validate().reference_valid);

        let mut missing_head = ReuseFixture::new();
        missing_head
            .observation
            .as_object_mut()
            .unwrap()
            .remove("expected_source_head_sha");
        missing_head.write();
        let report = evaluate_reuse(
            missing_head.dir.path(),
            true,
            Some(&missing_head.dir.path().join("attestations")),
            Some("source.json"),
            Some(&missing_head.dir.path().join("context.json")),
        )
        .unwrap();
        assert_eq!(report.decision, "RUN");
        assert!(!report.validation.unwrap().reference_valid);
    }

    #[test]
    fn revoked_duplicate_or_missing_context_never_matches() {
        let mut revoked = ReuseFixture::new();
        revoked.observation["revoked_fingerprints"] =
            serde_json::json!([format!("sha256:{}", "d".repeat(64))]);
        assert!(
            !revoked
                .validate()
                .structural_conditions
                .revocation_observation_match
        );

        let mut duplicate = ReuseFixture::new();
        let fingerprint = format!("sha256:{}", "d".repeat(64));
        duplicate.observation["revoked_fingerprints"] =
            serde_json::json!([fingerprint, fingerprint]);
        assert!(!duplicate.validate().all_structural_predicates_satisfied);

        let fixture = ReuseFixture::new();
        let report = validate_reuse_reference(
            fixture.dir.path(),
            Some(&fixture.dir.path().join("attestations")),
            Some("source.json"),
            "fast",
            None,
        );
        assert!(report.reference_valid);
        assert!(!report.all_structural_predicates_satisfied);
        assert!(report
            .reasons
            .iter()
            .any(|reason| reason.contains("context_missing")));
    }

    #[test]
    fn reuse_default_off_forces_run() {
        let root = repo_root_from_manifest();
        let report = evaluate_reuse(&root, true, None, None, None).unwrap();
        assert!(!report.reuse_enabled);
        assert_eq!(report.decision, "RUN");
    }
}
