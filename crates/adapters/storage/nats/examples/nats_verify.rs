//! NATS 库内自验证 CLI（live）。
//!
//! 用法：
//! ```text
//! FOUNDATIONX_NATS_URL=nats://127.0.0.1:4222 \
//!   cargo run -p natsx --example nats_verify
//! ```

use natsx::validation::{CheckLevel, NatsValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = NatsValidator::connect_from_env().await?;

    for level in [CheckLevel::Basic, CheckLevel::ReadWrite, CheckLevel::Full] {
        let report = validator.validate(level).await;
        println!(
            "Level={level:?} passed={} degraded={} total_ms={}",
            report.passed, report.degraded, report.total_ms
        );
        for item in &report.items {
            println!(
                "  {} [{:?}] {}ms baseline={:?} detail={}",
                item.id,
                item.status,
                item.latency_ms,
                item.baseline_ms,
                item.detail.as_deref().unwrap_or("-")
            );
        }
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
        if !report.passed {
            std::process::exit(1);
        }
    }
    Ok(())
}
