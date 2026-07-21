//! 轻量 JSON Schema 子集匹配（INFRA-003 / approval Evidence 共用）。
//!
//! 不是完整 draft-07 引擎；仅覆盖仓库 evidence-record.schema.json 实际用到的关键字：
//! `type` / `enum` / `required` / `properties` / `additionalProperties` /
//! `minLength` / `pattern` / `format` / `minimum` / `items`。
//! 未知 pattern/format 一律失败（fail-closed）。

use serde_json::Value;

/// 判断 `value` 是否满足 `schema`（子集语义）。
pub fn json_schema_matches(value: &Value, schema: &Value) -> bool {
    if schema
        .get("const")
        .is_some_and(|constant| constant != value)
    {
        return false;
    }
    if let Some(types) = schema.get("type") {
        let type_matches = match types {
            Value::String(kind) => json_type_matches(value, kind),
            Value::Array(kinds) => kinds
                .iter()
                .filter_map(Value::as_str)
                .any(|kind| json_type_matches(value, kind)),
            _ => false,
        };
        if !type_matches {
            return false;
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return false;
        }
    }
    if let Some(text) = value.as_str() {
        if schema
            .get("minLength")
            .and_then(Value::as_u64)
            .is_some_and(|minimum| text.chars().count() < minimum as usize)
        {
            return false;
        }
        if schema
            .get("pattern")
            .and_then(Value::as_str)
            .is_some_and(|pattern| !known_pattern_matches(text, pattern))
        {
            return false;
        }
        if schema
            .get("format")
            .and_then(Value::as_str)
            .is_some_and(|format| !known_format_matches(text, format))
        {
            return false;
        }
    }
    if schema
        .get("minimum")
        .and_then(Value::as_i64)
        .is_some_and(|minimum| value.as_i64().is_none_or(|number| number < minimum))
    {
        return false;
    }
    if let Some(object) = value.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            if required
                .iter()
                .filter_map(Value::as_str)
                .any(|field| !object.contains_key(field))
            {
                return false;
            }
        }
        for (field, field_value) in object {
            if let Some(field_schema) = properties.and_then(|known| known.get(field)) {
                if !json_schema_matches(field_value, field_schema) {
                    return false;
                }
                continue;
            }
            match schema.get("additionalProperties") {
                Some(Value::Bool(false)) => return false,
                Some(additional @ Value::Object(_))
                    if !json_schema_matches(field_value, additional) =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }
    if let Some(items) = value.as_array() {
        if schema
            .get("uniqueItems")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && items
                .iter()
                .enumerate()
                .any(|(index, item)| items[index + 1..].contains(item))
        {
            return false;
        }
        if let Some(item_schema) = schema.get("items") {
            if items
                .iter()
                .any(|item| !json_schema_matches(item, item_schema))
            {
                return false;
            }
        }
    }
    true
}

fn json_type_matches(value: &Value, kind: &str) -> bool {
    match kind {
        "null" => value.is_null(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        _ => false,
    }
}

fn known_pattern_matches(value: &str, pattern: &str) -> bool {
    match pattern {
        "^[0-9]+\\.[0-9]+\\.[0-9]+(-[A-Za-z0-9.-]+)?$" => {
            let (core, suffix) = value
                .split_once('-')
                .map_or((value, None), |(core, suffix)| (core, Some(suffix)));
            core.split('.').count() == 3
                && core
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                && suffix.is_none_or(|suffix| {
                    !suffix.is_empty()
                        && suffix
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
                })
        }
        "^[0-9a-f]{7,40}$" => {
            (7..=40).contains(&value.len())
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }
        "^[0-9a-fA-F]{40}$|^[0-9a-fA-F]{64}$" => {
            matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        }
        "^sha256:[0-9a-fA-F]{64}$" => value.strip_prefix("sha256:").is_some_and(|digest| {
            digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
        }),
        "^sha256:[0-9a-f]{64}$" => value.strip_prefix("sha256:").is_some_and(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }),
        "^(sha256:)?[0-9a-fA-F]{64}$|^sha256:PLACEHOLDER_[A-Za-z0-9_-]+$" => {
            matches_ci_digest(value, true)
        }
        "^sha256:[0-9a-fA-F]{64}$|^sha256:PLACEHOLDER_[A-Za-z0-9_-]+$" => {
            value.starts_with("sha256:") && matches_ci_digest(value, true)
        }
        _ => false,
    }
}

