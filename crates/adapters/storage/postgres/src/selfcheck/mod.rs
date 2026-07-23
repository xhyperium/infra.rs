//! postgresx 自验证（对齐 `.cargo/draft/verifyctl.md` / LIB-SELFCHECK-SPEC §6.1）。
//!
//! ## 与 `tools/verifyctl` 的边界
//!
//! | 组件 | 职责 |
//! |------|------|
//! | **本模块 `postgresx::selfcheck`** | 中间件依赖可用性 / 功能闭环（P1–P4 运行时自检） |
//! | **`tools/verifyctl`** | Goal Contract → 变更验证计划/执行（代码变更门禁） |
//!
//! 二者同名「验证」但 **不是同一系统**；本实现只覆盖规范中的 **postgres 检查项目录**
//!（module 段为 `postgres`，crate 为 `postgresx`）。
//!
//! ## 快速使用
//!
//! ```ignore
//! use postgresx::selfcheck::{CheckLevel, PostgresValidator, Validatable};
//!
//! let pool = postgresx::PostgresPool::connect_from_env().await?;
//! let report = PostgresValidator::new(pool).run(CheckLevel::ReadWrite).await;
//! assert!(report.passed);
//! ```
//!
//! ## 诚实边界
//!
//! - 需要可达 Postgres；`connect_lazy` + 不可达地址用于短路/catalog 单测
//! - `replication_lag` 在 `replica_check=false` 或无副本时为 `Skipped`
//! - 自检表命名 `_self_check_{token}`，运行结束主动 DROP
//! - **未**实现跨模块 `SelfValidator` 调度器 / HTTP 探针 / Prometheus 导出器

mod config;
mod context;
mod types;
mod validator;

pub use config::PostgresSelfCheckConfig;
pub use context::{CancelFlag, ValidationContext};
pub use types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};
pub use validator::{MODULE, PostgresValidator, Validatable};
