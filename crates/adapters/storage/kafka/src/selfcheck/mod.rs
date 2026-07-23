//! kafkax 自验证（对齐 `.cargo/draft/verifyctl.md` / LIB-SELFCHECK-SPEC §6.2）。
//!
//! ## 与 `tools/verifyctl` 的边界
//!
//! | 组件 | 职责 |
//! |------|------|
//! | **本模块 `kafkax::selfcheck`** | 中间件依赖可用性 / 功能闭环（P1–P4 运行时自检） |
//! | **`tools/verifyctl`** | Goal Contract → 变更验证计划/执行（代码变更门禁） |
//!
//! 二者同名「验证」但 **不是同一系统**；本实现只覆盖规范中的 **kafka 检查项目录**。
//!
//! ## 快速使用
//!
//! ```ignore
//! use kafkax::selfcheck::{CheckLevel, KafkaValidator, Validatable};
//!
//! let pool = kafkax::KafkaPool::connect_from_env().await?;
//! let report = KafkaValidator::new(pool).run(CheckLevel::ReadWrite).await;
//! assert!(report.passed);
//! ```
//!
//! ## 诚实边界
//!
//! - 需要可达 Kafka broker；`connect_and_run` 在连接失败时返回合成 Failed/Skipped 报告（不 panic）
//! - `group_lag` / `isr_health` 在 rskafka 默认栈上 **Skipped（NO-GO）**
//! - `offset_commit` 验证的是 **应用层** [`crate::OffsetCommitStore`] 语义，非 broker group commit
//! - `ordering_headers`：同分区顺序可测；公共 `KafkaMessage` **无 headers 面**（detail 标明 partial）
//! - **未**实现跨模块 `SelfValidator` 调度器 / HTTP 探针 / Prometheus 导出器
//! - **未**宣称 package stable

mod config;
mod context;
mod types;
mod validator;

pub use config::KafkaSelfCheckConfig;
pub use context::{CancelFlag, ValidationContext};
pub use types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};
pub use validator::{KafkaValidator, MODULE, Validatable};
