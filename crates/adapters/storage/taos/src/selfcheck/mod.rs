//! taosx 自验证（对齐 `.cargo/draft/verifyctl.md` / LIB-SELFCHECK-SPEC §6.7）。
//!
//! ## 与 `tools/verifyctl` 的边界
//!
//! | 组件 | 职责 |
//! |------|------|
//! | **本模块 `taosx::selfcheck`** | 中间件依赖可用性 / 功能闭环（P1–P4 运行时自检） |
//! | **`tools/verifyctl`** | Goal Contract → 变更验证计划/执行（代码变更门禁） |
//!
//! 二者同名「验证」但 **不是同一系统**；本实现只覆盖规范中的 **taos 检查项目录**
//!（module 段为 `taos`，crate 为 `taosx`）。
//!
//! ## 快速使用
//!
//! ```ignore
//! use taosx::selfcheck::{CheckLevel, TaosValidator, Validatable};
//!
//! let pool = taosx::TaosPool::connect_from_env().await?;
//! let report = TaosValidator::new(pool).run(CheckLevel::ReadWrite).await;
//! assert!(report.passed);
//! ```
//!
//! ## 诚实边界
//!
//! - 需要可达 TDengine REST；`connect_without_ping` + 不可达地址用于短路/catalog 单测
//! - `tmq_subscribe` 当前 **Skipped**（本 crate 未实现 TMQ）
//! - 自检超级表命名 `_sc_{token}`，运行结束主动 `DROP STABLE`
//! - **未**实现跨模块 `SelfValidator` 调度器 / HTTP 探针 / Prometheus 导出器

mod config;
mod context;
mod types;
mod validator;

pub use config::TaosSelfCheckConfig;
pub use context::{CancelFlag, ValidationContext};
pub use types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};
pub use validator::{MODULE, TaosValidator, Validatable};
