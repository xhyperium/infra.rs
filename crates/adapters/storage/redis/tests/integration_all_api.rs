//! redisx **集成测试**：覆盖公开 API 面（需 live Redis）。
//!
//! ```bash
//! # 凭据来自 ZoneCNH secrets/env/dev.md，不回显密码
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo test -p redisx --features pubsub --test integration_all_api -- --ignored --nocapture
//! ```

use std::time::Duration;

use contracts::KeyValueStore;
use kernel::ErrorKind;
use redisx::selfcheck::{CheckLevel, CheckStatus, RedisValidator};
use redisx::{RedisClient, RedisConfig, RedisLiveKv, RedisPool};

fn prefix() -> String {
    format!("redisx-it:{}:", std::process::id())
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_config_pool_client_lifecycle() {
    let cfg = RedisConfig::from_env().expect("FOUNDATIONX_REDISX_* 或 REDIS_URL");
    let ep = cfg.display_endpoint();
    assert!(ep.contains("***"), "password must be redacted in endpoint: {ep}");

    let pool = RedisPool::connect(cfg).await.expect("connect");
    let rtt = pool.ping().await.expect("ping");
    assert!(rtt < Duration::from_secs(3));
    assert_eq!(pool.stats().open, 1);
    assert!(pool.metrics_snapshot().commands_ok >= 1);

    let client = pool.client().with_call_deadline(Duration::from_secs(10));
    assert!(client.has_call_deadline());
    assert!(!client.endpoint().is_empty());

    let key = format!("{}lifecycle", prefix());
    client.set(&key, b"v".to_vec(), Some(Duration::from_secs(60))).await.expect("set");
    assert_eq!(client.get(&key).await.expect("get"), Some(b"v".to_vec()));
    assert!(client.exists(&key).await.expect("exists"));
    assert!(client.ttl(&key).await.expect("ttl").is_some());
    assert!(client.expire(&key, Duration::from_secs(120)).await.expect("expire"));
    assert!(client.delete(&key).await.expect("del"));

    pool.close(Duration::from_secs(3)).await.expect("close");
    assert!(pool.is_closed());
    let err = pool.client().get("x").await.expect_err("closed");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
    assert!(pool.metrics_snapshot().rejected_closed >= 1);
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_kv_bytes_mset_mget_pipeline() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let k1 = format!("{p}b1");
    let k2 = format!("{p}b2");
    let k3 = format!("{p}b3");
    let missing = format!("{p}missing");

    client.set_bytes(&k1, vec![0, 1, 255], Some(Duration::from_secs(90))).await.expect("set_bytes");
    assert_eq!(client.get_bytes(&k1).await.expect("get_bytes"), Some(vec![0, 1, 255]));

    client.mset(&[(&k2, b"two"), (&k3, b"three")]).await.expect("mset");
    let got = client.mget(&[&k2, &k3, &missing]).await.expect("mget");
    assert_eq!(got[0].as_deref(), Some(b"two".as_slice()));
    assert_eq!(got[1].as_deref(), Some(b"three".as_slice()));
    assert!(got[2].is_none());

    let p1 = format!("{p}p1");
    let p2 = format!("{p}p2");
    client
        .pipeline_set(&[(&p1, b"A".to_vec()), (&p2, b"B".to_vec())], Some(Duration::from_secs(60)))
        .await
        .expect("pipeline");
    assert_eq!(client.get(&p1).await.expect("g1"), Some(b"A".to_vec()));
    assert_eq!(client.get(&p2).await.expect("g2"), Some(b"B".to_vec()));

    for k in [&k1, &k2, &k3, &p1, &p2] {
        let _ = client.delete(k).await;
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_lua_and_dist_lock() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let p = prefix();
    let key = format!("{p}lua");
    client.set(&key, b"1".to_vec(), Some(Duration::from_secs(60))).await.expect("seed");
    let v =
        client.eval_script("return redis.call('GET', KEYS[1])", &[&key], &[]).await.expect("eval");
    match v {
        redis::Value::BulkString(b) => assert_eq!(b.as_slice(), b"1"),
        redis::Value::SimpleString(s) => assert_eq!(s.as_bytes(), b"1"),
        other => panic!("unexpected lua value {other:?}"),
    }

    let lock_key = format!("{p}lock");
    let lock = client.lock_acquire(&lock_key, Duration::from_secs(20)).await.expect("lock");
    assert!(lock.fence() >= 1);
    let conflict = client.lock_acquire(&lock_key, Duration::from_secs(5)).await;
    assert!(matches!(conflict, Err(e) if e.kind() == ErrorKind::Conflict));
    assert!(client.lock_extend(&lock, Duration::from_secs(30)).await.expect("extend"));
    assert!(client.lock_release(&lock).await.expect("release"));
    let lock2 = client.lock_acquire(&lock_key, Duration::from_secs(10)).await.expect("re");
    assert!(lock2.fence() > lock.fence());
    let _ = client.lock_release(&lock2).await;
    let _ = client.delete(&key).await;
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_contract_trait_and_invalid_ttl() {
    let store: RedisLiveKv = RedisLiveKv::connect_from_env().await.expect("connect");
    let kv: &dyn KeyValueStore = &store;
    let key = format!("{}trait", prefix());
    kv.set(&key, b"tv".to_vec(), None).await.expect("set");
    assert_eq!(kv.get(&key).await.expect("get"), Some(b"tv".to_vec()));
    let _ = store.delete(&key).await;

    let err = store.set(&key, b"x".to_vec(), Some(Duration::ZERO)).await.expect_err("ttl0");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_selfcheck_full() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let report = RedisValidator::new(client).run(CheckLevel::Full).await;
    assert_eq!(report.module, "redisx");
    assert_eq!(RedisValidator::static_catalog().len(), 11);
    for item in &report.items {
        if item.id == "redisx.full.cluster_slots" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        #[cfg(not(feature = "pubsub"))]
        if item.id == "redisx.full.pubsub" {
            assert_eq!(item.status, CheckStatus::Skipped, "{item:?}");
            continue;
        }
        assert!(
            matches!(item.status, CheckStatus::Passed | CheckStatus::Degraded),
            "unexpected {item:?}"
        );
    }
    assert!(report.passed, "failed: {:?}", report.items);
}

#[tokio::test]
#[ignore = "requires live Redis + feature pubsub"]
async fn it_pubsub_roundtrip() {
    #[cfg(feature = "pubsub")]
    {
        use futures_util::StreamExt;
        use redisx::RedisPubSub;
        use tokio::time::timeout;

        let cfg = RedisConfig::from_env().expect("cfg");
        let channel = format!("redisx-it-{}-ch", std::process::id());
        let session = RedisPubSub::connect_config(cfg, [channel.clone()]).await.expect("sub");
        let mut stream = session.into_message_stream().expect("stream");

        // 独立 publish 会话
        let pub_cfg = RedisConfig::from_env().expect("cfg2");
        let publisher =
            RedisPubSub::connect_config(pub_cfg, Vec::<String>::new()).await.expect("publisher");
        let payload = b"hello-pubsub";
        publisher.publish(&channel, payload).await.expect("publish");

        let ok = timeout(Duration::from_secs(3), async {
            while let Some(msg) = stream.next().await {
                if msg.payload.as_ref() == payload {
                    return true;
                }
            }
            false
        })
        .await
        .expect("timeout waiting pubsub");
        assert!(ok, "message not received");
    }
    #[cfg(not(feature = "pubsub"))]
    {
        panic!("run with --features pubsub");
    }
}

#[tokio::test]
#[ignore = "requires live Redis via FOUNDATIONX_REDISX_*"]
async fn it_connect_from_url_compat() {
    // 仅验证 from_env 路径已覆盖 URL；这里测 builder 显式连接
    let cfg = RedisConfig::from_env().expect("cfg");
    let pool = RedisPool::connect(cfg).await.expect("connect");
    pool.ping().await.expect("ping");
    let _ = pool.close(Duration::from_secs(2)).await;
}
