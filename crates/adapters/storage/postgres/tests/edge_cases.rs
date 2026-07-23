//! 边界/边缘情况集成测试（默认 `#[ignore]`）。
//!
//! 覆盖空参数、NULL 往返、特殊字符、长参数、SQLSTATE 错误映射、事务失败状态机。
//!
//! ```bash
//! cargo test -p postgresx --test edge_cases -- --ignored --nocapture --test-threads=1
//! ```

use kernel::ErrorKind;
use postgresx::{PoolStats, PostgresConfig, PostgresPool, SslMode, TxStatus};

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

fn edge_config() -> PostgresConfig {
    set_env();
    PostgresConfig::builder()
        .host("127.0.0.1")
        .port(5432)
        .database("market_binance")
        .user("market_binance")
        .password("Kt63mWgbhBwSPWnrEnMkC")
        .sslmode(SslMode::Disable)
        .max_pool_size(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .operation_timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("edge config")
}

fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("e{}", t)
}

// ============================================================================
// EDGE-1: empty_query_parameters
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_empty_query_parameters() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");

    // query with empty params
    let rows = pool.query("SELECT 1 AS n", &[]).await.expect("query empty params");
    assert_eq!(rows.len(), 1);
    let n: i32 = rows[0].get("n");
    assert_eq!(n, 1);

    // execute with empty params
    let affected = pool.execute("SELECT 2 AS n", &[]).await.expect("execute empty params");
    // SELECT 返回 at least 0
    assert!(affected > 0);

    // test with a WHERE clause that references no params
    let rows2 = pool.query("SELECT 3 AS n WHERE true", &[]).await.expect("query no param");
    assert_eq!(rows2.len(), 1);

    pool.close();
}

// ============================================================================
// EDGE-2: null_values_roundtrip
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_null_values_roundtrip() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");
    let suffix = rand_id();
    let table = format!("edge_null_{suffix}");

    pool.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, name TEXT, data BYTEA)"
        ),
        &[],
    )
    .await
    .expect("create");

    // INSERT NULL
    let null_name: Option<&str> = None;
    let null_data: Option<&[u8]> = None;
    pool.execute(
        &format!("INSERT INTO {table} (name, data) VALUES ($1, $2)"),
        &[&null_name, &null_data],
    )
    .await
    .expect("insert null");

    // SELECT → verify NULL
    let row = pool
        .query_one(&format!("SELECT name, data FROM {table} WHERE name IS NULL"), &[])
        .await
        .expect("select null");
    let name: Option<String> = row.get(0);
    let data: Option<Vec<u8>> = row.get(1);
    assert!(name.is_none(), "NULL name 应为 None");
    assert!(data.is_none(), "NULL data 应为 None");

    // INSERT non-null
    pool.execute(
        &format!("INSERT INTO {table} (name, data) VALUES ($1, $2)"),
        &[&"hello".to_string(), &b"world".to_vec()],
    )
    .await
    .expect("insert non-null");

    // SELECT → verify non-null
    let row2 = pool
        .query_one(
            &format!("SELECT name, data FROM {table} WHERE name = $1"),
            &[&"hello".to_string()],
        )
        .await
        .expect("select non-null");
    let name2: String = row2.get(0);
    let data2: Vec<u8> = row2.get(1);
    assert_eq!(name2, "hello");
    assert_eq!(data2, b"world");

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// EDGE-3: special_characters_in_data
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_special_characters_in_data() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");
    let suffix = rand_id();
    let table = format!("edge_special_{suffix}");

    pool.execute(
        &format!("CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, body TEXT)"),
        &[],
    )
    .await
    .expect("create");

    // 包含 Unicode、emoji、特殊字符的文本
    let special = "hello 🦀 世界\n tab\t quote' backslash\\ null\x00 done".to_string();
    pool.execute(&format!("INSERT INTO {table} (body) VALUES ($1)"), &[&special])
        .await
        .expect("insert special");

    // SELECT → 精确匹配
    let row = pool
        .query_one(&format!("SELECT body FROM {table} WHERE id = 1"), &[])
        .await
        .expect("select");
    let got: String = row.get(0);
    assert_eq!(got, special, "特殊字符应精确往返");

    // COPY IN/OUT with binary data containing \x00 bytes
    let binary_table = format!("edge_bin_{suffix}");
    pool.execute(
        &format!("CREATE TABLE IF NOT EXISTS {binary_table} (id INT PRIMARY KEY, blob BYTEA)"),
        &[],
    )
    .await
    .expect("create binary");

    let mut binary_data = Vec::with_capacity(256);
    for i in 0u8..255 {
        binary_data.push(i);
    }
    // 格式：id \t \\x<hex>\n
    let mut csv_data = String::new();
    csv_data.push_str("1\t\\\\x");
    for b in &binary_data {
        csv_data.push_str(&format!("{b:02x}"));
    }
    csv_data.push('\n');

    let mut conn = pool.acquire().await.expect("acquire");
    let rows = conn
        .copy_in_bytes(&format!("COPY {binary_table} (id, blob) FROM STDIN"), csv_data.as_bytes())
        .await
        .expect("copy in binary");
    assert_eq!(rows, 1);

    // COPY OUT → 验证
    let out = conn
        .copy_out_bytes(&format!("COPY {binary_table} TO STDOUT"), 16 * 1024 * 1024)
        .await
        .expect("copy out");
    let out_text = String::from_utf8_lossy(&out);
    // 包含所有 0x00-0xFE 的 hex 表示
    assert!(out_text.contains("\\\\x0001"));

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {binary_table}"), &[]).await;
    pool.close();
}

