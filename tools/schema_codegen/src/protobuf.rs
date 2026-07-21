//! protobuf 解析 + Rust 代码生成。
//!
//! 手写解析器，支持 proto3 语法（message/enum/字段/repeated/nested/oneof/map/service/import）。

use anyhow::{bail, Result};
use std::path::Path;

/// proto message 定义。
struct Message {
    name: String,
    fields: Vec<Field>,
    nested_messages: Vec<Message>,
    nested_enums: Vec<EnumDef>,
    oneofs: Vec<Oneof>,
}

/// proto 字段定义。
struct Field {
    proto_type: String,
    name: String,
    is_repeated: bool,
    #[allow(dead_code)]
    number: u32,
    map_key: Option<String>,
    map_value: Option<String>,
}

/// proto enum 定义。
struct EnumDef {
    name: String,
    variants: Vec<EnumVariant>,
}

/// proto enum 变体。
struct EnumVariant {
    name: String,
    #[allow(dead_code)]
    number: i32,
}

/// proto oneof 定义。
struct Oneof {
    name: String,
    fields: Vec<Field>,
}

/// proto service 定义。
struct Service {
    name: String,
    rpcs: Vec<Rpc>,
}

/// proto rpc 方法定义。
struct Rpc {
    name: String,
    request_type: String,
    response_type: String,
    is_streaming: bool,
}

/// proto 类型 → Rust 类型映射。
///
/// ADR-006：金额/价格/数量在 schema 中应使用 `string`（十进制文本），禁止用
/// `double`/`float` 承载金融字段。本映射对显式 `double`/`float` 仍生成 f64/f32，
/// 仅适用于非金融 wire 类型；market 行情字段见 `schemas/protobuf/market.proto`。
fn proto_type_to_rust(proto_type: &str) -> &str {
    match proto_type {
        "string" => "String",
        "double" => "f64",
        "float" => "f32",
        "int32" | "sint32" | "sfixed32" => "i32",
        "int64" | "sint64" | "sfixed64" => "i64",
        "uint32" | "fixed32" => "u32",
        "uint64" | "fixed64" => "u64",
        "bool" => "bool",
        "bytes" => "Vec<u8>",
        _ => "serde_json::Value",
    }
}

/// SCREAMING_SNAKE_CASE → PascalCase。
fn screaming_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(|c| c.to_lowercase()))
                    .collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// PascalCase → snake_case。
fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

/// 解析 .proto 文件，返回 (messages, services, imports)。
fn parse_proto(content: &str) -> Result<(Vec<Message>, Vec<Service>, Vec<String>)> {
    let mut messages = Vec::new();
    let mut enums = Vec::new();
    let mut services = Vec::new();
    let mut imports = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("message ") {
            let name = extract_name(rest);
            if name.is_empty() {
                bail!("message 名称为空");
            }
            messages.push(parse_message(&mut lines, &name));
        } else if let Some(rest) = trimmed.strip_prefix("enum ") {
            let name = extract_name(rest);
            if name.is_empty() {
                bail!("enum 名称为空");
            }
            enums.push(parse_enum(&mut lines, &name));
        } else if let Some(rest) = trimmed.strip_prefix("service ") {
            let name = extract_name(rest);
            if name.is_empty() {
                bail!("service 名称为空");
            }
            services.push(parse_service(&mut lines, &name));
        } else if let Some(rest) = trimmed.strip_prefix("import ") {
            let import = rest
                .trim_end_matches(';')
                .trim()
                .trim_matches('"')
                .to_string();
            imports.push(import);
        }
    }

    if !enums.is_empty() {
        messages.push(Message {
            name: String::new(),
            fields: Vec::new(),
            nested_messages: Vec::new(),
            nested_enums: enums,
            oneofs: Vec::new(),
        });
    }
    Ok((messages, services, imports))
}

/// 从 `XXX {` 提取名称。
fn extract_name(rest: &str) -> String {
    rest.split('{').next().unwrap_or("").trim().to_string()
}

/// 解析 message 体（递归处理 nested message + enum + oneof）。
fn parse_message(lines: &mut std::iter::Peekable<std::str::Lines>, name: &str) -> Message {
    let mut fields = Vec::new();
    let mut nested_messages = Vec::new();
    let mut nested_enums = Vec::new();
    let mut oneofs = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("message ") {
            let nested_name = extract_name(rest);
            nested_messages.push(parse_message(lines, &nested_name));
        } else if let Some(rest) = trimmed.strip_prefix("enum ") {
            let enum_name = extract_name(rest);
            nested_enums.push(parse_enum(lines, &enum_name));
        } else if let Some(rest) = trimmed.strip_prefix("oneof ") {
            let oneof_name = extract_name(rest);
            oneofs.push(parse_oneof(lines, &oneof_name));
        } else if let Some(field) = parse_field(trimmed) {
            fields.push(field);
        }
    }

    Message {
        name: name.to_string(),
        fields,
        nested_messages,
        nested_enums,
        oneofs,
    }
}

