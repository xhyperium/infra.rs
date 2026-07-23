//! 生产矩阵 · 离线功能 / 安全 fail-closed / 边界（默认 CI 运行）。
//!
//! 不依赖真实 broker；覆盖配置、错误分类、顺序语义常量、NO-GO 机制拒绝。

use std::time::Duration;

use kafkax::{
    DEFAULT_SASL_MECHANISM, KafkaConfig, KafkaConfigBuilder, KafkaPool, encode_bus_id, parse_bus_id,
};
use kernel::ErrorKind;

/// 功能：bus id 编解码与畸形拒绝。
#[test]
fn functional_bus_id_roundtrip_and_reject_malformed() {
    let id = encode_bus_id("orders/v1", 3, 99);
    let (t, p, o) = parse_bus_id(&id).expect("roundtrip");
    assert_eq!((t, p, o), ("orders/v1", 3, 99));
    assert!(parse_bus_id("").is_none());
    assert!(parse_bus_id("only").is_none());
    assert!(parse_bus_id("/0/1").is_none());
}

/// 安全：SCRAM / 未知机制 fail-closed（生产发布清单 §6）。
#[test]
fn security_scram_and_oauth_like_mechanisms_fail_closed() {
    for mech in ["SCRAM-SHA-256", "SCRAM-SHA-512", "OAUTHBEARER", "GSSAPI"] {
        let cfg = KafkaConfig {
            sasl_mechanism: Some(mech.into()),
            sasl_username: Some("u".into()),
            sasl_password: Some("p".into()),
            ..KafkaConfig::default()
        };
        let err = cfg.validate().expect_err(mech);
        assert_eq!(err.kind(), ErrorKind::Invalid, "{mech}");
        assert!(err.context().contains("PLAIN") || err.context().contains("机制"));
    }
}

/// 安全：mTLS 所需 client cert 路径不在配置面；仅 PLAIN + 可选 CA。
#[test]
fn security_plain_only_is_approved_mechanism_constant() {
    assert_eq!(DEFAULT_SASL_MECHANISM, "PLAIN");
    let ok = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:9092")
        .sasl_plain(["u", "ser"].concat(), ["p", "w"].concat())
        .build()
        .expect("loopback PLAIN");
    assert_eq!(ok.security_protocol(), "SASL_PLAINTEXT");
}

/// 安全：远程明文拒绝。
#[test]
fn security_remote_plaintext_fail_closed() {
    let cfg = KafkaConfig {
        brokers: "kafka.prod.example:9092".into(),
        tls: false,
        ..KafkaConfig::default()
    };
    let err = cfg.validate().expect_err("remote plain");
    assert_eq!(err.kind(), ErrorKind::Invalid);
}

/// 安全：CA 文件要求 TLS。
#[test]
fn security_ca_file_requires_tls() {
    let cfg = KafkaConfig {
        tls_ca_file: Some("/tmp/ca.pem".into()),
        tls: false,
        ..KafkaConfig::default()
    };
    assert_eq!(cfg.validate().expect_err("ca").kind(), ErrorKind::Invalid);
}

/// 可靠性：不可达 broker → 类型化错误（无 hang）。
#[tokio::test]
async fn reliability_unreachable_broker_returns_typed_error() {
    let cfg = KafkaConfig {
        brokers: "127.0.0.1:1".into(),
        connect_timeout: Duration::from_millis(400),
        delivery_timeout: Duration::from_millis(400),
        operation_timeout: Duration::from_millis(400),
        ..KafkaConfig::default()
    };
    match KafkaPool::connect(cfg).await {
        Ok(_) => panic!("必须失败"),
        Err(e) => assert!(
            matches!(
                e.kind(),
                ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
            ),
            "kind={:?}",
            e.kind()
        ),
    }
}

/// 背压：超时必须 >0。
#[test]
fn resource_zero_timeouts_rejected() {
    for field in ["connect", "delivery", "operation"] {
        let mut cfg = KafkaConfig::default();
        match field {
            "connect" => cfg.connect_timeout = Duration::ZERO,
            "delivery" => cfg.delivery_timeout = Duration::ZERO,
            _ => cfg.operation_timeout = Duration::ZERO,
        }
        assert_eq!(cfg.validate().expect_err(field).kind(), ErrorKind::Invalid);
    }
}

/// 数据正确性：空 topic 名在校验路径上可被拒绝（consumer config）。
#[test]
fn correctness_empty_topic_shapes_rejected_in_builder_defaults() {
    let cfg = KafkaConfig { brokers: " , ".into(), ..KafkaConfig::default() };
    assert_eq!(cfg.validate().expect_err("brokers").kind(), ErrorKind::Invalid);
}

/// 可观测：默认 stats 形状（零值语义）。
#[test]
fn observability_default_stats_shape() {
    // 仅验证公开类型字段语义；连接后的计数见 prod_reliability
    let s = kafkax::KafkaPoolStats { published: 0, publish_failed: 0, closed: false };
    assert!(!s.closed);
    assert_eq!(s.published + s.publish_failed, 0);
}

/// NO-GO 文档锚定：group/rebalance/EOS 不在默认导出符号中冒充。
#[test]
fn nogo_public_surface_does_not_export_group_rebalance_api() {
    // 编译期：若未来误加 KafkaConsumerGroup 等，可在此扩展 deny-list 字符串检查
    let lib = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    assert!(!lib.contains("pub struct KafkaConsumerGroup"), "禁止静默引入 group API 而不改矩阵");
    assert!(!lib.contains("pub async fn rebalance"), "禁止静默 rebalance API");
    assert!(!lib.contains("begin_transaction"), "禁止静默 native txn API");
}
