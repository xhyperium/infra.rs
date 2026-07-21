//! SQL DDL 解析 + Rust 代码生成。
//!
//! 从 CREATE TABLE 语句生成 Rust struct。
//! 支持：列定义 + NOT NULL（→ Option）。
//! 不支持：ALTER TABLE / CREATE INDEX / 触发器 / 视图。

use anyhow::Result;
use std::path::Path;

/// SQL 列定义。
struct Column {
    name: String,
    sql_type: String,
    is_not_null: bool,
}

/// SQL 类型 → Rust 类型映射。
///
/// ADR-006：`DECIMAL` / `NUMERIC` 禁止映射为 `f64`/`f32`；映射到
/// `decimalx::Decimal`（仓库金额/价格/数量唯一定义点）。
/// 显式 `DOUBLE`/`REAL`/`FLOAT*` 仍映射浮点，仅用于非金融显式浮点列。
fn sql_type_to_rust(sql_type: &str) -> &str {
    let upper = sql_type.to_uppercase();
    let base = upper.split_whitespace().next().unwrap_or(&upper);
    let base = base.split('(').next().unwrap_or(base);
    match base {
        "VARCHAR" | "TEXT" | "CHAR" | "CITEXT" | "NAME" => "String",
        "DOUBLE" | "REAL" | "FLOAT8" => "f64",
        "FLOAT" | "FLOAT4" => "f32",
        "INT" | "INTEGER" | "INT4" => "i32",
        "BIGINT" | "INT8" => "i64",
        "SMALLINT" | "INT2" => "i16",
        "BOOLEAN" | "BOOL" => "bool",
        "BLOB" | "BYTEA" | "BINARY" | "VARBINARY" => "Vec<u8>",
        "JSON" | "JSONB" => "serde_json::Value",
        "DATE" | "TIMESTAMP" | "TIMESTAMPTZ" | "TIME" | "INTERVAL" => "String",
        // ADR-006：金融精度类型 → decimalx，禁止 f64
        "DECIMAL" | "NUMERIC" => "decimalx::Decimal",
        _ => "String",
    }
}

/// 表名 snake_case → struct 名 PascalCase。
fn table_name_to_pascal(name: &str) -> String {
    name.split('_')
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

/// 解析 SQL DDL，提取表定义。
fn parse_sql(content: &str) -> Vec<(String, Vec<Column>)> {
    let mut tables = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if let Some(rest) = upper.strip_prefix("CREATE TABLE ") {
            let rest = rest.trim_start_matches("IF NOT EXISTS ").trim();
            let table_name = rest
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_lowercase();
            if table_name.is_empty() {
                continue;
            }
            let mut columns = Vec::new();
            for col_line in lines.by_ref() {
                let col_trimmed = col_line.trim().trim_end_matches(',');
                if col_trimmed == ")" || col_trimmed == ");" {
                    break;
                }
                if col_trimmed.is_empty() || col_trimmed.starts_with("--") {
                    continue;
                }
                let upper_col = col_trimmed.to_uppercase();
                if upper_col.starts_with("PRIMARY KEY")
                    || upper_col.starts_with("FOREIGN KEY")
                    || upper_col.starts_with("UNIQUE")
                    || upper_col.starts_with("CHECK")
                    || upper_col.starts_with("CONSTRAINT")
                    || upper_col.starts_with("INDEX")
                {
                    continue;
                }
                if let Some(col) = parse_column(col_trimmed) {
                    columns.push(col);
                }
            }
            tables.push((table_name, columns));
        }
    }
    tables
}

/// 解析列定义 `name TYPE [NOT NULL] [其他约束]`
fn parse_column(line: &str) -> Option<Column> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let name = parts[0].trim_matches('"').to_lowercase();
    let sql_type = parts[1]
        .split('(')
        .next()
        .unwrap_or(parts[1])
        .to_uppercase();
    let is_not_null = line.to_uppercase().contains("NOT NULL");
    Some(Column {
        name,
        sql_type,
        is_not_null,
    })
}

/// 列是否为 DECIMAL/NUMERIC（生成注释用）。
fn is_decimal_sql_type(sql_type: &str) -> bool {
    let upper = sql_type.to_uppercase();
    let base = upper.split('(').next().unwrap_or(&upper);
    matches!(base.trim(), "DECIMAL" | "NUMERIC")
}

/// 生成 Rust 代码。
fn generate_rust(tables: &[(String, Vec<Column>)]) -> String {
    let mut out = String::new();
    out.push_str("// 自动生成，请勿手动编辑。\n");
    out.push_str(
        "// DECIMAL/NUMERIC → decimalx::Decimal（ADR-006：禁止 f64 承载金额/价格/数量）。\n\n",
    );
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    for (table_name, columns) in tables {
        let struct_name = table_name_to_pascal(table_name);
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));
        for col in columns {
            let rust_type = sql_type_to_rust(&col.sql_type);
            if is_decimal_sql_type(&col.sql_type) {
                out.push_str("    /// SQL DECIMAL/NUMERIC；ADR-006 禁止 f64。\n");
            }
            if col.is_not_null {
                out.push_str(&format!("    pub {}: {rust_type},\n", col.name));
            } else {
                out.push_str(&format!("    pub {}: Option<{rust_type}>,\n", col.name));
            }
        }
        out.push_str("}\n\n");
    }
    out
}

