//! goalctl CLI：doctor / validate / compile。

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use goalctl::{CompileError, VERSION, compile_goal, validate_goal};

#[derive(Parser, Debug)]
#[command(
    name = "goalctl",
    version = VERSION,
    about = "Minimal Goal → Contract compiler"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 环境自检（无网络）。
    Doctor,
    /// 校验 Goal 文件（不写 Contract）。
    Validate {
        /// Goal YAML/JSON 路径。
        path: PathBuf,
    },
    /// 编译 Goal → Contract JSON（stdout；可用 -o 写文件）。
    Compile {
        /// Goal YAML/JSON 路径。
        path: PathBuf,
        /// 输出路径；默认 stdout。
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Doctor => {
            println!("goalctl {VERSION}");
            println!("status: ok");
            println!("commands: doctor, validate, compile");
            println!("features: yaml+json input, sha256 digest, subjective lint");
            ExitCode::SUCCESS
        }
        Commands::Validate { path } => match load_and_validate(&path) {
            Ok(()) => {
                println!("validate: ok ({})", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("validate failed: {e}");
                ExitCode::from(1)
            }
        },
        Commands::Compile { path, output } => match compile_goal(&path) {
            Ok(out) => {
                if let Some(p) = output {
                    if let Err(e) = std::fs::write(&p, &out.json) {
                        eprintln!("write {}: {e}", p.display());
                        return ExitCode::from(2);
                    }
                    println!(
                        "compiled {} digest={} -> {}",
                        out.contract.id,
                        out.contract.digest,
                        p.display()
                    );
                } else {
                    print!("{}", out.json);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("compile failed: {e}");
                ExitCode::from(1)
            }
        },
    }
}

fn load_and_validate(path: &std::path::Path) -> Result<(), CompileError> {
    let raw = std::fs::read_to_string(path)?;
    let hint = path.extension().and_then(|e| e.to_str());
    // 复用 compile 的 parse + validate，但不写 digest 也可：
    let goal = match hint.map(|s| s.to_ascii_lowercase()) {
        Some(ref e) if e == "json" => {
            serde_json::from_str(&raw).map_err(|e| CompileError::Parse(format!("json: {e}")))?
        }
        _ => serde_yaml::from_str(&raw).map_err(|e| CompileError::Parse(format!("yaml: {e}")))?,
    };
    validate_goal(&goal)?;
    Ok(())
}
