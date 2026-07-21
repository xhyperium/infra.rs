//! first-batch KeyValueStore 合同在生产 [`RedisClient`] 上的验证。
//!
//! 默认 ignore；有 Redis 时：
//! ```bash
//! cargo test -p redisx --test live_kv_conformance -- --ignored
//! ```

use std::time::Duration;

use contracts::KeyValueStore;
use redisx::{RedisConfig, RedisLiveKv, RedisPool};

async fn connect() -> RedisLiveKv {
    let cfg = RedisConfig::from_env().expect("RedisConfig::from_env");
    RedisPool::connect(cfg).await.expect("redis connect").client()
}

#[tokio::test]
#[ignore = "requires live Redis (infra-s9t.3 KV L3 subset)"]
async fn live_key_value_store_trait_get_set_missing() {
    let store = connect().await;
    let kv: &dyn KeyValueStore = &store;
    let prefix = format!("infra-s9t3:{}:", std::process::id());
    let missing = format!("{prefix}missing");
    let key = format!("{prefix}k");

    assert!(kv.get(&missing).await.expect("get").is_none());
    kv.set(&key, b"hello".to_vec(), Some(Duration::from_secs(60))).await.expect("set");
    let v = kv.get(&key).await.expect("get2").expect("hit");
    assert_eq!(v, b"hello");

    // overwrite
    kv.set(&key, b"world".to_vec(), None).await.expect("overwrite");
    assert_eq!(kv.get(&key).await.unwrap().unwrap(), b"world");
}

#[tokio::test]
#[ignore = "requires live Redis (infra-s9t.3 KV L3 subset)"]
async fn live_key_value_store_isolation() {
    let store = connect().await;
    let kv: &dyn KeyValueStore = &store;
    let prefix = format!("infra-s9t3-iso:{}:", std::process::id());
    let a = format!("{prefix}a");
    let b = format!("{prefix}b");
    kv.set(&a, b"1".to_vec(), None).await.unwrap();
    kv.set(&b, b"2".to_vec(), None).await.unwrap();
    assert_eq!(kv.get(&a).await.unwrap().unwrap(), b"1");
    assert_eq!(kv.get(&b).await.unwrap().unwrap(), b"2");
}
