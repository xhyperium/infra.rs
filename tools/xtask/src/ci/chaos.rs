//! Spec §26 manifest + isolated negative driver runner（PHASE-5-07）。

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::aggregate_dry;
use super::domain_gates;
use super::drift;
use super::flake;
use super::locks;
use super::verify_runner::{self, ObservedTool, RunnerObserved};

const MANIFEST_REL: &str = "tools/xtask/tests/ci_negative/manifest.toml";
const FIXTURE_ROOT_REL: &str = "tools/xtask/tests/ci_negative/fixtures";

const CANONICAL_CASES: [(&str, &str); 20] = [
    ("spec26-01-missing-lane", "Missing Lane"),
    ("spec26-02-cancelled-lane", "Cancelled Lane"),
    ("spec26-03-unexpected-skip", "Unexpected Skip"),
    ("spec26-04-invalid-na", "Invalid N/A"),
    (
        "spec26-05-invalid-reused-attestation",
        "Invalid Reused Attestation",
    ),
    ("spec26-06-fingerprint-mismatch", "Fingerprint Mismatch"),
    ("spec26-07-runner-digest-mismatch", "Runner Digest Mismatch"),
    ("spec26-08-tool-version-mismatch", "Tool Version Mismatch"),
    ("spec26-09-disk-insufficient", "Disk Insufficient"),
    ("spec26-10-planner-unknown", "Planner Unknown"),
    (
        "spec26-11-cargo-graph-parse-failure",
        "Cargo Graph Parse Failure",
    ),
    ("spec26-12-merge-group-event", "Merge Group Event"),
    ("spec26-13-fork-pr", "Fork PR"),
    ("spec26-14-github-api-429-5xx", "GitHub API 429/5xx"),
    ("spec26-15-generated-drift", "Generated Drift"),
    ("spec26-16-ruleset-drift", "Ruleset Drift"),
    ("spec26-17-cache-corruption", "Cache Corruption"),
    (
        "spec26-18-external-cleanup-failure",
        "External Cleanup Failure",
    ),
    ("spec26-19-flake-expiry", "Flake Expiry"),
    (
        "spec26-20-aggregate-unknown-state",
        "Aggregate Unknown State",
    ),
];

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Maturity {
    Executable,
    Stub,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum LogicalOutcome {
    Rejected,
    NotRun,
}

impl LogicalOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FinalState {
    Fail,
    InfraFailure,
    Drift,
    NotRun,
}

impl FinalState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fail => "FAIL",
            Self::InfraFailure => "INFRA_FAILURE",
            Self::Drift => "DRIFT",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum DriverId {
    AggregateMissingLane,
    AggregateCancelledLane,
    AggregateUnexpectedSkip,
    AggregateInvalidNa,
    AggregateInvalidReused,
    RunnerDigestMismatch,
    ToolVersionMismatch,
    RunnerDiskInsufficient,
    GeneratedDrift,
    FlakeExpiry,
    AggregateUnknownState,
}

impl DriverId {
    fn spec_id(self) -> &'static str {
        match self {
            Self::AggregateMissingLane => "spec26-01-missing-lane",
            Self::AggregateCancelledLane => "spec26-02-cancelled-lane",
            Self::AggregateUnexpectedSkip => "spec26-03-unexpected-skip",
            Self::AggregateInvalidNa => "spec26-04-invalid-na",
            Self::AggregateInvalidReused => "spec26-05-invalid-reused-attestation",
            Self::RunnerDigestMismatch => "spec26-07-runner-digest-mismatch",
            Self::ToolVersionMismatch => "spec26-08-tool-version-mismatch",
            Self::RunnerDiskInsufficient => "spec26-09-disk-insufficient",
            Self::GeneratedDrift => "spec26-15-generated-drift",
            Self::FlakeExpiry => "spec26-19-flake-expiry",
            Self::AggregateUnknownState => "spec26-20-aggregate-unknown-state",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::AggregateMissingLane => "aggregate_missing_lane",
            Self::AggregateCancelledLane => "aggregate_cancelled_lane",
            Self::AggregateUnexpectedSkip => "aggregate_unexpected_skip",
            Self::AggregateInvalidNa => "aggregate_invalid_na",
            Self::AggregateInvalidReused => "aggregate_invalid_reused",
            Self::RunnerDigestMismatch => "runner_digest_mismatch",
            Self::ToolVersionMismatch => "tool_version_mismatch",
            Self::RunnerDiskInsufficient => "runner_disk_insufficient",
            Self::GeneratedDrift => "generated_drift",
            Self::FlakeExpiry => "flake_expiry",
            Self::AggregateUnknownState => "aggregate_unknown_state",
        }
    }

    fn target_seam(self) -> &'static str {
        match self {
            Self::AggregateMissingLane
            | Self::AggregateCancelledLane
            | Self::AggregateUnexpectedSkip
            | Self::AggregateInvalidNa
            | Self::AggregateInvalidReused
            | Self::AggregateUnknownState => "ci::aggregate_dry",
            Self::RunnerDigestMismatch | Self::RunnerDiskInsufficient => {
                "ci::verify_runner::verify_runner"
            }
            Self::ToolVersionMismatch => "ci::locks::check_locks",
            Self::GeneratedDrift => "ci::drift::check_generated_drift",
            Self::FlakeExpiry => "ci::flake::check_flake_registry_text",
        }
    }

    fn canonical_fixture(self) -> Option<&'static str> {
        match self {
            Self::AggregateMissingLane => {
                Some("tools/xtask/tests/ci_negative/fixtures/missing_lane.json")
            }
            Self::AggregateCancelledLane => {
                Some("tools/xtask/tests/ci_negative/fixtures/cancelled_lane.json")
            }
            Self::AggregateUnexpectedSkip => {
                Some("tools/xtask/tests/ci_negative/fixtures/unexpected_skip.json")
            }
            Self::AggregateInvalidNa => {
                Some("tools/xtask/tests/ci_negative/fixtures/invalid_na.json")
            }
            Self::AggregateInvalidReused => {
                Some("tools/xtask/tests/ci_negative/fixtures/missing_reused_attestation.json")
            }
            Self::FlakeExpiry => Some("tools/xtask/tests/ci_negative/fixtures/flake_expired.toml"),
            Self::AggregateUnknownState => {
                Some("tools/xtask/tests/ci_negative/fixtures/aggregate_unknown_state.json")
            }
            Self::RunnerDigestMismatch
            | Self::ToolVersionMismatch
            | Self::RunnerDiskInsufficient
            | Self::GeneratedDrift => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NegativeManifest {
    schema_version: u32,
    spec_section: u32,
    #[serde(rename = "case")]
    cases: Vec<ManifestCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestCase {
    id: String,
    spec_name: String,
    maturity: Maturity,
    target_seam: String,
    driver: Option<DriverId>,
    fixture: Option<String>,
    expected_logical_outcome: LogicalOutcome,
    expected_final_state: FinalState,
    notes: String,
}

#[derive(Debug, Serialize)]
pub struct ChaosCaseResult {
    pub id: String,
    pub spec_name: String,
    pub maturity: String,
    pub target_seam: String,
    pub driver: Option<String>,
    pub fixture: Option<String>,
    pub expected_logical_outcome: String,
    pub expected_final_state: String,
    pub observed_logical_outcome: Option<String>,
    pub observed_final_state: Option<String>,
    pub executed: bool,
    pub status: String,
    /// 兼容旧 report 字段；STUB 恒 false，不能冒充负向 PASS。
    pub ok_expected_fail: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct SupplementalCaseResult {
    pub id: String,
    pub ok_expected_fail: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ChaosReport {
    pub ok: bool,
    pub gate_ok: bool,
    pub coverage_complete: bool,
    pub mode: &'static str,
    pub executable_count: usize,
    pub stub_count: usize,
    pub cases: Vec<ChaosCaseResult>,
    pub supplemental_cases: Vec<SupplementalCaseResult>,
    pub note: String,
}

#[derive(Debug)]
struct DriverOutcome {
    logical_outcome: LogicalOutcome,
    final_state: FinalState,
    detail: String,
}

/// 跑 manifest 中全部 EXECUTABLE；STUB 只如实报告，不视为 PASS。
pub fn run_negative_subset(root: &Path) -> Result<ChaosReport> {
    let manifest = load_and_validate_manifest(root)?;
    let mut cases = Vec::with_capacity(manifest.cases.len());
    let mut gate_ok = true;

    for case in manifest.cases {
        match case.maturity {
            Maturity::Stub => cases.push(stub_result(case)),
            Maturity::Executable => {
                let outcome = run_driver(root, &case);
                let result = match outcome {
                    Ok(observed) => {
                        let matches = observed.logical_outcome == case.expected_logical_outcome
                            && observed.final_state == case.expected_final_state;
                        gate_ok &= matches;
                        executable_result(case, observed, matches)
                    }
                    Err(error) => {
                        gate_ok = false;
                        driver_error_result(case, error)
                    }
                };
                cases.push(result);
            }
        }
    }

    let executable_count = cases.iter().filter(|case| case.executed).count();
    let stub_count = cases.len() - executable_count;
    let supplemental_cases = run_supplemental_cases(root)?;
    gate_ok &= supplemental_cases.iter().all(|case| case.ok_expected_fail);
    let report = ChaosReport {
        ok: gate_ok,
        gate_ok,
        coverage_complete: stub_count == 0,
        mode: "shadow",
        executable_count,
        stub_count,
        cases,
        supplemental_cases,
        note: "Spec §26 manifest gate; STUB is visible and not PASS; not 20/20 production chaos"
            .into(),
    };
    if !report.gate_ok {
        let mut failed = report
            .cases
            .iter()
            .filter(|case| case.executed && case.status != "PASS")
            .map(|case| case.id.clone())
            .collect::<Vec<_>>();
        failed.extend(
            report
                .supplemental_cases
                .iter()
                .filter(|case| !case.ok_expected_fail)
                .map(|case| case.id.clone()),
        );
        bail!("ci chaos: executable negative drivers failed: {failed:?}");
    }
    Ok(report)
}

fn load_and_validate_manifest(root: &Path) -> Result<NegativeManifest> {
    let path = root.join(MANIFEST_REL);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    validate_manifest_text(root, &raw)
}

fn validate_manifest_text(root: &Path, raw: &str) -> Result<NegativeManifest> {
    let manifest: NegativeManifest = toml::from_str(raw).context("parse Spec §26 manifest")?;
    if manifest.schema_version != 1 || manifest.spec_section != 26 {
        bail!("Spec §26 manifest header mismatch");
    }
    if manifest.cases.len() != CANONICAL_CASES.len() {
        bail!(
            "Spec §26 manifest must contain exactly {} cases",
            CANONICAL_CASES.len()
        );
    }
    let mut seen = BTreeSet::new();
    let fixture_root = root
        .join(FIXTURE_ROOT_REL)
        .canonicalize()
        .context("canonicalize CI negative fixture root")?;

    for (index, (case, (canonical_id, canonical_name))) in manifest
        .cases
        .iter()
        .zip(CANONICAL_CASES.iter())
        .enumerate()
    {
        if case.id != *canonical_id || case.spec_name != *canonical_name {
            bail!(
                "Spec §26 case {} identity/order mismatch: got {}/{}",
                index + 1,
                case.id,
                case.spec_name
            );
        }
        if !seen.insert(case.id.as_str()) {
            bail!("duplicate Spec §26 id: {}", case.id);
        }
        if case.target_seam.trim().is_empty() || case.notes.trim().is_empty() {
            bail!("Spec §26 case {} has empty seam/notes", case.id);
        }
        match case.maturity {
            Maturity::Stub => {
                if case.driver.is_some()
                    || case.fixture.is_some()
                    || case.expected_logical_outcome != LogicalOutcome::NotRun
                    || case.expected_final_state != FinalState::NotRun
                {
                    bail!(
                        "STUB {} must use NONE/NOT_RUN and cannot fake execution",
                        case.id
                    );
                }
            }
            Maturity::Executable => {
                let driver = case
                    .driver
                    .context("EXECUTABLE manifest case missing driver")?;
                if case.id != driver.spec_id() {
                    bail!("driver/case identity mismatch for {}", case.id);
                }
                if case.target_seam != driver.target_seam() {
                    bail!("driver/seam mismatch for {}", case.id);
                }
                if case.expected_logical_outcome != LogicalOutcome::Rejected
                    || case.expected_final_state == FinalState::NotRun
                {
                    bail!("EXECUTABLE {} has invalid expected outcome", case.id);
                }
                match (&case.fixture, driver.canonical_fixture()) {
                    (Some(relative), Some(canonical)) if relative == canonical => {
                        validate_fixture(root, &fixture_root, relative)?;
                    }
                    (None, None) => {}
                    _ => bail!("fixture/driver contract mismatch for {}", case.id),
                }
            }
        }
    }
    Ok(manifest)
}

fn validate_fixture(root: &Path, fixture_root: &Path, relative: &str) -> Result<()> {
    if relative.starts_with('/')
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        bail!("untrusted fixture path: {relative}");
    }
    let candidate = root
        .join(relative)
        .canonicalize()
        .with_context(|| format!("canonicalize fixture {relative}"))?;
    if !candidate.starts_with(fixture_root) || !candidate.is_file() {
        bail!("fixture escapes trusted root or is not a file: {relative}");
    }
    Ok(())
}

fn run_driver(root: &Path, case: &ManifestCase) -> Result<DriverOutcome> {
    let driver = case.driver.context("validated executable driver")?;
    match driver {
        DriverId::AggregateMissingLane => aggregate_driver(root, case, |report| {
            report.missing_lanes.iter().any(|lane| lane == "build_test")
        }),
        DriverId::AggregateCancelledLane => aggregate_driver(root, case, |report| {
            report.decisions["fast"]["decision"] == "CANCELLED"
        }),
        DriverId::AggregateUnexpectedSkip => aggregate_driver(root, case, |report| {
            report.decisions["fast"]["decision"] == "UNKNOWN"
                && report.decisions["fast"]["reason"] == "unknown_or_invalid_decision"
        }),
        DriverId::AggregateInvalidNa => aggregate_driver(root, case, |report| {
            report.decisions["build_test"]["reason"] == "missing_not_applicable_reason"
        }),
        DriverId::AggregateInvalidReused => aggregate_driver(root, case, |report| {
            report.decisions["fast"]["reason"] == "missing_reused_attestation"
        }),
        DriverId::AggregateUnknownState => aggregate_driver(root, case, |report| {
            report.decisions["fast"]["decision"] == "UNKNOWN"
        }),
        DriverId::RunnerDigestMismatch => runner_mutation_driver(true),
        DriverId::RunnerDiskInsufficient => runner_mutation_driver(false),
        DriverId::ToolVersionMismatch => tool_version_driver(),
        DriverId::GeneratedDrift => generated_drift_driver(root),
        DriverId::FlakeExpiry => flake_expiry_driver(root, case),
    }
}

fn aggregate_driver<F>(root: &Path, case: &ManifestCase, specific: F) -> Result<DriverOutcome>
where
    F: FnOnce(&super::AggregateReport) -> bool,
{
    let fixture = root.join(case.fixture.as_deref().context("aggregate fixture")?);
    let report = aggregate_dry(root, Some(&fixture), Some("fast,build_test"), false)?;
    if report.final_decision != "FAIL" || !specific(&report) {
        bail!("aggregate driver did not observe scenario-specific failure");
    }
    Ok(DriverOutcome {
        logical_outcome: LogicalOutcome::Rejected,
        final_state: FinalState::Fail,
        detail: format!(
            "final_decision={} missing={:?} decisions={:?}",
            report.final_decision, report.missing_lanes, report.decisions
        ),
    })
}

fn flake_expiry_driver(root: &Path, case: &ManifestCase) -> Result<DriverOutcome> {
    let fixture = root.join(case.fixture.as_deref().context("flake fixture")?);
    let raw = fs::read_to_string(&fixture)?;
    let report = flake::check_flake_registry_text(&raw, "2026-07-16")?;
    if report.ok || report.expired.is_empty() || !report.retry_allowances.is_empty() {
        bail!("flake driver did not isolate expired registry block");
    }
    Ok(DriverOutcome {
        logical_outcome: LogicalOutcome::Rejected,
        final_state: FinalState::Fail,
        detail: format!(
            "expired={:?} retry_mode={}",
            report.expired, report.retry_mode
        ),
    })
}

fn runner_mutation_driver(digest_mutation: bool) -> Result<DriverOutcome> {
    let scratch = ScratchRoot::new("runner")?;
    write_runner_contract(scratch.path())?;
    let mut observed = valid_runner_observation();
    let control = verify_runner::verify_runner(scratch.path(), &observed)?;
    if !control.ok || !control.failures.is_empty() {
        bail!("runner positive control failed: {:?}", control.failures);
    }
    if digest_mutation {
        observed.image_digest = format!("sha256:{}", "9".repeat(64));
    } else {
        observed.disk_free_gib = 19;
    }
    let report = verify_runner::verify_runner(scratch.path(), &observed)?;
    let expected_prefix = if digest_mutation {
        "digest_mismatch:"
    } else {
        "disk_free_gib_insufficient:"
    };
    if report.status != "INFRA_FAILURE"
        || report.failures.len() != 1
        || !report.failures[0].starts_with(expected_prefix)
    {
        bail!("runner mutation was not isolated: {:?}", report.failures);
    }
    Ok(DriverOutcome {
        logical_outcome: LogicalOutcome::Rejected,
        final_state: FinalState::InfraFailure,
        detail: report.failures[0].clone(),
    })
}

fn write_runner_contract(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join(".github/ci"))?;
    fs::write(
        root.join(".github/ci/runners.toml"),
        format!(
            r#"[runner.fast]
labels = ["self-hosted", "xhyper-ci-fast-v1"]
forbidden_labels = ["linux", "x64"]
trust_domain = "pr-untrusted"
cpu_min = 4
memory_gib_min = 8
disk_free_gib_min = 20
image_digest = "sha256:{}"
toolchain = "1.94.1"
native_dependencies = ["bash", "git", "python3"]
tool_binary_root = "/opt/xhyper/bin"
attestation_path = "/etc/xhyper/runner-attestation.json"
"#,
            "8".repeat(64)
        ),
    )?;
    fs::write(
        root.join(".github/ci/tools.lock.toml"),
        valid_tools_lock(true),
    )?;
    Ok(())
}

fn valid_runner_observation() -> RunnerObserved {
    RunnerObserved {
        runner_class: "fast".into(),
        image_digest: format!("sha256:{}", "8".repeat(64)),
        trust_domain: "pr-untrusted".into(),
        labels: vec!["self-hosted".into(), "xhyper-ci-fast-v1".into()],
        cpu_count: 4,
        memory_gib: 8,
        disk_free_gib: 20,
        toolchain_version: "1.94.1".into(),
        tools: tool_observations(),
        native_dependencies: vec!["bash".into(), "git".into(), "python3".into()],
        observation_source: "isolated-chaos-positive-control".into(),
    }
}

fn tool_observations() -> Vec<ObservedTool> {
    [
        ("nextest", "0.9.140", 'a'),
        ("cargo-deny", "0.20.2", 'b'),
        ("cargo-machete", "0.9.2", 'c'),
        ("cargo-llvm-cov", "0.8.7", 'd'),
    ]
    .into_iter()
    .map(|(name, version, digest)| ObservedTool {
        name: name.into(),
        version: version.into(),
        binary_sha256: format!("sha256:{}", digest.to_string().repeat(64)),
    })
    .collect()
}

fn valid_tools_lock(correct_nextest: bool) -> String {
    let nextest = if correct_nextest { "0.9.140" } else { "0.0.0" };
    format!(
        r#"schema_version = 1
[tool.nextest]
version = "{nextest}"
binary_sha256 = "sha256:{}"
[tool.cargo-deny]
version = "0.20.2"
binary_sha256 = "sha256:{}"
[tool.cargo-machete]
version = "0.9.2"
binary_sha256 = "sha256:{}"
[tool.cargo-llvm-cov]
version = "0.8.7"
binary_sha256 = "sha256:{}"
"#,
        "a".repeat(64),
        "b".repeat(64),
        "c".repeat(64),
        "d".repeat(64)
    )
}

fn tool_version_driver() -> Result<DriverOutcome> {
    let scratch = ScratchRoot::new("locks")?;
    fs::create_dir_all(scratch.path().join(".github/ci"))?;
    fs::write(
        scratch.path().join(".github/ci/toolchains.lock.toml"),
        "schema_version = 1\nprimary = \"1.94.1\"\nmsrv = \"1.94.1\"\n",
    )?;
    fs::write(
        scratch.path().join(".github/ci/tools.lock.toml"),
        valid_tools_lock(true),
    )?;
    let control = locks::check_locks(scratch.path())?;
    if !control.ok || !control.issues.is_empty() {
        bail!("locks positive control failed: {:?}", control.issues);
    }
    fs::write(
        scratch.path().join(".github/ci/tools.lock.toml"),
        valid_tools_lock(false),
    )?;
    let report = locks::check_locks(scratch.path())?;
    if report.issues.len() != 1
        || !report.issues[0].starts_with("tools_lock_install_mismatch:nextest:")
    {
        bail!(
            "tool version mutation was not isolated: {:?}",
            report.issues
        );
    }
    Ok(DriverOutcome {
        logical_outcome: LogicalOutcome::Rejected,
        final_state: FinalState::Fail,
        detail: report.issues[0].clone(),
    })
}

fn generated_drift_driver(repo_root: &Path) -> Result<DriverOutcome> {
    let scratch = ScratchRoot::new("drift")?;
    fs::create_dir_all(scratch.path().join(".github/ci"))?;
    fs::copy(
        repo_root.join(".github/ci/baseline.toml"),
        scratch.path().join(".github/ci/baseline.toml"),
    )?;
    let generated = scratch.path().join(".github/ci/generated");
    let rendered = super::render_to(scratch.path(), &generated)?;
    if !rendered.ok {
        bail!("drift positive control render failed");
    }
    let control = drift::check_generated_drift(scratch.path())?;
    if !control.ok || control.status != "MATCH" {
        bail!("drift positive control was not MATCH: {:?}", control.drifts);
    }
    let contract = generated.join("workflow-contract.json");
    let mut body = fs::read_to_string(&contract)?;
    body.push('\n');
    fs::write(&contract, body)?;
    let report = drift::check_generated_drift(scratch.path())?;
    if report.status != "DRIFT" || report.drifts != ["workflow-contract.json"] {
        bail!(
            "generated drift mutation was not isolated: {:?}",
            report.drifts
        );
    }
    Ok(DriverOutcome {
        logical_outcome: LogicalOutcome::Rejected,
        final_state: FinalState::Drift,
        detail: "drifts=[workflow-contract.json]".into(),
    })
}

fn run_supplemental_cases(root: &Path) -> Result<Vec<SupplementalCaseResult>> {
    let no_decisions = aggregate_dry(root, None, Some("fast,build_test"), false)?;
    let no_lookahead = domain_gates::no_lookahead_from_fixture(
        &root.join("tools/xtask/tests/ci_negative/fixtures/no_lookahead_violation.json"),
    )?;
    Ok(vec![
        SupplementalCaseResult {
            id: "aggregate_no_decisions_file".into(),
            ok_expected_fail: no_decisions.final_decision == "FAIL",
            detail: format!("final_decision={}", no_decisions.final_decision),
        },
        SupplementalCaseResult {
            id: "no_lookahead".into(),
            ok_expected_fail: !no_lookahead.ok,
            detail: format!("violations={:?}", no_lookahead.violations),
        },
    ])
}

fn stub_result(case: ManifestCase) -> ChaosCaseResult {
    ChaosCaseResult {
        id: case.id,
        spec_name: case.spec_name,
        maturity: "STUB".into(),
        target_seam: case.target_seam,
        driver: None,
        fixture: None,
        expected_logical_outcome: "not_run".into(),
        expected_final_state: "NOT_RUN".into(),
        observed_logical_outcome: None,
        observed_final_state: None,
        executed: false,
        status: "STUB".into(),
        ok_expected_fail: false,
        detail: case.notes,
    }
}

fn executable_result(
    case: ManifestCase,
    observed: DriverOutcome,
    matches: bool,
) -> ChaosCaseResult {
    ChaosCaseResult {
        id: case.id,
        spec_name: case.spec_name,
        maturity: "EXECUTABLE".into(),
        target_seam: case.target_seam,
        driver: case.driver.map(|driver| driver.as_str().into()),
        fixture: case.fixture,
        expected_logical_outcome: case.expected_logical_outcome.as_str().into(),
        expected_final_state: case.expected_final_state.as_str().into(),
        observed_logical_outcome: Some(observed.logical_outcome.as_str().into()),
        observed_final_state: Some(observed.final_state.as_str().into()),
        executed: true,
        status: if matches { "PASS" } else { "FAIL" }.into(),
        ok_expected_fail: matches,
        detail: observed.detail,
    }
}

fn driver_error_result(case: ManifestCase, error: anyhow::Error) -> ChaosCaseResult {
    ChaosCaseResult {
        id: case.id,
        spec_name: case.spec_name,
        maturity: "EXECUTABLE".into(),
        target_seam: case.target_seam,
        driver: case.driver.map(|driver| driver.as_str().into()),
        fixture: case.fixture,
        expected_logical_outcome: case.expected_logical_outcome.as_str().into(),
        expected_final_state: case.expected_final_state.as_str().into(),
        observed_logical_outcome: None,
        observed_final_state: None,
        executed: true,
        status: "FAIL".into(),
        ok_expected_fail: false,
        detail: format!("driver error: {error:#}"),
    }
}

struct ScratchRoot {
    path: PathBuf,
}

impl ScratchRoot {
    fn new(label: &str) -> Result<Self> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock before epoch")?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xhyper-ci-chaos-{label}-{}-{nonce}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).with_context(|| format!("create {}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;

    #[test]
    fn spec26_manifest_executes_eleven_and_exposes_nine_stubs() {
        let root = repo_root_from_manifest();
        let report = run_negative_subset(&root).expect("chaos");
        assert!(report.gate_ok, "{report:?}");
        assert!(!report.coverage_complete);
        assert_eq!(report.cases.len(), 20);
        assert_eq!(report.executable_count, 11);
        assert_eq!(report.stub_count, 9);
        assert!(report
            .cases
            .iter()
            .filter(|case| case.executed)
            .all(|case| case.ok_expected_fail && case.status == "PASS"));
        assert!(report
            .cases
            .iter()
            .filter(|case| !case.executed)
            .all(|case| !case.ok_expected_fail && case.status == "STUB"));
    }

    #[test]
    fn markdown_projection_matches_manifest_maturity() {
        let root = repo_root_from_manifest();
        let raw = fs::read_to_string(root.join(MANIFEST_REL)).unwrap();
        let manifest = validate_manifest_text(&root, &raw).unwrap();
        let projection =
            fs::read_to_string(root.join("tools/xtask/tests/ci_negative_fixtures.md")).unwrap();
        for (index, case) in manifest.cases.iter().enumerate() {
            let maturity = match case.maturity {
                Maturity::Executable => "EXECUTABLE",
                Maturity::Stub => "STUB",
            };
            let prefix = format!("| {} | {} | **{}** |", index + 1, case.spec_name, maturity);
            assert!(
                projection.lines().any(|line| line.starts_with(&prefix)),
                "missing projection row: {prefix}"
            );
        }
    }

    #[test]
    fn manifest_rejects_missing_duplicate_unknown_and_fake_execution() {
        let root = repo_root_from_manifest();
        let raw = fs::read_to_string(root.join(MANIFEST_REL)).unwrap();
        let mutations = [
            raw.replacen("[[case]]", "[[removed]]", 1),
            raw.replacen("spec26-02-cancelled-lane", "spec26-01-missing-lane", 1),
            raw.replacen(
                "notes = \"Expected lane",
                "unknown = true\nnotes = \"Expected lane",
                1,
            ),
            raw.replacen(
                "target_seam = \"ci::aggregate_dry\"",
                "target_seam = \"\"",
                1,
            ),
            raw.replacen("driver = \"aggregate_missing_lane\"\n", "", 1),
            raw.replacen(
                "maturity = \"STUB\"",
                "maturity = \"STUB\"\ndriver = \"aggregate_missing_lane\"",
                1,
            ),
            raw.replacen(
                "expected_logical_outcome = \"rejected\"",
                "expected_logical_outcome = \"accepted\"",
                1,
            ),
            raw.replacen(
                "maturity = \"STUB\"\ntarget_seam = \"ci::fingerprint::validate_reuse_reference\"\nexpected_logical_outcome = \"not_run\"\nexpected_final_state = \"NOT_RUN\"",
                "maturity = \"EXECUTABLE\"\ntarget_seam = \"ci::aggregate_dry\"\ndriver = \"aggregate_missing_lane\"\nfixture = \"tools/xtask/tests/ci_negative/fixtures/missing_lane.json\"\nexpected_logical_outcome = \"rejected\"\nexpected_final_state = \"FAIL\"",
                1,
            ),
            raw.replacen(
                "fixtures/missing_lane.json",
                "fixtures/cancelled_lane.json",
                1,
            ),
        ];
        for mutation in mutations {
            assert!(validate_manifest_text(&root, &mutation).is_err());
        }
    }

    #[test]
    fn isolated_runner_lock_and_drift_drivers_have_specific_failures() {
        for (driver, expected) in [
            (DriverId::RunnerDigestMismatch, "digest_mismatch:"),
            (
                DriverId::RunnerDiskInsufficient,
                "disk_free_gib_insufficient:",
            ),
            (
                DriverId::ToolVersionMismatch,
                "tools_lock_install_mismatch:nextest:",
            ),
            (DriverId::GeneratedDrift, "workflow-contract.json"),
        ] {
            let case = ManifestCase {
                id: "test".into(),
                spec_name: "test".into(),
                maturity: Maturity::Executable,
                target_seam: driver.target_seam().into(),
                driver: Some(driver),
                fixture: None,
                expected_logical_outcome: LogicalOutcome::Rejected,
                expected_final_state: match driver {
                    DriverId::RunnerDigestMismatch | DriverId::RunnerDiskInsufficient => {
                        FinalState::InfraFailure
                    }
                    DriverId::GeneratedDrift => FinalState::Drift,
                    _ => FinalState::Fail,
                },
                notes: "test".into(),
            };
            let outcome = run_driver(&repo_root_from_manifest(), &case).unwrap();
            assert!(outcome.detail.contains(expected), "{:?}", outcome);
        }
    }
}
