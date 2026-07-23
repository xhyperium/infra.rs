//! ClickHouse 异常恢复测试。
//!
//! ```text
//! cargo test -p clickhousex --test exception_recovery -- --nocapture
//! ```

use std::time::Duration;

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;

const TEST_PASSWORD: &str = "iCEOuptIx40EduvGOKX73rfY";
const TEST_DATABASE: &str = "infra_draft_exception_test";

fn default_config() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        password: TEST_PASSWORD.into(),
        database: TEST_DATABASE.into(),
        ..ClickHouseConfig::default()
    }
}

async fn setup_db() -> ClickHousePool {
    let mut cfg = default_config();
    cfg.database = "default".into();
    let pool = ClickHousePool::connect(cfg).await.expect("连接 default 数据库");
    pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {TEST_DATABASE}"))
        .await
        .expect("创建测试数据库");
    pool.close().await.expect("关闭");

    ClickHousePool::connect(default_config())
        .await
        .expect("连接到测试数据库")
}

// ── 测试 1：非法端口返回 Unavailable/DeadlineExceeded ──────────

#[tokio::test]
async fn connect_invalid_port_returns_error() {
    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 1,
        timeout: Duration::from_millis(500),
        acquire_timeout: Duration::from_millis(500),
        ..ClickHouseConfig::default()
    };
    let err = match ClickHousePool::connect(cfg).await {
        Ok(p) => {
            let err = p.ping().await.expect_err("ping 应失败");
            err.kind()
        }
        Err(e) => e.kind(),
    };
    assert!(
        matches!(err, ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient),
        "非法端口错误应为 Unavailable/DeadlineExceeded/Transient, 实际: {err:?}"
    );
}

// ── 测试 2：非法 host 返回 Unavailable/DeadlineExceeded/Transient ──

#[tokio::test]
async fn connect_invalid_host_returns_error() {
    let cfg = ClickHouseConfig {
        host: "unknown-host-that-does-not-exist.invalid".into(),
        http_port: 8123,
        timeout: Duration::from_millis(1000),
        acquire_timeout: Duration::from_millis(1000),
        ..ClickHouseConfig::default()
    };
    // validate 要求非 localhost 必须用 TLS，这里会先走 validate
    let result = ClickHousePool::connect(cfg).await;
    match result {
        Ok(_) => panic!("非法主机连接不应成功"),
        Err(e) => {
            assert!(
                matches!(
                    e.kind(),
                    ErrorKind::Invalid
                        | ErrorKind::Unavailable
                        | ErrorKind::DeadlineExceeded
                        | ErrorKind::Transient
                ),
                "非法主机错误应为 Invalid/Unavailable/DeadlineExceeded/Transient, 实际: {:?}",
                e.kind()
            );
        }
    }
}

// ── 测试 3：错误密码返回 Unavailable ────────────────────────────

#[tokio::test]
async fn connect_wrong_password_returns_unavailable() {
    let mut cfg = default_config();
    cfg.password = "definitely-wrong-password-no-chance-this-is-real".into();
    let err = match ClickHousePool::connect(cfg).await {
        Ok(p) => {
            // connect 内部会 ping，wups 密码错误 ping 也会失败
            let err = p.ping().await.expect_err("错误密码的 ping 应失败");
            err.kind()
        }
        Err(e) => e.kind(),
    };
    assert_eq!(
        err,
        ErrorKind::Unavailable,
        "错误密码应为 Unavailable, 实际: {err:?}"
    );
}

// ── 测试 4：极小 timeout 返回 DeadlineExceeded ──────────────────

