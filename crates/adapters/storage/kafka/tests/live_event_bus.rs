//! 需要 `--features sasl` 与真实 broker。
//! 真实 broker 往返（默认 ignore）。
//!
//! ```text
//! cargo test -p kafkax --test live_event_bus -- --ignored --nocapture
//! ```
//!
//! 说明：部分环境 group coordinator 会返回 `COORDINATOR_NOT_AVAILABLE`，
//! 此时消费组 `subscribe` 无法 join。live 内容断言使用 **手动 assign**
//!（不依赖 coordinator），并仍验证 `publish` delivery report 与 payload。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use contracts::EventBus;
use kafkax::{ConsumerConfig, KafkaConfig, KafkaEventBus, KafkaPool, encode_bus_id};

fn unique_suffix() -> String {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{}-{}", std::process::id(), ts)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[cfg_attr(not(feature = "sasl"), ignore = "needs feature sasl + live Kafka")]
#[cfg_attr(feature = "sasl", ignore = "requires live Kafka; run with --ignored when broker available")]
async fn live_publish_consume_content() {
    let cfg = KafkaConfig::from_env();
    let pool = KafkaPool::connect(cfg).await.expect("connect");

    let topic = format!("infra-draft-kafkax-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("ensure_topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let health = pool.health().await.expect("health");
    assert!(health.ready, "cluster not ready: {}", health.detail);

    let payload = format!("payload-{}", unique_suffix());
    let group = format!("kafkax-live-{}", unique_suffix());

    // group.id 仍需要（rdkafka client 要求），但用 assign 绕过 coordinator
    let mut ccfg = ConsumerConfig::new(&group);
    ccfg.auto_offset_reset = "earliest".into();
    ccfg.enable_auto_commit = false;
    let consumer = pool.consumer_with(ccfg).await.expect("consumer");
    // partition 0 from beginning（-2）
    consumer.assign(&topic, &[(0, -2)]).expect("assign");
    assert_eq!(consumer.assignment_count().expect("count"), 1);

    let delivery =
        pool.producer().publish(&topic, Bytes::from(payload.clone())).await.expect("publish");
    eprintln!(
        "published topic={topic} partition={} offset={}",
        delivery.partition, delivery.offset
    );
    assert_eq!(delivery.partition, 0);

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut found = None;
    while tokio::time::Instant::now() < deadline {
        match consumer.recv_timeout(Duration::from_secs(2)).await {
            Ok(Some(msg)) => {
                eprintln!(
                    "got part={} off={} len={}",
                    msg.partition,
                    msg.offset,
                    msg.payload.len()
                );
                if msg.payload.as_ref() == payload.as_bytes() {
                    found = Some(msg);
                    break;
                }
            }
            Ok(None) => eprintln!("recv timeout tick"),
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
#[cfg_attr(not(feature = "sasl"), ignore = "needs feature sasl + live Kafka")]
#[cfg_attr(feature = "sasl", ignore = "requires live Kafka; EventBus publish + id encoding")]
async fn live_event_bus_publish_and_id() {
    // EventBus::subscribe 依赖 group coordinator；本环境可能不可用。
    // 本用例验证 EventBus::publish（delivery report）+ 用 assign 消费校验 id 格式。
    let pool = KafkaPool::connect_from_env().await.expect("connect");
    let topic = format!("infra-draft-kafkax-eb-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("ensure_topic");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let bus = KafkaEventBus::new(pool.clone());
    let payload = format!("eb-{}", unique_suffix());

    let mut ccfg = ConsumerConfig::new(format!("kafkax-eb-{}", unique_suffix()));
    ccfg.enable_auto_commit = false;
    let consumer = pool.consumer_with(ccfg).await.expect("consumer");
    consumer.assign(&topic, &[(0, -2)]).expect("assign");

    EventBus::publish(&bus, &topic, Bytes::from(payload.clone())).await.expect("publish");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut hit = false;
    while tokio::time::Instant::now() < deadline {
        match consumer.recv_timeout(Duration::from_secs(2)).await {
            Ok(Some(msg)) if msg.payload.as_ref() == payload.as_bytes() => {
                let id = msg.bus_id();
                eprintln!("bus id={id}");
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
