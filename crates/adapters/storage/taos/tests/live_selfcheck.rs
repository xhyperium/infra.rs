//! taosx 自验证 live（默认 ignore）。
//!
//! ```bash
//! export FOUNDATIONX_TAOSX_PASSWORD=...
//! cargo test -p taosx --test live_selfcheck -- --ignored
//! ```

use taosx::TaosPool;
use taosx::selfcheck::{CheckLevel, CheckStatus, TaosValidator, Validatable};

#[tokio::test]
#[ignore = "requires live TDengine REST; set FOUNDATIONX_TAOSX_PASSWORD"]
async fn live_selfcheck_read_write_passes() {
    let pool = TaosPool::connect_from_env().await.expect("connect from env");
    let v = TaosValidator::new(pool);
    assert_eq!(v.catalog().len(), 9);
    let report = v.run(CheckLevel::ReadWrite).await;
    assert!(report.passed, "items={:?}", report.items);
    assert!(
        report.items.iter().any(|i| i.id == "taos.basic.ping" && i.status != CheckStatus::Failed)
    );
    assert!(
        report
            .items
            .iter()
            .any(|i| i.id == "taos.rw.insert_query" && i.status != CheckStatus::Failed)
    );
}

#[tokio::test]
#[ignore = "requires live TDengine; Full 含 DDL 与批量写入"]
async fn live_selfcheck_full_rest() {
    let pool = TaosPool::connect_from_env().await.expect("connect");
    let report = TaosValidator::new(pool).run(CheckLevel::Full).await;
    for item in &report.items {
        assert!(
            matches!(item.status, CheckStatus::Passed | CheckStatus::Degraded),
            "unexpected {item:?}"
        );
    }
    assert!(report.passed, "failed items: {:?}", report.items);
    assert!(report.items.iter().any(|i| i.id == "taos.full.tmq_subscribe"
        && matches!(i.status, CheckStatus::Passed | CheckStatus::Degraded)));
}
