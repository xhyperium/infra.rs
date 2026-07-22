//! postgresx 最小示例：配置构造与 Debug 脱敏（无网络）。
use postgresx::PostgresConfig;

fn main() {
    // 不在示例中硬编码密码；凭据经环境注入。
    let cfg = PostgresConfig::builder()
        .host("127.0.0.1")
        .port(5432)
        .database("postgres")
        .user("postgres")
        .build()
        .expect("config");
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("127.0.0.1"), "host visible: {dbg}");
    println!("postgresx example ok host={}", cfg.host);
}
