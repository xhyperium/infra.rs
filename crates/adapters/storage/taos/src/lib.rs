//! `taosx` — TDengine 时序存储适配器。
//!
//! - **默认**：[`TaosPool`] / [`TaosClient`] REST 生产客户端（端口 6041）。
//! - **feature `scaffold`**：`TaosAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::TimeSeriesStore`]（`Tick.ts` 为纳秒 epoch）。

#![forbid(unsafe_code)]

mod client;
mod config;

pub use client::{TaosClient, TaosExecResult, TaosPool};
pub use config::{TaosConfig, TsPrecision};

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
        let result = TaosExecResult {
            code: 0,
            rows: vec![vec!["1".into()]],
            columns: vec!["n".into()],
            affected_rows: Some(1),
        };
        assert_eq!(result.code, 0);
        fn assert_type<T: ?Sized>() {}
        assert_type::<TaosClient>();
        assert_type::<TaosPool>();
        assert_type::<TaosConfig>();
        assert_type::<TaosExecResult>();
    }
}
