//! 真实 Postgres 集成测（默认 `#[ignore]`）。
//!
//! 环境变量（任选其一）：
//! - `DATABASE_URL`
//! - 或 `FOUNDATIONX_POSTGRESX_{HOST,PORT,DATABASE,USER,PASSWORD,SSLMODE}`
//!
//! ```bash
//! cargo test -p postgresx --test live_postgres -- --ignored --nocapture
//! ```

use postgresx::{PostgresConfig, PostgresPool, TxState};
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
    let tx = conn.begin().await.expect("begin");
    assert_eq!(tx.state(), TxState::Active);

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
