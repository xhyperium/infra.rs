//! 简单微基准：配置构建 + OSS V1 签名（不打真实网络）。

use criterion::{Criterion, criterion_group, criterion_main};
use ossx::{OssConfig, sign_v1};

fn bench_config_and_sign(c: &mut Criterion) {
    c.bench_function("oss_config_build", |b| {
        b.iter(|| {
            let cfg = OssConfig::builder()
                .endpoint("https://oss-ap-northeast-1.aliyuncs.com")
                .bucket("bench-bucket")
                .access_key_id("AKIDEXAMPLE")
                .access_key_secret("secret-example")
                .region("ap-northeast-1")
                .build()
                .expect("cfg");
            std::hint::black_box(cfg);
        });
    });

    c.bench_function("oss_sign_v1", |b| {
        b.iter(|| {
            let sig = sign_v1(
                "secret-example",
                "PUT",
                "",
                "application/octet-stream",
                "Thu, 01 Jan 1970 00:00:00 GMT",
                "",
                "/bench-bucket/infra-draft/key",
            );
            std::hint::black_box(sig);
        });
    });
}

criterion_group!(benches, bench_config_and_sign);
criterion_main!(benches);
