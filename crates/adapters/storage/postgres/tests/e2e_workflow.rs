//! 端到端工作流集成测试（默认 `#[ignore]`）。
//!
//! 完整 E2E pipeline：配置构建 → 连接 → SQL → COPY → 事务 → Repository → Migration → Selfcheck。
//!
//! ```bash
//! cargo test -p postgresx --test e2e_workflow -- --ignored --nocapture --test-threads=1
//! ```

use contracts::Repository;
use kernel::{ErrorKind, XError};
use postgresx::{
    Migration, Migrator, PgRecord, PgRepository, PgRetryConfig, PoolStats, PostgresConfig,
    PostgresPool, SslMode, TxStatus, error_kind_from_sqlstate,
    selfcheck::{CheckLevel, CheckStatus, PostgresValidator, ValidationReport},
    with_budget_async, with_budget_async_safe, with_retry_async,
};
use resiliencx::{NoopInstrumentation, RetryBudget, RetrySafety};
use std::time::Duration;

// ============================================================================
// 环境变量设置
// ============================================================================

fn set_env() {
    // SAFETY: 仅通过 --test-threads=1 运行，无并发 set_var 竞争
    if std::env::var("FOUNDATIONX_POSTGRESX_HOST").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_HOST", "127.0.0.1");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_PORT").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_PORT", "5432");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_DATABASE").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_DATABASE", "market_binance");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_USER").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_USER", "market_binance");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_PASSWORD").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_PASSWORD", "Kt63mWgbhBwSPWnrEnMkC");
        }
    }
    if std::env::var("FOUNDATIONX_POSTGRESX_SSLMODE").is_err() {
        unsafe {
            std::env::set_var("FOUNDATIONX_POSTGRESX_SSLMODE", "disable");
        }
    }
}

fn live_config() -> PostgresConfig {
    set_env();
    PostgresConfig::from_foundationx_env().expect("加载 FOUNDATIONX_POSTGRESX_* 环境变量")
}

fn e2e_table_suffix() -> String {
    format!("e2e_{}", std::process::id())
}

// ============================================================================
// E2E-1: full_config_to_query_workflow
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_full_config_to_query_workflow() {
    // -- 配置构建 --
    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(5432)
        .database("market_binance")
        .user("market_binance")
        .password("Kt63mWgbhBwSPWnrEnMkC")
        .sslmode(SslMode::Disable)
        .max_pool_size(4)
        .acquire_timeout(Duration::from_secs(10))
        .operation_timeout(Duration::from_secs(10))
        .build()
        .expect("配置构建");

    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 5432);
    assert_eq!(cfg.database, "market_binance");

    // -- 连接 --
    let pool = PostgresPool::connect(&cfg).await.expect("connect");

    // -- health --
    pool.health().await.expect("health");

    // -- CREATE TABLE --
    let suffix = e2e_table_suffix();
    let table = format!("e2e_test_{suffix}");
    let create = format!(
        "CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, name TEXT, value NUMERIC)"
    );
    pool.execute(&create, &[]).await.expect("create table");

    // -- INSERT --
    let name: &str = "alice";
    let value: f64 = 100.5;
    pool.execute(&format!("INSERT INTO {table} (name, value) VALUES ($1, $2)"), &[&name, &value])
        .await
        .expect("insert");

    // -- query_one COUNT --
    let row = pool.query_one(&format!("SELECT COUNT(*) FROM {table}"), &[]).await.expect("count");
    let count: i64 = row.get(0);
    assert_eq!(count, 1);

    // -- query SELECT --
    let rows = pool.query(&format!("SELECT name, value FROM {table}"), &[]).await.expect("select");
    assert_eq!(rows.len(), 1);
    let row_name: String = rows[0].get(0);
    let row_value: f64 = rows[0].get(1);
    assert_eq!(row_name, "alice");
    assert_eq!(row_value, 100.5);

    // -- COPY OUT --
    let out =
        pool.copy_out_bytes(&format!("COPY {table} TO STDOUT"), 1024).await.expect("copy out");
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("alice"));

    // -- close --
    pool.close();
    let stats: PoolStats = pool.stats();
    assert!(stats.closed, "关闭后 stats.closed 应为 true");

    // -- acquire after close --
    match pool.acquire().await {
        Err(e) => assert_eq!(e.kind(), ErrorKind::Unavailable),
        Ok(_) => panic!("关闭后 acquire 应失败"),
    }

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
}

