//! OSS v0.4.0 全 API E2E 测试（需真实凭证，默认 #[ignore]）。

use bytes::Bytes;
use contract_testkit::assert_object_store;
use contracts::ObjectStore;
use ossx::{DownloadOptions, ObjectKey, OssConfig, OssPool, UploadOptions};
use std::time::Duration;

fn pool() -> OssPool {
    OssPool::connect(OssConfig::from_env().expect("env")).expect("connect")
}

fn live_key(p: &str) -> String {
    format!(
        "infra-draft/{}-{}-{}",
        p,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    )
}

#[tokio::test]
#[ignore]
async fn live_put_get_delete() {
    let p = pool();
    let k = live_key("basic");
    let d = Bytes::from(format!("ossx-live-{}", std::process::id()));
    let s: &dyn ObjectStore = &p;
    let sr = assert_object_store(s, &k, d.clone()).await;
    sr.expect("suite");
    let g = s.get_object(&k).await.expect("get");
    assert_eq!(g, d);
    p.delete_object(&k).await.expect("del");
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_head_meta() {
    let p = pool();
    let k = live_key("head");
    p.put_object(&k, Bytes::from("hi")).await.expect("put");
    let m = p.head(&k).await.expect("head");
    assert!(m.size > 0);
    assert!(m.etag.is_some());
    p.delete_object(&k).await.ok();
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_head_missing() {
    let p = pool();
    assert!(p.head(&live_key("no")).await.is_err());
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_delete_idempotent() {
    let p = pool();
    let k = live_key("del");
    assert!(p.delete_object(&k).await.is_ok());
    assert!(p.delete_object(&k).await.is_ok());
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_stats_health() {
    let p = pool();
    let s = p.stats();
    assert!(!s.closed);
    assert!(s.max_in_flight > 0);
    let h = p.health(Duration::from_secs(10)).await;
    assert!(h.ready, "health: {}", h.detail);
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_close_rejects() {
    let p = pool();
    p.close(Duration::from_secs(5)).await.unwrap();
    assert!(p.stats().closed);
    assert!(p.put_object("k", Bytes::from("v")).await.is_err());
}

#[tokio::test]
#[ignore]
async fn live_large_object() {
    let p = pool();
    let k = live_key("large");
    let d = Bytes::from(vec![0xABu8; 1024 * 1024]);
    p.put_object(&k, d.clone()).await.expect("put");
    assert_eq!(p.get_object(&k).await.expect("get"), d);
    assert_eq!(p.head(&k).await.expect("head").size as usize, 1024 * 1024);
    p.delete_object(&k).await.ok();
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_empty_object() {
    let p = pool();
    let k = live_key("empty");
    p.put_object(&k, Bytes::new()).await.expect("put");
    assert_eq!(p.get_object(&k).await.expect("get").len(), 0);
    p.delete_object(&k).await.ok();
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_stream_upload_download() {
    let p = pool();
    let k = live_key("stream");
    let d = Bytes::from(vec![0xCDu8; 100 * 1024]);
    let bs = ossx::byte_stream_from_bytes(d.clone());
    p.put_stream(&k, bs, UploadOptions::default()).await.expect("put_stream");
    let (m, mut stream) = p.get_stream(&k, DownloadOptions::default()).await.expect("get_stream");
    assert!(m.etag.is_some());
    use futures_util::StreamExt;
    let mut c = Vec::new();
    while let Some(chunk) = stream.next().await {
        c.extend_from_slice(&chunk.expect("chunk"));
    }
    assert_eq!(Bytes::from(c), d);
    p.delete_object(&k).await.ok();
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_range_download() {
    let p = pool();
    let k = live_key("range");
    p.put_object(&k, Bytes::from(b"0123456789".to_vec())).await.expect("put");
    let (_, mut stream) =
        p.get_stream(&k, DownloadOptions::with_range("bytes=0-4")).await.expect("get");
    use futures_util::StreamExt;
    let mut c = Vec::new();
    while let Some(chunk) = stream.next().await {
        c.extend_from_slice(&chunk.expect("chunk"));
    }
    assert_eq!(Bytes::from(c), Bytes::from(b"01234".to_vec()));
    p.delete_object(&k).await.ok();
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_oversized_rejected() {
    let c = OssConfig::builder()
        .endpoint(&OssConfig::from_env().expect("env").endpoint)
        .bucket(&OssConfig::from_env().unwrap().bucket)
        .access_key_id(&OssConfig::from_env().unwrap().access_key_id)
        .access_key_secret(&OssConfig::from_env().unwrap().access_key_secret)
        .region(&OssConfig::from_env().unwrap().region)
        .max_object_bytes(10)
        .build()
        .expect("cfg");
    let p = OssPool::connect(c).expect("pool");
    let k = live_key("big");
    assert!(p.put_object(&k, Bytes::from(vec![0u8; 100])).await.is_err());
    p.close(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
#[ignore]
async fn live_concurrent_operations() {
    let p = pool();
    let n = 10;
    let mut handles = Vec::new();
    for i in 0..n {
        let p = p.clone();
        let k = format!(
            "infra-draft/concurrent-{i}-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        );
        let d = Bytes::from(format!("item-{i}"));
        handles.push(tokio::spawn(async move {
            p.put_object(&k, d.clone()).await?;
            assert_eq!(p.get_object(&k).await?, d);
            p.delete_object(&k).await?;
            Ok::<_, kernel::XError>(())
        }));
    }
    for h in handles {
        h.await.expect("join").expect("concurrent");
    }
    p.close(Duration::from_secs(5)).await.ok();
}