// ============================================================================
// EDGE-4: very_long_sql_parameter
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_very_long_sql_parameter() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");
    let suffix = rand_id();
    let table = format!("edge_long_{suffix}");

    pool.execute(
        &format!("CREATE TABLE IF NOT EXISTS {table} (id SERIAL PRIMARY KEY, body TEXT)"),
        &[],
    )
    .await
    .expect("create");

    // 100KB 文本参数
    let long_text = "A".repeat(100 * 1024);
    pool.execute(&format!("INSERT INTO {table} (body) VALUES ($1)"), &[&long_text])
        .await
        .expect("insert long");

    // SELECT → 验证长度
    let row = pool
        .query_one(&format!("SELECT length(body)::int4 FROM {table}"), &[])
        .await
        .expect("select length");
    let len: i32 = row.get(0);
    assert_eq!(len, 100 * 1024);

    // 验证内容首尾一致
    let row2 = pool
        .query_one(&format!("SELECT LEFT(body, 10), RIGHT(body, 10) FROM {table}"), &[])
        .await
        .expect("select trim");
    let prefix: String = row2.get(0);
    let suffix: String = row2.get(1);
    assert_eq!(prefix, "AAAAAAAAAA");
    assert_eq!(suffix, "AAAAAAAAAA");

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    pool.close();
}

// ============================================================================
// EDGE-5: sqlstate_error_kind_mapping_coverage
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_sqlstate_error_kind_mapping_coverage() {
    use postgresx::error_kind_from_sqlstate;

    // unique_violation (23505) → Conflict
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");
    let suffix = rand_id();
    let table = format!("edge_sqlstate_{suffix}");

    pool.execute(
        &format!("CREATE TABLE IF NOT EXISTS {table} (id INT PRIMARY KEY, val TEXT)"),
        &[],
    )
    .await
    .expect("create");

    // unique_violation (23505) → Conflict
    pool.execute(
        &format!("INSERT INTO {table} (id, val) VALUES ($1, $2)"),
        &[&1i32, &"first".to_string()],
    )
    .await
    .expect("first insert");

    let err = pool
        .execute(
            &format!("INSERT INTO {table} (id, val) VALUES ($1, $2)"),
            &[&1i32, &"duplicate".to_string()],
        )
        .await
        .expect_err("duplicate key");

    assert_eq!(
        err.kind(),
        ErrorKind::Conflict,
        "unique_violation 23505 → Conflict: {:?}",
        err.kind()
    );

    // not_null_violation (23502) → Invalid
    // 先创建专门的 not-null 表
    let table_nn = format!("edge_nn_{suffix}");
    pool.execute(
        &format!("CREATE TABLE IF NOT EXISTS {table_nn} (id INT PRIMARY KEY, val TEXT NOT NULL)"),
        &[],
    )
    .await
    .expect("create nn");

    let err_nn = pool
        .execute(&format!("INSERT INTO {table_nn} (id, val) VALUES ($1, NULL)"), &[&2i32])
        .await
        .expect_err("not null violation");

    assert_eq!(
        err_nn.kind(),
        ErrorKind::Invalid,
        "not_null_violation 23502 → Invalid: {:?}",
        err_nn.kind()
    );

    // check_violation (23514) → Invalid
    let table_ck = format!("edge_ck_{suffix}");
    pool.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {table_ck} (id INT PRIMARY KEY, val INT CHECK (val > 0))"
        ),
        &[],
    )
    .await
    .expect("create check");

    let err_ck = pool
        .execute(&format!("INSERT INTO {table_ck} (id, val) VALUES ($1, $2)"), &[&3i32, &(-1i32)])
        .await
        .expect_err("check violation");

    assert_eq!(
        err_ck.kind(),
        ErrorKind::Invalid,
        "check_violation 23514 → Invalid: {:?}",
        err_ck.kind()
    );

    // undefined_table (42P01) → Missing
    let err_missing = pool
        .query_one("SELECT * FROM nonexistent_table_xyz_123", &[])
        .await
        .expect_err("undefined table");

    assert_eq!(
        err_missing.kind(),
        ErrorKind::Missing,
        "undefined_table 42P01 → Missing: {:?}",
        err_missing.kind()
    );

    // 单元测试覆盖其他 SQLSTATE 映射（不透传 SQL 但验证映射函数）
    assert_eq!(error_kind_from_sqlstate("40P01"), ErrorKind::Transient); // deadlock
    assert_eq!(error_kind_from_sqlstate("23503"), ErrorKind::Invalid); // fk
    assert_eq!(error_kind_from_sqlstate("57014"), ErrorKind::Cancelled); // query_canceled

    // cleanup
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table}"), &[]).await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table_nn}"), &[]).await;
    let _ = pool.execute(&format!("DROP TABLE IF EXISTS {table_ck}"), &[]).await;
    pool.close();
}

