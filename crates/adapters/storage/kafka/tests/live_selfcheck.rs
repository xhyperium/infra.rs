//! kafkax 自验证 live（默认 ignore）。
//!
//! ```bash
//! export FOUNDATIONX_KAFKAX_BROKERS=127.0.0.1:9092
//! cargo test -p kafkax --test live_selfcheck -- --ignored --nocapture
//! # 或: node scripts/kafka-broker-conformance.mjs  # 其它 live 路径
//! ```

use std::time::Duration;

use kafkax::KafkaConfigBuilder;
use kafkax::selfcheck::{CheckLevel, CheckStatus, KafkaValidator, Validatable};

#[tokio::test]
#[ignore = "requires live Kafka (FOUNDATIONX_KAFKAX_BROKERS)"]
async fn live_selfcheck_read_write_passes() {
    let pool = kafkax::KafkaPool::connect_from_env().await.expect("connect from env");
    let v = KafkaValidator::new(pool);
    assert_eq!(v.catalog().len(), 9);
    let report = v.run(CheckLevel::ReadWrite).await;
    assert!(report.passed, "items={:?}", report.items);
    let meta = report.items.iter().find(|i| i.id == "kafka.basic.metadata").expect("meta");
    assert!(matches!(meta.status, CheckStatus::Passed | CheckStatus::Degraded), "{meta:?}");
    let rw = report.items.iter().find(|i| i.id == "kafka.rw.produce_consume").expect("rw");
    assert!(matches!(rw.status, CheckStatus::Passed | CheckStatus::Degraded), "{rw:?}");
    let _ = v.pool().close(Duration::from_secs(3)).await;
}

#[tokio::test]
#[ignore = "requires live Kafka; Full 含 NO-GO Skipped"]
async fn live_selfcheck_full_with_nogo_skipped() {
    let pool = kafkax::KafkaPool::connect_from_env().await.expect("connect");
    let report = KafkaValidator::new(pool.clone()).run(CheckLevel::Full).await;
    for item in &report.items {
        match item.id.as_str() {
            "kafka.full.group_lag" | "kafka.full.isr_health" => {
                assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
                assert!(item.detail.as_ref().is_some_and(|d| d.contains("NO-GO")), "{item:?}");
            }
            _ => {
                assert!(
                    matches!(
                        item.status,
                        CheckStatus::Passed | CheckStatus::Degraded | CheckStatus::Skipped
                    ),
                    "unexpected {item:?}"
                );
                assert_ne!(item.status, CheckStatus::Failed, "failed {item:?}");
            }
        }
    }
    assert!(report.passed, "failed items: {:?}", report.items);
    let _ = pool.close(Duration::from_secs(3)).await;
}

#[tokio::test]
async fn offline_connect_and_run_is_deterministic() {
    let cfg = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:1")
        .connect_timeout(Duration::from_millis(200))
        .operation_timeout(Duration::from_millis(200))
        .delivery_timeout(Duration::from_millis(200))
        .build()
        .expect("cfg");
    let r1 = KafkaValidator::connect_and_run(cfg.clone(), CheckLevel::Full).await;
    let r2 = KafkaValidator::connect_and_run(cfg, CheckLevel::Full).await;
    assert_eq!(r1.module, "kafka");
    assert!(!r1.passed);
    assert_eq!(r1.items.len(), 9);
    assert_eq!(r1.passed, r2.passed);
    assert_eq!(r1.items.len(), r2.items.len());
}