// ============================================================================
// E2E-2: transaction_workflow_commit
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_transaction_workflow_commit() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let suffix = e2e_table_suffix();
    let table = format!("e2e_tx_commit_{suffix}");
    let marker = format!("commit-{}", rand_part());

    pool.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, marker TEXT NOT NULL)"
        ),
        &[],
    )
    .await
    .expect("create");

    // BEGIN → INSERT → SELECT in tx → COMMIT
    let conn = pool.acquire().await.expect("acquire");
    let mut tx = conn.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);

    tx.execute(&format!("INSERT INTO {table} (marker) VALUES ($1)"), &[&marker])
        .await
        .expect("insert");

    let row = tx
        .query_one(&format!("SELECT marker FROM {table} WHERE marker = $1"), &[&marker])
        .await
        .expect("select in tx");
    let got: String = row.get(0);
    assert_eq!(got, marker);

    tx.commit().await.expect("commit");

    // 事务外查询（验证持久化）
    let rows = pool
        .query(&format!("SELECT marker FROM {table} WHERE marker = $1"), &[&marker])
        .await
        .expect("select outside tx");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, String>(0), marker);

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// E2E-3: transaction_workflow_rollback
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_transaction_workflow_rollback() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let suffix = e2e_table_suffix();
    let table = format!("e2e_tx_rb_{suffix}");
    let marker = format!("rollback-{}", rand_part());

    pool.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, marker TEXT NOT NULL)"
        ),
        &[],
    )
    .await
    .expect("create");

    // BEGIN → INSERT → ROLLBACK
    let conn = pool.acquire().await.expect("acquire");
    let mut tx = conn.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);

    tx.execute(&format!("INSERT INTO {table} (marker) VALUES ($1)"), &[&marker])
        .await
        .expect("insert in tx");

    tx.rollback().await.expect("rollback");

    // 事务外查询（验证不可见）
    let rows = pool
        .query(&format!("SELECT marker FROM {table} WHERE marker = $1"), &[&marker])
        .await
        .expect("select outside tx");
    assert!(rows.is_empty(), "回滚后不应可见");

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// E2E-4: transaction_workflow_business_error
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_transaction_workflow_business_error() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let suffix = e2e_table_suffix();
    let table = format!("e2e_tx_biz_{suffix}");
    let marker = format!("biz-{}", rand_part());

    pool.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, marker TEXT NOT NULL)"
        ),
        &[],
    )
    .await
    .expect("create");

    // with_transaction 中返回 Err → 自动 rollback
    let result = pool
        .with_transaction(async |tx| {
            tx.execute(&format!("INSERT INTO {table} (marker) VALUES ($1)"), &[&marker]).await?;
            Err::<(), _>(XError::invalid("业务校验失败"))
        })
        .await;

    let err = result.expect_err("应返回业务错误");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.context().contains("业务校验失败"));

    // 事务外查询（验证 rollback）
    let rows = pool
        .query(&format!("SELECT marker FROM {table} WHERE marker = $1"), &[&marker])
        .await
        .expect("select");
    assert!(rows.is_empty(), "业务错误后应回滚");

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// E2E-5: repository_crud_workflow
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_repository_crud_workflow() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let repo = PgRepository::new(pool.clone());

    // ensure_table
    repo.ensure_table().await.expect("ensure_table");

    let id = format!("e2e-repo-{}", rand_part());

    // 清理残留
    let _ = pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await;

    // find 不存在 → None
    assert!(repo.find(id.clone()).await.expect("find miss").is_none());

    // save 插入
    let record = PgRecord { id: id.clone(), data: b"hello".to_vec() };
    repo.save(&record).await.expect("save insert");

    // find 命中
    let found = repo.find(id.clone()).await.expect("find hit").expect("row");
    assert_eq!(found.id, id);
    assert_eq!(found.data, b"hello");

    // save upsert
    let updated = PgRecord { id: id.clone(), data: b"updated".to_vec() };
    repo.save(&updated).await.expect("save upsert");

    let found2 = repo.find(id.clone()).await.expect("find after upsert").expect("row");
    assert_eq!(found2.id, id);
    assert_eq!(found2.data, b"updated");

    // find nonexistent → None
    assert!(repo.find("nonexistent-key".to_string()).await.expect("find nonexistent").is_none());

    // cleanup
    let _ = pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await;
    pool.close();
}