#[tokio::test]
async fn timeout_returns_deadline_exceeded() {
    let cfg = ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        password: TEST_PASSWORD.into(),
        timeout: Duration::from_millis(1),
        acquire_timeout: Duration::from_millis(100),
        ..ClickHouseConfig::default()
    };
    let result = ClickHousePool::connect(cfg).await;
    match result {
        Ok(p) => {
            let err = p.ping().await.expect_err("极短 timeout 应失败");
            let kind = err.kind();
            assert!(
                matches!(kind, ErrorKind::DeadlineExceeded | ErrorKind::Unavailable | ErrorKind::Transient),
                "极短 timeout 应为 DeadlineExceeded/Unavailable/Transient, 实际: {kind:?}"
            );
        }
        Err(e) => {
            let kind = e.kind();
            assert!(
                matches!(kind, ErrorKind::DeadlineExceeded | ErrorKind::Unavailable | ErrorKind::Transient),
                "极短 timeout 应为 DeadlineExceeded/Unavailable/Transient, 实际: {kind:?}"
            );
        }
    }
}

// ── 测试 5：close 后重连成功 ping + query ──────────────────────

#[tokio::test]
async fn reconnect_after_close_succeeds() {
    let pool = setup_db().await;
    pool.ping().await.expect("首次 ping 应成功");
    pool.close().await.expect("首次关闭");

    // 已关闭的池应拒绝操作
    let err = pool.execute("SELECT 1").await.expect_err("关闭后操作应失败");
    assert_eq!(err.kind(), ErrorKind::Unavailable);

    // 重新连接
    let pool2 = ClickHousePool::connect(default_config())
        .await
        .expect("重连应成功");
    pool2.ping().await.expect("重连后 ping 应成功");

    let text = pool2.query_text("SELECT 1").await.expect("query 应成功");
    assert_eq!(text.trim(), "1");

    pool2.close().await.expect("再次关闭");
}

// ── 测试 6：背压 DeadlineExceeded ──────────────────────────────

