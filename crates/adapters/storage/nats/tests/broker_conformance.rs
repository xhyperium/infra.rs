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
    NatsPool::connect(NatsConfig::from_env().expect("NATS 环境配置合法")).await.expect("连接 NATS")
}

#[tokio::test]
#[ignore = "需要隔离 NATS；请通过 scripts/broker-conformance.mjs 运行"]
async fn core_nats_does_not_replay_messages_published_before_subscribe() {
    let pool = live_pool().await;
    let subject = format!("infra.conformance.core.{}", unique_suffix());
    eprintln!("NATS Core 语义验证 subject：{subject}");
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

#[tokio::test]
#[ignore = "需要隔离 NATS；请通过 scripts/broker-conformance.mjs 运行"]
async fn bounded_subscription_reports_local_slow_consumer() {
    let mut config = NatsConfig::from_env().expect("NATS 环境配置合法");
    config.subscription_capacity = 1;
    config.operation_timeout = Duration::from_millis(100);
    let pool = NatsPool::connect(config).await.expect("连接 NATS");
    let subject = format!("infra.conformance.slow-consumer.{}", unique_suffix());
    let _subscription = pool.subscribe(&subject).await.expect("建立不消费的订阅");

    for sequence in 0..8u32 {
        pool.publish(&subject, Bytes::copy_from_slice(&sequence.to_be_bytes()))
            .await
            .expect("发布慢消费者样本");
    }
    for _ in 0..50 {
        if pool.stats().slow_consumers >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(pool.stats().slow_consumers >= 1, "本地有界转发超时必须可观察");
    pool.close().await.expect("关闭 NATS pool");
}

async fn isolated_jetstream(
    pool: &NatsPool,
    label: &str,
    ack_wait: Duration,
    max_deliver: i64,
    max_ack_pending: i64,
) -> (JetStream, String, String, JetStreamConsumer, JetStreamConsumerConfig) {
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
    let consumer = jetstream.consumer(&stream, cfg.clone()).await.expect("创建 durable consumer");
    eprintln!("NATS JetStream 语义验证 stream：{stream}；subject：{subject}");
    (jetstream, stream, subject, consumer, cfg)
}

#[tokio::test]
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn jetstream_redelivers_until_double_ack() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer, cfg) =
        isolated_jetstream(&pool, "redelivery", Duration::from_millis(300), 3, 1).await;

    jetstream.publish(&subject, Bytes::from_static(b"redeliver-me")).await.expect("发布");
    let first = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("第一次拉取")
        .expect("第一次投递");
    let sequence = first.metadata().stream_sequence;
    assert_eq!(first.metadata().delivery_attempts, 1);
    assert_eq!(first.subject(), subject);
    let debug = format!("{first:?}");
    assert!(debug.contains("payload_len"));
    assert!(!debug.contains("redeliver-me"));
    drop(first);
    drop(consumer);
    drop(jetstream);
    pool.close().await.expect("关闭第一次 NATS 连接");
    tokio::time::sleep(Duration::from_millis(450)).await;

    let pool = live_pool().await;
    let jetstream = JetStream::from_pool(&pool);
    let consumer = jetstream.consumer(&stream, cfg).await.expect("重连 durable consumer");
    let redelivery =
        consumer.next_timeout(Duration::from_secs(2)).await.expect("重投拉取").expect("发生重投");
    assert_eq!(redelivery.metadata().stream_sequence, sequence);
    assert!(redelivery.metadata().delivery_attempts >= 2);
    redelivery.double_ack().await.expect("服务端确认 ack");
    assert!(
        consumer.next_timeout(Duration::from_millis(500)).await.expect("确认后拉取").is_none(),
        "double_ack 后不应再次重投"
    );
    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn max_ack_pending_applies_backpressure_until_ack() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer, _cfg) =
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
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn nak_redelivers_and_progress_extends_ack_wait() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer, _cfg) =
        isolated_jetstream(&pool, "ack-control", Duration::from_millis(800), 3, 1).await;

    jetstream
        .publish(&subject, Bytes::from_static(b"without-progress"))
        .await
        .expect("发布无进度确认对照消息");
    let control = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("拉取无进度确认对照消息")
        .expect("对照消息存在");
    let control_sequence = control.metadata().stream_sequence;
    drop(control);
    tokio::time::sleep(Duration::from_millis(1_000)).await;
    let control_redelivery = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("拉取无进度确认对照重投")
        .expect("对照消息必须重投");
    assert_eq!(control_redelivery.metadata().stream_sequence, control_sequence);
    assert!(control_redelivery.metadata().delivery_attempts >= 2);
    control_redelivery.ack().await.expect("确认对照重投");

    jetstream.publish(&subject, Bytes::from_static(b"progress")).await.expect("发布 progress");
    let progressing = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("拉取 progress")
        .expect("进度确认消息存在");
    tokio::time::sleep(Duration::from_millis(600)).await;
    progressing.progress().await.expect("延长 ack wait");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        consumer
            .next_timeout(Duration::from_millis(300))
            .await
            .expect("进度确认后并发拉取")
            .is_none(),
        "进度确认后原消息不应在延长窗口内重投"
    );
    progressing.ack().await.expect("确认 progress 消息");

    jetstream.publish(&subject, Bytes::from_static(b"nak")).await.expect("发布 nak");
    let first = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("第一次拉取 nak")
        .expect("否定确认消息存在");
    let sequence = first.metadata().stream_sequence;
    first.nak(Some(Duration::from_millis(100))).await.expect("请求延迟重投");
    let redelivery = consumer
        .next_timeout(Duration::from_secs(2))
        .await
        .expect("拉取 nak 重投")
        .expect("否定确认重投存在");
    assert_eq!(redelivery.metadata().stream_sequence, sequence);
    assert!(redelivery.metadata().delivery_attempts >= 2);
    redelivery.ack().await.expect("确认 nak 重投");

    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn with_operation_timeout_zero_fails_closed_without_mutating_context() {
    let pool = live_pool().await;
    let jetstream = JetStream::from_pool(&pool);
    // JetStream 未实现 Debug，避免 expect_err。
    let error = match jetstream.with_operation_timeout(Duration::ZERO) {
        Ok(_) => panic!("零 operation_timeout 必须拒绝"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kernel::ErrorKind::Invalid);

    // 拒绝不消费原始上下文：重新构造并确认原始 timeout 下仍可正常发布。
    let jetstream = JetStream::from_pool(&pool);
    let (_stream, _subject) = {
        let suffix = unique_suffix();
        let stream = format!("INFRA_TIMEOUT_ZERO_{}", suffix.replace('-', "_"));
        let subject = format!("infra.conformance.timeout-zero.{suffix}");
        jetstream
            .get_or_create_stream(StreamConfig::new(&stream, &subject))
            .await
            .expect("创建 stream 验证 jetstream 仍可用");
        jetstream.publish(&subject, Bytes::from_static(b"still-usable")).await.expect("仍可发布");
        jetstream.context().delete_stream(&stream).await.expect("删除 stream");
        (stream, subject)
    };
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn get_pull_consumer_rejects_invalid_names_and_missing_targets() {
    let pool = live_pool().await;
    let jetstream = JetStream::from_pool(&pool);

    let stream_error = jetstream
        .get_pull_consumer("bad.stream.name", "consumer")
        .await
        .expect_err("非法 stream 名必须在发起请求前拒绝");
    assert_eq!(stream_error.kind(), kernel::ErrorKind::Invalid);

    let suffix = unique_suffix();
    let stream = format!("INFRA_GETPULL_{}", suffix.replace('-', "_"));
    let subject = format!("infra.conformance.get-pull-consumer.{suffix}");
    jetstream
        .get_or_create_stream(StreamConfig::new(&stream, &subject))
        .await
        .expect("创建 stream");

    let consumer_name_error = jetstream
        .get_pull_consumer(&stream, "")
        .await
        .expect_err("空 consumer 名必须在发起请求前拒绝");
    assert_eq!(consumer_name_error.kind(), kernel::ErrorKind::Invalid);

    let missing_stream_error = jetstream
        .get_pull_consumer("INFRA_GETPULL_DOES_NOT_EXIST", "consumer")
        .await
        .expect_err("不存在的 stream 必须返回 Unavailable");
    assert_eq!(missing_stream_error.kind(), kernel::ErrorKind::Unavailable);

    let missing_consumer_error = jetstream
        .get_pull_consumer(&stream, "consumer-does-not-exist")
        .await
        .expect_err("stream 存在但 consumer 不存在必须返回 Unavailable");
    assert_eq!(missing_consumer_error.kind(), kernel::ErrorKind::Unavailable);

    let mut cfg = JetStreamConsumerConfig::durable(format!("durable-getpull-{suffix}"));
    cfg.filter_subject = Some(subject.clone());
    jetstream.consumer(&stream, cfg.clone()).await.expect("创建 durable consumer");
    let handle = jetstream
        .get_pull_consumer(&stream, &cfg.durable_name)
        .await
        .expect("已创建的 durable consumer 必须可获取");
    assert_eq!(handle.cached_info().name, cfg.durable_name);

    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}

#[tokio::test]
#[ignore = "需要隔离 NATS JetStream；请通过 scripts/broker-conformance.mjs 运行"]
async fn max_deliver_and_term_do_not_publish_conventional_dlq_subject() {
    let pool = live_pool().await;
    let (jetstream, stream, subject, consumer, _cfg) =
        isolated_jetstream(&pool, "poison", Duration::from_millis(250), 2, 1).await;
    let dlq_subject = format!("{subject}.DLQ");
    let mut dlq = pool.subscribe(&dlq_subject).await.expect("订阅显式 DLQ 探针");

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
        .expect("拉取待终止投递")
        .expect("待终止消息存在");
    terminal.term().await.expect("终止重投");
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert!(consumer.next_timeout(Duration::from_millis(500)).await.expect("终止后拉取").is_none());
    assert!(
        tokio::time::timeout(Duration::from_millis(300), dlq.recv()).await.is_err(),
        "term/max_deliver 不应自动发布到约定外的 DLQ subject"
    );
    jetstream.context().delete_stream(&stream).await.expect("删除 stream");
    pool.close().await.expect("关闭 NATS pool");
}
