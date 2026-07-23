//! Postgresx 集成测试——覆盖全部公开 API 面。
//!
//! Live Postgres 环境变量：FOUNDATIONX_POSTGRESX_* 或 DATABASE_URL

use contracts::{Repository, run_tx_lifecycle};
use kernel::{ErrorKind, XError};
use postgresx::{
    DEFAULT_MAX_POOL_SIZE, DEFAULT_PORT, Migration, Migrator, PgRecord, PgRepository,
    PgRetryConfig, PgTxRunner, PoolStats, PostgresConfig, PostgresPool, SslMode, TxStatus,
    with_retry_sync,
};
use std::sync::Arc;
use std::time::Duration;

fn config() -> PostgresConfig {
    PostgresConfig::from_env().expect("need PG env vars")
}

// ═══════════════════════════════════════════
// 1 ─ 连接池生命周期
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn pool_connect_health_stats_close() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    pool.health().await.expect("health");
    let s = pool.stats();
    assert!(s.max_size >= 1);
    assert!(!s.closed);
    assert!(!pool.summary().is_empty());
    pool.close();
    assert!(pool.stats().closed);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn pool_acquire_and_execute() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let t = format!("int_pool_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT PRIMARY KEY, val TEXT)"), &[])
        .await
        .expect("create");
    let n = pool
        .execute(&format!("INSERT INTO {t} VALUES ($1,$2),($3,$4)"), &[&1i32, &"a", &2i32, &"b"])
        .await
        .expect("insert");
    assert_eq!(n, 2);
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn pool_query_one_and_opt() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let row = pool
        .query_one("SELECT $1::int AS n, $2::text AS t", &[&42i32, &"ok"])
        .await
        .expect("query_one");
    assert_eq!(row.get::<&str, i32>("n"), 42);
    assert_eq!(row.get::<&str, &str>("t"), "ok");
    let none = pool.query_opt("SELECT 1 WHERE false", &[]).await.expect("query_opt");
    assert!(none.is_none());
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn pool_query_multi_and_copy() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let rows = pool.query("SELECT generate_series(1,5) AS n", &[]).await.expect("query");
    assert_eq!(rows.len(), 5);

    let t = format!("int_copy_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT, data BYTEA)"), &[])
        .await
        .expect("create");
    pool.copy_in_bytes(&format!("COPY {t} FROM STDIN CSV"), b"1,hello\n2,world\n")
        .await
        .expect("copy_in");
    let out =
        pool.copy_out_bytes(&format!("COPY {t} TO STDOUT CSV"), 1024).await.expect("copy_out");
    assert!(!out.is_empty());
    drop(pool);
}

// ═══════════════════════════════════════════
// 2 ─ 事务
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn tx_begin_commit_rollback() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let t = format!("int_tx_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT PRIMARY KEY)"), &[])
        .await
        .expect("create");

    let mut tx = pool.begin().await.expect("begin");
    tx.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&1i32]).await.expect("insert");
    tx.commit().await.expect("commit");

    let mut tx2 = pool.begin().await.expect("begin");
    tx2.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&2i32]).await.expect("insert");
    tx2.rollback().await.expect("rollback");

    let rows = pool.query(&format!("SELECT id FROM {t} ORDER BY id"), &[]).await.expect("query");
    assert_eq!(rows.len(), 1);
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn tx_with_transaction_commit_and_rollback() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let t = format!("int_wtx_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT)"), &[]).await.expect("create");

    let result = pool
        .with_transaction(async |tx| {
            tx.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&1i32]).await?;
            Ok::<_, XError>(42u32)
        })
        .await
        .expect("commit");
    assert_eq!(result, 42);

    let err = pool
        .with_transaction(async |tx| {
            tx.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&2i32]).await?;
            Err::<(), _>(XError::invalid("业务失败"))
        })
        .await
        .expect_err("rollback");
    assert_eq!(err.kind(), ErrorKind::Invalid);

    let n: i64 = pool
        .query_one(&format!("SELECT COUNT(*) FROM {t}"), &[])
        .await
        .expect("count")
        .get("count");
    assert_eq!(n, 1, "rollback should undo insert");
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn tx_status_failed_on_error() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let t = format!("int_tx_s_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT PRIMARY KEY)"), &[])
        .await
        .expect("create");

    let mut tx = pool.begin().await.expect("begin");
    tx.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&1i32]).await.expect("insert");
    let dup = tx.execute(&format!("INSERT INTO {t} VALUES ($1)", t = t), &[&1i32]).await;
    assert!(dup.is_err());
    assert_eq!(dup.expect_err("dup key").kind(), ErrorKind::Conflict);
    drop(pool);
}