// ============================================================================
// E2E-6: migration_verify_and_apply
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_migration_verify_and_apply() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let suffix = rand_part();
    let table1 = format!("e2e_mig_t1_{suffix}");
    let table2 = format!("e2e_mig_t2_{suffix}");

    // 使用足够大的版本号避免冲突
    let base = 90_000 + (std::process::id() % 1000) as i64;

    // 清理旧记录
    let _ = pool
        .execute(
            "DELETE FROM infra_schema_migrations WHERE version IN ($1, $2)",
            &[&base, &(base + 1)],
        )
        .await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table1}"), &[]).await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table2}"), &[]).await;

    // 构造两个 migration
    let m1 = Migration::new(
        base,
        format!("e2e_mig_a_{suffix}"),
        format!("CREATE TABLE IF NOT EXISTS {table1} (id INT PRIMARY KEY)"),
    )
    .expect("m1");

    let m2 = Migration::new(
        base + 1,
        format!("e2e_mig_b_{suffix}"),
        format!("CREATE TABLE IF NOT EXISTS {table2} (id INT PRIMARY KEY)"),
    )
    .expect("m2");

    let migrator = Migrator::new(pool.clone(), vec![m1.clone(), m2.clone()]).expect("migrator");

    // verify → 报告 pending
    let st = migrator.verify().await.expect("verify");
    assert!(st.is_boot_ok());
    assert!(st.pending.contains(&base));
    assert!(st.pending.contains(&(base + 1)));

    // apply → 运行两个
    let report = migrator.apply().await.expect("apply");
    assert!(report.applied_now.contains(&base));
    assert!(report.applied_now.contains(&(base + 1)));
    assert!(report.status.pending.is_empty());

    // apply again → no pending
    let report2 = migrator.apply().await.expect("re-apply");
    assert!(report2.applied_now.is_empty());

    // 修改已应用 migration 的 SQL → verify 校验通过
    let st2 = migrator.verify().await.expect("verify after apply");
    assert!(st2.mismatches.is_empty());
    assert!(st2.pending.is_empty());

    // 变更 checksum → verify 失败
    let m1_bad = Migration::new(
        base,
        format!("e2e_mig_a_{suffix}"),
        format!("CREATE TABLE IF NOT EXISTS {table1} (id INT PRIMARY KEY, x INT)"),
    )
    .expect("m1 bad");
    let migrator_bad = Migrator::new(pool.clone(), vec![m1_bad]).expect("bad migrator");
    let err = migrator_bad.verify().await.expect_err("checksum 不一致必须失败");
    assert_eq!(err.kind(), ErrorKind::Conflict);

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table1}"), &[]).await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table2}"), &[]).await;
    let _ = pool
        .execute(
            "DELETE FROM infra_schema_migrations WHERE version IN ($1, $2)",
            &[&base, &(base + 1)],
        )
        .await;
    pool.close();
}

// ============================================================================
// E2E-7: resilience_retry_workflow
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_resilience_retry_workflow() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");

    // with_retry_async：成功查询
    let cfg = PgRetryConfig::fixed(2, 0);
    let n = with_retry_async(&cfg, "e2e_retry_select", || {
        let pool = pool.clone();
        async move {
            let row = pool.query_one("SELECT 42 AS n", &[]).await?;
            let v: i32 = row.try_get(0).map_err(postgresx::map_tokio_error)?;
            Ok::<_, XError>(v)
        }
    })
    .await
    .expect("retry async ok");
    assert_eq!(n, 42);

    // with_budget_async：预算耗尽
    let budget = RetryBudget::new(0);
    let err = with_budget_async(&budget, 3, "e2e_budget", &NoopInstrumentation, || async {
        Err::<(), _>(XError::transient("临时错误"))
    })
    .await
    .expect_err("预算耗尽");
    assert!(
        err.to_string().contains("budget")
            || err.to_string().contains("exceeded")
            || err.kind() == ErrorKind::Unavailable
            || err.kind() == ErrorKind::Transient,
        "预算耗尽错误: {:?}",
        err.kind()
    );

    // with_budget_async_safe：安全操作（ReadOnly）可重试
    let budget_safe = RetryBudget::new(2);
    let counter = std::sync::atomic::AtomicU32::new(0);
    let value = with_budget_async_safe(
        &budget_safe,
        3,
        RetrySafety::ReadOnly,
        "e2e_budget_safe",
        &NoopInstrumentation,
        || {
            let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            async move {
                if attempt == 1 {
                    Err(XError::transient("临时读取失败"))
                } else {
                    Ok::<_, XError>(attempt)
                }
            }
        },
    )
    .await
    .expect("安全重试");
    assert_eq!(value, 2);
    assert_eq!(budget_safe.remaining(), 1);

    pool.close();
}

