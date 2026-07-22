# adapters/storage/oss — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `ossx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `OssClient / OssConfig + sign_v1` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_OSSX_{ENDPOINT,BUCKET,ACCESS_KEY_ID,ACCESS_KEY_SECRET,REGION}` |
| S-4 | 离线测试 | PASS | cargo test -p ossx |
| S-5 | live ignore 入口 | PASS | `tests/live_object_store.rs` |
| S-6 | bench 有界 | PASS | `benches/put_get.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | multipart 基础面 | PASS | XML escaping、part/count、loopback abort/orphan 状态机 |
| S-11 | 远程传输安全 | PASS | 非 loopback HTTP fail-closed 单测 |
| S-12 | 资源硬上界 | PASS | object/buffer/error/in-flight + 多片共享总 deadline 单测 |
| S-13 | lifecycle / STS / 流式 TB 对象 | OPEN | 不在本版本完成声明 |
