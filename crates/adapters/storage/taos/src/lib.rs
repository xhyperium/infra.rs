//! `taosx` — TDengine 时序存储适配器。
//!
//! - **默认**：[`TaosPool`] / [`TaosClient`] REST 生产客户端（端口 6041）。
//! - **Native WS**：握手探测 + [`exec_sql_ws`] 短会话 SQL；[`probe_native_tcp`] 原生端口探测。
//! - **流/批/重试/TMQ/soak**：[`TaosQueryStream`] / [`WriteBatcher`] / [`RetryPolicy`] / [`TmqConsumer`] / [`run_soak`]。
//! - **自验证**：[`selfcheck`]（LIB-SELFCHECK-SPEC §6.7）。
//! - **feature `scaffold`**：`TaosAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::TimeSeriesStore`]（`Tick.ts` 为纳秒 epoch）。

#![forbid(unsafe_code)]

mod batcher;
mod client;
mod config;
mod metrics;
mod native;
mod retry;
mod soak;
mod stream;
mod tmq;

pub mod selfcheck;

pub use batcher::{WriteBatcher, WriteBatcherConfig};
pub use client::{
    BatchWritePartialError, BatchWriteReport, TaosClient, TaosExecResult, TaosHealth, TaosPool,
    TaosPoolStats, build_insert_sql_chunks,
};
pub use config::{
    HARD_MAX_BATCH_BYTES, HARD_MAX_BATCH_ROWS, HARD_MAX_CLOSE_TIMEOUT, HARD_MAX_IN_FLIGHT,
    HARD_MAX_QUERY_ROWS, HARD_MAX_RESPONSE_BYTES, TaosConfig, TransportMode, TsPrecision,
};
pub use metrics::{TaosMetricsSnapshot, ws_probe_totals};
pub use native::{
    build_native_ws_url, connect_native_ws, exec_sql_ws, probe_native_tcp, validate_mode,
};
pub use retry::RetryPolicy;
pub use soak::{SoakConfig, SoakReport, run_soak};
pub use stream::TaosQueryStream;
pub use tmq::TmqConsumer;

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::TaosAdapter;

#[cfg(test)]
mod public_api_surface {
    use super::*;
    use std::time::Duration;

    /// 默认 feature crate-root 导出：类型、常量、自由函数均被点名。
    #[test]
    fn default_exports_named() {
        let cfg = TaosConfig::default();
        let _ = TsPrecision::Ms;
        let _ = TsPrecision::Us;
        let _ = TsPrecision::Ns;
        let _ = TransportMode::Rest;
        let _ = TransportMode::NativeWs;
        let result = TaosExecResult {
            code: 0,
            rows: vec![vec!["1".into()]],
            columns: vec!["n".into()],
            affected_rows: Some(1),
        };
        assert_eq!(result.code, 0);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(build_native_ws_url(&cfg), "ws://127.0.0.1:6041/rest/ws");
        validate_mode(&cfg).expect("mode");
        let empty = build_insert_sql_chunks("t", &[], TsPrecision::Ms, 10).unwrap();
        assert!(empty.is_empty());

        // 硬上限常量被引用（防止死导出）；运行时求和避免 clippy assertions_on_constants
        let hard_sum = HARD_MAX_IN_FLIGHT
            + HARD_MAX_BATCH_BYTES
            + HARD_MAX_BATCH_ROWS
            + HARD_MAX_RESPONSE_BYTES
            + HARD_MAX_QUERY_ROWS
            + HARD_MAX_CLOSE_TIMEOUT.as_millis() as usize;
        assert!(hard_sum > 1_000);

        fn assert_type<T: ?Sized>() {}
        assert_type::<TaosClient>();
        assert_type::<TaosPool>();
        assert_type::<TaosConfig>();
        assert_type::<TaosExecResult>();
        assert_type::<TaosPoolStats>();
        assert_type::<TransportMode>();
        assert_type::<TsPrecision>();
    }

