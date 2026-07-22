//! Core NATS 与 JetStream broker conformance（默认 ignore）。
//!
//! 单节点结果不证明 Cluster/HA、跨账户、TLS 或自动 DLQ。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use natsx::{
    JetStream, JetStreamConsumer, JetStreamConsumerConfig, NatsConfig, NatsPool, StreamConfig,
};

fn unique_suffix() -> String {
    let nanos =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    format!("{}-{nanos}", std::process::id())
}

async fn live_pool() -> NatsPool {
    NatsPool::connect(NatsConfig::from_env()).await.expect("连接 NATS")
}

#[tokio::test]
#[ignore = "requires isolated NATS; run via scripts/broker-conformance.sh"]
async fn core_nats_does_not_replay_messages_published_before_subscribe() {
    let pool = live_pool().await;
    let subject = format!("infra.conformance.core.{}", unique_suffix());
    eprintln!("nats core conformance subject={subject}");
    pool.publish(&subject, Bytes::from_static(b"before-subscribe")).await.expect("订阅前发布");
    let mut subscription = pool.subscribe(&subject).await.expect("订阅");
    let missed = tokio::time::timeout(Duration::from_millis(400), subscription.recv()).await;
    assert!(missed.is_err(), "Core NATS 不应回放订阅前消息");

    pool.publish(&subject, Bytes::from_static(b"after-subscribe")).await.expect("订阅后发布");
    let observed = tokio::time::timeout(Duration::from_secs(5), subscription.recv())
        .await
        .expect("等待订阅后消息")
        .expect("订阅保持打开");
    assert_eq!(observed.payload, Bytes::from_static(b"after-subscribe"));
    pool.close().await.expect("关闭 NATS pool");
}

async fn isolated_jetstream(
    pool: &NatsPool,
    label: &str,
    ack_wait: Duration,
    max_deliver: i64,
    max_ack_pending: i64,
) -> (JetStream, String, String, JetStreamConsumer) {
    let suffix = unique_suffix();
    let stream = format!("INFRA_{}_{}", label.to_ascii_uppercase(), suffix.replace('-', "_"));
    let subject = format!("infra.conformance.{label}.{suffix}");
    let durable = format!("durable-{label}-{suffix}");
    let jetstream = JetStream::from_pool(pool);
    jetstream
        .get_or_create_stream(StreamConfig::new(&stream, &subject))
        .await
        .expect("创建 stream");
    let mut cfg = JetStreamConsumerConfig::durable(durable);
    cfg.filter_subject = Some(subject.clone());
    cfg.ack_wait = ack_wait;
    cfg.max_deliver = max_deliver;
    cfg.max_ack_pending = max_ack_pending;
    let consumer = jetstream.consumer(&stream, cfg).await.expect("创建 durable consumer");
    eprintln!("nats jetstream conformance stream={stream} subject={subject}");
    (jetstream, stream, subject, consumer)
}

#[tokio::test]
#[ignore = "requires isolated NATS JetStream; run via scripts/broker-conformance.sh"]
async fn jetstream_redelivers_until_double_ack() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer) =
        isolated_jetstream(&pool, "redelivery", Duration::from_millis(300), 3, 1).await;

    jetstream.publish(&subject, Bytes::from_static(b"redeliver-me")).await.expect("发布");
    let first = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("第一次拉取")
        .expect("第一次投递");
    let sequence = first.metadata().stream_sequence;
    assert_eq!(first.metadata().delivery_attempts, 1);
    drop(first);
    tokio::time::sleep(Duration::from_millis(450)).await;

    let redelivery =
        consumer.next_timeout(Duration::from_secs(2)).await.expect("重投拉取").expect("发生重投");
    assert_eq!(redelivery.metadata().stream_sequence, sequence);
    assert!(redelivery.metadata().delivery_attempts >= 2);
    redelivery.double_ack().await.expect("服务端确认 ack");
    assert!(
        consumer.next_timeout(Duration::from_millis(500)).await.expect("ack 后拉取").is_none(),
        "double_ack 后不应再次重投"
    );
    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "requires isolated NATS JetStream; run via scripts/broker-conformance.sh"]
async fn max_ack_pending_applies_backpressure_until_ack() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer) =
        isolated_jetstream(&pool, "backpressure", Duration::from_secs(5), 3, 1).await;
    jetstream.publish(&subject, Bytes::from_static(b"one")).await.expect("发布 one");
    jetstream.publish(&subject, Bytes::from_static(b"two")).await.expect("发布 two");

    let first = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("第一条拉取")
        .expect("第一条存在");
    assert!(
        consumer.next_timeout(Duration::from_millis(500)).await.expect("背压拉取").is_none(),
        "max_ack_pending=1 时未 ack 不应投递第二条"
    );
    first.ack().await.expect("确认第一条");
    let second = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("解除背压后拉取")
        .expect("第二条存在");
    assert_eq!(second.payload(), &Bytes::from_static(b"two"));
    second.ack().await.expect("确认第二条");
    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "requires isolated NATS JetStream; run via scripts/broker-conformance.sh"]
async fn max_deliver_and_term_stop_redelivery_without_creating_dlq() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer) =
        isolated_jetstream(&pool, "poison", Duration::from_millis(250), 2, 1).await;

    jetstream.publish(&subject, Bytes::from_static(b"max-deliver")).await.expect("发布毒消息");
    drop(
        consumer
            .next_timeout(Duration::from_secs(2))
            .await
            .expect("第一次投递")
            .expect("第一次存在"),
    );
    tokio::time::sleep(Duration::from_millis(350)).await;
    let last = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("第二次投递")
        .expect("第二次存在");
    assert_eq!(last.metadata().delivery_attempts, 2);
    drop(last);
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert!(consumer.next_timeout(Duration::from_millis(500)).await.expect("达到上限").is_none());

    jetstream.publish(&subject, Bytes::from_static(b"term")).await.expect("发布 term 消息");
    let terminal = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("term 投递")
        .expect("term 消息存在");
    terminal.term().await.expect("终止重投");
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert!(
        consumer.next_timeout(Duration::from_millis(500)).await.expect("term 后拉取").is_none()
    );
    // 未配置或发布任何 DLQ subject；term/max_deliver 只停止重投，不构成自动隔离。
    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}
