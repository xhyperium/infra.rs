//! ClickHouse 版本兼容配置集成测试。
//!
//! - 版本检测和 DDL 兼容性需要本地 ClickHouse。
//! - 配置校验和环境变量解析为离线测试。
#![allow(unused_unsafe)] // set_env/remove_env wrappers internally handle unsafe

use std::sync::Mutex;
use std::time::Duration;

// env_helpers: wrap unsafe set_var/remove_var for Rust 2024
fn set_env(k: &str, v: &str) {
    unsafe {
        std::env::set_var(k, v);
    }
}
fn remove_env(k: &str) {
    unsafe {
        std::env::remove_var(k);
    }
}

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;

// ═══════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════

/// 所有可能影响 from_env 的环境变量。
const CH_ENV_VARS: &[&str] = &[
    "FOUNDATIONX_CLICKHOUSEX_HOST",
    "FOUNDATIONX_CLICKHOUSEX_HTTP_PORT",
    "FOUNDATIONX_CLICKHOUSEX_PORT",
    "FOUNDATIONX_CLICKHOUSEX_TLS",
    "FOUNDATIONX_CLICKHOUSEX_TLS_CA_FILE",
    "FOUNDATIONX_CLICKHOUSEX_USER",
    "FOUNDATIONX_CLICKHOUSEX_PASSWORD",
    "FOUNDATIONX_CLICKHOUSEX_DATABASE",
    "FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS",
    "FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST",
    "FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT",
    "FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS",
];

/// 串行化环境变量操作。
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// 清理所有相关环境变量。
fn clear_ch_env_vars() {
    for var in CH_ENV_VARS {
        unsafe { remove_env(var) };
    }
}

/// 在隔离环境中执行测试闭包：锁 mutex → 清理 → 执行 → 清理。
fn with_clean_env<F: FnOnce() + std::panic::UnwindSafe>(f: F) {
    let _lock = ENV_MUTEX.lock().unwrap();
    clear_ch_env_vars();
    let result = std::panic::catch_unwind(f);
    clear_ch_env_vars();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

/// 本地 ClickHouse 连接配置。
fn ch_cfg() -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: 8123,
        user: "default".into(),
        password: "iCEOuptIx40EduvGOK73XrfY".into(),
        ..ClickHouseConfig::default()
    }
}

