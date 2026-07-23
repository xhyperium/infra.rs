//! 生产矩阵 · 集成 / 可靠性 / E2E / 恢复（默认 ignore）。
//!
//! ```text
//! node scripts/kafka-prod-matrix.mjs
//! # 或：
//! cargo test -p kafkax --test prod_reliability -- --ignored --nocapture --test-threads=1
//! ```
//!
//! 不证明：group/rebalance、自动重连、native EOS、HA、7×24 默认门禁。

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use contracts::EventBus;
use kafkax::selfcheck::{CheckLevel, CheckStatus, KafkaSelfCheckConfig, KafkaValidator};
use kafkax::{
    AtLeastOnceConsumer, ConsumerConfig, KafkaConfig, KafkaEventBus, KafkaPool, MemoryOffsetStore,
    OffsetCommitStore, encode_bus_id,
};
use kernel::ErrorKind;

fn unique_suffix() -> String {
    let ns = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{}-{ns}", std::process::id())
}

async fn pool() -> KafkaPool {
    KafkaPool::connect(KafkaConfig::from_env().expect("Kafka 环境配置合法"))
        .await
        .expect("连接 Kafka")
}

/// 无新依赖的稳定指纹（FNV-1a 64）。
fn checksum(payload: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x100_0000_01b3;
    let mut hash = OFFSET;
    for byte in payload {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// §1 分区内顺序。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn partition_order_preserved_for_sequential_publish() {
    let pool = pool().await;
    let topic = format!("infra-prod-order-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(400)).await;

    let n = 20u64;
    for i in 0..n {
        pool.producer().publish(&topic, Bytes::from(format!("seq-{i:04}"))).await.expect("publish");
    }

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("ord-{}", unique_suffix())))
        .await
        .expect("consumer");
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    while got.len() < n as usize && Instant::now() < deadline {
        if let Ok(Some(msg)) = consumer.recv_timeout(Duration::from_secs(2)).await {
            got.push(String::from_utf8_lossy(&msg.payload).into_owned());
        }
    }
    assert_eq!(got.len(), n as usize, "应收到全部顺序消息");
    for i in 0..n {
        assert_eq!(got[i as usize], format!("seq-{i:04}"));
    }
    // offset 单调
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §8 E2E 条数 + checksum。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn e2e_count_and_payload_checksum() {
    let pool = pool().await;
    let topic = format!("infra-prod-cksum-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(400)).await;

    let payloads: Vec<Bytes> =
        (0..15).map(|i| Bytes::from(format!("body-{i}-{}", unique_suffix()))).collect();
    let mut expect = Vec::new();
    for p in &payloads {
        let d = pool.producer().publish(&topic, p.clone()).await.expect("pub");
        expect.push((d.offset, checksum(p.as_ref()), p.clone()));
    }

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("ck-{}", unique_suffix())))
        .await
        .expect("c");
    let mut hits = 0usize;
    let deadline = Instant::now() + Duration::from_secs(30);
    while hits < expect.len() && Instant::now() < deadline {
        if let Ok(Some(msg)) = consumer.recv_timeout(Duration::from_secs(2)).await {
            let sum = checksum(msg.payload.as_ref());
            let found = expect.iter().any(|(off, s, raw)| {
                *off == msg.offset && *s == sum && raw.as_ref() == msg.payload.as_ref()
            });
            assert!(found, "offset={} checksum 未匹配已发布集合", msg.offset);
            assert_eq!(msg.bus_id(), encode_bus_id(&msg.topic, msg.partition, msg.offset));
            hits += 1;
        }
    }
    assert_eq!(hits, expect.len());
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §3 大消息 1MiB。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn large_message_1mib_roundtrip() {
    let pool = pool().await;
    let topic = format!("infra-prod-1m-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(400)).await;

    let mut raw = vec![0u8; 1024 * 1024];
    for (i, b) in raw.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let payload = Bytes::from(raw);
    let sum = checksum(payload.as_ref());
    let delivery = pool.producer().publish(&topic, payload.clone()).await.expect("1MiB publish");

    let mut consumer = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("1m-{}", unique_suffix())))
        .await
        .expect("c");
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut ok = false;
    while Instant::now() < deadline {
        if let Ok(Some(msg)) = consumer.recv_timeout(Duration::from_secs(3)).await {
            if msg.offset == delivery.offset {
                assert_eq!(msg.payload.len(), 1024 * 1024);
                assert_eq!(checksum(msg.payload.as_ref()), sum);
                ok = true;
                break;
            }
        }
    }
    assert!(ok, "未收到 1MiB 消息");
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §2 突发并发 publish。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn burst_concurrent_publish() {
    let pool = pool().await;
    let topic = format!("infra-prod-burst-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(400)).await;

    let n = 40usize;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let producer = pool.producer();
        let topic = topic.clone();
        handles.push(tokio::spawn(async move {
            producer.publish(&topic, Bytes::from(format!("burst-{i}"))).await
        }));
    }
    let mut ok = 0usize;
    let mut fail = 0usize;
    for h in handles {
        match h.await.expect("join") {
            Ok(_) => ok += 1,
            Err(_) => fail += 1,
        }
    }
    assert!(ok >= n * 9 / 10, "并发 publish 成功率过低 ok={ok} fail={fail}");
    let stats = pool.stats();
    assert!(stats.published >= ok as u64);
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §9 灾难恢复：未 ack 重建同 offset；ack 后推进。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn disaster_recovery_alo_checkpoint() {
    let pool = pool().await;
    let topic = format!("infra-prod-dr-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(400)).await;

    let p1 = Bytes::from(format!("dr-a-{}", unique_suffix()));
    let p2 = Bytes::from(format!("dr-b-{}", unique_suffix()));
    pool.producer().publish(&topic, p1.clone()).await.expect("p1");
    pool.producer().publish(&topic, p2.clone()).await.expect("p2");

    let store: Arc<dyn OffsetCommitStore> = MemoryOffsetStore::new().shared();
    let mut c1 = AtLeastOnceConsumer::connect(
        pool.clone(),
        ConsumerConfig::assign(&topic, 0, "dr-g"),
        Arc::clone(&store),
    )
    .await
    .expect("c1");

    let first = loop {
        match c1.recv_timeout(Duration::from_secs(2)).await.expect("recv") {
            Some(m) if m.payload == p1 || m.payload == p2 => break m,
            Some(_) => {}
            None => continue,
        }
    };
    let first_off = first.offset;
    // 未 ack 即 drop → 重建应同 offset
    drop(c1);

    let mut c2 = AtLeastOnceConsumer::connect(
        pool.clone(),
        ConsumerConfig::assign(&topic, 0, "dr-g"),
        Arc::clone(&store),
    )
    .await
    .expect("c2");
    let again = c2.recv_timeout(Duration::from_secs(5)).await.expect("r").expect("msg");
    assert_eq!(again.offset, first_off, "未 ack 必须重投同 offset");

    c2.ack().await.expect("ack");
    let after = store.committed(&topic, 0).await.expect("committed");
    assert_eq!(after, Some(first_off + 1));

    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §1 关闭后拒绝新 I/O。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn close_rejects_new_publish() {
    let pool = pool().await;
    let topic = format!("infra-prod-close-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    pool.close(Duration::from_secs(5)).await.expect("close");
    let err = pool.producer().publish(&topic, Bytes::from_static(b"x")).await.expect_err("closed");
    assert_eq!(err.kind(), ErrorKind::Cancelled);
}

/// §7 可观测：成功路径更新 stats。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn observability_stats_increment_on_publish() {
    let pool = pool().await;
    let topic = format!("infra-prod-stats-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    let before = pool.stats().published;
    pool.producer().publish(&topic, Bytes::from_static(b"s")).await.expect("p");
    let after = pool.stats();
    assert!(after.published > before);
    assert!(!after.closed);
    let health = pool.health().await.expect("health");
    assert!(health.ready, "{}", health.detail);
    let _ = pool.close(Duration::from_secs(3)).await;
    assert!(pool.stats().closed);
}

/// §7 **严格**：`publish_timeouts` 或 `publish_cancelled` 必须在 produce 有界等待路径上递增。
///
/// 策略（确定性）：
/// 1. 若提供 `KAFKAX_DOCKER_CONTAINER`：pause 容器使 broker I/O 挂起，短 `delivery_timeout` 命中 timeout 臂；
/// 2. 否则：并发 produce + 立即 close，要求 `publish_cancelled > 0` 或 `publish_timeouts > 0`
///    （禁止仅用 published/failed 凑绿）。
///
/// 离线确定性覆盖见 `producer::tests::limited_await_*_increments_*`（shipped `limited_produce_await`）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn stats_cancel_or_timeout_increment_on_close_during_publish() {
    // 路径 A：docker pause → produce 超时臂（严格 publish_timeouts）
    if let Ok(container) = std::env::var("KAFKAX_DOCKER_CONTAINER") {
        if !container.is_empty() {
            let mut cfg = KafkaConfig::from_env().expect("cfg");
            cfg.delivery_timeout = Duration::from_millis(80);
            cfg.operation_timeout = Duration::from_millis(500);
            let pool = KafkaPool::connect(cfg).await.expect("connect");
            let topic = format!("infra-prod-stats-to-{}", unique_suffix());
            pool.ensure_topic(&topic, 1, 1).await.expect("topic");
            tokio::time::sleep(Duration::from_millis(300)).await;
            // 先成功一条，确保 topic 就绪
            pool.producer().publish(&topic, Bytes::from_static(b"warm")).await.expect("warm");
            let before = pool.stats().publish_timeouts;

            let pause = std::process::Command::new("docker")
                .args(["pause", &container])
                .status()
                .expect("docker pause");
            assert!(pause.success(), "docker pause {container}");

            let err = pool
                .producer()
                .publish(&topic, Bytes::from_static(b"after-pause"))
                .await
                .expect_err("pause 后应失败");
            let _ = std::process::Command::new("docker").args(["unpause", &container]).status();

            let after_to = pool.stats().publish_timeouts;
            let after_ca = pool.stats().publish_cancelled;
            // partition_client 也可能先超时；此时 produce 超时计数可能不增。
            // 若未进入 produce 臂，至少 close 路径再验证 cancel。
            if after_to > before {
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::DeadlineExceeded | ErrorKind::Unavailable | ErrorKind::Transient
                    ),
                    "timeout path kind={:?}",
                    err.kind()
                );
                let _ = pool.close(Duration::from_secs(3)).await;
                return;
            }
            // fall through to close-race if pause 未打到 produce timeout
            let _ = pool.close(Duration::from_secs(3)).await;
            eprintln!(
                "docker pause 未递增 publish_timeouts (to={after_to} ca={after_ca}); 改走 close 竞态"
            );
        }
    }

    // 路径 B：并发 produce + 快速 close，严格要求 cancelled|timeouts
    let pool = Arc::new(pool().await);
    let topic = format!("infra-prod-stats-cancel-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let before = pool.stats();
    // 先启动一批会占用 produce 的任务
    let mut handles = Vec::new();
    for i in 0..64 {
        let p = Arc::clone(&pool);
        let t = topic.clone();
        handles.push(tokio::spawn(async move {
            // 略微错开，增加处于 select 内的窗口
            if i > 0 {
                tokio::task::yield_now().await;
            }
            p.producer().publish(&t, Bytes::from(vec![0u8; 64 * 1024])).await
        }));
    }
    // 几乎立即关闭，逼 produce 的 shutdown 臂
    tokio::task::yield_now().await;
    let pool_c = Arc::clone(&pool);
    let closer = tokio::spawn(async move {
        let _ = pool_c.close(Duration::from_secs(3)).await;
    });
    for h in handles {
        let _ = h.await;
    }
    let _ = closer.await;
    let after = pool.stats();
    assert!(after.closed, "pool 应已关闭");
    assert!(
        after.publish_cancelled > before.publish_cancelled
            || after.publish_timeouts > before.publish_timeouts,
        "严格要求 publish_cancelled 或 publish_timeouts 递增（禁止仅 published/failed）: before={before:?} after={after:?}"
    );
}

/// EventBus::with_group 真实 publish/subscribe 路径。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn event_bus_with_group_publish_subscribe() {
    let pool = pool().await;
    let topic = format!("infra-prod-bus-g-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let bus = KafkaEventBus::with_group(pool.clone(), format!("prod-g-{}", unique_suffix()));
    bus.publish(&topic, Bytes::from_static(b"with-group-payload")).await.expect("pub");

    // 直接 consumer 从最新前一条读回
    let mut c = pool
        .consumer(ConsumerConfig::assign(&topic, 0, format!("cg-{}", unique_suffix())))
        .await
        .expect("c");
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut got = None;
    while Instant::now() < deadline {
        if let Ok(Some(m)) = c.recv_timeout(Duration::from_secs(1)).await {
            if m.payload.as_ref() == b"with-group-payload" {
                got = Some(m);
                break;
            }
        }
    }
    assert!(got.is_some(), "with_group EventBus publish 应可被消费");
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// ALO nack / drop_pending / is_terminated 在真实 consumer 上的行为。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn alo_nack_and_drop_pending_on_live_consumer() {
    let pool = pool().await;
    let topic = format!("infra-prod-alo-gate-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    tokio::time::sleep(Duration::from_millis(200)).await;
    let d = pool.producer().publish(&topic, Bytes::from_static(b"alo-gate")).await.expect("p");

    let store = MemoryOffsetStore::new().shared();
    let mut c = AtLeastOnceConsumer::connect(
        pool.clone(),
        ConsumerConfig::assign(&topic, 0, "alo-g").with_start_offset(d.offset),
        Arc::clone(&store) as Arc<dyn OffsetCommitStore>,
    )
    .await
    .expect("alo");
    let m = c.recv_timeout(Duration::from_secs(10)).await.expect("r").expect("msg");
    assert_eq!(m.payload.as_ref(), b"alo-gate");
    c.nack_keep_pending();
    assert!(!c.is_terminated());
    assert_eq!(c.pending().map(|x| x.offset), Some(m.offset));
    c.drop_pending_unacked();
    assert!(c.is_terminated());
    let err = c.recv_timeout(Duration::from_millis(50)).await.expect_err("term");
    assert_eq!(err.kind(), ErrorKind::Cancelled);
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// KafkaValidator::with_config 真实 skip 行为。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka；kafka-prod-matrix.mjs"]
async fn selfcheck_with_config_skips_metadata() {
    let pool = pool().await;
    let mut sc = KafkaSelfCheckConfig::default();
    sc.skip.insert("kafka.basic.metadata".into());
    let v = KafkaValidator::new(pool.clone()).with_config(sc);
    assert!(v.config().is_skipped("kafka.basic.metadata"));
    let report = v.run(CheckLevel::Basic).await;
    let meta = report.items.iter().find(|i| i.id == "kafka.basic.metadata").expect("meta");
    assert_eq!(meta.status, CheckStatus::Skipped, "{meta:?}");
    let _ = pool.close(Duration::from_secs(3)).await;
}

/// §2 可选短 soak：受 `KAFKAX_SOAK_SECONDS` 控制（默认 0 跳过）。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要 Kafka + KAFKAX_SOAK_SECONDS>0"]
async fn optional_bounded_soak_loop() {
    let secs: u64 =
        std::env::var("KAFKAX_SOAK_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    if secs == 0 {
        eprintln!("KAFKAX_SOAK_SECONDS=0，跳过 soak（非失败）");
        return;
    }
    let pool = pool().await;
    let topic = format!("infra-prod-soak-{}", unique_suffix());
    pool.ensure_topic(&topic, 1, 1).await.expect("topic");
    let end = Instant::now() + Duration::from_secs(secs.min(3600));
    let mut i = 0u64;
    let mut errors = 0u64;
    while Instant::now() < end {
        match pool.producer().publish(&topic, Bytes::from(format!("soak-{i}"))).await {
            Ok(_) => i += 1,
            Err(_) => errors += 1,
        }
        if i % 50 == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
    eprintln!("soak done published={i} errors={errors} wall={secs}s");
    assert!(i > 0, "soak 期间应至少成功一条");
    let _ = pool.close(Duration::from_secs(5)).await;
}

/// §1 故障注入辅助：当 `KAFKAX_EXPECT_BROKER_DOWN=1` 时连接应失败。
#[tokio::test]
#[ignore = "由 kafka-prod-matrix.mjs --fault-restart 在停 broker 后调用"]
async fn fault_broker_down_connect_fails() {
    if std::env::var("KAFKAX_EXPECT_BROKER_DOWN").ok().as_deref() != Some("1") {
        eprintln!("未设置 KAFKAX_EXPECT_BROKER_DOWN=1，跳过");
        return;
    }
    let cfg = KafkaConfig::from_env().expect("cfg");
    let result = tokio::time::timeout(Duration::from_secs(8), KafkaPool::connect(cfg)).await;
    match result {
        Ok(Ok(_)) => panic!("broker 应不可达"),
        Ok(Err(e)) => assert!(
            matches!(
                e.kind(),
                ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
            ),
            "{:?}",
            e.kind()
        ),
        Err(_) => {} // 超时亦可接受
    }
}
