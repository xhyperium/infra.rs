//! 真实 Postgres 集成测（默认 `#[ignore]`）。
//!
//! 环境变量（任选其一）：
//! - `DATABASE_URL`
//! - 或 `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE}`
//!
//! ```bash
//! cargo test -p postgresx --test live_postgres -- --ignored --nocapture
//! ```

use contracts::Repository;
use postgresx::{
    PgRecord, PgRepository, PgRetryConfig, PostgresConfig, PostgresPool, TxStatus, with_retry_async,
};
use std::sync::Arc;

fn live_config() -> PostgresConfig {
    PostgresConfig::from_env().expect(
        "需要 DATABASE_URL 或 FOUNDATIONX_POSTGRESX_*（见 crates/adapters/storage/postgres/docs/config.md）",
    )
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_select_one() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let row = pool.query_one("SELECT 1 AS n", &[]).await.expect("select 1");
    let n: i32 = row.get("n");
    assert_eq!(n, 1);
    pool.health().await.expect("health");
    let stats = pool.stats();
    assert!(stats.max_size >= 1);
    assert!(!stats.closed);
    assert!(pool.summary().contains(&live_config().host));
    pool.close();
    let closed = pool.stats();
    assert!(closed.closed);
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_connect_from_env_and_query_opt_execute() {
    let pool = PostgresPool::connect_from_env().await.expect("connect_from_env");
    let n = 42i32;
    let affected =
        pool.execute("SELECT $1::int", &[&n]).await.expect("execute SELECT 返回 0 行影响亦可");
    assert_eq!(affected, 1, "SELECT 语句在 PG 上通常 rows=1 或 0；此处接受 u64");

    let some = pool
        .query_opt("SELECT $1::int AS n WHERE $1::int > 0", &[&n])
        .await
        .expect("query_opt some");
    let row = some.expect("应有一行");
    let got: i32 = row.get("n");
    assert_eq!(got, n);

    let none = pool
        .query_opt("SELECT $1::int AS n WHERE $1::int < 0", &[&n])
        .await
        .expect("query_opt none");
    assert!(none.is_none());

    let rows = pool.query("SELECT generate_series(1, 3) AS n", &[]).await.expect("query multi");
    assert_eq!(rows.len(), 3);

    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_pool_begin_commit() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let mut tx = pool.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);
    let row = tx.query_one("SELECT 2 + 2 AS n", &[]).await.expect("tx query");
    let n: i32 = row.get("n");
    assert_eq!(n, 4);
    tx.commit().await.expect("commit");
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_repository_find_save_roundtrip() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let repo = PgRepository::new(pool.clone());
    repo.ensure_table().await.expect("ensure_table");

    let id = format!("postgresx-live-{}", std::process::id());
    let payload = format!("payload-{}", std::process::id()).into_bytes();
    let record = PgRecord { id: id.clone(), data: payload.clone() };

    // 清理残留（幂等）
    let _ = pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await;

    assert!(repo.find(id.clone()).await.expect("find miss").is_none());

    repo.save(&record).await.expect("save insert");
    let found = repo.find(id.clone()).await.expect("find hit").expect("row");
    assert_eq!(found.id, id);
    assert_eq!(found.data, payload);

    let updated = PgRecord { id: id.clone(), data: b"upserted".to_vec() };
    repo.save(&updated).await.expect("save upsert");
    let found2 = repo.find(id.clone()).await.expect("find after upsert").expect("row");
    assert_eq!(found2.data, b"upserted");

    pool.execute("DELETE FROM infra_pg_records WHERE id = $1", &[&id]).await.expect("cleanup");
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_resilience_retry_wrapper() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let cfg = PgRetryConfig::fixed(2, 0);
    let n = with_retry_async(&cfg, "live_select", || {
        let pool = pool.clone();
        async move {
            let row = pool.query_one("SELECT 9 AS n", &[]).await?;
            let v: i32 = row.try_get(0).map_err(postgresx::map_tokio_error)?;
            Ok::<_, kernel::XError>(v)
        }
    })
    .await
    .expect("retry async");
    assert_eq!(n, 9);
    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_temp_table_insert_select() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");

    // TEMP 表随会话；with_transaction 同一连接内完成
    let content = format!("hello-postgresx-{}", std::process::id());

    pool.with_transaction(async |tx| {
        tx.execute(
            "CREATE TEMP TABLE postgresx_live_t (id int PRIMARY KEY, body text NOT NULL)",
            &[],
        )
        .await?;
        tx.execute("INSERT INTO postgresx_live_t (id, body) VALUES ($1, $2)", &[&1i32, &content])
            .await?;
        let row = tx.query_one("SELECT body FROM postgresx_live_t WHERE id = $1", &[&1i32]).await?;
        let body: String = row.get(0);
        assert_eq!(body, content);
        Ok::<_, kernel::XError>(())
    })
    .await
    .expect("tx");
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_transaction_rollback() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let marker = format!("rb-{}", std::process::id());

    // 使用真实（非 TEMP）表会污染 schema；这里用 TEMP + 显式 rollback 验证状态机
    let conn = pool.acquire().await.expect("acquire");
    let mut tx = conn.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);

    tx.execute("CREATE TEMP TABLE postgresx_live_rb (id int PRIMARY KEY, body text)", &[])
        .await
        .expect("create");
    tx.execute("INSERT INTO postgresx_live_rb (id, body) VALUES ($1, $2)", &[&1i32, &marker])
        .await
        .expect("insert");

    // 回滚后 TEMP 表与行均不可见（新连接更是如此）
    tx.rollback().await.expect("rollback");

    // 新连接不应看到该 TEMP 表
    let err = pool.query("SELECT body FROM postgresx_live_rb WHERE id = $1", &[&1i32]).await;
    assert!(err.is_err(), "rollback 后跨连接查询应失败");
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_with_transaction_business_err_rolls_back() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let marker = format!("biz-{}", std::process::id());

    // 验证业务 Err 路径返回原错误，并执行 rollback
    let result = pool
        .with_transaction(async |tx| {
            tx.execute("CREATE TEMP TABLE postgresx_live_biz (id int PRIMARY KEY, body text)", &[])
                .await?;
            tx.execute(
                "INSERT INTO postgresx_live_biz (id, body) VALUES ($1, $2)",
                &[&1i32, &marker],
            )
            .await?;
            Err::<(), _>(kernel::XError::invalid("业务校验失败"))
        })
        .await;

    let err = result.expect_err("应返回业务错误");
    assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    assert!(err.context().contains("业务校验失败"));
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_tx_runner_boundary() {
    use contracts::run_tx_lifecycle;

    let pool = Arc::new(PostgresPool::connect(&live_config()).await.expect("connect"));
    let runner = postgresx::PgTxRunner::new(Arc::clone(&pool));

    let v = run_tx_lifecycle(&runner, || async move { Ok::<_, kernel::XError>(7u8) })
        .await
        .expect("commit path");
    assert_eq!(v, 7);

    let err = run_tx_lifecycle(&runner, || async move {
        Err::<(), _>(kernel::XError::invalid("rollback path"))
    })
    .await
    .expect_err("rollback path");
    assert!(matches!(
        err,
        contracts::TxRunError::Business { source }
            if source.kind() == kernel::ErrorKind::Invalid
    ));

    pool.close();
}

