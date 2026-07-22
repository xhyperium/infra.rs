//! PostgreSQL 真实截止时间与连接隔离实验（默认忽略）。

use std::error::Error;
use std::time::{Duration, Instant};

use kernel::ErrorKind;
use postgresx::{PostgresConfig, PostgresPool, SslMode};

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
    drop(connection);

    let recovered = pool.query_one("SELECT 1", &[]).await.expect("丢弃超时连接后新建连接");
    assert_eq!(recovered.get::<_, i32>(0), 1);
    pool.close();
}
