//! verifyctl CLI：plan / execute / report。

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use verifyctl::{
    PlanOptions, RunStatus, VERSION, aggregate_report, build_plan, execute_plan, write_report,
};

#[derive(Parser, Debug)]
#[command(
    name = "verifyctl",
    version = VERSION,
    about = "Minimal verification planner / executor / reporter"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 生成 VerificationPlan JSON。
    Plan {
        /// Goal Contract JSON 路径（可省略，则 digest 空）。
        #[arg(long)]
        contract: Option<PathBuf>,
        /// 变更路径（可重复）。
        #[arg(long = "changed")]
        changed: Vec<String>,
        /// 输出路径；默认 stdout。
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 执行 plan JSON。
    Execute {
        /// VerificationPlan JSON 路径。
        plan: PathBuf,
        /// 仓库根（默认 cwd）。
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// 运行结果输出路径。
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 读取 RunResult 并打印状态；可选写规范化报告。
    Report {
        /// RunResult JSON 路径。
        run: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Plan { contract, changed, output } => {
            let contract_json = match contract {
                Some(p) => match std::fs::read_to_string(&p) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("read contract {}: {e}", p.display());
                        return ExitCode::from(2);
                    }
                },
                None => String::new(),
            };
            match build_plan(&contract_json, &changed, &PlanOptions::from_env()) {
                Ok(plan) => {
                    let json = serde_json::to_string_pretty(&plan).expect("serialize plan");
                    if let Some(p) = output {
                        if let Err(e) = std::fs::write(&p, &json) {
                            eprintln!("write {}: {e}", p.display());
                            return ExitCode::from(2);
                        }
                        println!("plan_digest={} -> {}", plan.plan_digest, p.display());
                    } else {
                        print!("{json}");
                    }
                    #[cfg(feature = "with-evidence")]
                    {
                        if let Ok(ev) = std::env::var("VERIFYCTL_EVIDENCE") {
                            let _ = verifyctl::append_evidence(
                                std::path::Path::new(&ev),
                                "verifyctl.plan",
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("plan failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Commands::Execute { plan, cwd, output } => {
            let raw = match std::fs::read_to_string(&plan) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("read plan {}: {e}", plan.display());
                    return ExitCode::from(2);
                }
            };
            let plan_obj: verifyctl::VerificationPlan = match serde_json::from_str(&raw) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("parse plan: {e}");
                    return ExitCode::from(1);
                }
            };
            let cwd = cwd.unwrap_or_else(|| PathBuf::from("."));
            match execute_plan(&plan_obj, &cwd) {
                Ok(run) => {
                    let json = serde_json::to_string_pretty(&run).expect("serialize run");
                    if let Some(p) = output {
                        if let Err(e) = std::fs::write(&p, &json) {
                            eprintln!("write {}: {e}", p.display());
                            return ExitCode::from(2);
                        }
                    } else {
                        print!("{json}");
                    }
                    #[cfg(feature = "with-evidence")]
                    {
                        if let Ok(ev) = std::env::var("VERIFYCTL_EVIDENCE") {
                            let name = match run.status {
                                RunStatus::Pass => "verifyctl.execute.pass",
                                RunStatus::Fail => "verifyctl.execute.fail",
                            };
                            let _ = verifyctl::append_evidence(std::path::Path::new(&ev), name);
                        }
                    }
                    match run.status {
                        RunStatus::Pass => ExitCode::SUCCESS,
                        RunStatus::Fail => ExitCode::from(1),
                    }
                }
                Err(e) => {
                    eprintln!("execute failed: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Commands::Report { run, output } => {
            let raw = match std::fs::read_to_string(&run) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("read run {}: {e}", run.display());
                    return ExitCode::from(2);
                }
            };
            let parsed: verifyctl::RunResult = match serde_json::from_str(&raw) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("parse run: {e}");
                    return ExitCode::from(1);
                }
            };
            let report = aggregate_report(parsed);
            println!(
                "status={:?} plan_digest={} checks={}",
                report.status,
                report.plan_digest,
                report.checks.len()
            );
            if let Some(p) = output {
                if let Err(e) = write_report(&p, &report) {
                    eprintln!("write report: {e}");
                    return ExitCode::from(2);
                }
            }
            match report.status {
                RunStatus::Pass => ExitCode::SUCCESS,
                RunStatus::Fail => ExitCode::from(1),
            }
        }
    }
}
