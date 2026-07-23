//! `oss-verify` — OSS 独立自验证工具。
//!
//! 提供 6 层 24 项验证检查，覆盖配置、连接、基本操作、流式、高级功能、安全并发。
//!
//! ## CLI
//!
//! ```bash
//! oss-verify run    # 执行所有验证检查
//! oss-verify run -l 0,1,2  # 仅执行指定层
//! oss-verify list   # 列出所有检查
//! ```

pub mod suite;
pub mod types;

pub use suite::{all_check_specs, execute_check, aggregate_results};
pub use types::*;
