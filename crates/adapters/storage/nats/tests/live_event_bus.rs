//! 真实 NATS 往返（默认 ignore）。
//!
//! ```text
//! cargo test -p natsx --test live_event_bus -- --ignored --nocapture
//! ```

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use contracts::EventBus;
use futures_util::StreamExt;
use natsx::{NatsConfig, NatsEventBus, NatsPool};

fn unique_payload() -> String {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("natsx-live-{}-{}", std::process::id(), ts)
}

#[tokio::test]
#[ignore = "requires live NATS; run with --ignored when server available"]
async fn live_pub_sub_content() {
    let cfg = NatsConfig::from_env();
    let pool = NatsPool::connect(cfg).await.expect("connect");
    let health = pool.health().await.expect("health");
    assert!(health.ready, "not ready: {}", health.detail);

    let subject = format!("infra.draft.natsx.{}", std::process::id());
    let payload = unique_payload();

    let mut sub = pool.subscribe(&subject).await.expect("subscribe");
    // 订阅注册后短暂等待
    tokio::time::sleep(Duration::from_millis(200)).await;

    pool.publish(&subject, Bytes::from(payload.clone())).await.expect("publish");

    let msg = tokio::time::timeout(Duration::from_secs(10), sub.recv())
        .await
        .expect("timeout waiting message")
        .expect("subscription closed");
    assert_eq!(msg.payload.as_ref(), payload.as_bytes());
    assert_eq!(msg.subject, subject);

    let _ = pool.close().await;
}

#[tokio::test]
#[ignore = "requires live NATS; EventBus facade"]
async fn live_event_bus_roundtrip() {
    let pool = NatsPool::connect_from_env().await.expect("connect");
    let bus = NatsEventBus::new(pool.clone());
    let subject = format!("infra.draft.natsx.eb.{}", std::process::id());
    let payload = unique_payload();

    let mut stream = bus.subscribe(&subject).await.expect("sub");
    tokio::time::sleep(Duration::from_millis(200)).await;
    bus.publish(&subject, Bytes::from(payload.clone())).await.expect("pub");

    let m = tokio::time::timeout(Duration::from_secs(10), stream.next())
        .await
        .expect("timeout")
        .expect("msg");
    assert_eq!(m.payload.as_ref(), payload.as_bytes());
    assert!(m.id.contains(&subject), "id={}", m.id);

    let _ = pool.close().await;
}
