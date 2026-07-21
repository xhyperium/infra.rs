//! 真实 broker 往返（默认 ignore）。
//!
//! ```text
//! cargo test -p kafkax --test live_event_bus -- --ignored --nocapture
//! ```
//!
//! 使用纯 Rust `rskafka` + 分区消费（不依赖 group coordinator）。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use contracts::EventBus;
use kafkax::{ConsumerConfig, KafkaConfig, KafkaEventBus, KafkaPool, encode_bus_id};

fn unique_suffix() -> String {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{}-{}", std::process::id(), ts)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires live Kafka; run with --ignored when broker available"]
async fn live_publish_consume_content() {
    let cfg = KafkaConfig::from_env();
    let pool = KafkaPool::connect(cfg).await.expect("connect");

    let topic = format!("infra-draft-kafkax-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("ensure_topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let health = pool.health().await.expect("health");
    assert!(health.ready, "cluster not ready: {}", health.detail);

    let payload = format!("payload-{}", unique_suffix());
    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("g-{}", unique_suffix())))
        .await
        .expect("consumer");

    let delivery =
        pool.producer().publish(&topic, Bytes::from(payload.clone())).await.expect("publish");
    assert_eq!(delivery.partition, 0);

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut found = None;
    while tokio::time::Instant::now() < deadline {
        match consumer.recv_timeout(Duration::from_secs(2)).await {
            Ok(Some(msg)) => {
                if msg.payload.as_ref() == payload.as_bytes() {
                    found = Some(msg);
                    break;
                }
            }
            Ok(None) => {}
            Err(e) => eprintln!("recv err: {e}"),
        }
    }
    let msg = found.expect("did not receive published payload in time");
    assert_eq!(msg.payload.as_ref(), payload.as_bytes());
    assert_eq!(msg.bus_id(), encode_bus_id(&msg.topic, msg.partition, msg.offset));
    assert_eq!(msg.offset, delivery.offset);

    let _ = pool.close(Duration::from_secs(3)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires live Kafka; EventBus publish + id encoding"]
async fn live_event_bus_publish_and_id() {
    let pool = KafkaPool::connect_from_env().await.expect("connect");
    let topic = format!("infra-draft-kafkax-eb-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("ensure_topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let bus = KafkaEventBus::new(pool.clone());
    let payload = format!("eb-{}", unique_suffix());

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("eb-{}", unique_suffix())))
        .await
        .expect("consumer");

    EventBus::publish(&bus, &topic, Bytes::from(payload.clone())).await.expect("publish");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut hit = false;
    while tokio::time::Instant::now() < deadline {
        match consumer.recv_timeout(Duration::from_secs(2)).await {
            Ok(Some(msg)) if msg.payload.as_ref() == payload.as_bytes() => {
                let id = msg.bus_id();
                assert_eq!(id, encode_bus_id(&msg.topic, msg.partition, msg.offset));
                let (t, p, o) = kafkax::parse_bus_id(&id).expect("parse id");
                assert_eq!(t, topic.as_str());
                assert_eq!(p, msg.partition);
                assert_eq!(o, msg.offset);
                hit = true;
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => eprintln!("recv err {e}"),
        }
    }
    assert!(hit, "did not observe EventBus published payload");
    let _ = pool.close(Duration::from_secs(3)).await;
}