/// 解析 enum 体。
fn parse_enum(lines: &mut std::iter::Peekable<std::str::Lines>, name: &str) -> EnumDef {
    let mut variants = Vec::new();
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(v) = parse_enum_variant(trimmed) {
            variants.push(v);
        }
    }
    EnumDef {
        name: name.to_string(),
        variants,
    }
}

/// 解析 oneof 体。
fn parse_oneof(lines: &mut std::iter::Peekable<std::str::Lines>, name: &str) -> Oneof {
    let mut fields = Vec::new();
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(field) = parse_field(trimmed) {
            fields.push(field);
        }
    }
    Oneof {
        name: name.to_string(),
        fields,
    }
}

/// 解析 service 体。
fn parse_service(lines: &mut std::iter::Peekable<std::str::Lines>, name: &str) -> Service {
    let mut rpcs = Vec::new();
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(rpc) = parse_rpc(trimmed) {
            rpcs.push(rpc);
        }
    }
    Service {
        name: name.to_string(),
        rpcs,
    }
}

/// 解析 rpc 方法 `rpc Method(Request) returns (Response);` 或 `... returns (stream Response);`
fn parse_rpc(line: &str) -> Option<Rpc> {
    let line = line.split("//").next()?.trim();
    let line = line.trim_end_matches(';').trim();
    let rest = line.strip_prefix("rpc ")?;
    let paren_pos = rest.find('(')?;
    let name = rest[..paren_pos].trim().to_string();
    let after_name = &rest[paren_pos..];
    let parts: Vec<&str> = after_name.split("returns").collect();
    if parts.len() != 2 {
        return None;
    }
    let request_type = parts[0]
        .trim()
        .trim_matches(|c| c == '(' || c == ')')
        .trim()
        .to_string();
    let response_inner = parts[1]
        .trim()
        .trim_matches(|c| c == '(' || c == ')')
        .trim();
    let (is_streaming, response_type) =
        if let Some(stream_rest) = response_inner.strip_prefix("stream ") {
            (true, stream_rest.trim().to_string())
        } else {
            (false, response_inner.to_string())
        };
    Some(Rpc {
        name,
        request_type,
        response_type,
        is_streaming,
    })
}

/// 解析 enum 变体 `NAME = number;`
fn parse_enum_variant(line: &str) -> Option<EnumVariant> {
    let line = line.split("//").next()?.trim();
    let line = line.trim_end_matches(';').trim();
    let eq_pos = line.find('=')?;
    let name = line[..eq_pos].trim().to_string();
    let number: i32 = line[eq_pos + 1..].trim().parse().ok()?;
    Some(EnumVariant { name, number })
}

/// 解析字段 `[repeated] <type> <name> = <number>;` 或 `map<K,V> <name> = <number>;`
fn parse_field(line: &str) -> Option<Field> {
    let line = line.split("//").next()?.trim();
    let line = line.trim_end_matches(';').trim();
    let eq_pos = line.find('=')?;
    let left = line[..eq_pos].trim();
    let number: u32 = line[eq_pos + 1..].trim().parse().ok()?;

    if left.starts_with("map<") {
        let close = left.find('>')?;
        let kv: Vec<&str> = left[4..close].split(',').map(str::trim).collect();
        if kv.len() != 2 {
            return None;
        }
        let after_map = left[close + 1..].trim();
        let name = after_map.split_whitespace().next()?.to_string();
        return Some(Field {
            proto_type: "map".to_string(),
            name,
            is_repeated: false,
            number,
            map_key: Some(kv[0].to_string()),
            map_value: Some(kv[1].to_string()),
        });
    }

    let parts: Vec<&str> = left.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let (is_repeated, proto_type, name) = if parts[0] == "repeated" {
        if parts.len() < 3 {
            return None;
        }
        (true, parts[1].to_string(), parts[2].to_string())
    } else {
        (false, parts[0].to_string(), parts[1].to_string())
    };
    Some(Field {
        proto_type,
        name,
        is_repeated,
        number,
        map_key: None,
        map_value: None,
    })
}

/// 递归生成 Rust 代码。
fn generate_rust(messages: &[Message], services: &[Service], imports: &[String]) -> String {
    let mut out = String::new();
    out.push_str("// 自动生成，请勿手动编辑。\n\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    if !imports.is_empty() {
        out.push_str("// imports（未递归解析，外部类型原样引用）\n");
        for imp in imports {
            out.push_str(&format!("// import \"{imp}\"\n"));
        }
        out.push('\n');
    }

    for msg in messages {
        generate_message(msg, "", &mut out);
    }
    for svc in services {
        generate_service(svc, &mut out);
    }
    out
}

