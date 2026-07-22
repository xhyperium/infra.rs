//! Redis PubSub 可移植 surface suite 的真实 adapter 接线。
//!
//! 默认 ignore；本测试只证明订阅/发布入口在生产 facade 上可调用，不宣称必达。

#![cfg(feature = "pubsub")]

use bytes::Bytes;
use contract_testkit::assert_pub_sub_surface;
use redisx::{RedisConfig, RedisPubSubFacade};

#[tokio::test]
#[ignore = "需要 live Redis 与 pubsub feature"]
async fn live_pubsub_portable_surface() {
    let cfg = RedisConfig::from_env().expect("RedisConfig::from_env");
    let facade = RedisPubSubFacade::connect(cfg).await.expect("连接 Redis PubSub");
    let channel = format!("infra-contract-pubsub-{}", std::process::id());
    assert_pub_sub_surface(&facade, &channel, Bytes::from_static(b"surface"))
        .await
        .expect("可移植 PubSub surface suite");
}
