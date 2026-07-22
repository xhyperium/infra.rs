//! 真实 OSS live 测：put / get / delete。
//!
//! 需要环境变量（见 `scripts/live/export-foundationx-env.sh`）：
//! - `FOUNDATIONX_OSSX_ENDPOINT`
//! - `FOUNDATIONX_OSSX_BUCKET`
//! - `FOUNDATIONX_OSSX_ACCESS_KEY_ID`
//! - `FOUNDATIONX_OSSX_ACCESS_KEY_SECRET`
//! - `FOUNDATIONX_OSSX_REGION`（可选）
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo test -p ossx --test live_object_store -- --ignored
//! ```
//!
//! 网络/鉴权失败时**如实** `Err`，不 mock 通过。

use bytes::Bytes;
use contract_testkit::assert_object_store;
use contracts::ObjectStore;
use ossx::OssClient;

#[tokio::test]
#[ignore = "requires live Aliyun OSS credentials (FOUNDATIONX_OSSX_*)"]
async fn live_put_get_delete_under_infra_draft() {
    let client = match OssClient::from_env() {
        Ok(c) => c,
        Err(e) => panic!("oss from_env failed (missing/invalid env): {e}"),
    };

    let key = format!(
        "infra-draft/live-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let payload = Bytes::from(format!("ossx-live-{}", std::process::id()));

    // ObjectStore trait 面
    let store: &dyn ObjectStore = &client;
    let suite_result = assert_object_store(store, &key, payload.clone()).await;
    let get_result = store.get_object(&key).await;

    // 无论 suite/get 成败都先执行清理，避免失败路径遗留计费对象。
    let cleanup_result = client.delete_object(&key).await;
    client.close();

    suite_result.expect("可移植 ObjectStore suite");
    let got = get_result.expect("get_object failed");
    assert_eq!(got, payload, "round-trip bytes mismatch");
    cleanup_result.expect("delete_object cleanup failed");
}
