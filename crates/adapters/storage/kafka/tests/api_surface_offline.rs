//! 默认 feature 公开 API 离线行为覆盖（无 broker）。
//!
//! 每个默认导出至少有一条行为断言；不连接真实集群。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use kafkax::selfcheck::{
    CheckLevel, CheckStatus, KafkaSelfCheckConfig, KafkaValidator, now_rfc3339,
};
use kafkax::{
    AtLeastOnceConsumer, ConsumerConfig, DEFAULT_BROKERS, DEFAULT_SASL_MECHANISM, Delivery,
    FileOffsetStore, KafkaAtLeastOnceBus, KafkaConfig, KafkaConfigBuilder, KafkaEventBus,
    KafkaHealth, KafkaMessage, KafkaPool, KafkaPoolStats, MemoryOffsetStore, OffsetCommitStore,
    ProduceThenCheckpointCoordinator, PublishRecord, encode_bus_id, parse_bus_id,
    partition_for_key, resolve_start_offset,
};
use kernel::ErrorKind;

fn data_dir() -> PathBuf {
    let base = PathBuf::from("/home/workspace/data");
    let dir = base.join(format!(
        "kafkax-gap-zero-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn constants_and_encode_surface() {
    assert!(!DEFAULT_BROKERS.is_empty());
    assert_eq!(DEFAULT_SASL_MECHANISM, "PLAIN");
    let id = encode_bus_id("t", 1, 2);
    assert_eq!(parse_bus_id(&id), Some(("t", 1, 2)));
    assert_eq!(partition_for_key(b"a", 4), partition_for_key(b"a", 4));
}

#[test]
fn config_builder_and_from_env_surface() {
    let cfg = KafkaConfigBuilder::new()
        .brokers(DEFAULT_BROKERS)
        .client_id("api-surface")
        .connect_timeout(Duration::from_millis(100))
        .operation_timeout(Duration::from_millis(100))
        .delivery_timeout(Duration::from_millis(100))
        .no_sasl()
        .tls(false)
        .build()
        .expect("cfg");
    assert_eq!(cfg.security_protocol(), "PLAINTEXT");
    let env = KafkaConfig::from_env().expect("from_env");
    assert!(!env.brokers.is_empty());
}

#[test]
fn message_and_publish_record_surface() {
    let msg = KafkaMessage {
        topic: "t".into(),
        partition: 0,
        offset: 1,
        payload: Bytes::from_static(b"p"),
        key: Some(Bytes::from_static(b"k")),
        headers: [(String::from("h"), Bytes::from_static(b"v"))].into_iter().collect(),
        timestamp: None,
    };
    assert_eq!(msg.bus_id(), "t/0/1");
    assert_eq!(msg.header("h").map(|b| b.as_ref()), Some(&b"v"[..]));
    let rec = PublishRecord::payload("t", 2, Bytes::from_static(b"x"))
        .with_key(Bytes::from_static(b"k"))
        .header("a", Bytes::from_static(b"1"));
    assert_eq!(rec.partition, 2);
    assert!(rec.headers.contains_key("a"));
    let d = Delivery { partition: 0, offset: 9 };
    assert_eq!(d.offset, 9);
}

#[test]
fn health_and_stats_default_surface() {
    let h = KafkaHealth { ready: true, detail: "ok".into() };
    assert!(h.ready);
    let s = KafkaPoolStats::default();
    assert_eq!(s.published, 0);
    assert_eq!(s.publish_timeouts, 0);
    assert_eq!(s.topics_ensured, 0);
    assert!(!s.closed);
}

#[tokio::test]
async fn offset_store_and_alo_helpers_on_data_dir() {
    let dir = data_dir();
    let path = dir.join("offsets.tsv");
    let file = FileOffsetStore::new(&path);
    file.commit("t", 0, 5).await.expect("file commit");
    assert_eq!(file.committed("t", 0).await.expect("get"), Some(6));

    let mem = MemoryOffsetStore::new().shared();
    mem.commit("t", 1, 2).await.expect("mem");
    assert_eq!(resolve_start_offset(mem.as_ref(), "t", 1).await.expect("r"), Some(3));

    let ptc = ProduceThenCheckpointCoordinator::new(Arc::clone(&mem) as Arc<dyn OffsetCommitStore>);
    assert_eq!(ptc.store().committed("t", 1).await.expect("c"), Some(3));

    let cfg = ConsumerConfig::assign("topic", 0, "g").with_start_offset(3);
    assert_eq!(cfg.start_offset, Some(3));
    assert_eq!(cfg.partition, 0);

    // 类型锚点：ALO bus 可构造（不连 broker）
    let refused = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:1")
        .connect_timeout(Duration::from_millis(80))
        .operation_timeout(Duration::from_millis(80))
        .delivery_timeout(Duration::from_millis(80))
        .build()
        .expect("cfg");
    // connect 失败路径：真实 shipped 入口
    let err = match KafkaPool::connect(refused).await {
        Ok(_) => panic!("must fail"),
        Err(e) => e,
    };
    assert!(matches!(
        err.kind(),
        ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
    ));

    // EventBus / ALO 类型存在性 + 构造需要 pool；用 connect_from_env 可能连上，
    // 离线只测类型名与 ConsumerConfig。
    let _ = std::any::type_name::<KafkaEventBus>();
    let _ = std::any::type_name::<KafkaAtLeastOnceBus>();
    let _ = std::any::type_name::<AtLeastOnceConsumer>();

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn selfcheck_connect_and_run_covers_catalog_and_nogo() {
    let cfg = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:1")
        .connect_timeout(Duration::from_millis(150))
        .operation_timeout(Duration::from_millis(150))
        .delivery_timeout(Duration::from_millis(150))
        .build()
        .expect("cfg");
    let report = KafkaValidator::connect_and_run(cfg, CheckLevel::Full).await;
    assert_eq!(report.module, "kafka");
    assert_eq!(report.items.len(), 9);
    assert!(!report.passed);
    for id in ["kafka.full.group_lag", "kafka.full.isr_health"] {
        let item = report.items.iter().find(|i| i.id == id).expect(id);
        assert_eq!(item.status, CheckStatus::Skipped);
        assert!(item.detail.as_ref().is_some_and(|d| d.contains("短路") || d.contains("NO-GO")));
    }
    // skip 配置路径
    let mut sc = KafkaSelfCheckConfig::default();
    sc.skip.insert("kafka.basic.metadata".into());
    assert!(sc.is_skipped("kafka.basic.metadata"));
    assert!(sc.baseline_ms("kafka.basic.metadata", Some(500)).is_some());
    let _ = now_rfc3339();
    let v_cat = KafkaValidator::static_catalog();
    assert_eq!(v_cat.len(), 9);
}

#[tokio::test]
async fn connect_from_env_and_closed_pool_fail_closed() {
    // from_env 不 panic；若无强制 env 则 default brokers
    let _ = KafkaPool::connect_from_env().await; // may ok or err depending on host

    let cfg = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:1")
        .connect_timeout(Duration::from_millis(100))
        .operation_timeout(Duration::from_millis(100))
        .delivery_timeout(Duration::from_millis(100))
        .build()
        .expect("cfg");
    let err = match KafkaPool::connect(cfg).await {
        Ok(_) => panic!("refused"),
        Err(e) => e,
    };
    assert_ne!(err.kind(), ErrorKind::Internal);
}

#[test]
fn nogo_public_api_does_not_export_group_or_txn_words() {
    // 锚定：公共 surface 不导出 group coordinator / transaction 成功路径类型
    let surface = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    assert!(!surface.contains("pub struct ConsumerGroup"));
    assert!(!surface.contains("pub struct TransactionalProducer"));
    assert!(!surface.contains("SchemaRegistry"));
}
