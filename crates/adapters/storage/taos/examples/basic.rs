//! taosx 最小示例：默认 REST 配置（无网络）。
use taosx::TaosConfig;

fn main() {
    let cfg = TaosConfig::default();
    println!("taosx example ok rest={}", cfg.rest_sql_url());
}