// ═══════════════════════════════════════════════════════════════
// 版本检测
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn clickhouse_version_is_detectable() {
    let pool = ClickHousePool::connect(ch_cfg()).await.expect("connect");
    let text = pool.query_text("SELECT version()").await.expect("version");
    assert!(!text.trim().is_empty());
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn clickhouse_version_contains_dots() {
    let pool = ClickHousePool::connect(ch_cfg()).await.expect("connect");
    let text = pool.query_text("SELECT version()").await.expect("version");
    assert!(text.contains('.'));
}

// ═══════════════════════════════════════════════════════════════
// DDL 兼容性
// ═══════════════════════════════════════════════════════════════

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn create_table_if_not_exists_is_idempotent() {
    let db = "gap_zero_ddl";
    let pool = ClickHousePool::connect(ClickHouseConfig { database: db.into(), ..ch_cfg() })
        .await
        .expect("connect");

    let _ = pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {db}")).await;

    let sql = "CREATE TABLE IF NOT EXISTS ddl_idem (x UInt8) ENGINE = MergeTree ORDER BY x";
    pool.execute(sql).await.expect("首次建表");
    pool.execute(sql).await.expect("二次 CREATE IF NOT EXISTS 也应成功");

    // 清理
    let _ = pool.execute("DROP TABLE IF EXISTS ddl_idem").await;
    let _ = pool.execute(&format!("DROP DATABASE IF EXISTS {db}")).await;
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn create_or_replace_table() {
    let db = "gap_zero_replace";
    let pool = ClickHousePool::connect(ClickHouseConfig { database: db.into(), ..ch_cfg() })
        .await
        .expect("connect");

    let _ = pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {db}")).await;

    pool.execute("CREATE OR REPLACE TABLE rep_t (val String) ENGINE = MergeTree ORDER BY val")
        .await
        .expect("首次建表");
    // 再次替换应成功
    pool.execute(
        "CREATE OR REPLACE TABLE rep_t (val String, extra UInt8) ENGINE = MergeTree ORDER BY val",
    )
    .await
    .expect("替换建表");

    // 清理
    let _ = pool.execute("DROP TABLE IF EXISTS rep_t").await;
    let _ = pool.execute(&format!("DROP DATABASE IF EXISTS {db}")).await;
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn alter_table_add_column() {
    let db = "gap_zero_alter";
    let pool = ClickHousePool::connect(ClickHouseConfig { database: db.into(), ..ch_cfg() })
        .await
        .expect("connect");

    let _ = pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {db}")).await;

    pool.execute("CREATE TABLE IF NOT EXISTS alt_tab (id UInt8) ENGINE = MergeTree ORDER BY id")
        .await
        .expect("建表");

    pool.execute("ALTER TABLE alt_tab ADD COLUMN IF NOT EXISTS name String")
        .await
        .expect("增加列应成功");

    // 清理
    let _ = pool.execute("DROP TABLE IF EXISTS alt_tab").await;
    let _ = pool.execute(&format!("DROP DATABASE IF EXISTS {db}")).await;
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn drop_table_if_exists() {
    let db = "gap_zero_drop";
    let pool = ClickHousePool::connect(ClickHouseConfig { database: db.into(), ..ch_cfg() })
        .await
        .expect("connect");

    let _ = pool.execute(&format!("CREATE DATABASE IF NOT EXISTS {db}")).await;

    pool.execute("CREATE TABLE IF NOT EXISTS drp_me (x UInt8) ENGINE = MergeTree ORDER BY x")
        .await
        .expect("建表");
    pool.execute("DROP TABLE IF EXISTS drp_me").await.expect("删表");
    // 二次删除不报错
    pool.execute("DROP TABLE IF EXISTS drp_me").await.expect("二次 DROP IF EXISTS 也应成功");

    // 清理
    let _ = pool.execute(&format!("DROP DATABASE IF EXISTS {db}")).await;
}

// ═══════════════════════════════════════════════════════════════
// 配置兼容性
// ═══════════════════════════════════════════════════════════════

#[test]
fn default_config_is_valid() {
    ClickHouseConfig::default().validate().expect("默认配置必须有效");
}

#[test]
fn from_env_respects_overrides() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HOST", "ch2.example.com");
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "9440");
            set_env("FOUNDATIONX_CLICKHOUSEX_TLS", "true");
            set_env("FOUNDATIONX_CLICKHOUSEX_USER", "reader");
            set_env("FOUNDATIONX_CLICKHOUSEX_PASSWORD", "secret123");
            set_env("FOUNDATIONX_CLICKHOUSEX_DATABASE", "analytics");
            set_env("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS", "5000");
            set_env("FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST", "4");
        }

        let cfg = ClickHouseConfig::from_env().expect("from_env 解析应成功");
        assert_eq!(cfg.host, "ch2.example.com");
        assert_eq!(cfg.http_port, 9440);
        assert!(cfg.tls);
        assert_eq!(cfg.user, "reader");
        assert_eq!(cfg.password, "secret123");
        assert_eq!(cfg.database, "analytics");
        assert_eq!(cfg.timeout, Duration::from_millis(5000));
        assert_eq!(cfg.max_idle_per_host, 4);
    });
}

#[test]
fn from_env_empty_string_does_not_override() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HOST", "");
            set_env("FOUNDATIONX_CLICKHOUSEX_USER", "");
            set_env("FOUNDATIONX_CLICKHOUSEX_DATABASE", "");
        }

        let cfg = ClickHouseConfig::from_env().expect("空 env 用默认值");
        assert_eq!(cfg.host, "127.0.0.1", "空 HOST 应保持默认");
        assert_eq!(cfg.user, "default", "空 USER 应保持默认");
        assert_eq!(cfg.database, "default", "空 DATABASE 应保持默认");
    });
}

#[test]
fn from_env_with_no_vars_uses_defaults() {
    with_clean_env(|| {
        let cfg = ClickHouseConfig::from_env().expect("无 env 返回默认配置");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.http_port, 8123);
        assert_eq!(cfg.user, "default");
        assert_eq!(cfg.database, "default");
        assert_eq!(cfg.max_in_flight, 64);
        assert_eq!(cfg.max_idle_per_host, 8);
    });
}

// ═══════════════════════════════════════════════════════════════
// 配置错误
// ═══════════════════════════════════════════════════════════════

