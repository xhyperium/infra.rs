//! Live 自验证：Basic + ReadWrite 必须通过；Full 在可达环境尽量跑通。
//!
//! ```bash
//! node scripts/live/build-foundationx-env.mjs --env dev --out /tmp/fx.env
//! set -a; source /tmp/fx.env; set +a
//! cargo test -p postgresx --test live_selfcheck -- --ignored --test-threads=1
//! ```

use postgresx::selfcheck::{CheckLevel, CheckStatus, PostgresValidator, Validatable};

fn live_env_present() -> bool {
    std::env::var("FOUNDATIONX_POSTGRESX_HOST").is_ok()
        || std::env::var("DATABASE_URL").is_ok()
        || std::env::var("FOUNDATIONX_POSTGRESX_URL").is_ok()
}

#[tokio::test]
#[ignore = "需要 FOUNDATIONX_POSTGRESX_* 或 DATABASE_URL"]
async fn live_selfcheck_basic_and_readwrite() {
    if !live_env_present() {
        eprintln!("skip: 无 live 环境变量");
        return;
    }
    let v = PostgresValidator::connect_from_env().await.expect("connect_from_env");
    let report = v.run(CheckLevel::ReadWrite).await;
    eprintln!(
        "selfcheck RW module={} passed={} degraded={} items={}",
        report.module,
        report.passed,
        report.degraded,
        report.items.len()
    );
    for i in &report.items {
        eprintln!("  {} {:?} {}ms {:?}", i.id, i.status, i.latency_ms, i.detail);
    }
    assert_eq!(report.module, "postgres");
    assert_eq!(report.items.len(), 5, "basic×2 + rw×3");
    assert!(
        report.passed,
        "Basic+RW 不得 Failed: {:?}",
        report
            .items
            .iter()
            .filter(|i| i.status == CheckStatus::Failed)
            .map(|i| (&i.id, &i.detail))
            .collect::<Vec<_>>()
    );
    // 无残留：token 表应已 DROP（抽查 catalog 表名模式）
    let token_like = report.items.iter().any(|i| i.id.contains("crud") || i.id.contains("tx_"));
    assert!(token_like);
}

#[tokio::test]
#[ignore = "需要 FOUNDATIONX_POSTGRESX_* 或 DATABASE_URL"]
async fn live_selfcheck_full_honest() {
    if !live_env_present() {
        eprintln!("skip: 无 live 环境变量");
        return;
    }
    let v = PostgresValidator::connect_from_env().await.expect("connect_from_env");
    let report = v.run(CheckLevel::Full).await;
    eprintln!(
        "selfcheck Full passed={} degraded={} total_ms={}",
        report.passed, report.degraded, report.total_ms
    );
    for i in &report.items {
        eprintln!("  {} {:?} {}ms detail={:?}", i.id, i.status, i.latency_ms, i.detail);
    }
    assert_eq!(report.module, "postgres");
    assert_eq!(report.items.len(), 11);

    // replication_lag 默认 Skipped
    let lag = report
        .items
        .iter()
        .find(|i| i.id == "postgres.full.replication_lag")
        .expect("replication_lag");
    assert_eq!(lag.status, CheckStatus::Skipped, "默认 replica_check=false 必须 Skipped");
    assert!(lag.detail.as_ref().is_some_and(|d| d.contains("replica_check")), "detail 说明原因");

    // 非可选 Full 项不得静默 Skipped（除非 basic 失败——此处应已连通）
    for i in &report.items {
        if i.id.starts_with("postgres.full.") && i.id != "postgres.full.replication_lag" {
            assert_ne!(i.status, CheckStatus::Skipped, "{} 不应被短路 Skipped", i.id);
        }
    }

    // catalog 与运行项 ID 一致
    let cat_ids: Vec<_> = v.catalog().into_iter().map(|d| d.id).collect();
    for i in &report.items {
        assert!(cat_ids.contains(&i.id), "report id 不在 catalog: {}", i.id);
    }

    assert!(
        report.passed,
        "Full 非可选项应通过（允许 Degraded）: failed={:?}",
        report
            .items
            .iter()
            .filter(|i| i.status == CheckStatus::Failed)
            .map(|i| (&i.id, &i.detail))
            .collect::<Vec<_>>()
    );

    // 清理痕迹：不应再存在 _self_check_% 表（尽力检查）
    let leftover = v
        .pool()
        .query(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename LIKE '_self_check_%'",
            &[],
        )
        .await
        .expect("leftover query");
    assert!(
        leftover.is_empty(),
        "残留自检表: {:?}",
        leftover.iter().map(|r| r.try_get::<_, String>(0).unwrap_or_default()).collect::<Vec<_>>()
    );
}
