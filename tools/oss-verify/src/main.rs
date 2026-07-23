//! OSS 独立自验证 CLI。
//!
//! 需要 `FOUNDATIONX_OSSX_*` 环境变量（参见 secrets/env/dev.md）。

use std::time::Instant;
use clap::{Parser, Subcommand};
use oss_verify::suite::{all_check_specs, execute_check, aggregate_results};
use ossx::{OssConfig, OssPool};

#[derive(Parser)]
#[command(name = "oss-verify", about = "OSS 独立自验证工具", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 执行验证
    Run {
        /// 仅执行指定层的检查（逗号分隔: 0,1,2,3,4,5）
        #[arg(short = 'l', long)]
        layers: Option<String>,
        /// 输出 JSON 文件路径
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
    /// 列出所有检查
    List,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::List => cmd_list(),
        Commands::Run { layers, output } => cmd_run(layers, output).await,
    }
}

fn cmd_list() {
    println!("OSS 验证检查清单 (ossx v0.4.0):\n");
    let specs = all_check_specs();
    let mut current_layer: u8 = 255;
    for (id, kind, layer) in &specs {
        if *layer != current_layer {
            current_layer = *layer;
            let layer_name = match layer {
                0 => "L0: 配置验证 (offline)",
                1 => "L1: 连接检测 (online)",
                2 => "L2: 基本操作",
                3 => "L3: 流式操作",
                4 => "L4: 高级功能",
                5 => "L5: 安全与并发",
                _ => "未知",
            };
            println!("\n  {layer_name}");
        }
        println!("    [{kind:?}] {id}");
    }
    println!("\n共 {} 项检查", specs.len());
}

async fn cmd_run(layers: Option<String>, output: Option<String>) {
    // 解析层过滤
    let layer_filter: Option<Vec<u8>> = layers.map(|l| {
        l.split(',').filter_map(|s| s.trim().parse().ok()).collect()
    });

    // 加载配置
    let config = match OssConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("配置加载失败: {e:?}");
            eprintln!("请设置 FOUNDATIONX_OSSX_* 环境变量");
            std::process::exit(1);
        }
    };

    println!("OSS 自验证开始...");
    println!("  endpoint: {}", config.endpoint);
    println!("  bucket:   {}", config.bucket);
    println!("  region:   {}", config.region);
    println!();

    // 连接
    let pool = match OssPool::connect(config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("OSS 连接失败: {e:?}");
            std::process::exit(1);
        }
    };

    let start = Instant::now();
    let specs = all_check_specs();
    let mut results = Vec::new();

    for (id, kind, layer) in &specs {
        if let Some(ref filter) = layer_filter {
            if !filter.contains(layer) { continue; }
        }

        print!("[{:>2}] {:<30} ... ", layer, id);
        let result = execute_check(&pool, id, *kind, id).await;
        let status = if result.passed { "PASS" } else { "FAIL" };
        println!("{} ({:>4}ms) {}", status, result.duration_ms, result.message);
        if let Some(ref detail) = result.detail {
            if !result.passed {
                println!("       └─ {detail}");
            }
        }
        results.push(result);
    }

    let total_duration = start.elapsed().as_millis() as u64;
    let run_result = aggregate_results(results, total_duration);

    println!();
    println!("═══════════════════════════════════════");
    println!("  结果: {}  |  {}/{} passed  |  {}ms",
        match run_result.status {
            oss_verify::types::RunStatus::Pass => "PASS",
            oss_verify::types::RunStatus::Fail => "FAIL",
            oss_verify::types::RunStatus::Partial => "PARTIAL",
        },
        run_result.passed,
        run_result.total,
        total_duration
    );
    println!("═══════════════════════════════════════");

    // 输出 JSON
    if let Some(path) = output {
        let json = serde_json::to_string_pretty(&run_result).unwrap_or_default();
        std::fs::write(&path, &json).unwrap_or_else(|e| {
            eprintln!("写入报告失败: {e}");
        });
        println!("\n报告已写入: {path}");
    }

    pool.close(std::time::Duration::from_secs(5)).await.ok();
}
