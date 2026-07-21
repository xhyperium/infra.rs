//! Flake registry + failure fingerprint（PHASE-3-03/04）。

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 初始合同只允许单次显式重试；默认值仍为 0（safe-off）。
pub const MAX_ALLOWED_RETRIES: u32 = 1;

#[derive(Debug, Serialize)]
pub struct FlakeCheckReport {
    pub ok: bool,
    pub mode: &'static str,
    pub registry_present: bool,
    pub retry_mode: &'static str,
    pub open_count: usize,
    pub expired: Vec<String>,
    /// 仅包含未过期、完整登记的 test；缺省为空即不允许 retry。
    pub retry_allowances: BTreeMap<String, u32>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct FailureFingerprintReport {
    pub fingerprint: String,
    pub lane: String,
    pub mode: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FlakeRegistryFile {
    #[serde(default)]
    flake: Vec<FlakeEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FlakeEntry {
    test: String,
    owner: String,
    tracking_issue: u64,
    first_seen: String,
    expires: String,
    allowed_retries: u32,
}

/// 读取 flakes.toml。文件缺失等价于空注册表：不允许任何 retry（safe-off）。
pub fn check_flake_registry(root: &Path, today: &str) -> Result<FlakeCheckReport> {
    let path = root.join(".github/ci/flakes.toml");
    let (raw, registry_present) = match fs::read_to_string(&path) {
        Ok(raw) => (raw, true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (String::new(), false),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    check_flake_registry_text_with_presence(&raw, today, registry_present)
}

/// 纯文本入口（单测 / 负向 fixture 注入，不依赖仓库根 flakes.toml）。
pub fn check_flake_registry_text(raw: &str, today: &str) -> Result<FlakeCheckReport> {
    check_flake_registry_text_with_presence(raw, today, true)
}

fn check_flake_registry_text_with_presence(
    raw: &str,
    today: &str,
    registry_present: bool,
) -> Result<FlakeCheckReport> {
    let today = parse_ymd(today).context("invalid --today; expected a real YYYY-MM-DD date")?;
    let entries = parse_registry(raw)?;
    let mut seen_tests = BTreeSet::new();
    let mut expired = Vec::new();
    let mut retry_allowances = BTreeMap::new();

    for (index, entry) in entries.iter().enumerate() {
        let entry_number = index + 1;
        validate_identity(&entry.test, "test", entry_number)?;
        validate_identity(&entry.owner, "owner", entry_number)?;
        if !entry.owner.starts_with('@') || entry.owner.len() == 1 {
            bail!("flake entry {entry_number}: owner must be a non-empty @handle");
        }
        if entry.tracking_issue == 0 {
            bail!("flake entry {entry_number}: tracking_issue must be positive");
        }
        if entry.allowed_retries > MAX_ALLOWED_RETRIES {
            bail!(
                "flake entry {entry_number}: allowed_retries {} exceeds maximum {MAX_ALLOWED_RETRIES}",
                entry.allowed_retries
            );
        }
        let first_seen = parse_ymd(&entry.first_seen)
            .with_context(|| format!("flake entry {entry_number}: invalid first_seen"))?;
        let expires = parse_ymd(&entry.expires)
            .with_context(|| format!("flake entry {entry_number}: invalid expires"))?;
        if expires < first_seen {
            bail!("flake entry {entry_number}: expires precedes first_seen");
        }
        if !seen_tests.insert(entry.test.clone()) {
            bail!("duplicate flake test entry: {}", entry.test);
        }
        if first_seen > today {
            bail!("flake {}: first_seen is in the future", entry.test);
        }
        if expires < today {
            expired.push(entry.test.clone());
        } else if entry.allowed_retries > 0 {
            retry_allowances.insert(entry.test.clone(), entry.allowed_retries);
        }
        // 读取字段可防止合同未来被悄悄退化成只校验日期。
        let _registry_identity = (&entry.owner, entry.tracking_issue);
    }

    let ok = expired.is_empty();
    if !ok {
        // 总门已阻断时不暴露局部 allowance，避免未来 consumer 忽略 `ok`。
        retry_allowances.clear();
    }
    let retry_mode = if !ok {
        "BLOCKED"
    } else if retry_allowances.is_empty() {
        "SAFE_OFF"
    } else {
        "REGISTERED_ONLY"
    };
    Ok(FlakeCheckReport {
        ok,
        mode: "shadow",
        registry_present,
        retry_mode,
        open_count: entries.len(),
        expired,
        retry_allowances,
        note: format!(
            "strict Spec §13.1 registry; missing/empty registry is safe-off; max retries={MAX_ALLOWED_RETRIES}"
        ),
    })
}

fn parse_registry(raw: &str) -> Result<Vec<FlakeEntry>> {
    let registry: FlakeRegistryFile =
        toml::from_str(raw).context("parse strict Spec §13.1 flake registry TOML")?;
    Ok(registry.flake)
}

fn validate_identity(value: &str, field: &str, entry_number: usize) -> Result<()> {
    if value.is_empty() || value != value.trim() {
        bail!("flake entry {entry_number}: {field} must be non-empty without edge whitespace");
    }
    if value.chars().any(char::is_control) {
        bail!("flake entry {entry_number}: {field} must not contain control characters");
    }
    Ok(())
}

pub fn failure_fingerprint(
    lane: &str,
    command: &str,
    error_class: &str,
) -> FailureFingerprintReport {
    let raw = format!("{lane}|{command}|{error_class}");
    FailureFingerprintReport {
        fingerprint: format!("sha256:{:x}", Sha256::digest(raw.as_bytes())),
        lane: lane.to_string(),
        mode: "shadow",
    }
}

/// 返回当前 UTC civil date；不读取本地时区。
pub fn utc_today() -> Result<String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs();
    let days = i64::try_from(seconds / 86_400).context("UTC day count overflow")?;
    let (year, month, day) = civil_from_days(days);
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

fn parse_ymd(value: &str) -> Result<i64> {
    if value.len() != 10
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
    {
        bail!("date must use YYYY-MM-DD");
    }
    let year: i64 = value[0..4].parse().context("invalid year")?;
    let month: i64 = value[5..7].parse().context("invalid month")?;
    let day: i64 = value[8..10].parse().context("invalid day")?;
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        bail!("date year/month out of range");
    }
    let max_day = days_in_month(year, month);
    if !(1..=max_day).contains(&day) {
        bail!("date day out of range");
    }
    Ok(days_from_civil(year, month, day))
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::repo_root_from_manifest;

    const VALID: &str = r#"
[[flake]]
test = "crate::module::test"
owner = "@owner"
tracking_issue = 123
first_seen = "2026-07-01"
expires = "2026-07-31"
allowed_retries = 1
"#;

    #[test]
    fn empty_registry_is_safe_off() {
        let report = check_flake_registry_text("# empty\n", "2026-07-17").unwrap();
        assert!(report.ok);
        assert!(report.registry_present);
        assert_eq!(report.retry_mode, "SAFE_OFF");
        assert_eq!(report.open_count, 0);
        assert!(report.retry_allowances.is_empty());
    }

    #[test]
    fn missing_registry_is_explicit_safe_off() {
        let root = tempfile::tempdir().unwrap();
        let report = check_flake_registry(root.path(), "2026-07-17").unwrap();
        assert!(report.ok);
        assert!(!report.registry_present);
        assert_eq!(report.retry_mode, "SAFE_OFF");
        assert!(report.retry_allowances.is_empty());
    }

    #[test]
    fn valid_registry_is_retry_eligible_until_expiry() {
        let report = check_flake_registry_text(VALID, "2026-07-17").unwrap();
        assert!(report.ok, "{report:?}");
        assert_eq!(report.retry_mode, "REGISTERED_ONLY");
        assert_eq!(report.open_count, 1);
        assert_eq!(report.retry_allowances["crate::module::test"], 1);
    }

    #[test]
    fn expiry_blocks() {
        let report = check_flake_registry_text(VALID, "2026-08-01").unwrap();
        assert!(!report.ok);
        assert_eq!(report.retry_mode, "BLOCKED");
        assert_eq!(report.expired, ["crate::module::test"]);
        assert!(report.retry_allowances.is_empty());
    }

    #[test]
    fn one_expired_entry_blocks_all_retry_allowances() {
        let expired = VALID
            .replace("crate::module::test", "crate::module::expired")
            .replace("2026-07-31", "2026-07-16");
        let report = check_flake_registry_text(&format!("{VALID}{expired}"), "2026-07-17").unwrap();
        assert!(!report.ok);
        assert_eq!(report.retry_mode, "BLOCKED");
        assert!(report.retry_allowances.is_empty());
    }

    #[test]
    fn missing_unknown_and_duplicate_fields_fail_closed() {
        for field_line in [
            "test = \"crate::module::test\"\n",
            "owner = \"@owner\"\n",
            "tracking_issue = 123\n",
            "first_seen = \"2026-07-01\"\n",
            "expires = \"2026-07-31\"\n",
            "allowed_retries = 1\n",
        ] {
            let raw = VALID.replace(field_line, "");
            assert!(
                check_flake_registry_text(&raw, "2026-07-17").is_err(),
                "missing {field_line:?} must fail"
            );
        }
        for (name, raw) in [
            (
                "unknown entry key",
                VALID.replace("allowed_retries = 1", "surprise = 1"),
            ),
            (
                "unknown top-level key",
                format!("default_retry = true\n{VALID}"),
            ),
            ("duplicate key", format!("{VALID}owner = \"@other\"\n")),
            ("invalid TOML", "[[flake]\n".to_string()),
        ] {
            assert!(
                check_flake_registry_text(&raw, "2026-07-17").is_err(),
                "{name} must fail"
            );
        }
    }

    #[test]
    fn formal_toml_syntax_is_parsed_before_contract_validation() {
        let valid_toml = VALID
            .replace(
                "test = \"crate::module::test\"",
                "test = \"crate::module::test\" # inline",
            )
            .replace("tracking_issue = 123", "tracking_issue = 1_23");
        assert!(check_flake_registry_text(&valid_toml, "2026-07-17").is_ok());

        for raw in [
            VALID.replace("tracking_issue = 123", "tracking_issue = 0123"),
            VALID.replace(
                "test = \"crate::module::test\"",
                "test = \" crate::module::test \"",
            ),
            VALID.replace("owner = \"@owner\"", "owner = \"@ \""),
            VALID.replace(
                "test = \"crate::module::test\"",
                "test = \"crate::\\u0000test\"",
            ),
            VALID.replace("first_seen = \"2026-07-01\"", "first_seen = 2026-07-01"),
        ] {
            assert!(
                check_flake_registry_text(&raw, "2026-07-17").is_err(),
                "{raw}"
            );
        }
    }

    #[test]
    fn duplicate_test_entries_fail_closed() {
        assert!(check_flake_registry_text(&format!("{VALID}{VALID}"), "2026-07-17").is_err());
    }

    #[test]
    fn invalid_dates_fail_closed() {
        for today in ["", "2026-2-01", "2026-02-31", "not-a-date"] {
            assert!(
                check_flake_registry_text(VALID, today).is_err(),
                "today={today:?}"
            );
        }
        for bad in ["2026-02-31", "2025-02-29", "2026-13-01"] {
            let raw = VALID.replace("2026-07-31", bad);
            assert!(
                check_flake_registry_text(&raw, "2026-07-17").is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn reversed_dates_future_first_seen_and_retry_overflow_fail() {
        let reversed = VALID.replace("2026-07-31", "2026-06-30");
        let future = VALID.replace("2026-07-01", "2026-07-18");
        let retries = VALID.replace("allowed_retries = 1", "allowed_retries = 2");
        for raw in [reversed, future, retries] {
            assert!(check_flake_registry_text(&raw, "2026-07-17").is_err());
        }
    }

    #[test]
    fn expiry_boundary_and_zero_retry_are_safe() {
        let boundary = VALID.replace("2026-07-31", "2026-07-17");
        let report = check_flake_registry_text(&boundary, "2026-07-17").unwrap();
        assert!(report.ok);
        assert_eq!(report.retry_allowances["crate::module::test"], 1);

        let disabled = VALID.replace("allowed_retries = 1", "allowed_retries = 0");
        let report = check_flake_registry_text(&disabled, "2026-07-17").unwrap();
        assert!(report.ok);
        assert_eq!(report.retry_mode, "SAFE_OFF");
        assert!(report.retry_allowances.is_empty());
    }

    #[test]
    fn civil_date_round_trip_and_leap_validation() {
        for value in ["1970-01-01", "2000-02-29", "2026-07-17", "9999-12-31"] {
            let days = parse_ymd(value).unwrap();
            let (year, month, day) = civil_from_days(days);
            assert_eq!(format!("{year:04}-{month:02}-{day:02}"), value);
        }
        assert!(parse_ymd("1900-02-29").is_err());
    }

    #[test]
    fn shipped_registry_and_utc_today_are_valid() {
        let root = repo_root_from_manifest();
        let today = utc_today().unwrap();
        assert!(parse_ymd(&today).is_ok());
        assert!(check_flake_registry(&root, &today).unwrap().ok);
    }

    #[test]
    fn failure_fp_stable() {
        let a = failure_fingerprint("fast", "cargo test", "assert");
        let b = failure_fingerprint("fast", "cargo test", "assert");
        assert_eq!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn expired_fixture_blocks() {
        let root = repo_root_from_manifest();
        let fixture = root.join("tools/xtask/tests/ci_negative/fixtures/flake_expired.toml");
        let raw = fs::read_to_string(&fixture).unwrap();
        let report = check_flake_registry_text(&raw, "2026-07-16").unwrap();
        assert!(!report.ok);
    }
}
