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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要可用 Kafka；只读验证认证、连接与集群健康"]
async fn live_cluster_health() {
    let cfg = KafkaConfig::from_env().expect("Kafka 环境配置合法");
    let pool = KafkaPool::connect(cfg).await.expect("连接 Kafka");
    let health = pool.health().await.expect("读取健康状态");
    assert!(health.ready, "集群未就绪：{}", health.detail);
    pool.close(Duration::from_secs(3)).await.expect("关闭 Kafka pool");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "需要可用 Kafka；请在 broker 就绪后使用 --ignored 运行"]
async fn live_publish_consume_content() {
    let cfg = KafkaConfig::from_env().expect("Kafka 环境配置合法");
    let pool = KafkaPool::connect(cfg).await.expect("连接 Kafka");

    let topic = format!("infra-draft-kafkax-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("创建主题");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let health = pool.health().await.expect("读取健康状态");
    assert!(health.ready, "集群未就绪：{}", health.detail);

    let payload = format!("payload-{}", unique_suffix());
    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("g-{}", unique_suffix())))
        .await
        .expect("创建消费者");

    let delivery =
        pool.producer().publish(&topic, Bytes::from(payload.clone())).await.expect("发布消息");
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
            Err(e) => eprintln!("接收消息失败：{e}"),
        }
    }
    let msg = found.expect("未在截止时间内收到已发布 payload");
    assert_eq!(msg.payload.as_ref(), payload.as_bytes());
    assert_eq!(msg.bus_id(), encode_bus_id(&msg.topic, msg.partition, msg.offset));
    assert_eq!(msg.offset, delivery.offset);

    let _ = pool.close(Duration::from_secs(3)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "需要可用 Kafka；验证 EventBus 发布与消息 ID 编码"]
async fn live_event_bus_publish_and_id() {
    let pool = KafkaPool::connect_from_env().await.expect("连接 Kafka");
    let topic = format!("infra-draft-kafkax-eb-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("创建主题");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let bus = KafkaEventBus::new(pool.clone());
    let payload = format!("eb-{}", unique_suffix());

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("eb-{}", unique_suffix())))
        .await
        .expect("创建消费者");

    EventBus::publish(&bus, &topic, Bytes::from(payload.clone())).await.expect("发布消息");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut hit = false;
    while tokio::time::Instant::now() < deadline {
        match consumer.recv_timeout(Duration::from_secs(2)).await {
            Ok(Some(msg)) if msg.payload.as_ref() == payload.as_bytes() => {
                let id = msg.bus_id();
                assert_eq!(id, encode_bus_id(&msg.topic, msg.partition, msg.offset));
                let (t, p, o) = kafkax::parse_bus_id(&id).expect("解析消息 ID");
                assert_eq!(t, topic.as_str());
                assert_eq!(p, msg.partition);
                assert_eq!(o, msg.offset);
                hit = true;
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(e) => eprintln!("接收消息失败：{e}"),
        }
    }
    assert!(hit, "未观察到 EventBus 发布的 payload");
    let _ = pool.close(Duration::from_secs(3)).await;
}
