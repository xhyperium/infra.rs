# adapters/storage/redis — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `redisx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `RedisPool / RedisClient / RedisConfig` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `ADDR,USERNAME,PASSWORD,DB,TLS,MODE,NODES,SENTINEL_MASTER` |
| S-4 | 离线测试 | PASS | cargo test -p redisx |
| S-5 | live ignore 入口 | PASS | `tests/live_kv.rs · tests/live_kv_conformance.rs` |
| S-6 | bench 有界 | PASS | `benches/kv_hot_path.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | Cluster 命令路径 | OPEN | 代码/离线失败测试存在；真实 Cluster live 未运行 |
| S-11 | Sentinel 命令路径 | OPEN | 代码/离线失败测试存在；真实 Sentinel/failover 未运行 |
| S-12 | TLS 安全路径 | OPEN | secure 构造测试存在；真实 TLS 握手未运行 |
| S-13 | Pub/Sub 配置同源 | PASS | 复用建池 config；Cluster/Sentinel 失败关闭测试 |
| S-14 | 重试/原子性合同 | PASS | client 参数细分：ReadOnly、无 TTL SET/MSET Idempotent、相对 TTL SET/DEL/PEXPIRE UnsafeSideEffect、PUBLISH NeverAutomatic |
