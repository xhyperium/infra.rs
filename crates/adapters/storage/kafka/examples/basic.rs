//! kafkax 最小示例：from_env 默认构造（无网络）。
use kafkax::KafkaConfig;

fn main() {
    let cfg = KafkaConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("brokers") || dbg.contains("KafkaConfig"));
    println!("kafkax example ok brokers={:?}", cfg.brokers);
}