// ═══════════════════════════════════════════
// 3 ─ Repository + contracts
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn repo_crud_and_contracts() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let repo = PgRepository::new(pool.clone());
    repo.ensure_table().await.expect("ensure_table");

    let rec = PgRecord { id: "int-repo-1".into(), data: b"data".to_vec() };
    Repository::<PgRecord, String>::save(&repo, &rec).await.expect("save");
    let found =
        Repository::<PgRecord, String>::find(&repo, "int-repo-1".into()).await.expect("find");
    assert_eq!(found, Some(PgRecord { id: "int-repo-1".into(), data: b"data".to_vec() }));

    // upsert
    Repository::<PgRecord, String>::save(
        &repo,
        &PgRecord { id: "int-repo-1".into(), data: b"v2".to_vec() },
    )
    .await
    .expect("upsert");
    let found2 =
        Repository::<PgRecord, String>::find(&repo, "int-repo-1".into()).await.expect("find");
    assert_eq!(found2.unwrap().data, b"v2");

    // missing
    assert_eq!(
        Repository::<PgRecord, String>::find(&repo, "nonexistent".into()).await.expect("find"),
        None
    );

    pool.execute("DELETE FROM infra_pg_records WHERE id=$1", &[&"int-repo-1"]).await.ok();
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn tx_runner_lifecycle() {
    let pool = Arc::new(PostgresPool::connect(&config()).await.expect("connect"));
    let runner = PgTxRunner::new(pool.clone());

    let result =
        run_tx_lifecycle(&runner, || async { Ok::<_, XError>(42u32) }).await.expect("commit");
    assert_eq!(result, 42);

    let err = run_tx_lifecycle(&runner, || async { Err::<u32, _>(XError::invalid("fail")) }).await;
    assert!(err.is_err());
    drop(pool);
}

// ═══════════════════════════════════════════
// 4 ─ Migration
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn migrator_verify_and_apply() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let p = format!("int_mig_{}", std::process::id());

    let m =
        Migration::new(1, "m1", format!("CREATE TABLE IF NOT EXISTS {p}_t (id INT)")).expect("m");
    let mig = Migrator::new(pool.clone(), vec![m]).expect("migrator");
    mig.apply().await.expect("apply");

    pool.query_one(&format!("SELECT 1 FROM {p}_t LIMIT 1"), &[]).await.expect("table exists");

    let status = mig.status().await.expect("status");
    assert!(!status.applied.is_empty());

    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {p}_t"), &[]).await;
    let _ = pool.execute("DELETE FROM infra_schema_migrations WHERE version=$1", &[&1i64]).await;
    drop(pool);
}

// ═══════════════════════════════════════════
// 5 ─ 错误处理
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn error_unique_violation_maps_to_conflict() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let t = format!("int_err_{}", std::process::id());
    pool.execute(&format!("CREATE TEMP TABLE {t} (id INT PRIMARY KEY)"), &[])
        .await
        .expect("create");
    pool.execute(&format!("INSERT INTO {t} VALUES (1)"), &[]).await.expect("insert");
    let err = pool.execute(&format!("INSERT INTO {t} VALUES (1)"), &[]).await.expect_err("dup");
    assert_eq!(err.kind(), ErrorKind::Conflict);
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn error_undefined_table_maps_to_missing() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let err =
        pool.execute("SELECT 1 FROM nonexistent_table_xyz", &[]).await.expect_err("missing table");
    assert_eq!(err.kind(), ErrorKind::Missing);
    drop(pool);
}

