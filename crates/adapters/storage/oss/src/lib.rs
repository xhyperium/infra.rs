//! `ossx` — 阿里云 OSS 对象存储适配。
//!
//! ## 生产入口
//!
//! - [`OssConfig`] / [`OssConfigBuilder`]：配置与 `FOUNDATIONX_OSSX_*` 环境变量
//! - [`OssClient`]：reqwest + OSS Signature V1；实现 [`contracts::ObjectStore`]
//!
//! ## Scaffold
//!
//! feature `scaffold` 暴露进程内 [`OssAdapter`]（**非**生产）。

#![forbid(unsafe_code)]

mod client;
mod config;
mod sign;

pub use client::OssClient;
pub use config::{
    ENV_ACCESS_KEY_ID, ENV_ACCESS_KEY_SECRET, ENV_BUCKET, ENV_ENDPOINT, ENV_REGION, OssConfig,
    OssConfigBuilder,
};
pub use sign::{authorization_header, canonicalized_resource, sign_v1};

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::OssAdapter;
