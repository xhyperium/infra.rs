//! OpenAPI 解析 + Rust 代码生成。
//!
//! 从 OpenAPI 3.0 的 components.schemas 提取 schema 定义，
//! 复用 jsonschema 的 generate_struct 生成 Rust struct。
//! 不解析 paths（只生成类型，不生成 client/server 代码）。

use crate::jsonschema::generate_struct;
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// 解析 OpenAPI 文件并生成 Rust 代码。
pub fn run(openapi_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(openapi_path)
        .map_err(|e| anyhow::anyhow!("读取 OpenAPI 文件失败: {e}"))?;
    let spec: Value =
        serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("解析 JSON 失败: {e}"))?;

    let mut out = String::new();
    out.push_str("// 自动生成，请勿手动编辑。\n\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    // 提取 components.schemas
    let schemas = &spec["components"]["schemas"];
    if let Some(obj) = schemas.as_object() {
        for (name, schema) in obj {
            // 给 schema 补上 title（如果缺失），用 schema 名
            let mut schema_with_title = schema.clone();
            if schema_with_title["title"].is_null() {
                schema_with_title["title"] = Value::String(name.clone());
            }
            generate_struct(&schema_with_title, &mut out);
        }
    }

    Ok(out)
}