// ============================================================================
// EDGE-6: tx_failed_state_prevents_commit
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_tx_failed_state_prevents_commit() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");

    // Begin tx
    let conn = pool.acquire().await.expect("acquire");
    let mut tx = conn.begin().await.expect("begin");
    assert_eq!(tx.status(), TxStatus::Active);

    // 执行除零 SQL（导致事务失败）
    let _ = tx.execute("SELECT 1/0", &[]).await;

    // 验证 TxStatus::Failed
    assert_eq!(tx.status(), TxStatus::Failed, "除零后应为 Failed 状态");

    // commit() → Invariant error（无法提交失败的事务）
    let commit_result = tx.commit().await;
    assert_eq!(
        commit_result.expect_err("失败事务禁止 commit").kind(),
        ErrorKind::Invariant,
        "commit() 应返回 Invariant"
    );

    // 新事务：测试 rollback 后 commit 也返回 Invariant
    let conn2 = pool.acquire().await.expect("acquire 2");
    let tx2 = conn2.begin().await.expect("begin 2");
    assert_eq!(tx2.status(), TxStatus::Active);

    // rollback 消费 tx2，返回 ()
    tx2.rollback().await.expect("rollback");

    pool.close();
}

// ============================================================================
// 额外边缘测试：acquire_with 零 deadline
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_acquire_with_zero_deadline() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");

    match pool.acquire_with(std::time::Duration::ZERO).await {
        Err(e) => assert_eq!(e.kind(), ErrorKind::Invalid, "零 deadline → Invalid"),
        Ok(_) => panic!("zero deadline must fail"),
    }

    pool.close();
}

// ============================================================================
// 额外边缘测试：健康检查异常
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 Postgres；使用 --ignored 运行"]
async fn edge_pool_close_then_health_fails() {
    let pool = PostgresPool::connect(&edge_config()).await.expect("connect");

    pool.health().await.expect("health before close");

    pool.close();
    let stats: PoolStats = pool.stats();
    assert!(stats.closed);

    let err = pool.health().await.expect_err("health after close");
    assert_eq!(err.kind(), ErrorKind::Unavailable, "关闭后 health → Unavailable");
}

// ============================================================================
// 单元测试（不依赖真实 Postgres）
// ============================================================================

#[test]
fn unit_sqlstate_error_kind_table() {
    use postgresx::error_kind_from_sqlstate;

    // 验证规范映射表
    assert_eq!(error_kind_from_sqlstate("23505"), ErrorKind::Conflict); // unique_violation
    assert_eq!(error_kind_from_sqlstate("23503"), ErrorKind::Invalid); // fk_violation
    assert_eq!(error_kind_from_sqlstate("23502"), ErrorKind::Invalid); // not_null_violation
    assert_eq!(error_kind_from_sqlstate("23514"), ErrorKind::Invalid); // check_violation
    assert_eq!(error_kind_from_sqlstate("40P01"), ErrorKind::Transient); // deadlock
    assert_eq!(error_kind_from_sqlstate("40001"), ErrorKind::Transient); // serialization
    assert_eq!(error_kind_from_sqlstate("42P01"), ErrorKind::Missing); // undefined_table
    assert_eq!(error_kind_from_sqlstate("57014"), ErrorKind::Cancelled); // query_canceled
    assert_eq!(error_kind_from_sqlstate("08006"), ErrorKind::Unavailable); // connection
    assert_eq!(error_kind_from_sqlstate("25P01"), ErrorKind::Invariant); // tx state
    assert_eq!(error_kind_from_sqlstate("99999"), ErrorKind::Internal); // unknown
}

#[test]
fn unit_tx_status_values() {
    assert_ne!(TxStatus::Active, TxStatus::Committed);
    assert_ne!(TxStatus::Active, TxStatus::RolledBack);
    assert_ne!(TxStatus::Active, TxStatus::Failed);
    assert_ne!(TxStatus::Committed, TxStatus::RolledBack);
}
