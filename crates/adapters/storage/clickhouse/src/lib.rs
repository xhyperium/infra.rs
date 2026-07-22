//! `clickhousex` — ClickHouse 分析汇聚适配器。
//!
//! - **默认**：[`ClickHousePool`] / [`ClickHouseClient`] HTTP 生产客户端（端口 8123）。
//! - **feature `scaffold`**：`ClickHouseAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::AnalyticsSink`]。

#![forbid(unsafe_code)]

mod client;
mod config;

pub use client::{ANALYTICS_TABLE, ClickHouseClient, ClickHousePool};
pub use config::ClickHouseConfig;

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::ClickHouseAdapter;

#[cfg(test)]
mod public_api_surface {
    use super::*;

    /// 默认 feature crate-root 导出均被单元测试点名。
    #[test]
    fn default_exports_named() {
        assert!(!ANALYTICS_TABLE.is_empty());
        let _cfg = ClickHouseConfig::default();
        fn assert_type<T: ?Sized>() {}
        assert_type::<ClickHouseClient>();
        assert_type::<ClickHousePool>();
        assert_type::<ClickHouseConfig>();
    }
}
