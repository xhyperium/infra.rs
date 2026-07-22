//! redisx 最小示例：配置构造与 Debug 脱敏（无网络）。
use redisx::RedisConfig;

fn main() {
    let secret = ["super", "-", "secret"].concat();
    let cfg = RedisConfig::builder()
        .addr("127.0.0.1:6379")
        .username("default")
        .password(secret.clone())
        .db(0)
        .build()
        .expect("config");
    let dbg = format!("{cfg:?}");
    assert!(!dbg.contains(&secret), "password must be redacted in Debug: {dbg}");
    println!("redisx example ok {}", cfg.display_endpoint());
}