/// 解析字段类型：nested → 扁平化名，内置 → Rust 类型，map → HashMap，顶层自定义 → 原样。
fn resolve_field_type(proto_type: &str, parent_name: &str, msg: &Message) -> String {
    if msg.nested_messages.iter().any(|n| n.name == proto_type) {
        return format!("{parent_name}_{proto_type}");
    }
    if msg.nested_enums.iter().any(|e| e.name == proto_type) {
        return format!("{parent_name}_{proto_type}");
    }
    let rust_type = proto_type_to_rust(proto_type);
    if rust_type != "serde_json::Value" {
        return rust_type.to_string();
    }
    proto_type.to_string()
}

/// 递归生成 message + nested + oneof + enum。
fn generate_message(msg: &Message, prefix: &str, out: &mut String) {
    if msg.name.is_empty() {
        for enum_def in &msg.nested_enums {
            generate_enum(enum_def, "", out);
        }
        return;
    }

    let full_name = if prefix.is_empty() {
        msg.name.clone()
    } else {
        format!("{prefix}_{}", msg.name)
    };

    for oneof in &msg.oneofs {
        generate_oneof(oneof, &full_name, msg, out);
    }

    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {full_name} {{\n"));
    for field in &msg.fields {
        if field.proto_type == "map" {
            let key = proto_type_to_rust(field.map_key.as_deref().unwrap_or("string"));
            let value = proto_type_to_rust(field.map_value.as_deref().unwrap_or("string"));
            out.push_str(&format!(
                "    pub {}: std::collections::HashMap<{key}, {value}>,\n",
                field.name
            ));
        } else {
            let final_type = resolve_field_type(&field.proto_type, &full_name, msg);
            if field.is_repeated {
                out.push_str(&format!("    pub {}: Vec<{final_type}>,\n", field.name));
            } else {
                out.push_str(&format!("    pub {}: {final_type},\n", field.name));
            }
        }
    }
    for oneof in &msg.oneofs {
        let pascal = screaming_to_pascal(&oneof.name);
        let enum_name = format!("{full_name}_{pascal}");
        out.push_str(&format!("    pub {}: Option<{}>,\n", oneof.name, enum_name));
    }
    out.push_str("}\n\n");

    for nested in &msg.nested_messages {
        generate_message(nested, &full_name, out);
    }
    for enum_def in &msg.nested_enums {
        generate_enum(enum_def, &full_name, out);
    }
}

/// 生成 oneof 的 Rust enum。
fn generate_oneof(oneof: &Oneof, parent_full_name: &str, msg: &Message, out: &mut String) {
    let pascal_name = screaming_to_pascal(&oneof.name);
    let enum_name = format!("{parent_full_name}_{pascal_name}");
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub enum {enum_name} {{\n"));
    for field in &oneof.fields {
        let pascal = screaming_to_pascal(&field.name);
        let final_type = resolve_field_type(&field.proto_type, parent_full_name, msg);
        out.push_str(&format!("    {pascal}({final_type}),\n"));
    }
    out.push_str("}\n\n");
}

/// 生成 Rust enum。
fn generate_enum(enum_def: &EnumDef, prefix: &str, out: &mut String) {
    let full_name = if prefix.is_empty() {
        enum_def.name.clone()
    } else {
        format!("{prefix}_{}", enum_def.name)
    };

    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub enum {full_name} {{\n"));
    for variant in &enum_def.variants {
        let pascal = screaming_to_pascal(&variant.name);
        out.push_str(&format!("    {pascal},\n"));
    }
    out.push_str("}\n\n");
}

/// 生成 service 的 Rust trait。
fn generate_service(service: &Service, out: &mut String) {
    out.push_str(&format!("pub trait {} {{\n", service.name));
    for rpc in &service.rpcs {
        let method = pascal_to_snake(&rpc.name);
        if rpc.is_streaming {
            out.push_str(&format!(
                "    fn {}(&self, request: {}) -> anyhow::Result<Vec<{}>>;\n",
                method, rpc.request_type, rpc.response_type
            ));
        } else {
            out.push_str(&format!(
                "    fn {}(&self, request: {}) -> anyhow::Result<{}>;\n",
                method, rpc.request_type, rpc.response_type
            ));
        }
    }
    out.push_str("}\n\n");
}

/// 解析 .proto 文件并生成 Rust 代码。
pub fn run(proto_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(proto_path)
        .map_err(|e| anyhow::anyhow!("读取 .proto 文件失败: {e}"))?;
    let (messages, services, imports) = parse_proto(&content)?;
    let rust_code = generate_rust(&messages, &services, &imports);
    Ok(rust_code)
}
