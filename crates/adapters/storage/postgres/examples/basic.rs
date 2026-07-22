//! postgresx 最小示例：配置构造与 Debug 脱敏（无网络）。
use postgresx::PostgresConfig;

fn main() {
    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(5432)
        .database("postgres")
        .user("postgres")
        .password("super-secret")
        .build()
        .expect("config");
    let dbg = format!("{cfg:?}");
    assert!(!dbg.contains("super-secret"), "password must be redacted in Debug: {dbg}");
    println!("postgresx example ok host={}", cfg.host);
}