// ═══════════════════════════════════════════
// 6 ─ 并发 / deadline
// ═══════════════════════════════════════════

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn pool_acquire_with_deadline() {
    let pool = PostgresPool::connect(&config()).await.expect("connect");
    let conn = pool.acquire_with(Duration::from_secs(5)).await.expect("acquire");
    drop(conn);

    let zero = pool.acquire_with(Duration::ZERO).await;
    assert!(zero.is_err());
    if let Err(e) = zero {
        assert!(matches!(e.kind(), ErrorKind::Invalid | ErrorKind::DeadlineExceeded))
    }
    drop(pool);
}

#[tokio::test]
#[ignore = "需要 Postgres"]
async fn resilience_with_retry_sync() {
    let cfg = PgRetryConfig::fixed(2, 0);
    let result = with_retry_sync(&cfg, "test", || Ok::<i32, XError>(42)).expect("retry ok");
    assert_eq!(result, 42);
}

// ═══════════════════════════════════════════
// 7 ─ 配置验证（离线）
// ═══════════════════════════════════════════

#[test]
fn config_defaults_and_builder() {
    assert_eq!(DEFAULT_PORT, 5432);
    assert_eq!(DEFAULT_MAX_POOL_SIZE, 16);
    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .database("test")
        .user("test")
        .sslmode(SslMode::Disable)
        .max_pool_size(4)
        .build()
        .expect("valid");
    assert_eq!(cfg.max_pool_size, 4);
}

#[test]
fn config_sslmode_parsing() {
    assert_eq!(SslMode::parse("disable").unwrap(), SslMode::Disable);
    assert_eq!(SslMode::parse("require").unwrap(), SslMode::Require);
    assert!(SslMode::parse("invalid").is_err());
}

#[test]
fn config_rejects_remote_plaintext() {
    let err = PostgresConfig::builder()
        .host("db.example.com")
        .database("x")
        .user("x")
        .build()
        .expect_err("remote without ssl");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn migration_new_and_checksum() {
    let m = Migration::new(1, "test", "CREATE TABLE t (id INT)").unwrap();
    assert_eq!(m.version, 1);
    assert_eq!(m.name, "test");
    assert!(!m.checksum().is_empty());
    // checksum is stable
    assert_eq!(m.checksum(), m.checksum());
}

#[test]
fn migration_duplicate_version_rejected() {
    let m1 = Migration::new(1, "a", "SELECT 1").unwrap();
    let m2 = Migration::new(1, "b", "SELECT 2").unwrap();
    let pool_cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(1)
        .database("x")
        .user("x")
        .sslmode(SslMode::Disable)
        .acquire_timeout(Duration::from_millis(50))
        .max_pool_size(1)
        .build()
        .unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(PostgresPool::connect_lazy(&pool_cfg)).unwrap();
    let err = Migrator::new(pool, vec![m1, m2]).expect_err("dup versions");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn pool_stats() {
    let s = PoolStats { max_size: 16, size: 0, available: 0, waiting: 0, closed: false };
    assert!(!s.closed);
    assert_eq!(s.max_size, 16);
}

#[test]
fn tx_status_discriminants() {
    assert_eq!(TxStatus::Active, TxStatus::Active);
    assert_eq!(TxStatus::Committed, TxStatus::Committed);
    assert_eq!(TxStatus::Failed, TxStatus::Failed);
    assert_ne!(TxStatus::Active, TxStatus::Committed);
    assert_ne!(TxStatus::Active, TxStatus::Failed);
}

#[test]
fn pg_record_fields() {
    let r = PgRecord { id: "a".into(), data: b"123".to_vec() };
    assert_eq!(r.id, "a");
    assert_eq!(r.data, b"123");
}
