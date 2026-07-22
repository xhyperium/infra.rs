# adapters/storage/redis — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `redisx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `RedisPool / RedisClient / RedisConfig` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_REDISX_{ADDR,USERNAME,PASSWORD,DB,TLS}` |
| S-4 | 离线测试 | PASS | cargo test -p redisx |
| S-5 | live ignore 入口 | PASS | `tests/live_kv.rs · tests/live_kv_conformance.rs` |
| S-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | DEFER 能力 | OPEN | Cluster / Sentinel / Streams full / pubsub 默认关闭 |
