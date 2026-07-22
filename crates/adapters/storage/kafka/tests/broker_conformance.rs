//! Kafka 单节点 broker conformance（默认 ignore）。
//!
//! 这些场景只证明手动分区、单 owner、应用自管 checkpoint 的边界；不证明
//! consumer group、rebalance、multi-owner fencing、TLS、HA 或 broker 原生 EOS。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use kafkax::{
    ConsumerConfig, FileOffsetStore, KafkaAtLeastOnceBus, KafkaConfig, KafkaPool,
    MemoryOffsetStore, OffsetCommitStore, ProduceThenCheckpointCoordinator,
};
use kernel::{ErrorKind, XError, XResult};

fn unique_suffix() -> String {
    let nanos =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    format!("{}-{nanos}", std::process::id())
}

async fn live_pool() -> KafkaPool {
    KafkaPool::connect(KafkaConfig::from_env()).await.expect("连接 Kafka")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires isolated Kafka; run via scripts/broker-conformance.sh"]
async fn unacked_restarts_same_offset_and_acked_advances() {
    let pool = live_pool().await;
    let topic = format!("infra-conformance-checkpoint-{}", unique_suffix());
    eprintln!("kafka conformance topic={topic}");
    pool.ensure_topic(&topic, 1, 1).await.expect("创建 topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let first_payload = Bytes::from(format!("first-{}", unique_suffix()));
    let second_payload = Bytes::from(format!("second-{}", unique_suffix()));
    pool.producer().publish(&topic, first_payload.clone()).await.expect("发布第一条");

    let dir = std::env::temp_dir().join(format!("kafkax-conformance-{}", unique_suffix()));
    std::fs::create_dir_all(&dir).expect("创建 checkpoint 临时目录");
    let path = dir.join("offsets.tsv");
    let first_store: Arc<dyn OffsetCommitStore> = Arc::new(FileOffsetStore::new(&path));
    let bus = KafkaAtLeastOnceBus::new(pool.clone(), first_store);
    let mut first = bus
        .consumer(ConsumerConfig::assign(&topic, 0, "single-owner"))
        .await
        .expect("创建第一次 consumer");
    let first_message =
        first.recv_timeout(Duration::from_secs(10)).await.expect("拉取第一条").expect("第一条存在");
    assert_eq!(first_message.payload, first_payload);
    let first_offset = first_message.offset;
    drop(first);

    let second_store: Arc<dyn OffsetCommitStore> = Arc::new(FileOffsetStore::new(&path));
    let bus = KafkaAtLeastOnceBus::new(pool.clone(), second_store);
    let mut restarted = bus
        .consumer(ConsumerConfig::assign(&topic, 0, "single-owner"))
        .await
        .expect("重建 consumer");
    let redelivery =
        restarted.recv_timeout(Duration::from_secs(10)).await.expect("拉取重投").expect("重投存在");
    assert_eq!(redelivery.offset, first_offset);
    assert_eq!(redelivery.payload, first_payload);
    restarted.ack().await.expect("确认第一条");
    drop(restarted);

    pool.producer().publish(&topic, second_payload.clone()).await.expect("发布第二条");
    let third_store: Arc<dyn OffsetCommitStore> = Arc::new(FileOffsetStore::new(&path));
    let bus = KafkaAtLeastOnceBus::new(pool.clone(), third_store);
    let mut advanced = bus
        .consumer(ConsumerConfig::assign(&topic, 0, "single-owner"))
        .await
        .expect("创建已推进 consumer");
    let next = advanced
        .recv_timeout(Duration::from_secs(10))
        .await
        .expect("拉取下一条")
        .expect("下一条存在");
    assert!(next.offset > first_offset);
    assert_eq!(next.payload, second_payload);
    advanced.ack().await.expect("确认第二条");

    std::fs::remove_dir_all(&dir).expect("清理 checkpoint 临时目录");
    pool.close(Duration::from_secs(3)).await.expect("关闭 Kafka pool");
}

struct FailFirstCommitStore {
    failed: AtomicBool,
    inner: MemoryOffsetStore,
}

impl FailFirstCommitStore {
    fn new() -> Self {
        Self { failed: AtomicBool::new(false), inner: MemoryOffsetStore::new() }
    }
}

#[async_trait]
impl OffsetCommitStore for FailFirstCommitStore {
    async fn committed(&self, topic: &str, partition: i32) -> XResult<Option<i64>> {
        self.inner.committed(topic, partition).await
    }

    async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()> {
        if !self.failed.swap(true, Ordering::SeqCst) {
            return Err(XError::unavailable("测试注入：checkpoint 首次失败"));
        }
        self.inner.commit(topic, partition, offset).await
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires isolated Kafka; run via scripts/broker-conformance.sh"]
async fn successful_produce_then_failed_checkpoint_has_duplicate_window() {
    let pool = live_pool().await;
    let topic = format!("infra-conformance-duplicate-{}", unique_suffix());
    eprintln!("kafka conformance duplicate-window topic={topic}");
    pool.ensure_topic(&topic, 1, 1).await.expect("创建 topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let payload = Bytes::from(format!("stable-id-{}", unique_suffix()));
    let store: Arc<dyn OffsetCommitStore> = Arc::new(FailFirstCommitStore::new());
    let coordinator = ProduceThenCheckpointCoordinator::new(store);
    let producer = pool.producer();
    let first = coordinator
        .produce_then_commit(&producer, &topic, payload.clone(), "input", 0, 7)
        .await
        .expect_err("produce 成功后 checkpoint 应按注入失败");
    assert_eq!(first.kind(), ErrorKind::Unavailable);
    coordinator
        .produce_then_commit(&producer, &topic, payload.clone(), "input", 0, 7)
        .await
        .expect("重试成功");

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, "single-owner"))
        .await
        .expect("创建观测 consumer");
    let mut matching = 0;
    while matching < 2 {
        let message = consumer
            .recv_timeout(Duration::from_secs(10))
            .await
            .expect("拉取重复窗口")
            .expect("消息存在");
        if message.payload == payload {
            matching += 1;
        }
    }
    assert_eq!(matching, 2, "非原子 produce/checkpoint 重试应可观察到重复");
    pool.close(Duration::from_secs(3)).await.expect("关闭 Kafka pool");
}
