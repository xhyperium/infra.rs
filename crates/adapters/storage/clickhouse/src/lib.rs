//! `clickhousex` — ClickHouse 分析汇聚适配器。
//!
//! - **默认**：[`ClickHousePool`] / [`ClickHouseClient`] HTTP 生产客户端（端口 8123）。
//! - **feature `scaffold`**：`ClickHouseAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::AnalyticsSink`]。
//!
//! # 配置校验（离线）
//!
//! ```
//! use std::time::Duration;
//! use clickhousex::ClickHouseConfig;
//!
//! let cfg = ClickHouseConfig::default();
//! cfg.validate().expect("默认配置应通过校验");
//!
//! // 非法配置应被拒绝
//! let bad = ClickHouseConfig { max_in_flight: 0, ..Default::default() };
//! assert!(bad.validate().is_err());
//! ```
//!
//! # 标识符校验
//!
//! ```
//! use clickhousex::validate_ident;
//!
//! assert!(validate_ident("valid_table").is_ok());
//! assert!(validate_ident("").is_err());
//! assert!(validate_ident("1bad").is_err());
//! assert!(validate_ident("a;drop").is_err());
//! ```
//!
//! # 分块计算
//!
//! ```
//! use clickhousex::chunk_ranges;
//!
//! assert_eq!(chunk_ranges(5, 2), vec![(0, 2), (2, 4), (4, 5)]);
//! assert!(chunk_ranges(0, 10).is_empty());
//! ```
//!
//! # TabSeparated 行解析
//!
//! ```
//! use clickhousex::parse_tab_separated_rows;
//!
//! let rows = parse_tab_separated_rows("a\tb\nc\td\n");
//! assert_eq!(rows, vec![vec!["a", "b"], vec!["c", "d"]]);
//!
//! assert!(parse_tab_separated_rows("").is_empty());
//! ```

#![forbid(unsafe_code)]

mod client;
mod config;

pub use client::{
    ANALYTICS_TABLE, BatchInsertOptions, ClickHouseClient, ClickHousePool, ClickHousePoolStats,
    chunk_ranges, parse_tab_separated_rows, validate_ident,
};
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
        let _ = BatchInsertOptions::default();
        let ranges = chunk_ranges(5, 2);
        assert_eq!(ranges.len(), 3);
        let rows = parse_tab_separated_rows("a\tb\n");
        assert_eq!(rows, vec![vec!["a".to_string(), "b".to_string()]]);
        fn assert_type<T: ?Sized>() {}
        assert_type::<ClickHouseClient>();
        assert_type::<ClickHousePool>();
        assert_type::<ClickHouseConfig>();
        assert_type::<ClickHousePoolStats>();
        assert_type::<BatchInsertOptions>();
    }
}
