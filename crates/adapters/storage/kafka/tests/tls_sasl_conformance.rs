//! 隔离 Kafka TLS + SASL/PLAIN 真实 broker conformance。
//!
//! 默认 ignored；由 `scripts/kafka-tls-sasl-conformance.mjs` 生成临时 CA/凭据并运行。

use std::path::PathBuf;
use std::time::Duration;

use bytes::Bytes;
use kafkax::{DEFAULT_SASL_MECHANISM, KafkaConfig, KafkaPool};
use kernel::ErrorKind;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("缺少测试环境变量 {name}"))
}

fn config(password: String, ca_file: PathBuf) -> KafkaConfig {
    KafkaConfig {
        brokers: required_env("INFRA_KAFKA_TLS_BROKER"),
        client_id: format!("kafkax-tls-test-{}", std::process::id()),
        sasl_mechanism: Some(DEFAULT_SASL_MECHANISM.into()),
        sasl_username: Some(required_env("INFRA_KAFKA_TLS_USERNAME")),
        sasl_password: Some(password),
        tls: true,
        tls_ca_file: Some(ca_file),
        connect_timeout: Duration::from_secs(30),
        operation_timeout: Duration::from_secs(15),
        delivery_timeout: Duration::from_secs(15),
        ..KafkaConfig::default()
    }
}

#[tokio::test]
#[ignore = "需要脚本启动隔离 TLS+SASL Kafka"]
async fn trusted_ca_and_plain_credentials_publish_to_real_broker() {
    let config = config(
        required_env("INFRA_KAFKA_TLS_PASSWORD"),
        required_env("INFRA_KAFKA_TLS_CA_FILE").into(),
    );
    assert_eq!(config.security_protocol(), "SASL_SSL");
    let pool = KafkaPool::connect(config).await.expect("TLS+PLAIN 连接真实 Kafka");
    let topic = format!("infra_tls_{}_{}", std::process::id(), unix_nanos());
    pool.ensure_topic(&topic, 1, 1).await.expect("创建 TLS topic");
    let delivery = pool
        .producer()
        .publish(&topic, Bytes::from_static(b"tls-sasl-plain"))
        .await
        .expect("经 TLS+PLAIN 发布");
    assert_eq!(delivery.partition, 0);
    assert!(delivery.offset >= 0);
}

#[tokio::test]
#[ignore = "需要脚本启动隔离 TLS+SASL Kafka"]
async fn wrong_ca_and_wrong_password_fail_closed() {
    let bad_ca = config(
        required_env("INFRA_KAFKA_TLS_PASSWORD"),
        required_env("INFRA_KAFKA_TLS_BAD_CA_FILE").into(),
    );
    let ca_error = match KafkaPool::connect(bad_ca).await {
        Err(error) => error,
        Ok(_) => panic!("错误 CA 必须拒绝"),
    };
    assert!(
        matches!(ca_error.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
        "kind={:?}",
        ca_error.kind()
    );
    assert!(std::error::Error::source(&ca_error).is_some());

    let bad_password =
        config("definitely-wrong-password".into(), required_env("INFRA_KAFKA_TLS_CA_FILE").into());
    let auth_error = match KafkaPool::connect(bad_password).await {
        Err(error) => error,
        Ok(_) => panic!("错误密码必须拒绝"),
    };
    assert!(
        matches!(auth_error.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
        "kind={:?}",
        auth_error.kind()
    );
    assert!(std::error::Error::source(&auth_error).is_some());
}

fn unix_nanos() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("系统时钟").as_nanos()
}