#[test]
fn invalid_env_variables_cause_errors() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT", "not-a-number");
        }
        let err = ClickHouseConfig::from_env().expect_err("非法数字必须失败");
        assert!(err.context().contains("MAX_IN_FLIGHT"));
    });
}

#[test]
fn empty_host_validate_fails() {
    let cfg = ClickHouseConfig { host: String::new(), ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("空 host 必须失败");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[test]
fn whitespace_only_host_validate_fails() {
    let cfg = ClickHouseConfig { host: "  ".into(), ..ClickHouseConfig::default() };
    let err = cfg.validate().expect_err("空白 host 必须失败");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[ignore = "requires live ClickHouse"]
#[tokio::test]
async fn zero_max_in_flight_connect_fails() {
    let cfg = ClickHouseConfig { max_in_flight: 0, ..ClickHouseConfig::default() };
    let err = match ClickHousePool::connect(cfg).await {
        Ok(_) => panic!("零 max_in_flight 必须失败"),
        Err(e) => e,
    };
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

// ═══════════════════════════════════════════════════════════════
// 端口别名
// ═══════════════════════════════════════════════════════════════

#[test]
fn only_http_port_set() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "8443");
        }
        let cfg = ClickHouseConfig::from_env().expect("from_env");
        assert_eq!(cfg.http_port, 8443);
    });
}

#[test]
fn only_port_alias_set() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_PORT", "9440");
        }
        let cfg = ClickHouseConfig::from_env().expect("from_env");
        assert_eq!(cfg.http_port, 9440);
    });
}

#[test]
fn both_same_value() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "9440");
            set_env("FOUNDATIONX_CLICKHOUSEX_PORT", "9440");
        }
        let cfg = ClickHouseConfig::from_env().expect("from_env");
        assert_eq!(cfg.http_port, 9440);
    });
}

#[test]
fn port_conflict_fails() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "8123");
            set_env("FOUNDATIONX_CLICKHOUSEX_PORT", "8443");
        }
        let err = ClickHouseConfig::from_env().expect_err("端口冲突必须失败");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    });
}

// ═══════════════════════════════════════════════════════════════
// from_env 完整覆盖
// ═══════════════════════════════════════════════════════════════

#[test]
fn from_env_full_coverage() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HOST", "ch.internal");
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "9440");
            set_env("FOUNDATIONX_CLICKHOUSEX_TLS", "true");
            set_env("FOUNDATIONX_CLICKHOUSEX_USER", "admin");
            set_env("FOUNDATIONX_CLICKHOUSEX_PASSWORD", "p@ss");
            set_env("FOUNDATIONX_CLICKHOUSEX_DATABASE", "metrics");
            set_env("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS", "6000");
            set_env("FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST", "12");
            set_env("FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT", "128");
            set_env("FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS", "4000");
        }

        let cfg = ClickHouseConfig::from_env().expect("from_env 解析应成功");
        assert_eq!(cfg.host, "ch.internal");
        assert_eq!(cfg.http_port, 9440);
        assert!(cfg.tls);
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.password, "p@ss");
        assert_eq!(cfg.database, "metrics");
        assert_eq!(cfg.timeout, Duration::from_millis(6000));
        assert_eq!(cfg.max_idle_per_host, 12);
        assert_eq!(cfg.max_in_flight, 128);
        assert_eq!(cfg.acquire_timeout, Duration::from_millis(4000));
    });
}

#[test]
fn from_env_invalid_number_fails() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS", "xyz");
        }
        let err = ClickHouseConfig::from_env().expect_err("非数字必须失败");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    });
}

#[test]
fn invalid_port_number_fails() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_PORT", "-1");
        }
        let err = ClickHouseConfig::from_env().expect_err("非法端口必须失败");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    });
}

#[test]
fn partial_env_config_validates() {
    with_clean_env(|| {
        unsafe {
            set_env("FOUNDATIONX_CLICKHOUSEX_HOST", "127.0.0.1");
            set_env("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", "9123");
            set_env("FOUNDATIONX_CLICKHOUSEX_DATABASE", "custom_db");
        }

        let cfg = ClickHouseConfig::from_env().expect("from_env 解析应成功");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.http_port, 9123);
        assert_eq!(cfg.database, "custom_db");
        // 默认值
        assert_eq!(cfg.user, "default");
        assert_eq!(cfg.max_in_flight, 64);
        assert_eq!(cfg.max_idle_per_host, 8);
        assert_eq!(cfg.timeout, Duration::from_secs(10));
    });
}
