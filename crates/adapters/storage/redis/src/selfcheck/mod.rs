//! redisx 自验证（对齐 `.cargo/draft/verifyctl.md` / LIB-SELFCHECK-SPEC §6.5）。
//!
//! ## 与 `tools/verifyctl` 的边界
//!
//! | 组件 | 职责 |
//! |------|------|
//! | **本模块 `redisx::selfcheck`** | 中间件依赖可用性 / 功能闭环（P1–P4 运行时自检） |
//! | **`tools/verifyctl`** | Goal Contract → 变更验证计划/执行（代码变更门禁） |
//!
//! 二者同名「验证」但 **不是同一系统**；本实现只覆盖规范中的 **redisx 检查项目录**。
//!
//! ## 快速使用
//!
//! ```ignore
//! use redisx::selfcheck::{CheckLevel, RedisValidator, Validatable};
//!
//! let client = redisx::RedisClient::connect_from_env().await?;
//! let report = RedisValidator::new(client).run(CheckLevel::ReadWrite).await;
//! assert!(report.passed);
//! ```
//!
//! ## 诚实边界
//!
//! - 需要可达 Redis；离线 probe 路径用于短路/catalog 测试
//! - `cluster_slots` 在非 Cluster 拓扑为 `Skipped`
//! - `pubsub` 检查需 feature `pubsub`
//! - **未**实现跨模块 `SelfValidator` 调度器 / HTTP 探针 / Prometheus 导出器
//! - 提供 `ValidationReport::to_json_string` / `RedisValidator::run_json` 作为 §10 JSON 通道

mod config;
mod context;
mod types;
mod validator;

pub use config::RedisSelfCheckConfig;
pub use context::{CancelFlag, ValidationContext};
pub use types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};
pub use validator::{MODULE, RedisValidator, Validatable};
