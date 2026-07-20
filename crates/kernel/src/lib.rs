#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # `kernel` — xhyper.rs L0 语义信任根
//!
//! `kernel` 定义全系统必须唯一且长期稳定的三类语义：
//!
//! 1. **错误分类与响应**（[`error`]）—— 按"调用方应如何反应"分类，不按模块来源分类；
//! 2. **时间获取与表示**（[`clock`]）—— 墙钟与单调钟分离，时间源必须显式注入；
//! 3. **生命周期与关停信号**（[`lifecycle`]）—— 关停一次触发、多方观察、不可逆。
//!
//! `kernel` 不提供配置、日志、网络、异步运行时、依赖注入、持久化或业务能力。
//! 任何新增公开项、依赖或 feature 都必须走 RFC。

pub mod clock;
pub mod error;
pub mod lifecycle;

pub use clock::{Clock, ClockError, MonotonicInstant, SystemClock, Timestamp};
pub use error::{BoxError, ErrorKind, XError, XResult};
pub use lifecycle::{ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal};
