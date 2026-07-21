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
//! source scripts/live/export-foundationx-env.sh /path/to/oss.env
//! cargo test -p ossx --test live_object_store -- --ignored --nocapture
//! ```
//!
//! 网络/鉴权失败时**如实** `Err`，不 mock 通过。

use bytes::Bytes;
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
    if let Err(e) = store.put_object(&key, payload.clone()).await {
        panic!("put_object failed (auth/network/config): {e}");
    }
    let got = match store.get_object(&key).await {
        Ok(b) => b,
        Err(e) => panic!("get_object failed: {e}"),
    };
    assert_eq!(got, payload, "round-trip bytes mismatch");

    // 客户端扩展：delete 清理
    if let Err(e) = client.delete_object(&key).await {
        panic!("delete_object cleanup failed: {e}");
    }
    client.close();
}
