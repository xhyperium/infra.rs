//! bootstrap → 正式 contracts → 真实 Redis/NATS 的无生产密钥 E2E。
//!
//! 默认 ignored；必须由 `scripts/storage-composition-conformance.mjs` 启动固定镜像后执行。
//! 两个槽位是彼此独立的资源，本测试不声明跨资源事务。

use std::sync::Arc;
use std::time::Duration;

use bootstrap::{Bootstrap, ContractStoreSet};
use bytes::Bytes;
use contracts::{EventBus, KeyValueStore};
use futures_util::StreamExt;
use natsx::{NatsConfig, NatsEventBus, NatsPool};
use redisx::RedisClient;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("缺少测试环境变量 {name}"))
}

#[tokio::test]
#[ignore = "需要脚本启动隔离 Redis/NATS 容器"]
async fn real_storage_contracts_are_callable_through_bootstrap() {
    let redis_url = required_env("INFRA_BOOTSTRAP_E2E_REDIS_URL");
    let nats_url = required_env("INFRA_BOOTSTRAP_E2E_NATS_URL");

    let redis = RedisClient::connect(&redis_url).await.expect("连接隔离 Redis");
    let nats_config = NatsConfig {
        url: nats_url,
        connect_timeout: Duration::from_secs(5),
        operation_timeout: Duration::from_secs(5),
        subscription_capacity: 16,
        client_capacity: 32,
        max_reconnects: 8,
        reconnect_max_delay: Duration::from_millis(500),
        name: format!("bootstrap-e2e-{}", std::process::id()),
        ..NatsConfig::default()
    };
    let nats_pool = NatsPool::connect(nats_config).await.expect("连接隔离 NATS");
    let bus = NatsEventBus::new(nats_pool.clone());

    let stores = ContractStoreSet::new()
        .with_kv(Arc::new(redis.clone()) as Arc<dyn KeyValueStore>)
        .with_event_bus(Arc::new(bus) as Arc<dyn EventBus>);
    let app = Bootstrap::new().with_contract_store_set(stores).try_build_app().expect("bootstrap");

    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("系统时钟")
            .as_nanos()
    );
    let key = format!("infra:bootstrap:e2e:{suffix}");
    let topic = format!("infra.bootstrap.e2e.{suffix}");
    let payload = Bytes::from_static(b"formal-contract-path");

    let contracts = app.context().contract_store_set();
    contracts
        .kv()
        .expect("正式 KV contract")
        .set(&key, payload.to_vec(), Some(Duration::from_secs(30)))
        .await
        .expect("经 bootstrap 写 Redis");
    assert_eq!(
        contracts.kv().expect("正式 KV contract").get(&key).await.expect("经 bootstrap 读 Redis"),
        Some(payload.to_vec())
    );

    let mut subscription = contracts
        .event_bus()
        .expect("正式 EventBus contract")
        .subscribe(&topic)
        .await
        .expect("经 bootstrap 订阅 NATS");
    contracts
        .event_bus()
        .expect("正式 EventBus contract")
        .publish(&topic, payload.clone())
        .await
        .expect("经 bootstrap 发布 NATS");
    let message = tokio::time::timeout(Duration::from_secs(5), subscription.next())
        .await
        .expect("等待 NATS 消息不得超时")
        .expect("NATS stream 不得提前结束");
    assert_eq!(message.payload, payload);

    let signal = app.context().shutdown_signal().clone();
    app.trigger_shutdown();
    assert!(signal.is_triggered());

    redis.pool().close(Duration::from_secs(5)).await.expect("关闭 Redis pool");
    nats_pool.close().await.expect("关闭 NATS pool");
}
