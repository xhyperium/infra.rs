//! postgresx 功能 API 全覆盖测试套件。
//!
//! # 运行分类
//!
//! | 类别 | 命令 |
//! |------|------|
//! | 离线（默认） | `cargo test -p postgresx --test functional_api` |
//! | Live（需要 PG） | `cargo test -p postgresx --test functional_api -- --ignored` |
//! | Scaffold | `cargo test -p postgresx --test functional_api --features scaffold` |
//!
//! Live 测试环境变量（`FOUNDATIONX_POSTGRESX_*`或`DATABASE_URL`）。

// ==== 1. Config ====

#[test]
fn config_defaults_are_stable() {
    assert_eq!(postgresx::DEFAULT_PORT, 5432);
    assert_eq!(postgresx::DEFAULT_MAX_POOL_SIZE, 16);
    assert_eq!(postgresx::DEFAULT_COPY_IN_MAX_BYTES, 16 * 1024 * 1024);
    assert_eq!(postgresx::DEFAULT_COPY_OUT_MAX_BYTES, 16 * 1024 * 1024);
}

#[test]
fn config_builder_roundtrip_all_fields() {
    use std::time::Duration;
    use postgresx::{PostgresConfig, SslMode};

    let cfg = PostgresConfig::builder()
        .host("pg.example.io")
        .port(15432)
        .database("analytics")
        .user("reporter")
        .password("s3cret!")
        .sslmode(SslMode::Require)
        .max_pool_size(8)
        .application_name("func-test")
        .connect_timeout(Duration::from_secs(15))
        .acquire_timeout(Duration::from_secs(3))
        .operation_timeout(Duration::from_secs(30))
        .tls_server_name("cert.example.io")
        .build()
        .expect("valid config");

    assert_eq!(cfg.host, "pg.example.io");
    assert_eq!(cfg.port, 15432);
    assert_eq!(cfg.database, "analytics");
    assert_eq!(cfg.user, "reporter");
    assert_eq!(cfg.password, "s3cret!");
    assert_eq!(cfg.sslmode, SslMode::Require);
    assert_eq!(cfg.max_pool_size, 8);
    assert_eq!(cfg.application_name.as_deref(), Some("func-test"));
    assert_eq!(cfg.connect_timeout, Some(Duration::from_secs(15)));
    assert_eq!(cfg.acquire_timeout, Duration::from_secs(3));
    assert_eq!(cfg.operation_timeout, Duration::from_secs(30));
    assert_eq!(cfg.tls_server_name.as_deref(), Some("cert.example.io"));
}

#[test]
fn config_sslmode_parse_all_variants() {
    use postgresx::SslMode;
    assert_eq!(SslMode::parse("disable").unwrap(), SslMode::Disable);
    assert_eq!(SslMode::parse("DISABLE").unwrap(), SslMode::Disable);
    assert_eq!(SslMode::parse("false").unwrap(), SslMode::Disable);
    assert_eq!(SslMode::parse("0").unwrap(), SslMode::Disable);
    assert_eq!(SslMode::parse("prefer").unwrap(), SslMode::Prefer);
    assert_eq!(SslMode::parse("allow").unwrap(), SslMode::Prefer);
    assert_eq!(SslMode::parse("require").unwrap(), SslMode::Require);
    assert_eq!(SslMode::parse("verify-ca").unwrap(), SslMode::Require);
    assert_eq!(SslMode::parse("verify-full").unwrap(), SslMode::Require);
    assert!(SslMode::parse("tls-v99").is_err());
    assert!(SslMode::parse("").is_err());
}

#[test]
fn config_sslmode_as_str_roundtrip() {
    use postgresx::SslMode;
    assert_eq!(SslMode::Disable.as_str(), "disable");
    assert_eq!(SslMode::Prefer.as_str(), "prefer");
    assert_eq!(SslMode::Require.as_str(), "require");
    assert_eq!(SslMode::default(), SslMode::Disable);
}

#[test]
fn config_debug_redacts_password() {
    use postgresx::PostgresConfig;
    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .database("db")
        .user("alice")
        .password("top-secret!")
        .build()
        .expect("cfg");
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("***"));
    assert!(!dbg.contains("top-secret"));
    assert!(dbg.contains("127.0.0.1"));
}

