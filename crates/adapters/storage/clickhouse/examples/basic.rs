//! clickhousex 最小示例：默认配置（无网络）。
use clickhousex::ClickHouseConfig;

fn main() {
    let cfg = ClickHouseConfig::default();
    println!("clickhousex example ok host={} port={}", cfg.host, cfg.http_port);
}
