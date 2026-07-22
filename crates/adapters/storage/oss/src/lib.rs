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

#[cfg(test)]
mod public_api_surface {
    use super::*;

    /// 默认 feature crate-root 导出均被单元测试点名（含 ENV_* / Config）。
    #[test]
    fn default_exports_named() {
        assert_eq!(ENV_ENDPOINT, "FOUNDATIONX_OSSX_ENDPOINT");
        assert_eq!(ENV_BUCKET, "FOUNDATIONX_OSSX_BUCKET");
        assert_eq!(ENV_ACCESS_KEY_ID, "FOUNDATIONX_OSSX_ACCESS_KEY_ID");
        assert_eq!(ENV_ACCESS_KEY_SECRET, "FOUNDATIONX_OSSX_ACCESS_KEY_SECRET");
        assert_eq!(ENV_REGION, "FOUNDATIONX_OSSX_REGION");

        let builder: OssConfigBuilder = OssConfig::builder();
        let cfg: OssConfig = builder
            .endpoint("https://oss.example.com")
            .bucket("b")
            .access_key_id("id")
            .access_key_secret("sec")
            .region("r")
            .build()
            .expect("cfg");
        assert_eq!(cfg.bucket, "b");

        let resource = canonicalized_resource("b", "/k");
        let sig = sign_v1("sec", "GET", "", "", "date", "", &resource);
        let _ = authorization_header("id", &sig);

        fn assert_type<T: ?Sized>() {}
        assert_type::<OssClient>();
        assert_type::<OssConfig>();
        assert_type::<OssConfigBuilder>();
    }
}
