//! JSON Schema 解析 + Rust 代码生成。
//!
//! 支持 draft-07 基本语法：title/type/properties/required/array。
//! 不支持 $ref/allOf/anyOf/oneOf/patternProperties。

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// JSON 类型 → Rust 类型（简单映射）。
///
/// ADR-006：金额/价格/数量在 schema 中应使用 `string`（十进制文本），禁止用
/// JSON `number`（IEEE754）承载金融字段。本映射对显式 `number` 仍生成 `f64`，
/// 仅适用于非金融通用字段；market 见 `schemas/jsonschema/quote.json` /
/// `schemas/openapi/market.json`（`DecimalString`）。
fn json_type_to_rust_simple(json_type: &str) -> &str {
    match json_type {
        "string" => "String",
        "number" => "f64",
        "integer" => "i64",
        "boolean" => "bool",
        _ => "serde_json::Value",
    }
}

/// 解析 property 的 Rust 类型（含 array）。
fn json_type_to_rust(prop: &Value) -> String {
    let json_type = prop["type"].as_str().unwrap_or("string");
    match json_type {
        "array" => {
            let item_type = prop["items"]["type"].as_str().unwrap_or("string");
            format!("Vec<{}>", json_type_to_rust_simple(item_type))
        }
        _ => json_type_to_rust_simple(json_type).to_string(),
    }
}

/// 生成 Rust struct（pub(crate) 供 openapi 复用）。
pub(crate) fn generate_struct(schema: &Value, out: &mut String) {
    let title = schema["title"].as_str().unwrap_or("Unnamed");
    let properties = schema["properties"].as_object();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {title} {{\n"));

    if let Some(props) = properties {
        for (name, prop) in props {
            let rust_type = json_type_to_rust(prop);
            if required.contains(&name.as_str()) {
                out.push_str(&format!("    pub {name}: {rust_type},\n"));
            } else {
                out.push_str(&format!("    pub {name}: Option<{rust_type}>,\n"));
            }
        }
    }
    out.push_str("}\n\n");
}

/// 解析 JSON Schema 文件并生成 Rust 代码。
pub fn run(schema_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(schema_path)
        .map_err(|e| anyhow::anyhow!("读取 JSON Schema 文件失败: {e}"))?;
    let schema: Value =
        serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("解析 JSON 失败: {e}"))?;

    let mut out = String::new();
    out.push_str("// 自动生成，请勿手动编辑。\n\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");
    generate_struct(&schema, &mut out);
    Ok(out)
}
