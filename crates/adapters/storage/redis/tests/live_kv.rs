//! 生产 Redis 客户端 live 验证（默认 ignore）。
//!
//! ```bash
//! export FOUNDATIONX_REDISX_ADDR=127.0.0.1:6379
//! export FOUNDATIONX_REDISX_USERNAME=default
//! export FOUNDATIONX_REDISX_PASSWORD=...
//! export FOUNDATIONX_REDISX_DB=0
//! export FOUNDATIONX_REDISX_TLS=false
//! cargo test -p redisx --test live_kv -- --ignored
//! # 或 REDIS_URL=redis://:pass@127.0.0.1:6379/0
//! ```

use std::time::Duration;

use contracts::KeyValueStore;
use kernel::ErrorKind;
use redisx::{RedisClient, RedisConfig, RedisLiveKv, RedisPool};

async fn connect_pool() -> RedisPool {
    RedisPool::connect(RedisConfig::from_env().expect("config from env"))
        .await
        .expect("redis pool connect")
}

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_pool_ping_and_stats() {
    let pool = connect_pool().await;
    let rtt = pool.ping().await.expect("ping");
    assert!(rtt < Duration::from_secs(5));
    let st = pool.stats();
    assert_eq!(st.open, 1);
    assert_eq!(st.in_flight, 0);
    pool.close(Duration::from_secs(2)).await.expect("close");
    assert!(pool.is_closed());
    let err = pool.ping().await.expect_err("closed");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
}

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_client_kv_extensions() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let prefix = format!("redisx-live:{}:", std::process::id());
    let key = format!("{prefix}k");
    let missing = format!("{prefix}missing");

    assert!(!client.exists(&missing).await.expect("exists"));
    assert!(client.get(&missing).await.expect("get").is_none());

    client.set(&key, b"hello".to_vec(), Some(Duration::from_secs(60))).await.expect("set");
    assert_eq!(client.get(&key).await.expect("get2"), Some(b"hello".to_vec()));
    assert!(client.exists(&key).await.expect("exists2"));

    let ttl = client.ttl(&key).await.expect("ttl");
    assert!(ttl.is_some());
    assert!(ttl.unwrap() <= Duration::from_secs(60));

    client.expire(&key, Duration::from_secs(120)).await.expect("expire");

    // binary
    let bin = format!("{prefix}bin");
    client.set(&bin, vec![0, 1, 2, 255], None).await.expect("bin set");
    assert_eq!(client.get(&bin).await.expect("bin get"), Some(vec![0, 1, 2, 255]));

    // mset / mget
    let a = format!("{prefix}a");
    let b = format!("{prefix}b");
    client.mset(&[(&a, b"1"), (&b, b"2")]).await.expect("mset");
    let got = client.mget(&[&a, &b, &missing]).await.expect("mget");
    assert_eq!(got[0], Some(b"1".to_vec()));
    assert_eq!(got[1], Some(b"2".to_vec()));
    assert_eq!(got[2], None);

    assert!(client.delete(&key).await.expect("del"));
    assert!(!client.delete(&key).await.expect("del2"));
}

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_ttl_zero_rejected() {
    let client = RedisClient::connect_from_env().await.expect("connect");
    let key = format!("redisx-live-ttl0:{}", std::process::id());
    let err = client.set(&key, b"x".to_vec(), Some(Duration::ZERO)).await.expect_err("ttl 0");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_key_value_store_trait() {
    let store: RedisLiveKv = RedisLiveKv::connect_from_env().await.expect("connect");
    let kv: &dyn KeyValueStore = &store;
    let key = format!("redisx-live-trait:{}", std::process::id());
    kv.set(&key, b"v".to_vec(), None).await.expect("set");
    assert_eq!(kv.get(&key).await.expect("get"), Some(b"v".to_vec()));
}

#[tokio::test]
#[ignore = "requires live Redis (FOUNDATIONX_REDISX_* or REDIS_URL)"]
async fn live_close_rejects_new_ops() {
    let pool = connect_pool().await;
    let client = pool.client();
    pool.close(Duration::from_secs(2)).await.expect("close");
    let err = client.get("any").await.expect_err("must reject after close");
    assert_eq!(err.kind(), ErrorKind::Unavailable);
}
