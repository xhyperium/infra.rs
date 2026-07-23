# adapters/storage/kafka — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `kafkax` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `KafkaPool / KafkaProducer / KafkaConsumer / KafkaEventBus / KafkaConfigBuilder` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` |
| S-4 | 离线测试 | PASS | cargo test -p kafkax |
| S-5 | live ignore 入口 | PASS | `tests/live_event_bus.rs`（真 secrets 本会话 3/3） |
| S-6 | bench 有界 | PASS | `benches/hot_path.rs` |
| S-7 | crate docs | PASS | docs/usage·config·operations·标准 |
| S-8 | SSOT 11 层 + dual spec | PASS | 本树 + `cmp` dual |
| S-9 | deadline / 有界背压 / close / 脱敏错误 | PASS | lifecycle + 离线失败测试 |
| S-10 | package stable | **NO-GO** | 未宣称 |
| S-11 | group / rebalance / 自动重连 / native EOS / schema registry / SCRAM·OAuth·mTLS | **NO-GO** | rskafka 边界；见 evidence/kafkax-10pass-matrix.md |
| S-12 | draft Part2 量化栈 | **OOS** | embedded / io-uring / µs 热路径等不在本仓 |
| S-13 | 十轮 draft 对照收敛 | PASS | `evidence/kafkax-10pass-matrix.md` |