// ============================================================================
// E2E-8: selfcheck_full_workflow
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn e2e_selfcheck_full_workflow() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");

    // Basic
    let basic_report: ValidationReport =
        PostgresValidator::new(pool.clone()).run(CheckLevel::Basic).await;
    assert_eq!(basic_report.module, "postgres");
    assert!(basic_report.passed, "Basic 自检应通过");
    assert_eq!(basic_report.level, CheckLevel::Basic);
    // ping + version
    assert!(
        basic_report
            .items
            .iter()
            .any(|i| i.id == "postgres.basic.ping" && i.status == CheckStatus::Passed)
    );
    assert!(
        basic_report
            .items
            .iter()
            .any(|i| i.id == "postgres.basic.version" && i.status == CheckStatus::Passed)
    );

    // ReadWrite
    let rw_report: ValidationReport =
        PostgresValidator::new(pool.clone()).run(CheckLevel::ReadWrite).await;
    assert!(rw_report.passed, "ReadWrite 自检应通过");
    // crud + tx_commit + tx_rollback
    assert!(
        rw_report
            .items
            .iter()
            .any(|i| i.id == "postgres.rw.crud_roundtrip" && i.status == CheckStatus::Passed)
    );
    assert!(
        rw_report
            .items
            .iter()
            .any(|i| i.id == "postgres.rw.tx_commit" && i.status == CheckStatus::Passed)
    );
    assert!(
        rw_report
            .items
            .iter()
            .any(|i| i.id == "postgres.rw.tx_rollback" && i.status == CheckStatus::Passed)
    );

    // Full
    let full_report: ValidationReport =
        PostgresValidator::new(pool.clone()).run(CheckLevel::Full).await;
    // Full 级别应包含全部 11 项
    assert_eq!(full_report.items.len(), 11, "Full 应包含 11 项检查");
    assert!(full_report.passed, "Full 自检应通过（除 replication_lag Skipped）");

    // 分类验证
    let basic_count =
        full_report.items.iter().filter(|i| i.id.starts_with("postgres.basic.")).count();
    let rw_count = full_report.items.iter().filter(|i| i.id.starts_with("postgres.rw.")).count();
    let full_count_only =
        full_report.items.iter().filter(|i| i.id.starts_with("postgres.full.")).count();
    assert_eq!(basic_count, 2);
    assert_eq!(rw_count, 3);
    assert_eq!(full_count_only, 6);

    // replication_lag 应为 Skipped（无副本环境）
    let lag = full_report
        .items
        .iter()
        .find(|i| i.id == "postgres.full.replication_lag")
        .expect("replication_lag");
    assert_eq!(lag.status, CheckStatus::Skipped);

    // 验证 _self_check_* 表已清理（Full 中的 crud 检查会创建 UNLOGGED 表并 DROP）
    let rows = pool
        .query(
            "SELECT tablename FROM pg_catalog.pg_tables WHERE tablename LIKE '_self_check_%'",
            &[],
        )
        .await
        .expect("query tables");
    assert!(rows.is_empty(), "自检临时表应已清理");

    pool.close();
}

// ============================================================================
// 辅助函数
// ============================================================================

fn rand_part() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t:x}")
}

// ============================================================================
// 单元测试（不依赖真实 Postgres）
// ============================================================================

#[test]
fn unit_sqlstate_error_kind_mapping_table() {
    // 验证规范表映射
    assert_eq!(error_kind_from_sqlstate("23505"), ErrorKind::Conflict); // unique_violation
    assert_eq!(error_kind_from_sqlstate("42P01"), ErrorKind::Missing); // undefined_table
    assert_eq!(error_kind_from_sqlstate("40P01"), ErrorKind::Transient); // deadlock
    assert_eq!(error_kind_from_sqlstate("23502"), ErrorKind::Invalid); // not_null
    assert_eq!(error_kind_from_sqlstate("23514"), ErrorKind::Invalid); // check
    assert_eq!(error_kind_from_sqlstate("23503"), ErrorKind::Invalid); // fk
    assert_eq!(error_kind_from_sqlstate("57014"), ErrorKind::Cancelled); // query_canceled
    assert_eq!(error_kind_from_sqlstate("08006"), ErrorKind::Unavailable); // connection
    assert_eq!(error_kind_from_sqlstate("25P01"), ErrorKind::Invariant); // tx state
}