/// 解析 SQL DDL 文件并生成 Rust 代码。
pub fn run(sql_path: &Path) -> Result<String> {
    let content =
        std::fs::read_to_string(sql_path).map_err(|e| anyhow::anyhow!("读取 SQL 文件失败: {e}"))?;
    let tables = parse_sql(&content);
    let rust_code = generate_rust(&tables);
    Ok(rust_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_type_to_rust() {
        assert_eq!(sql_type_to_rust("VARCHAR"), "String");
        assert_eq!(sql_type_to_rust("varchar(32)"), "String");
        assert_eq!(sql_type_to_rust("DOUBLE"), "f64");
        assert_eq!(sql_type_to_rust("BIGINT"), "i64");
        assert_eq!(sql_type_to_rust("BOOLEAN"), "bool");
        assert_eq!(sql_type_to_rust("BLOB"), "Vec<u8>");
        assert_eq!(sql_type_to_rust("JSON"), "serde_json::Value");
        // ADR-006：DECIMAL/NUMERIC 禁止 f64
        assert_eq!(sql_type_to_rust("DECIMAL"), "decimalx::Decimal");
        assert_eq!(sql_type_to_rust("NUMERIC"), "decimalx::Decimal");
        assert_eq!(sql_type_to_rust("NUMERIC(38,18)"), "decimalx::Decimal");
        assert_eq!(sql_type_to_rust("decimal(38, 18)"), "decimalx::Decimal");
        assert_ne!(sql_type_to_rust("DECIMAL"), "f64");
        assert_ne!(sql_type_to_rust("NUMERIC"), "f64");
    }

    #[test]
    fn test_table_name_to_pascal() {
        assert_eq!(table_name_to_pascal("quote"), "Quote");
        assert_eq!(table_name_to_pascal("trade_history"), "TradeHistory");
        assert_eq!(table_name_to_pascal("order_book"), "OrderBook");
    }

    #[test]
    fn test_parse_column_not_null() {
        let col = parse_column("symbol VARCHAR(32) NOT NULL").unwrap();
        assert_eq!(col.name, "symbol");
        assert_eq!(col.sql_type, "VARCHAR");
        assert!(col.is_not_null);
    }

    #[test]
    fn test_parse_column_nullable() {
        let col = parse_column("exchange VARCHAR(32)").unwrap();
        assert_eq!(col.name, "exchange");
        assert!(!col.is_not_null);
    }

    #[test]
    fn test_parse_sql_single_table() {
        let sql = "CREATE TABLE quote (\n    symbol VARCHAR(32) NOT NULL,\n    bid DOUBLE NOT NULL,\n);\n";
        let tables = parse_sql(sql);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].0, "quote");
        assert_eq!(tables[0].1.len(), 2);
    }

    #[test]
    fn test_parse_sql_skips_constraints() {
        let sql = "CREATE TABLE quote (\n    symbol VARCHAR(32) NOT NULL,\n    PRIMARY KEY (symbol),\n);\n";
        let tables = parse_sql(sql);
        assert_eq!(tables[0].1.len(), 1);
    }

    #[test]
    fn test_generate_rust_not_null() {
        let tables = vec![(
            "quote".to_string(),
            vec![Column {
                name: "symbol".to_string(),
                sql_type: "VARCHAR".to_string(),
                is_not_null: true,
            }],
        )];
        let code = generate_rust(&tables);
        assert!(code.contains("pub struct Quote {"));
        assert!(code.contains("pub symbol: String,"));
        assert!(!code.contains("Option"));
    }

    #[test]
    fn test_generate_rust_nullable() {
        let tables = vec![(
            "quote".to_string(),
            vec![Column {
                name: "exchange".to_string(),
                sql_type: "VARCHAR".to_string(),
                is_not_null: false,
            }],
        )];
        let code = generate_rust(&tables);
        assert!(code.contains("pub exchange: Option<String>,"));
    }

    #[test]
    fn test_run_end_to_end() {
        // DOUBLE 仍为显式非金融浮点 → f64；DECIMAL/NUMERIC → decimalx::Decimal
        let sql = "CREATE TABLE quote (\n    symbol VARCHAR(32) NOT NULL,\n    bid NUMERIC(38, 18) NOT NULL,\n    score DOUBLE NOT NULL,\n    exchange VARCHAR(32),\n);\n";
        let path = std::env::temp_dir().join("test_sql_codegen.sql");
        std::fs::write(&path, sql).unwrap();
        let code = run(&path).unwrap();
        assert!(code.contains("pub struct Quote {"));
        assert!(code.contains("pub symbol: String,"));
        assert!(code.contains("pub bid: decimalx::Decimal,"));
        assert!(!code.contains("pub bid: f64,"));
        assert!(code.contains("pub score: f64,"));
        assert!(code.contains("pub exchange: Option<String>,"));
        assert!(code.contains("ADR-006"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_generate_rust_decimal_not_f64() {
        let tables = vec![(
            "trade".to_string(),
            vec![
                Column {
                    name: "price".to_string(),
                    sql_type: "DECIMAL".to_string(),
                    is_not_null: true,
                },
                Column {
                    name: "volume".to_string(),
                    sql_type: "NUMERIC".to_string(),
                    is_not_null: true,
                },
            ],
        )];
        let code = generate_rust(&tables);
        assert!(code.contains("pub price: decimalx::Decimal,"));
        assert!(code.contains("pub volume: decimalx::Decimal,"));
        assert!(!code.contains(": f64,"));
        assert!(!code.contains(": f32,"));
    }
}
