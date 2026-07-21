//! schema_codegen —— 从 `schemas/` 生成 Rust 类型。
//!
//! 子命令：
//! - `protobuf`：从 protobuf 定义生成 Rust 类型
//! - `jsonschema`：从 JSON Schema 生成 Rust 类型
//! - `openapi`：从 OpenAPI 定义生成 Rust 类型
//! - `sql`：从 SQL DDL 生成 Rust 类型

mod jsonschema;
mod openapi;
mod protobuf;
mod sql;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "schema_codegen", about = "从 schemas/ 生成 Rust 类型")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 从 protobuf 定义生成 Rust 类型
    Protobuf {
        /// 输入 .proto 文件路径
        #[arg(short, long)]
        input: PathBuf,
    },
    /// 从 JSON Schema 生成 Rust 类型
    #[command(name = "jsonschema")]
    JsonSchema {
        /// 输入 .json 文件路径
        #[arg(short, long)]
        input: PathBuf,
    },
    /// 从 OpenAPI 定义生成 Rust 类型
    #[command(name = "openapi")]
    Openapi {
        /// 输入 .json 文件路径
        #[arg(short, long)]
        input: PathBuf,
    },
    /// 从 SQL DDL 生成 Rust 类型
    #[command(name = "sql")]
    Sql {
        /// 输入 .sql 文件路径
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Protobuf { input } => {
            let code = protobuf::run(&input)?;
            println!("{code}");
            Ok(())
        }
        Command::JsonSchema { input } => {
            let code = jsonschema::run(&input)?;
            println!("{code}");
            Ok(())
        }
        Command::Openapi { input } => {
            let code = openapi::run(&input)?;
            println!("{code}");
            Ok(())
        }
        Command::Sql { input } => {
            let code = sql::run(&input)?;
            println!("{code}");
            Ok(())
        }
    }
}
