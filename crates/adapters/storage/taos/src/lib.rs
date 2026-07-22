//! `taosx` — TDengine 时序存储适配器。
//!
//! - **默认**：[`TaosPool`] / [`TaosClient`] REST 生产客户端（端口 6041）。
//! - **Native WS**：`TransportMode::NativeWs` + native 连通性探测。
//! - **feature `scaffold`**：`TaosAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::TimeSeriesStore`]（`Tick.ts` 为纳秒 epoch）。

#![forbid(unsafe_code)]

mod client;
mod config;
mod native;

pub use client::{TaosClient, TaosExecResult, TaosPool, TaosPoolStats, build_insert_sql_chunks};
pub use config::{
    HARD_MAX_BATCH_BYTES, HARD_MAX_BATCH_ROWS, HARD_MAX_CLOSE_TIMEOUT, HARD_MAX_IN_FLIGHT,
    HARD_MAX_QUERY_ROWS, HARD_MAX_RESPONSE_BYTES, TaosConfig, TransportMode, TsPrecision,
};
pub use native::{build_native_ws_url, connect_native_ws, validate_mode};

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::TaosAdapter;

#[cfg(test)]
mod public_api_surface {
    use super::*;

    /// 默认 feature crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        let _cfg = TaosConfig::default();
        let _ = TsPrecision::Ms;
        let _ = TransportMode::Rest;
        let result = TaosExecResult {
            code: 0,
            rows: vec![vec!["1".into()]],
            columns: vec!["n".into()],
            affected_rows: Some(1),
        };
        assert_eq!(result.code, 0);
        assert_eq!(build_native_ws_url(&_cfg), "ws://127.0.0.1:6041/rest/ws");
        validate_mode(&_cfg).expect("mode");
        let empty = build_insert_sql_chunks("t", &[], TsPrecision::Ms, 10).unwrap();
        assert!(empty.is_empty());
        fn assert_type<T: ?Sized>() {}
        assert_type::<TaosClient>();
        assert_type::<TaosPool>();
        assert_type::<TaosConfig>();
        assert_type::<TaosExecResult>();
        assert_type::<TaosPoolStats>();
        assert_type::<TransportMode>();
    }
}
