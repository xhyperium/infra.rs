//! e2e：key / headers / stats / FileOffset 真实路径（默认 ignore）。
//!
//! ```bash
//! # 隔离或 secrets 注入 FOUNDATIONX_KAFKAX_* 后：
//! cargo test -p kafkax --test e2e_api_roundtrip -- --ignored --nocapture
//! ```
//!
//! FileOffset 落盘：`/home/workspace/data/kafkax-gap-zero-*`。

use std::path::PathBuf;
use std::time::Duration;

use bytes::Bytes;
use kafkax::selfcheck::{CheckLevel, CheckStatus, KafkaValidator, Validatable};
use kafkax::{
    ConsumerConfig, FileOffsetStore, KafkaPool, OffsetCommitStore, PublishRecord,
    partition_for_key, resolve_start_offset,
};

fn data_root() -> PathBuf {
    let stamp = format!(
        "kafkax-gap-zero-e2e-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let preferred = PathBuf::from("/home/workspace/data").join(&stamp);
    if std::fs::create_dir_all(&preferred).is_ok() {
        return preferred;
    }
    let fallback = std::env::temp_dir().join(&stamp);
    std::fs::create_dir_all(&fallback).expect("create temp data dir");
    fallback
}

#[tokio::test]
#[ignore = "requires live Kafka (FOUNDATIONX_KAFKAX_* or isolation broker)"]
async fn e2e_key_headers_stats_and_file_offset() {
    let pool = KafkaPool::connect_from_env().await.expect("connect");
    let topic = format!(
        "_kafkax_e2e_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    pool.ensure_topic(&topic, 3, 1).await.expect("ensure");

    let key = Bytes::from_static(b"e2e-key");
    let part = partition_for_key(key.as_ref(), 3);
    let payload = Bytes::from_static(b"e2e-payload-with-headers");
    let rec = PublishRecord::payload(&topic, part, payload.clone())
        .with_key(key.clone())
        .header("x-selfcheck", Bytes::from_static(b"1"))
        .header("x-trace", Bytes::from_static(b"abc"));
    let d = pool.producer().publish_record(rec).await.expect("publish");
    assert_eq!(d.partition, part);

    let mut c = pool
        .consumer(ConsumerConfig::assign(&topic, part, "e2e-g").with_start_offset(d.offset))
        .await
        .expect("consumer");
    let msg = c.recv_timeout(Duration::from_secs(10)).await.expect("recv").expect("some");
    assert_eq!(msg.payload, payload);
    assert_eq!(msg.key.as_ref().map(|k| k.as_ref()), Some(key.as_ref()));
    assert_eq!(msg.header("x-selfcheck").map(|v| v.as_ref()), Some(&b"1"[..]));
    assert_eq!(msg.header("x-trace").map(|v| v.as_ref()), Some(&b"abc"[..]));

    let stats = pool.stats();
    assert!(stats.published >= 1);
    assert!(stats.topics_ensured >= 1);

    let dir = data_root();
    let store = FileOffsetStore::new(dir.join("off.tsv"));
    store.commit(&topic, part, msg.offset).await.expect("commit");
    assert_eq!(
        resolve_start_offset(&store, &topic, part).await.expect("start"),
        Some(msg.offset + 1)
    );

    let _ = pool.delete_topic(&topic).await;
    let st2 = pool.stats();
    assert!(st2.topics_deleted >= 1);
    let _ = pool.close(Duration::from_secs(3)).await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
#[ignore = "requires live Kafka"]
async fn e2e_selfcheck_full_headers_pass() {
    let pool = KafkaPool::connect_from_env().await.expect("connect");
    let report = KafkaValidator::new(pool.clone()).run(CheckLevel::Full).await;
    assert!(report.passed, "items={:?}", report.items);
    let ord = report.items.iter().find(|i| i.id == "kafka.full.ordering_headers").expect("ord");
    assert!(matches!(ord.status, CheckStatus::Passed | CheckStatus::Degraded), "{ord:?}");
    assert!(
        ord.detail.as_ref().is_none_or(|d| !d.contains("partial") || d.contains("延迟")),
        "headers should not be partial: {ord:?}"
    );
    for id in ["kafka.full.group_lag", "kafka.full.isr_health"] {
        let i = report.items.iter().find(|x| x.id == id).expect(id);
        assert_eq!(i.status, CheckStatus::Skipped);
        assert!(i.detail.as_ref().is_some_and(|d| d.contains("NO-GO")));
    }
    assert_eq!(KafkaValidator::new(pool.clone()).catalog().len(), 9);
    let _ = pool.close(Duration::from_secs(3)).await;
}
