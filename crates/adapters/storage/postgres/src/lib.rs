//! `postgresx` — Postgres 存储适配（**生产 SQL/连接池为默认导出**）。
//!
//! ## 默认（生产）
//!
//! - [`PostgresConfig`] / [`PostgresConfigBuilder`]：`DATABASE_URL` 或
//!   `FOUNDATIONX_POSTGRESX_*` 环境变量
//! - [`PostgresPool`]：`connect` / `acquire` / `execute` / `query` / `query_one` /
//!   `with_transaction` / `health` / `stats` / `close`
//! - [`PgConnection`] / [`PgTransaction`]（[`TxState`]）
//! - [`PgTxRunner`]：`contracts::TxRunner` 边界适配（**不**传 SQL 句柄，见模块文档）
//! - [`PgRepository`] / [`PgRecord`]：生产 `contracts::Repository`
//! - [`MakeRustlsConnect`]：`SslMode::Prefer` / `Require` 的 rustls TLS
//! - [`with_retry_sync`] / [`with_retry_async`]：resiliencx 重试
//! - SQLSTATE → [`kernel::ErrorKind`] 映射：[`error_kind_from_sqlstate`]
//!
//! ## 可选 scaffold
//!
//! feature `scaffold`：进程内 `PostgresAdapter` / `ObservingPostgresAdapter`
//! （**非**真实 Postgres）。
//!
//! ## 安全
//!
//! 所有查询 API 仅接受参数化 `$N` + [`ToSql`]；
//! 禁止把用户输入拼进 SQL 字符串。

#![forbid(unsafe_code)]

mod config;
mod conn;
mod error;
mod pool;
mod repository;
mod resilience;
mod runner;
mod tls;
mod tx;

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
mod mock;

pub use config::{
    DEFAULT_MAX_POOL_SIZE, DEFAULT_PORT, PostgresConfig, PostgresConfigBuilder, SslMode,
};
pub use conn::PgConnection;
pub use error::{error_kind_from_sqlstate, map_pool_error, map_tokio_error, xerror_from_sqlstate};
pub use pool::{PoolStats, PostgresPool};
pub use repository::{PgRecord, PgRepository};
pub use resilience::{
    PgRetryConfig, with_budget, with_budget_async, with_budget_async_noop, with_budget_noop,
    with_retry_async, with_retry_async_no_wait, with_retry_sync,
};
pub use runner::PgTxRunner;
pub use tls::{MakeRustlsConnect, build_client_config};
pub use tx::{PgTransaction, TxState};

#[cfg(feature = "scaffold")]
pub use adapter::{PostgresAdapter, Record};
#[cfg(feature = "scaffold")]
pub use mock::{MockPostgresBackend, MockTxContext, ObservingPostgresAdapter, TxObservability};

/// 常用 re-export：行类型与参数 trait。
pub use tokio_postgres::{Row, types::ToSql};

#[cfg(test)]
mod unit_smoke {
    use super::*;

    #[test]
    fn public_types_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PostgresPool>();
        assert_send_sync::<PostgresConfig>();
        assert_send_sync::<PostgresConfigBuilder>();
        assert_send_sync::<PgTxRunner>();
        assert_send_sync::<PoolStats>();
        assert_send_sync::<SslMode>();
        assert_send_sync::<PgRepository>();
        assert_send_sync::<PgRecord>();
        assert_send_sync::<MakeRustlsConnect>();
    }

    /// 默认 feature crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        assert_eq!(DEFAULT_PORT, 5432);
        assert_eq!(DEFAULT_MAX_POOL_SIZE, 16);
        let _ = SslMode::Disable.as_str();
        let _ = SslMode::Prefer.as_str();
        let _ = SslMode::Require.as_str();

        fn assert_type<T: ?Sized>() {}
        assert_type::<PgConnection>();
        assert_type::<PgTransaction>();
        assert_type::<Row>();
        assert_type::<dyn ToSql>();
        assert_type::<PgRepository>();
        assert_type::<PgRecord>();
        let _ = map_pool_error;
        let _ = map_tokio_error;
        let _ = error_kind_from_sqlstate;
        let err = xerror_from_sqlstate("42P01", "missing");
        assert_eq!(err.kind(), kernel::ErrorKind::Missing);

        let tls = MakeRustlsConnect::with_webpki_roots().expect("tls");
        let _ = format!("{tls:?}");
        let _ = build_client_config().expect("cfg");

        let cfg = PgRetryConfig::fixed(1, 0);
        let v = with_retry_sync(&cfg, "surface", || Ok(1_i32)).expect("retry");
        assert_eq!(v, 1);
    }
}
