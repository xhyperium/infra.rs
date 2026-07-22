//! natsx 最小示例：默认配置（无网络）。
use natsx::NatsConfig;

fn main() {
    let cfg = NatsConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(!dbg.to_lowercase().contains("password: \"") || dbg.contains("***"));
    println!("natsx example ok url={}", cfg.url);
}
