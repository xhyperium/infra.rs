//! NATS 驱动重连、订阅恢复与慢消费者上界的真实 broker 实验。
//!
//! 默认 ignored；由 `scripts/nats-reconnect-conformance.mjs` 启动固定镜像并提供容器名。

use std::process::Command;
use std::time::{Duration, Instant};

use bytes::Bytes;
use natsx::{NatsConfig, NatsPool};

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("缺少测试环境变量 {name}"))
}

async fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("条件未在 {timeout:?} 内满足");
}

#[tokio::test]
#[ignore = "需要脚本启动并重启隔离 NATS 容器"]
async fn reconnect_restores_subscription_and_slow_consumer_is_observable() {
    let url = required_env("INFRA_NATS_RECONNECT_URL");
    let container = required_env("INFRA_NATS_RECONNECT_CONTAINER");
    let config = NatsConfig {
        url,
        connect_timeout: Duration::from_secs(5),
        operation_timeout: Duration::from_secs(5),
        subscription_capacity: 1,
        client_capacity: 32,
        max_reconnects: 40,
        reconnect_max_delay: Duration::from_millis(250),
        ignore_discovered_servers: true,
        name: format!("nats-reconnect-test-{}", std::process::id()),
        ..NatsConfig::default()
    };
    let pool = NatsPool::connect(config).await.expect("连接隔离 NATS");
    let subject = format!("infra.reconnect.{}", std::process::id());
    let mut subscription = pool.subscribe(&subject).await.expect("订阅");
    pool.publish(&subject, Bytes::from_static(b"before")).await.expect("重启前发布");
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), subscription.recv())
            .await
            .expect("重启前接收不得超时")
            .expect("重启前消息")
            .payload,
        Bytes::from_static(b"before")
    );

    let restart = tokio::task::spawn_blocking(move || {
        Command::new("timeout")
            .args(["--signal=TERM", "--kill-after=5s", "30s", "docker", "restart", &container])
            .output()
    });
    let output = tokio::time::timeout(Duration::from_secs(35), restart)
        .await
        .expect("docker restart 任务不得超时")
        .expect("docker restart join")
        .expect("执行 docker restart");
    assert!(
        output.status.success(),
        "docker restart 失败: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    wait_until(Duration::from_secs(30), || {
        let stats = pool.stats();
        stats.disconnected >= 1 && stats.connected >= 2
    })
    .await;

    let publish_start = Instant::now();
    loop {
        match pool.publish(&subject, Bytes::from_static(b"after")).await {
            Ok(()) => break,
            Err(_) if publish_start.elapsed() < Duration::from_secs(30) => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Err(error) => panic!("预算内重连后发布仍失败: {error}"),
        }
    }
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(10), subscription.recv())
            .await
            .expect("重连后接收不得超时")
            .expect("重连后订阅应恢复")
            .payload,
        Bytes::from_static(b"after")
    );

    // 不再消费一个容量为 1 的转发队列，大量发布迫使驱动报告 SlowConsumer。
    let raw = pool.client();
    for sequence in 0..256u32 {
        raw.publish(subject.clone(), Bytes::from(sequence.to_be_bytes().to_vec()))
            .await
            .expect("填充慢消费者队列");
    }
    raw.flush().await.expect("flush 慢消费者样本");
    wait_until(Duration::from_secs(10), || pool.stats().slow_consumers >= 1).await;

    let stats = pool.stats();
    assert!(stats.disconnected >= 1);
    assert!(stats.connected >= 2);
    assert!(stats.slow_consumers >= 1);
    pool.close().await.expect("关闭 NATS pool");
}
