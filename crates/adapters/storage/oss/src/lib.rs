//! `ossx` — 阿里云 OSS 对象存储适配。
//!
//! ## 生产入口
//!
//! - [`OssConfig`] / [`OssConfigBuilder`]：配置与 `FOUNDATIONX_OSSX_*` 环境变量
//! - [`OssClient`]：reqwest + OSS Signature V1；multipart；`resiliencx` 重试
//! - 实现 [`contracts::ObjectStore`]
//!
//! ## Scaffold
//!
//! feature `scaffold` 暴露进程内 `OssAdapter`（**非**生产）。

#![forbid(unsafe_code)]

mod client;
mod config;
mod retry;
mod sign;

pub use client::{
    MAX_MULTIPART_PART_BYTES, MAX_MULTIPART_PARTS, MAX_OBJECT_KEY_BYTES, MIN_MULTIPART_PART_BYTES,
    MultipartOrphanAudit, ORPHAN_AUDIT_CAPACITY, OssClient,
};
pub use config::{
    ENV_ACCESS_KEY_ID, ENV_ACCESS_KEY_SECRET, ENV_ACQUIRE_TIMEOUT_MS, ENV_BUCKET, ENV_ENDPOINT,
    ENV_MAX_BUFFER_BYTES, ENV_MAX_ERROR_BODY_BYTES, ENV_MAX_IN_FLIGHT, ENV_MAX_OBJECT_BYTES,
    ENV_OPERATION_DEADLINE_MS, ENV_REGION, ENV_REQUEST_TIMEOUT_MS, HARD_MAX_BUFFER_BYTES,
    HARD_MAX_ERROR_BODY_BYTES, HARD_MAX_IN_FLIGHT, HARD_MAX_OBJECT_BYTES, OssConfig,
    OssConfigBuilder,
};
pub use retry::{
    MAX_RETRY_ATTEMPTS, default_retry_config, is_oss_retryable, with_retry, with_retry_deadline,
    with_retry_default,
};
pub use sign::{
    authorization_header, canonicalized_resource, canonicalized_resource_with_subresources,
    sign_v1, split_parts,
};

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

        let multi = canonicalized_resource_with_subresources("b", "k", &[("uploads", None)]);
        assert!(multi.ends_with("?uploads"));
        assert!(split_parts(b"abc", 2).len() == 2);

        let _ = default_retry_config();
        assert!(is_oss_retryable(&kernel::XError::transient("t")));

        fn assert_type<T: ?Sized>() {}
        assert_type::<OssClient>();
        assert_type::<MultipartOrphanAudit>();
        assert_type::<OssConfig>();
        assert_type::<OssConfigBuilder>();
    }
}