#[test]
fn config_validation_rejects_empty_host() {
    let err = postgresx::PostgresConfig::builder()
        .host("")
        .database("db")
        .user("u")
        .build()
        .expect_err("empty host");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

#[test]
fn config_validation_rejects_zero_port() {
    let err = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .port(0)
        .database("db")
        .user("u")
        .build()
        .expect_err("zero port");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

#[test]
fn config_validation_rejects_zero_max_pool_size() {
    let err = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .database("db")
        .user("u")
        .max_pool_size(0)
        .build()
        .expect_err("zero pool");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

#[test]
fn config_validation_rejects_zero_timeouts() {
    use std::time::Duration;
    for result in [
        postgresx::PostgresConfig::builder()
            .host("127.0.0.1").database("db").user("u")
            .connect_timeout(Duration::ZERO).build(),
        postgresx::PostgresConfig::builder()
            .host("127.0.0.1").database("db").user("u")
            .acquire_timeout(Duration::ZERO).build(),
        postgresx::PostgresConfig::builder()
            .host("127.0.0.1").database("db").user("u")
            .operation_timeout(Duration::ZERO).build(),
    ] {
        assert_eq!(result.expect_err("zero timeout").kind(), kernel::ErrorKind::Invalid);
    }
}

#[test]
fn config_remote_plaintext_is_rejected() {
    use postgresx::SslMode;
    for mode in [SslMode::Disable, SslMode::Prefer] {
        let err = postgresx::PostgresConfig::builder()
            .host("db.example.com")
            .database("db")
            .user("user")
            .sslmode(mode)
            .build()
            .expect_err("remote non-require");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }
}

#[test]
fn config_missing_user_is_rejected() {
    let err = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .database("db")
        .build()
        .expect_err("missing user");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

#[test]
fn config_database_url_parse() {
    let cfg = postgresx::PostgresConfig::from_database_url(
        "postgres://alice:s3cret@127.0.0.1:5433/market?sslmode=disable",
    )
    .expect("url");
    assert_eq!(cfg.user, "alice");
    assert_eq!(cfg.password, "s3cret");
    assert_eq!(cfg.port, 5433);
    assert_eq!(cfg.database, "market");
}

// ━━━━━━━━━━━━━━━━━━━━━━━ 2. Error 映射 ━━━━━━━━━━━━━━━━━━━━

#[test]
fn error_kind_from_sqlstate_all_classes() {
    use postgresx::error_kind_from_sqlstate;
    use kernel::ErrorKind;

    // Class 08 → Unavailable
    assert_eq!(error_kind_from_sqlstate("08006"), ErrorKind::Unavailable);
    assert_eq!(error_kind_from_sqlstate("08000"), ErrorKind::Unavailable);
    // Class 22 → Invalid
    assert_eq!(error_kind_from_sqlstate("22001"), ErrorKind::Invalid);
    // Class 23 — unique → Conflict, FK → Invalid
    assert_eq!(error_kind_from_sqlstate("23505"), ErrorKind::Conflict);
    assert_eq!(error_kind_from_sqlstate("23503"), ErrorKind::Invalid);
    assert_eq!(error_kind_from_sqlstate("23502"), ErrorKind::Invalid);
    assert_eq!(error_kind_from_sqlstate("23514"), ErrorKind::Invalid);
    // Class 25 → Invariant
    assert_eq!(error_kind_from_sqlstate("25001"), ErrorKind::Invariant);
    // Class 28 → Invalid
    assert_eq!(error_kind_from_sqlstate("28P01"), ErrorKind::Invalid);
    // Class 3D → Invalid
    assert_eq!(error_kind_from_sqlstate("3D000"), ErrorKind::Invalid);
    // Class 40 → Transient
    assert_eq!(error_kind_from_sqlstate("40P01"), ErrorKind::Transient);
    assert_eq!(error_kind_from_sqlstate("40001"), ErrorKind::Transient);
    // Class 42 — Missing
    assert_eq!(error_kind_from_sqlstate("42P01"), ErrorKind::Missing);
    assert_eq!(error_kind_from_sqlstate("42703"), ErrorKind::Invalid);
    // Class 53 → Transient
    assert_eq!(error_kind_from_sqlstate("53300"), ErrorKind::Transient);
    // Class 55 → Transient
    assert_eq!(error_kind_from_sqlstate("55P03"), ErrorKind::Transient);
    // Class 57 → Cancelled
    assert_eq!(error_kind_from_sqlstate("57014"), ErrorKind::Cancelled);
    // Class 58 → Unavailable
    assert_eq!(error_kind_from_sqlstate("58000"), ErrorKind::Unavailable);
    // Class XX → Internal
    assert_eq!(error_kind_from_sqlstate("XX000"), ErrorKind::Internal);
    // Unknown → Internal
    assert_eq!(error_kind_from_sqlstate("ZZ999"), ErrorKind::Internal);
    // P0001 → Invalid
    assert_eq!(error_kind_from_sqlstate("P0001"), ErrorKind::Invalid);
}

#[test]
fn xerror_from_sqlstate_maps_missing() {
    let err = postgresx::xerror_from_sqlstate("42P01", "relation \"foo\" does not exist");
    assert_eq!(err.kind(), kernel::ErrorKind::Missing);
    assert!(err.context().contains("42P01"));
    assert!(err.context().contains("foo"));
}

#[test]
fn xerror_from_sqlstate_maps_conflict() {
    let err = postgresx::xerror_from_sqlstate("23505", "duplicate key value");
    assert_eq!(err.kind(), kernel::ErrorKind::Conflict);
}

#[test]
fn xerror_from_sqlstate_maps_transient() {
    let err = postgresx::xerror_from_sqlstate("40P01", "deadlock detected");
    assert_eq!(err.kind(), kernel::ErrorKind::Transient);
}

#[test]
fn map_pool_error_closed_is_unavailable() {
    let err = postgresx::map_pool_error(deadpool_postgres::PoolError::Closed);
    assert_eq!(err.kind(), kernel::ErrorKind::Unavailable);
}

// PoolError::Timeout has a parameter type that varies by deadpool version.
// Deadpool 0.12 使用 TimeoutType；我们仅测试精确枚举变体。

#[test]
fn transaction_rollback_failure_structure() {
    use std::error::Error;
    let original = kernel::XError::deadline_exceeded("业务超时");
    let rollback_err = kernel::XError::unavailable("回滚断连");
    let composite = postgresx::TransactionRollbackFailure::new(original, rollback_err);
    assert_eq!(
        composite.original().kind(),
        kernel::ErrorKind::DeadlineExceeded
    );
    assert_eq!(composite.rollback().kind(), kernel::ErrorKind::Unavailable);
    assert!(Error::source(&composite).is_some());
    assert_eq!(composite.to_string(), "事务原操作失败且回滚失败");
}

#[test]
fn transaction_rollback_failure_source_chain_downcast() {
    use std::error::Error;
    use postgresx::TransactionRollbackFailure;
    let original = kernel::XError::conflict("业务冲突");
    let rollback = kernel::XError::unavailable("回滚断连");
    let composite = TransactionRollbackFailure::new(original, rollback);
    // source() 返回 original XError
    let source: &kernel::XError = Error::source(&composite)
        .and_then(|s| s.downcast_ref::<kernel::XError>())
        .expect("source must be XError");
    assert_eq!(source.kind(), kernel::ErrorKind::Conflict);
}

// ==== 3. Resilience ====

#[test]
fn retry_config_fixed_fields() {
    let cfg = postgresx::PgRetryConfig::fixed(3, 10);
    assert_eq!(cfg.max_attempts, 3);
    assert_eq!(cfg.base_delay_ms, 10);
}

#[test]
fn with_retry_sync_success() {
    let cfg = postgresx::PgRetryConfig::fixed(3, 0);
    let v: i32 = postgresx::with_retry_sync(&cfg, "surface", || Ok(42)).unwrap();
    assert_eq!(v, 42);
}

#[test]
fn with_retry_sync_exhausts() {
    let cfg = postgresx::PgRetryConfig::fixed(2, 0);
    let err = postgresx::with_retry_sync::<i32, _>(&cfg, "fail", || {
        Err(kernel::XError::transient("retry fail"))
    })
    .expect_err("exhausted");
    assert_eq!(err.kind(), kernel::ErrorKind::Transient);
}

#[tokio::test]
async fn with_retry_async_success() {
    let cfg = postgresx::PgRetryConfig::fixed(2, 0);
    let v = postgresx::with_retry_async(&cfg, "surface", || async { Ok(42) })
        .await
        .unwrap();
    assert_eq!(v, 42);
}

#[tokio::test]
async fn with_retry_async_retries_transient() {
    use std::sync::atomic::{AtomicU32, Ordering};
    let cfg = postgresx::PgRetryConfig::fixed(3, 0);
    let calls = AtomicU32::new(0);
    let v: u32 = postgresx::with_retry_async(&cfg, "retry-transient", || {
        let c = calls.fetch_add(1, Ordering::SeqCst) + 1;
        async move {
            if c < 2 {
                Err(kernel::XError::transient("fail"))
            } else {
                Ok(c)
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(v, 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn with_budget_noop_success() {
    use resiliencx::RetryBudget;
    let budget = RetryBudget::new(10);
    let v = postgresx::with_budget_noop(&budget, 3, "test", || Ok::<i32, kernel::XError>(7))
        .unwrap();
    assert_eq!(v, 7);
}

#[test]
fn with_budget_exhausted() {
    use resiliencx::RetryBudget;
    let budget = RetryBudget::new(1);
    let err = postgresx::with_budget_noop(&budget, 4, "pg.query", || {
        Err::<(), _>(kernel::XError::transient("conn reset"))
    })
    .unwrap_err();
    assert!(
        matches!(
            err.kind(),
            kernel::ErrorKind::Unavailable | kernel::ErrorKind::Transient
        )
    );
}

#[test]
fn with_budget_safe_read_only_success() {
    use resiliencx::{RetryBudget, RetrySafety};
    let budget = RetryBudget::new(2);
    let v = postgresx::with_budget_safe_noop(
        &budget,
        2,
        RetrySafety::ReadOnly,
        "pg.query",
        || Ok(42_u8),
    )
    .unwrap();
    assert_eq!(v, 42);
}

// ==== 4. TLS ====

#[test]
fn build_client_config_with_webpki() {
    let _cfg = postgresx::build_client_config().expect("rustls client config");
}

#[test]
fn make_rustls_connect_rejects_empty_domain() {
    let maker = postgresx::MakeRustlsConnect::with_webpki_roots().expect("maker");
    let err = maker.for_domain("").expect_err("empty domain");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

#[test]
fn make_rustls_connect_accepts_valid_domain() {
    let maker = postgresx::MakeRustlsConnect::with_webpki_roots().expect("maker");
    let connect = maker.for_domain("db.example.io").expect("sni");
    let dbg = format!("{connect:?}");
    assert!(dbg.contains("RustlsConnect"));
}

#[test]
fn extra_ca_missing_file_fails_closed() {
    let err = postgresx::build_client_config_with_ca(Some(std::path::Path::new("/no/such/ca.pem")))
        .expect_err("missing ca");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
}

// ==== 5. Selfcheck ====

#[test]
fn selfcheck_config_defaults() {
    let cfg = postgresx::selfcheck::PostgresSelfCheckConfig::default();
    assert!(!cfg.replica_check);
    assert!(cfg.skip.is_empty());
    assert_eq!(cfg.min_version, 13);
    assert_eq!(cfg.max_replication_lag_ms, 1_000);
}

#[test]
fn selfcheck_config_skip_items() {
    use postgresx::selfcheck::PostgresSelfCheckConfig;
    let mut cfg = PostgresSelfCheckConfig::default();
    cfg.skip.insert("postgres.basic.ping".into());
    cfg.skip.insert("postgres.basic.version".into());
    assert!(cfg.is_skipped("postgres.basic.ping"));
    assert!(cfg.is_skipped("postgres.basic.version"));
    assert!(!cfg.is_skipped("postgres.rw.crud_roundtrip"));
}

#[test]
fn selfcheck_level_ordering() {
    use postgresx::selfcheck::CheckLevel;
    assert!(CheckLevel::Full.includes(CheckLevel::Basic));
    assert!(CheckLevel::Full.includes(CheckLevel::ReadWrite));
    assert!(CheckLevel::ReadWrite.includes(CheckLevel::Basic));
    assert!(!CheckLevel::Basic.includes(CheckLevel::Full));
}

#[test]
fn selfcheck_level_duration() {
    use std::time::Duration;
    use postgresx::selfcheck::CheckLevel;
    assert_eq!(CheckLevel::Basic.max_duration(), Duration::from_secs(3));
    assert_eq!(CheckLevel::ReadWrite.max_duration(), Duration::from_secs(15));
    assert_eq!(CheckLevel::Full.max_duration(), Duration::from_secs(120));
}

#[test]
fn selfcheck_level_as_str() {
    use postgresx::selfcheck::CheckLevel;
    assert_eq!(CheckLevel::Basic.as_str(), "basic");
    assert_eq!(CheckLevel::ReadWrite.as_str(), "read_write");
    assert_eq!(CheckLevel::Full.as_str(), "full");
}

#[test]
fn selfcheck_report_structure() {
    use std::time::Duration;
    use postgresx::selfcheck::{CheckLevel, CheckStatus, CheckItem, ValidationReport, now_rfc3339};
    let items = vec![
        CheckItem::finish("postgres.basic.ping", CheckStatus::Passed, Duration::from_millis(5), Some(50), None, now_rfc3339()),
        CheckItem::finish("postgres.basic.version", CheckStatus::Passed, Duration::from_millis(3), None, None, now_rfc3339()),
    ];
    let report = ValidationReport::from_items("postgres", CheckLevel::Basic, items);
    assert_eq!(report.module, "postgres");
    assert_eq!(report.level, CheckLevel::Basic);
    assert!(report.passed);
    assert!(!report.degraded);
    assert_eq!(report.items.len(), 2);
}

#[test]
fn selfcheck_report_degraded_still_passed() {
    use std::time::Duration;
    use postgresx::selfcheck::{CheckLevel, CheckStatus, CheckItem, ValidationReport, now_rfc3339};
    let items = vec![
        CheckItem::finish("a", CheckStatus::Passed, Duration::from_millis(100), Some(10), None, now_rfc3339()),
    ];
    let r = ValidationReport::from_items("postgres", CheckLevel::Basic, items);
    assert!(r.passed);
    assert!(r.degraded);
    assert_eq!(r.items[0].status, CheckStatus::Degraded);
    assert!(r.items[0].detail.as_ref().is_some_and(|d| d.contains("超基线")));
}

#[test]
fn selfcheck_failed_is_not_passed() {
    use std::time::Duration;
    use postgresx::selfcheck::{CheckLevel, CheckStatus, CheckItem, ValidationReport, now_rfc3339};
    let items = vec![
        CheckItem::finish("x", CheckStatus::Failed, Duration::ZERO, None, Some("失败".into()), now_rfc3339()),
    ];
    let r = ValidationReport::from_items("postgres", CheckLevel::Basic, items);
    assert!(!r.passed);
}

#[test]
fn selfcheck_catalog_matches_spec_11_items() {
    use postgresx::selfcheck::PostgresValidator;
    let cat = PostgresValidator::static_catalog();
    assert_eq!(cat.len(), 11);
    let ids: Vec<_> = cat.iter().map(|d| d.id.as_str()).collect();
    for expected in [
        "postgres.basic.ping",
        "postgres.basic.version",
        "postgres.rw.crud_roundtrip",
        "postgres.rw.tx_commit",
        "postgres.rw.tx_rollback",
        "postgres.full.batch_insert_1k",
        "postgres.full.jsonb_roundtrip",
        "postgres.full.listen_notify",
        "postgres.full.pool_saturation",
        "postgres.full.pool_recovery",
        "postgres.full.replication_lag",
    ] {
        assert!(ids.contains(&expected), "missing {expected}");
    }
}

#[test]
fn selfcheck_item_skipped_structure() {
    use postgresx::selfcheck::{CheckStatus, CheckItem, now_rfc3339};
    let item = CheckItem::skipped("postgres.full.replication_lag", "未启用", now_rfc3339());
    assert_eq!(item.status, CheckStatus::Skipped);
    assert_eq!(item.latency_ms, 0);
    assert_eq!(item.detail.as_deref(), Some("未启用"));
}

#[tokio::test]
async fn selfcheck_unreachable_pool_basic_fails_short_circuits() {
    use std::time::Duration;
    use postgresx::selfcheck::{CheckLevel, CheckStatus, PostgresValidator};
    use postgresx::{PostgresPool, PostgresConfig, SslMode};

    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .password("")
        .sslmode(SslMode::Disable)
        .max_pool_size(1)
        .acquire_timeout(Duration::from_millis(80))
        .operation_timeout(Duration::from_millis(80))
        .build()
        .expect("unreachable cfg");
    let pool = PostgresPool::connect_lazy(&cfg).await.expect("lazy");
    let v = PostgresValidator::new(pool);
    let report = v.run(CheckLevel::Full).await;
    assert_eq!(report.module, "postgres");
    assert!(!report.passed);
    assert_eq!(report.items.len(), 11);
    // basic items should be Failed
    let basic: Vec<_> = report.items.iter().filter(|i| i.id.starts_with("postgres.basic.")).collect();
    assert_eq!(basic.len(), 2);
    assert!(basic.iter().all(|i| i.status == CheckStatus::Failed));
    // rw/full items should be Skipped with 短路 reason
    for i in &report.items {
        if i.id.starts_with("postgres.rw.") || i.id.starts_with("postgres.full.") {
            assert_eq!(i.status, CheckStatus::Skipped, "{} -> {:?}", i.id, i.status);
            assert!(
                i.detail.as_ref().is_some_and(|d| d.contains("短路")),
                "detail: {:?}",
                i.detail
            );
        }
    }
}

#[tokio::test]
async fn selfcheck_skip_config_prevents_fail() {
    use std::time::Duration;
    use postgresx::selfcheck::{CheckLevel, CheckStatus, PostgresValidator, PostgresSelfCheckConfig};
    use postgresx::{PostgresPool, PostgresConfig, SslMode};

    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .password("")
        .sslmode(SslMode::Disable)
        .max_pool_size(1)
        .acquire_timeout(Duration::from_millis(80))
        .operation_timeout(Duration::from_millis(80))
        .build()
        .expect("unreachable cfg");
    let pool = PostgresPool::connect_lazy(&cfg).await.expect("lazy");
    let config = {
        let mut c = PostgresSelfCheckConfig::default();
        c.skip.insert("postgres.basic.ping".into());
        c.skip.insert("postgres.basic.version".into());
        c
    };
    let v = PostgresValidator::new(pool).with_config(config);
    let report = v.run(CheckLevel::Basic).await;
    assert!(report.passed, "skip 后无 Failed 项 → passed");
    assert_eq!(report.items.len(), 2);
    assert!(report.items.iter().all(|i| i.status == CheckStatus::Skipped));
}

// ━━━━━━━━━━━━━━━━━━━━━━━ 6. Repository SQL shapes ━━━━━━━━

#[test]
fn repository_sql_is_parameterized() {
    let find = postgresx::PgRepository::find_sql();
    let save = postgresx::PgRepository::save_sql();
    assert!(find.contains("$1"));
    assert!(save.contains("$1"));
    assert!(save.contains("$2"));
    assert!(save.contains("ON CONFLICT"));
    assert!(!find.contains("{}"));
    assert!(!save.contains("{}"));
    assert!(!find.contains("format!"));
    assert!(!save.contains("format!"));
}

#[test]
fn repository_ensure_table_sql_contains_correct_table() {
    let sql = postgresx::PgRepository::ensure_table_sql();
    assert!(sql.contains("infra_pg_records"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS"));
}

// ==== 7. Migration unit ====

#[test]
fn migration_checksum_stable() {
    let m = postgresx::Migration::new(1, "init", "CREATE TABLE t (id int);").unwrap();
    let c1 = m.checksum();
    let c2 = m.checksum();
    assert_eq!(c1, c2);
    assert_eq!(c1.len(), 64);
}

#[test]
fn migration_checksum_differs_on_whitespace() {
    let m1 = postgresx::Migration::new(1, "a", "SELECT 1;").unwrap();
    let m2 = postgresx::Migration::new(1, "a", "SELECT 1; ").unwrap();
    assert_ne!(m1.checksum(), m2.checksum());
}

#[test]
fn migration_rejects_bad_metadata() {
    assert!(postgresx::Migration::new(0, "name", "SQL").is_err());
    assert!(postgresx::Migration::new(1, "", "SQL").is_err());
    assert!(postgresx::Migration::new(1, "name", "  ").is_err());
}

#[test]
fn migration_rejects_duplicate_versions() {
    let m1 = postgresx::Migration::new(2, "a", "SELECT 1;").unwrap();
    let m2 = postgresx::Migration::new(2, "b", "SELECT 2;").unwrap();
    let pool_cfg = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .password("")
        .sslmode(postgresx::SslMode::Disable)
        .acquire_timeout(std::time::Duration::from_millis(50))
        .operation_timeout(std::time::Duration::from_millis(50))
        .max_pool_size(1)
        .build()
        .unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(postgresx::PostgresPool::connect_lazy(&pool_cfg)).unwrap();
    let err = postgresx::Migrator::new(pool, vec![m1, m2]).expect_err("dup versions");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    assert!(err.context().contains("重复"));
}

#[test]
fn migration_status_is_clean_and_boot_ok() {
    use postgresx::{MigrationStatus, ChecksumMismatch};
    let ok = MigrationStatus {
        applied: vec![],
        pending: vec![1],
        mismatches: vec![],
        unknown_applied: vec![],
    };
    assert!(!ok.is_clean());
    assert!(ok.is_boot_ok());
    assert!(postgresx::ensure_boot_ok(&ok).is_ok());

    let bad = MigrationStatus {
        applied: vec![],
        pending: vec![],
        mismatches: vec![ChecksumMismatch {
            version: 1,
            expected: "a".into(),
            actual: "b".into(),
        }],
        unknown_applied: vec![],
    };
    assert!(!bad.is_boot_ok());
    assert_eq!(
        postgresx::ensure_boot_ok(&bad).unwrap_err().kind(),
        kernel::ErrorKind::Conflict
    );
}

#[test]
fn migration_constants() {
    assert_eq!(postgresx::SCHEMA_MIGRATIONS_TABLE, "infra_schema_migrations");
    assert_ne!(postgresx::MIGRATION_LOCK_KEY1, 0);
    assert_ne!(postgresx::MIGRATION_LOCK_KEY2, 0);
}

// ==== 8. 类型与常量 ====

#[test]
fn public_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<postgresx::PostgresPool>();
    assert_send_sync::<postgresx::PostgresConfig>();
    assert_send_sync::<postgresx::PostgresConfigBuilder>();
    assert_send_sync::<postgresx::PgTxRunner>();
    assert_send_sync::<postgresx::PoolStats>();
    assert_send_sync::<postgresx::SslMode>();
    assert_send_sync::<postgresx::PgRepository>();
    assert_send_sync::<postgresx::PgRecord>();
    assert_send_sync::<postgresx::MakeRustlsConnect>();
}

#[test]
fn pool_stats() {
    let ps = postgresx::PoolStats {
        max_size: 16,
        size: 2,
        available: 1,
        waiting: 0,
        closed: false,
    };
    assert_eq!(ps.max_size, 16);
    assert_eq!(ps.size, 2);
    assert_eq!(ps.available, 1);
    assert!(!ps.closed);
}

#[test]
fn tx_status_discriminants_are_distinct() {
    use postgresx::TxStatus;
    assert_ne!(TxStatus::Active, TxStatus::Committed);
    assert_ne!(TxStatus::Active, TxStatus::RolledBack);
    assert_ne!(TxStatus::Active, TxStatus::Failed);
    assert_ne!(TxStatus::Committed, TxStatus::RolledBack);
    assert_ne!(TxStatus::Committed, TxStatus::Failed);
    assert_ne!(TxStatus::RolledBack, TxStatus::Failed);
}

// ━━━━━━━━━━━━━━━━━━━━━━━ 9. 离线 connect 拒绝 ━━━━━━━━━━━━

#[tokio::test]
async fn connect_lazy_can_be_constructed_offline() {
    let cfg = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .password("")
        .sslmode(postgresx::SslMode::Disable)
        .acquire_timeout(std::time::Duration::from_millis(200))
        .operation_timeout(std::time::Duration::from_millis(200))
        .max_pool_size(1)
        .build()
        .unwrap();
    let pool = postgresx::PostgresPool::connect_lazy(&cfg).await;
    assert!(pool.is_ok(), "connect_lazy should succeed even with unreachable host");
}

#[tokio::test]
async fn connect_to_unreachable_returns_error() {
    let cfg = postgresx::PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .password("")
        .sslmode(postgresx::SslMode::Disable)
        .connect_timeout(std::time::Duration::from_millis(300))
        .build()
        .unwrap();
    let res = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        postgresx::PostgresPool::connect(&cfg),
    )
    .await;
    match res {
        Ok(Err(err)) => {
            assert!(
                matches!(
                    err.kind(),
                    kernel::ErrorKind::Unavailable
                        | kernel::ErrorKind::DeadlineExceeded
                        | kernel::ErrorKind::Transient
                ),
                "kind={:?}",
                err.kind()
            );
        }
        Ok(Ok(_)) => panic!("unexpected success"),
        Err(_) => panic!("connect must have internal timeout"),
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━ 10. Live 集成 (ignore) ━━━━━━━━━━
// 设置环境变量后运行: cargo test -p postgresx --test functional_api -- --ignored

fn live_config() -> postgresx::PostgresConfig {
    postgresx::PostgresConfig::from_env().expect(
        "需要 DATABASE_URL 或 FOUNDATIONX_POSTGRESX_*",
    )
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_connect_health_stats_close() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    pool.health().await.expect("health");
    let stats = pool.stats();
    assert!(stats.max_size >= 1);
    assert!(!stats.closed);
    pool.close();
    assert!(pool.stats().closed);
    let err = pool.health().await.expect_err("closed pool health");
    assert_eq!(err.kind(), kernel::ErrorKind::Unavailable);
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_execute_insert_and_select() {
    use postgresx::PostgresPool;
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_exec (id INT PRIMARY KEY, val TEXT)",
        &[],
    )
    .await
    .expect("create");
    let n = pool.execute(
        "INSERT INTO fa_exec (id, val) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET val = EXCLUDED.val",
        &[&1i32, &"hello"],
    )
    .await
    .expect("insert");
    assert_eq!(n, 1);
    let row = pool.query_one("SELECT val FROM fa_exec WHERE id = $1", &[&1i32])
        .await.expect("select");
    let val: String = row.try_get(0).unwrap();
    assert_eq!(val, "hello");
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_query_one_and_opt() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    let row = pool.query_one("SELECT 1 AS one", &[]).await.expect("query_one");
    let one: i32 = row.try_get(0).unwrap();
    assert_eq!(one, 1);

    let some = pool.query_opt("SELECT 1 WHERE true", &[]).await.expect("some");
    assert!(some.is_some());

    let none = pool.query_opt("SELECT 1 WHERE false", &[]).await.expect("none");
    assert!(none.is_none());
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_query_multi_row() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    let rows = pool.query("SELECT generate_series(1, 5) AS n", &[]).await.expect("query");
    assert_eq!(rows.len(), 5);
    for (i, row) in rows.iter().enumerate() {
        let n: i32 = row.try_get(0).unwrap();
        assert_eq!(n, (i + 1) as i32);
    }
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_copy_in_out_roundtrip() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_copy (id INT, name TEXT)",
        &[],
    )
    .await
    .expect("create");
    pool.execute("TRUNCATE fa_copy", &[]).await.ok();
    let n = pool.copy_in_bytes(
        "COPY fa_copy (id, name) FROM STDIN WITH (FORMAT csv)",
        b"1,alice\n2,bob\n",
    )
    .await
    .expect("copy_in");
    assert_eq!(n, 2);
    let out = pool.copy_out_bytes(
        "COPY fa_copy (id, name) TO STDOUT WITH (FORMAT csv)",
        0,
    )
    .await
    .expect("copy_out");
    assert!(!out.is_empty());
    let s = String::from_utf8_lossy(&out);
    assert!(s.contains("1"));
    assert!(s.contains("2"));
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_with_transaction_commit() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_txc (k TEXT PRIMARY KEY, v INT)",
        &[],
    )
    .await
    .expect("create");
    let result = pool
        .with_transaction(async |tx| {
            tx.execute("INSERT INTO fa_txc (k, v) VALUES ($1, $2)", &[&"a", &99i32])
                .await?;
            Ok::<_, kernel::XError>(42_i32)
        })
        .await
        .expect("tx commit");
    assert_eq!(result, 42);
    let row = pool.query_one("SELECT v FROM fa_txc WHERE k = $1", &[&"a"]).await.expect("select");
    let v: i32 = row.try_get(0).unwrap();
    assert_eq!(v, 99);
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_with_transaction_rollback() {
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_txr (k TEXT PRIMARY KEY, v INT)",
        &[],
    )
    .await
    .expect("create");
    let err = pool
        .with_transaction(async |tx| {
            tx.execute("INSERT INTO fa_txr (k, v) VALUES ($1, $2)", &[&"rb", &88i32]).await?;
            Err::<(), _>(kernel::XError::invalid("business error"))
        })
        .await
        .expect_err("should rollback");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    let none = pool
        .query_opt("SELECT v FROM fa_txr WHERE k = $1", &[&"rb"])
        .await
        .expect("query_opt");
    assert!(none.is_none(), "rollback 后不应可见");
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_begin_commit_and_rollback() {
    use postgresx::{PostgresPool, TxStatus};
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_btx (id INT PRIMARY KEY)",
        &[],
    )
    .await
    .expect("create");
    // commit
    let mut tx = pool.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);
    tx.execute("INSERT INTO fa_btx VALUES ($1)", &[&1i32]).await.expect("insert");
    tx.commit().await.expect("commit");
    // rollback
    let mut tx2 = pool.begin().await.expect("begin");
    tx2.execute("INSERT INTO fa_btx VALUES ($1)", &[&2i32]).await.expect("insert");
    tx2.rollback().await.expect("rollback");
    // verify
    let rows = pool.query("SELECT id FROM fa_btx ORDER BY id", &[]).await.expect("query");
    assert_eq!(rows.len(), 1);
    let id: i32 = rows[0].try_get(0).unwrap();
    assert_eq!(id, 1);
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_tx_failed_state_on_duplicate_key() {
    use postgresx::{PostgresPool, TxStatus};
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    pool.execute(
        "CREATE TEMP TABLE IF NOT EXISTS fa_txf (id INT PRIMARY KEY)",
        &[],
    )
    .await
    .expect("create");
    let mut tx = pool.begin().await.expect("begin");
    tx.execute("INSERT INTO fa_txf VALUES ($1)", &[&1i32]).await.expect("first");
    let err = tx.execute("INSERT INTO fa_txf VALUES ($1)", &[&1i32]).await.expect_err("dup");
    assert_eq!(err.kind(), kernel::ErrorKind::Conflict);
    assert_eq!(tx.status(), TxStatus::Failed, "SQL 错误后进入 Failed 态");
    // commit 应返回错误
    let _ = tx.commit().await.expect_err("commit after failed SQL");
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_conn_execute_and_query_parameterized() {
    use postgresx::PostgresPool;
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let mut conn = pool.acquire().await.expect("acquire");
    conn.batch_execute("CREATE TEMP TABLE IF NOT EXISTS fa_conn (id INT, val TEXT)")
        .await
        .expect("create");
    let rows = conn
        .execute("INSERT INTO fa_conn (id, val) VALUES ($1, $2)", &[&100i32, &"pg"])
        .await
        .expect("execute");
    assert_eq!(rows, 1);
    let row = conn.query_one("SELECT val FROM fa_conn WHERE id = $1", &[&100i32])
        .await.expect("query_one");
    let val: String = row.try_get(0).unwrap();
    assert_eq!(val, "pg");
    // query_opt None
    let none = conn.query_opt("SELECT 1 WHERE false", &[]).await.expect("q_opt");
    assert!(none.is_none());
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_conn_copy_bytes_roundtrip() {
    use postgresx::PostgresPool;
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let mut conn = pool.acquire().await.expect("acquire");
    conn.batch_execute("CREATE TEMP TABLE IF NOT EXISTS fa_ccopy (payload BYTEA)")
        .await
        .expect("create");
    conn.batch_execute("TRUNCATE fa_ccopy").await.ok();
    let data = b"binary-data-stream-001";
    let rows = conn
        .copy_in_bytes("COPY fa_ccopy (payload) FROM STDIN WITH (FORMAT binary)", data)
        .await
        .expect("copy_in");
    assert_eq!(rows, 1);
    let out = conn
        .copy_out_bytes("COPY fa_ccopy (payload) TO STDOUT WITH (FORMAT binary)", 0)
        .await
        .expect("copy_out");
    assert!(!out.is_empty());
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_repository_crud() {
    use postgresx::{PostgresPool, PgRepository, PgRecord};
    use contracts::Repository;

    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let repo = PgRepository::new(pool.clone());
    repo.ensure_table().await.expect("ensure_table");

    let id = format!("fa-repo-{}", std::process::id());
    // cleanup
    let _ = pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await;

    let record = PgRecord { id: id.clone(), data: b"hello".to_vec() };
    assert!(repo.find(id.clone()).await.expect("find miss").is_none());
    repo.save(&record).await.expect("save");
    let found = repo.find(id.clone()).await.expect("find hit").expect("row");
    assert_eq!(found.id, id);
    assert_eq!(found.data, b"hello");

    // upsert
    let updated = PgRecord { id: id.clone(), data: b"updated".to_vec() };
    repo.save(&updated).await.expect("save upsert");
    let found2 = repo.find(id.clone()).await.expect("find 2").expect("row");
    assert_eq!(found2.data, b"updated");

    // missing
    assert!(repo.find("nonexistent-9999".to_string()).await.expect("find").is_none());

    // cleanup
    let _ = pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await;
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_migrator_apply_and_verify() {
    use postgresx::{PostgresPool, Migration, Migrator};

    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let m = Migration::new(
        99001,
        "fa-mig-single",
        "CREATE TABLE IF NOT EXISTS fa_mig_tbl (id SERIAL PRIMARY KEY);",
    )
    .expect("migration");
    let migrator = Migrator::new(pool.clone(), vec![m]).expect("migrator");

    // cleanup before
    let _ = pool.execute("DROP TABLE IF EXISTS fa_mig_tbl", &[]).await;
    let _ = pool.execute(
        &format!("DELETE FROM {} WHERE version = 99001", postgresx::SCHEMA_MIGRATIONS_TABLE),
        &[],
    )
    .await;

    // apply
    let report = migrator.apply().await.expect("apply");
    assert!(report.applied_now.contains(&99001));
    assert!(report.status.applied.iter().any(|a| a.version == 99001));

    // second apply should be idempotent
    let report2 = migrator.apply().await.expect("second apply");
    assert!(!report2.applied_now.contains(&99001));

    // verify should pass
    let status = migrator.verify().await.expect("verify");
    assert!(status.mismatches.is_empty());
    assert!(status.unknown_applied.is_empty());

    // cleanup
    let _ = pool.execute("DROP TABLE IF EXISTS fa_mig_tbl", &[]).await;
    let _ = pool.execute(
        &format!("DELETE FROM {} WHERE version = 99001", postgresx::SCHEMA_MIGRATIONS_TABLE),
        &[],
    )
    .await;
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres"]
async fn live_selfcheck_basic_passes() {
    use postgresx::selfcheck::{CheckLevel, PostgresValidator};
    let pool = postgresx::PostgresPool::connect(&live_config()).await.expect("connect");
    let v = PostgresValidator::new(pool.clone());
    let report = v.run(CheckLevel::Basic).await;
    assert!(report.passed, "basic should pass: {:?}", report.items);
    assert_eq!(report.items.len(), 2);
    assert!(report.items.iter().all(|i| i.status == postgresx::selfcheck::CheckStatus::Passed));
    pool.close();
}

// ━━━━━━━━━━━━━━━━━━━━━━━ 11. Scaffold (feature gate) ━━━━━

#[cfg(all(test, feature = "scaffold"))]
mod scaffold {
    use postgresx::{PostgresAdapter, Record, ObservingPostgresAdapter};

    #[test]
    fn record_fields() {
        let r = Record { id: "a".into(), data: b"xyz".to_vec() };
        assert_eq!(r.id, "a");
        assert_eq!(r.data, b"xyz");
    }

    #[tokio::test]
    async fn adapter_save_and_find() {
        let adapter = PostgresAdapter::default();
        let record = Record { id: "1".into(), data: b"hello".to_vec() };
        adapter.save(&record).await.expect("save");
        let found = adapter.find("1".into()).await.expect("find");
        assert_eq!(found, Some(record));
    }

    #[tokio::test]
    async fn adapter_find_missing() {
        let adapter = PostgresAdapter::default();
        assert_eq!(adapter.find("nonexistent".into()).await.expect("find"), None);
    }

    #[tokio::test]
    async fn observing_adapter_commit() {
        use contracts::run_tx_lifecycle;
        use kernel::XError;

        let adapter = ObservingPostgresAdapter::default();
        let record = Record { id: "oc1".into(), data: b"val".to_vec() };
        let a = adapter.clone();
        let r = record.clone();
        let result = run_tx_lifecycle(&adapter, || {
            let a = a.clone();
            let r = r.clone();
            async move {
                a.save(&r).await?;
                Ok::<_, XError>(42u32)
            }
        })
        .await
        .expect("commit");
        assert_eq!(result, 42);
        let found = adapter.find("oc1".into()).await.expect("find");
        assert_eq!(found, Some(record));
    }
}

// ==== 12. PgTxRunner ====

#[test]
fn pg_tx_runner_constructible() {
    use postgresx::PgTxRunner;
    use std::sync::Arc;
    // 离线构造：connect_lazy + Arc
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(async {
        postgresx::PostgresPool::connect_lazy(
            &postgresx::PostgresConfig::builder()
                .host("127.0.0.1")
                .port(1)
                .database("x")
                .user("x")
                .password("")
                .sslmode(postgresx::SslMode::Disable)
                .acquire_timeout(std::time::Duration::from_millis(50))
                .operation_timeout(std::time::Duration::from_millis(50))
                .max_pool_size(1)
                .build()
                .expect("cfg"),
        )
        .await
        .expect("lazy pool")
    });
    let pool = Arc::new(pool);
    let runner = PgTxRunner::new(pool.clone());
    let _ = runner.pool();
}
