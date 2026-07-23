//! redisx 自验证 live（默认 ignore）。
//!
//! ```bash
//! export FOUNDATIONX_REDISX_ADDR=127.0.0.1:6379
//! export FOUNDATIONX_REDISX_PASSWORD=...
//! cargo test -p redisx --test live_selfcheck --features pubsub -- --ignored
//! ```

use redisx::RedisClient;
use redisx::selfcheck::{CheckLevel, CheckStatus, RedisValidator, Validatable};

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_selfcheck_read_write_passes() {
    let client = RedisClient::connect_from_env().await.expect("connect from env");
    let v = RedisValidator::new(client);
    assert_eq!(v.catalog().len(), 11);
    let report = v.run(CheckLevel::ReadWrite).await;
    assert!(report.passed, "items={:?}", report.items);
    assert!(
        report.items.iter().any(|i| i.id == "redisx.basic.ping" && i.status != CheckStatus::Failed)
    );
}

#[tokio::test]
#[ignore = "requires live Redis; Full 含 TTL sleep 与可选 pubsub"]
async fn live_selfcheck_full_standalone() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let report = RedisValidator::new(client).run(CheckLevel::Full).await;
    // Standalone：cluster_slots 应为 Skipped；其余应 Passed/Degraded
    for item in &report.items {
        if item.id == "redisx.full.cluster_slots" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        #[cfg(not(feature = "pubsub"))]
        if item.id == "redisx.full.pubsub" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        assert!(
            matches!(item.status, CheckStatus::Passed | CheckStatus::Degraded),
            "unexpected {item:?}"
        );
    }
    assert!(report.passed, "failed items: {:?}", report.items);
}