fn matches_ci_digest(value: &str, allow_placeholder: bool) -> bool {
    if allow_placeholder
        && value
            .strip_prefix("sha256:PLACEHOLDER_")
            .is_some_and(|suffix| {
                !suffix.is_empty()
                    && suffix
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            })
    {
        return true;
    }
    let digest = value.strip_prefix("sha256:").unwrap_or(value);
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn known_format_matches(value: &str, format: &str) -> bool {
    match format {
        "date-time" => is_rfc3339_utc(value),
        "date" => {
            let candidate = format!("{value}T00:00:00Z");
            is_rfc3339_utc(&candidate)
        }
        "uri" => value
            .split_once("://")
            .is_some_and(|(scheme, rest)| !scheme.is_empty() && !rest.is_empty()),
        _ => false,
    }
}

/// 严格 `YYYY-MM-DDTHH:MM:SSZ`。
pub fn is_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let number = |range: std::ops::Range<usize>| -> Option<u32> {
        std::str::from_utf8(&bytes[range]).ok()?.parse().ok()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
        number(17..19),
    ) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_additional_property_when_disallowed() {
        let schema = serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["a"],
            "properties": { "a": { "type": "string" } }
        });
        let ok = serde_json::json!({ "a": "x" });
        let bad = serde_json::json!({ "a": "x", "extra": 1 });
        assert!(json_schema_matches(&ok, &schema));
        assert!(!json_schema_matches(&bad, &schema));
    }

    #[test]
    fn rejects_enum_and_pattern_violations() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["result", "commit"],
            "properties": {
                "result": { "type": "string", "enum": ["PASS", "FAIL"] },
                "commit": { "type": "string", "pattern": "^[0-9a-f]{7,40}$" }
            }
        });
        let ok = serde_json::json!({ "result": "PASS", "commit": "deadbeef" });
        let bad_result = serde_json::json!({ "result": "YES", "commit": "deadbeef" });
        let bad_commit = serde_json::json!({ "result": "PASS", "commit": "NOTHEX" });
        assert!(json_schema_matches(&ok, &schema));
        assert!(!json_schema_matches(&bad_result, &schema));
        assert!(!json_schema_matches(&bad_commit, &schema));
    }

    #[test]
    fn enforces_const_and_unique_items() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["kind", "lanes"],
            "properties": {
                "kind": { "const": "aggregate" },
                "lanes": {
                    "type": "array",
                    "uniqueItems": true,
                    "items": { "type": "string" }
                }
            }
        });
        assert!(json_schema_matches(
            &serde_json::json!({"kind": "aggregate", "lanes": ["fast", "test"]}),
            &schema
        ));
        assert!(!json_schema_matches(
            &serde_json::json!({"kind": "wrong", "lanes": ["fast", "test"]}),
            &schema
        ));
        assert!(!json_schema_matches(
            &serde_json::json!({"kind": "aggregate", "lanes": ["fast", "fast"]}),
            &schema
        ));
    }

    #[test]
    fn supports_ci_digest_patterns_without_accepting_placeholders() {
        let schema = serde_json::json!({
            "type": "string",
            "pattern": "^sha256:[0-9a-fA-F]{64}$"
        });
        assert!(json_schema_matches(
            &serde_json::Value::String(format!("sha256:{}", "a".repeat(64))),
            &schema
        ));
        assert!(!json_schema_matches(
            &serde_json::Value::String("sha256:PLACEHOLDER_DIGEST".into()),
            &schema
        ));
    }
}