    /// `TaosConfig` 公开方法与 URL 构造；Debug 脱敏必须遮住注入的假密码。
    #[test]
    fn config_public_methods_exercised() {
        let cfg = TaosConfig { password: "fake-pass-value-42".into(), ..TaosConfig::default() };
        assert_eq!(cfg.rest_sql_url(), "http://127.0.0.1:6041/rest/sql");
        assert!(cfg.rest_sql_db_url().contains("/rest/sql/"));
        assert_eq!(cfg.native_ws_url(), "ws://127.0.0.1:6041/rest/ws");
        cfg.validate().expect("default validate");

        assert_eq!(TsPrecision::parse("ms"), Some(TsPrecision::Ms));
        assert_eq!(TsPrecision::parse("us"), Some(TsPrecision::Us));
        assert_eq!(TsPrecision::parse("ns"), Some(TsPrecision::Ns));
        assert_eq!(TsPrecision::Ms.from_nanos(1_000_000), 1);
        assert_eq!(TsPrecision::Ms.to_nanos(1), 1_000_000);
        assert_eq!(TransportMode::parse("rest"), Some(TransportMode::Rest));
        assert_eq!(TransportMode::parse("ws"), Some(TransportMode::NativeWs));

        let debug = format!("{cfg:?}");
        assert!(debug.contains("***"), "Debug 必须脱敏: {debug}");
        assert!(!debug.contains("fake-pass-value-42"), "明文密码不得出现在 Debug: {debug}");

        let report = BatchWriteReport { accepted: 3, failed: 1, chunks_ok: 1, chunks_total: 2 };
        assert!(!report.is_complete());
        assert_eq!(report.accepted + report.failed, 4);
        fn assert_type<T: ?Sized>() {}
        assert_type::<BatchWriteReport>();
        assert_type::<BatchWritePartialError>();
        assert_type::<TaosMetricsSnapshot>();
        assert_type::<TaosHealth>();
        let _ = ws_probe_totals();
        assert_type::<selfcheck::TaosValidator>();
        assert_type::<selfcheck::ValidationReport>();
        assert_eq!(selfcheck::MODULE, "taos");
        assert_eq!(selfcheck::TaosValidator::static_catalog().len(), 9);
        assert_type::<WriteBatcher>();
        assert_type::<WriteBatcherConfig>();
        assert_type::<TaosQueryStream>();
        assert_type::<RetryPolicy>();
        assert_type::<TmqConsumer>();
        assert_type::<SoakConfig>();
        assert_type::<SoakReport>();
        let _ = RetryPolicy::for_read();
        let _ = RetryPolicy::for_idempotent_write();
        let text = TaosMetricsSnapshot::default().to_prometheus_text();
        assert!(text.contains("taosx_ops_total"));
        let _ = exec_sql_ws;
        let _ = probe_native_tcp;
        let _ = run_soak;
    }

    /// `from_env` 在无变量时回到默认，且不 panic。
    #[test]
    fn config_from_env_defaults_when_unset() {
        // 不注入密钥；仅验证 API 可调用且默认值合法。
        let cfg = TaosConfig::from_env();
        assert!(!cfg.host.is_empty());
        assert_eq!(cfg.port, 6041);
    }

    /// 池公开同步面：`connect_without_ping` / `client` / `config` / `precision` / `stats` / `is_closed`。
    #[test]
    fn pool_sync_surface_methods() {
        let cfg = TaosConfig::default();
        let pool = TaosPool::connect_without_ping(cfg).expect("offline build");
        let client: TaosClient = pool.client();
        assert_eq!(client.config().host, "127.0.0.1");
        assert_eq!(pool.precision(), TsPrecision::Ms);
        let stats = pool.stats();
        assert_eq!(stats.in_flight, 0);
        assert!(!stats.closed);
        assert!(!pool.is_closed());
    }

    /// crate-root 导出的 `connect_native_ws` 必须被点名（Rest 模式应 Invalid）。
    #[tokio::test]
    async fn connect_native_ws_export_is_exercised() {
        let cfg = TaosConfig {
            transport: TransportMode::Rest,
            timeout: Duration::from_millis(100),
            ..TaosConfig::default()
        };
        let err = connect_native_ws(&cfg).await.expect_err("rest mode must fail");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }
}
