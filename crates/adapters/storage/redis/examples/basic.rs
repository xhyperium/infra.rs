//! redisx 最小示例：配置构造与 Debug 脱敏（无网络）。
use redisx::RedisConfig;

fn main() {
    // 不在示例中硬编码密码；凭据经环境注入。
    let cfg = RedisConfig::builder()
        .addr("127.0.0.1:6379")
        .username("default")
        .db(0)
        .build()
        .expect("config");
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("127.0.0.1:6379"), "endpoint visible: {dbg}");
    println!("redisx example ok {}", cfg.display_endpoint());
}
