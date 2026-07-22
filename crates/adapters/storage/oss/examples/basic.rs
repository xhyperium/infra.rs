//! ossx 最小示例：builder 校验（无网络）。
use ossx::OssConfig;

fn main() {
    let cfg = OssConfig::builder()
        .endpoint("https://oss-ap-northeast-1.aliyuncs.com")
        .bucket("demo")
        .access_key_id("AKIDEMO")
        .access_key_secret("secret")
        .region("ap-northeast-1")
        .build()
        .expect("config");
    println!("ossx example ok bucket={}", cfg.bucket);
}
