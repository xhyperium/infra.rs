//! PostgreSQL 真实截止时间与连接隔离实验（默认忽略）。

use std::error::Error;
use std::time::{Duration, Instant};

use kernel::{ErrorKind, XError};
use postgresx::{PostgresConfig, PostgresPool, SslMode, TxState};

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("缺少环境变量 {name}"))
}

#[tokio::test]
#[ignore = "由 scripts/postgres-deadline-conformance.mjs 启动固定镜像后执行"]
async fn pool_and_query_deadlines_fail_closed_then_recover() {
    let port = required("INFRA_POSTGRES_TEST_PORT").parse::<u16>().expect("合法端口");
    let config = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(port)
        .database("postgres")
        .user("postgres")
        .password(required("INFRA_POSTGRES_TEST_PASSWORD"))
        .sslmode(SslMode::Disable)
        .max_pool_size(1)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_millis(250))
        .operation_timeout(Duration::from_millis(300))
        .build()
        .expect("本机测试配置合法");
    let pool = PostgresPool::connect(&config).await.expect("连接 PostgreSQL");

    let held = pool.acquire().await.expect("占用唯一连接");
    let started = Instant::now();
    let acquire_error = match pool.acquire().await {
        Ok(_) => panic!("池饱和时不应获取第二条连接"),
        Err(error) => error,
    };
    assert_eq!(acquire_error.kind(), ErrorKind::DeadlineExceeded);
    assert!(started.elapsed() < Duration::from_secs(2), "获取截止时间必须有界");
    drop(held);

    let row = pool.query_one("SELECT 1", &[]).await.expect("释放后恢复获取连接");
    assert_eq!(row.get::<_, i32>(0), 1);

    // 关闭服务端 statement_timeout，强制由调用侧截止时间取消 future；适配器必须
    // 丢弃处于未知状态的连接，而不是把它归还到池中。
    let mut connection = pool.acquire().await.expect("获取慢查询连接");
    connection.execute("SET statement_timeout = 0", &[]).await.expect("关闭服务端超时");
    let query_error = connection
        .query_one("SELECT pg_sleep(2)", &[])
        .await
        .expect_err("慢查询必须由调用侧截止时间终止");
    assert_eq!(query_error.kind(), ErrorKind::DeadlineExceeded);
    assert!(Error::source(&query_error).is_some(), "截止错误必须保留 source");
    assert_eq!(pool.stats().size, 0, "内部超时连接必须永久移出池");
    assert_eq!(pool.stats().available, 0, "内部超时连接不得重新变为可用");
    drop(connection);

    let recovered = pool.query_one("SELECT 1", &[]).await.expect("丢弃超时连接后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);

    // 外层 deadline 比适配器 operation_timeout 更短时，future cancellation 也必须触发
    // RAII guard，把仍在执行的连接永久移出 deadpool。
    let mut externally_cancelled = pool.acquire().await.expect("获取外层取消连接");
    externally_cancelled.execute("SET statement_timeout = 0", &[]).await.expect("关闭服务端超时");
    let outer = tokio::time::timeout(
        Duration::from_millis(100),
        externally_cancelled.query_one("SELECT pg_sleep(2)", &[]),
    )
    .await;
    assert!(outer.is_err(), "外层 deadline 必须先取消查询 future");
    assert_eq!(pool.stats().size, 0, "外层取消连接必须永久移出池");
    assert_eq!(pool.stats().available, 0, "外层取消连接不得重新变为可用");
    drop(externally_cancelled);
    let recovered = pool.query_one("SELECT 1", &[]).await.expect("外层取消后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);

    // 事务内 SQL 的外层取消同样不得让 open transaction 回池。
    let mut transaction = pool.begin().await.expect("开启取消测试事务");
    transaction
        .execute("SET LOCAL statement_timeout = 0", &[])
        .await
        .expect("关闭事务内服务端超时");
    let outer_tx = tokio::time::timeout(
        Duration::from_millis(100),
        transaction.query_one("SELECT pg_sleep(2)", &[]),
    )
    .await;
    assert!(outer_tx.is_err(), "外层 deadline 必须取消事务查询 future");
    assert_eq!(transaction.state(), TxState::Failed, "取消后事务必须进入失败态");
    assert!(!transaction.is_active(), "取消后的事务不得继续接受 SQL");
    assert_eq!(pool.stats().size, 0, "事务取消连接必须永久移出池");
    assert_eq!(pool.stats().available, 0, "事务取消连接不得重新变为可用");
    drop(transaction);
    let recovered = pool.query_one("SELECT 1", &[]).await.expect("事务取消后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);

    // PostgreSQL 语句错误会把事务置为 aborted；适配器必须进入 rollback-only Failed，
    // 不能让后续 COMMIT 把服务端 ROLLBACK 的 CommandComplete 误报成提交成功。
    let mut failed_transaction = pool.begin().await.expect("开启语句失败测试事务");
    failed_transaction.query_one("SELECT 1 / 0", &[]).await.expect_err("除零必须使事务失败");
    assert_eq!(failed_transaction.state(), TxState::Failed);
    failed_transaction.rollback().await.expect("失败事务仍允许显式回滚");
    let recovered = pool.query_one("SELECT 1", &[]).await.expect("显式回滚后连接可复用");
    assert_eq!(recovered.get::<_, i32>(0), 1);

    let mut false_commit = pool.begin().await.expect("开启伪提交防护事务");
    false_commit.query_one("SELECT 1 / 0", &[]).await.expect_err("除零必须使事务失败");
    false_commit.commit().await.expect_err("Failed 事务禁止 COMMIT");
    assert_eq!(pool.stats().size, 0, "拒绝伪提交后必须关闭失败事务连接");
    let recovered = pool.query_one("SELECT 1", &[]).await.expect("拒绝伪提交后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);

    // with_transaction 的业务 future 若因内部 deadline 失败，自动 rollback 也会因连接
    // 已丢弃而失败；返回值仍必须保持原 DeadlineExceeded 分类与 source chain。
    let wrapped_timeout = pool
        .with_transaction(async |tx| {
            tx.execute("SET LOCAL statement_timeout = 0", &[]).await?;
            let _ = tx.query_one("SELECT pg_sleep(2)", &[]).await?;
            Ok::<(), XError>(())
        })
        .await
        .expect_err("事务 SQL 必须由内部 deadline 终止");
    assert_eq!(wrapped_timeout.kind(), ErrorKind::DeadlineExceeded);
    assert!(Error::source(&wrapped_timeout).is_some(), "双错误必须保留原 source chain");
    assert_eq!(pool.stats().size, 0, "双错误路径不得把未知连接归池");
    let recovered = pool.query_one("SELECT 1", &[]).await.expect("双错误后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);
    pool.close();
}
