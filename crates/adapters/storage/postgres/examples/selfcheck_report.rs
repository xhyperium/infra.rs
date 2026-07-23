//! 捕获 Full 级自检 JSON 报告（live / 本地运维）。
use postgresx::selfcheck::{CheckLevel, PostgresValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v = PostgresValidator::connect_from_env().await?;
    let report = v.run(CheckLevel::Full).await;
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    if !report.passed {
        std::process::exit(1);
    }
    Ok(())
}