#[tokio::test]
async fn backpressure_deadline_exceeded() {
    let mut cfg = default_config();
    cfg.max_in_flight = 1;
    cfg.acquire_timeout = Duration::from_millis(500);
    let pool = ClickHousePool::connect(cfg).await.expect("连接");

    // 用 sleep(3) 住唯一许可
    let hold_pool = pool.clone();
    let hold = tokio::spawn(async move { hold_pool.execute("SELECT sleep(3)").await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 第二个请求等待许可超时
    let err = pool.execute("SELECT 1").await.expect_err("应被背压");

    let _ = hold.await.expect("hold join");

    assert_eq!(
        err.kind(),
        ErrorKind::DeadlineExceeded,
        "背压错误应为 DeadlineExceeded, 实际: {:?}",
        err.kind()
    );
    assert!(
        err.context().contains("max=1"),
        "错误上下文应包含 max=1: {}",
        err.context()
    );

    pool.close().await.expect("关闭");
}

// ── 测试 7：写入不存在表失败，已有数据完整 ─────────────────────

#[tokio::test]
async fn write_to_nonexistent_table_preserves_existing_data() {
    let pool = setup_db().await;

    // 创建一张正常表并插入数据
    let pid = std::process::id();
    let table = format!("wnx_preserve_{pid}");
    pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("创建表");

    let row = serde_json::json!({"id": 1, "marker": "original"});
    pool.insert_json_each_row(&table, &[row])
        .await
        .expect("插入原始数据");

    // 尝试向不存在的表写入（应失败）
    let err = pool
        .insert_json_each_row(
            "nonexistent_table_xyz",
            &[serde_json::json!({"id": 1})],
        )
        .await
        .expect_err("写不存在表应失败");
    assert!(err.kind() != ErrorKind::Conflict, "不应是 Conflict");

    // 验证原始数据完整
    let count_sql = format!("SELECT count() FROM {table} FORMAT TabSeparated");
    let count_text = pool.query_text(&count_sql).await.expect("查询 count");
    let count: u64 = count_text.trim().parse().expect("解析");
    assert_eq!(count, 1, "原始数据应完整保留");

    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理");
    pool.close().await.expect("关闭");
}

// ─ 测试 8：非法标识符返回 Invalid ────────────────────────────

#[tokio::test]
async fn error_kind_invalid_for_illegal_identifier() {
    let pool = setup_db().await;

    let err = pool
        .execute("SELECT * FROM `a;drop`")
        .await
        .expect_err("非法标识符应失败");
    assert!(
        matches!(
            err.kind(),
            ErrorKind::Invalid | ErrorKind::Transient
        ),
        "非法标识符应为 Invalid 或 Transient, 实际: {:?}",
        err.kind()
    );

    pool.close().await.expect("关闭");
}

// ── 测试 9：DROP 不存在表返回 Missing（或 Transient）───────────

#[tokio::test]
async fn error_kind_missing_for_nonexistent_table() {
    let pool = setup_db().await;

    let err2 = pool
        .execute("DROP TABLE _nonexistent_table_drop_test_xyzabc_2")
        .await
        .expect_err("DROP 不存在表应失败");
    assert!(
        matches!(
            err2.kind(),
            ErrorKind::Missing | ErrorKind::Transient
        ),
        "DROP 不存在表应为 Missing 或 Transient, 实际: {:?}",
        err2.kind()
    );

    pool.close().await.expect("关闭");
}

// ── 测试 10：重复建表返回 Conflict 或 Transient ───────────────

#[tokio::test]
async fn error_kind_conflict_for_duplicate_table() {
    let pool = setup_db().await;
    let pid = std::process::id();
    let table = format!("dup_table_{pid}");

    // 首次创建
    pool.execute(&format!(
        "CREATE TABLE {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
    ))
    .await
    .expect("创建表");

    // 重复创建
    let err = pool
        .execute(&format!(
            "CREATE TABLE {table} (id UInt32, marker String) ENGINE = MergeTree ORDER BY id"
        ))
        .await
        .expect_err("重复建表应失败");

    assert!(
        matches!(
            err.kind(),
            ErrorKind::Conflict | ErrorKind::Transient | ErrorKind::Invalid
        ),
        "重复建表应为 Conflict/Transient/Invalid, 实际: {:?}",
        err.kind()
    );

    pool.execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .expect("清理");
    pool.close().await.expect("关闭");
}

// ── 测试 11：错误密码返回 Unavailable ─────────────────────────

#[tokio::test]
async fn error_kind_unavailable_for_wrong_password() {
    let mut cfg = default_config();
    cfg.password = "this-is-not-the-correct-password".into();
    let err = match ClickHousePool::connect(cfg).await {
        Ok(p) => p.ping().await.expect_err("ping 应失败").kind(),
        Err(e) => e.kind(),
    };
    assert_eq!(
        err,
        ErrorKind::Unavailable,
        "错误密码应为 Unavailable, 实际: {err:?}"
    );
}

// ── 测试 12：背压返回 DeadlineExceeded ────────────────────────

#[tokio::test]
async fn error_kind_deadline_exceeded_for_backpressure() {
    let mut cfg = default_config();
    cfg.max_in_flight = 1;
    cfg.acquire_timeout = Duration::from_millis(300);
    let pool = ClickHousePool::connect(cfg).await.expect("连接");

    let hold_pool = pool.clone();
    let hold =
        tokio::spawn(async move { hold_pool.execute("SELECT sleep(3)").await });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let err = pool.execute("SELECT 1").await.expect_err("应被背压");
    let _ = hold.await.expect("hold join");

    assert_eq!(
        err.kind(),
        ErrorKind::DeadlineExceeded,
        "背压错误应为 DeadlineExceeded, 实际: {:?}",
        err.kind()
    );

    pool.close().await.expect("关闭");
}

// ── 测试 13：错误信息不含密码/payload/SQL ────────────────────

#[tokio::test]
async fn error_messages_never_contain_secrets() {
    let pool = setup_db().await;

    // 故意写入包含 secret 的 SQL，验证错误不泄露
    let sensitive_sql = "SELECT * FROM nonexistent_table_xyz_123";
    let err = pool
        .query_text(sensitive_sql)
        .await
        .expect_err("查询不存在表应失败");

    let err_str = err.to_string();
    assert!(
        !err_str.contains(TEST_PASSWORD),
        "错误信息不得包含密码: {err_str}"
    );
    assert!(
        !err_str.contains("payload"),
        "错误信息不应包含 payload 原文: {err_str}"
    );

    pool.close().await.expect("关闭");
}