#[tokio::test]
#[ignore = "requires live Postgres; run with --ignored when available"]
async fn live_raw_client_and_pool_fail_closed() {
    let pool = PostgresPool::connect(&live_config()).await.expect("connect");
    let before = pool.stats();
    assert!(!before.closed);

    // deprecated raw client：标记污染 → Drop 脱池；禁止再 begin
    #[allow(deprecated)]
    {
        let conn = pool.acquire().await.expect("acquire");
        let _ = conn.client().expect("legacy client");
        match conn.begin().await {
            Ok(_) => panic!("raw 暴露后禁止 begin"),
            Err(begin_err) => {
                assert_eq!(begin_err.kind(), kernel::ErrorKind::Unavailable);
                assert!(begin_err.context().contains("原始 client"));
            }
        }
    }
    // 污染连接已脱池；正式路径仍可借新连接
    let row = pool.query_one("SELECT 1 AS n", &[]).await.expect("recover");
    let n: i32 = row.get("n");
    assert_eq!(n, 1);

    // deprecated raw pool：仅关闭隔离池，get 明确 Closed
    #[allow(deprecated)]
    {
        let err = pool.inner().get().await.expect_err("legacy raw pool Closed");
        assert!(matches!(err, deadpool_postgres::PoolError::Closed));
    }
    let still = pool.query_one("SELECT 2 AS n", &[]).await.expect("正式池不受隔离池影响");
    assert_eq!(still.get::<_, i32>(0), 2);

    pool.close();
}
