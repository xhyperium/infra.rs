# adapters/storage/kafka — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `kafkax` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `KafkaPool / KafkaProducer / KafkaConsumer / KafkaEventBus` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` |
| S-4 | 离线测试 | PASS | cargo test -p kafkax |
| S-5 | live ignore 入口 | PASS | `tests/live_event_bus.rs` |
| S-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | deadline / 有界背压 / close / 脱敏错误 | PASS | `src/lifecycle.rs` + 离线失败测试 |
| S-10 | package stable | OPEN | 未宣称 |
| S-11 | DEFER 能力 | OPEN | EOS / transactional producer / schema registry / group coordinator / 自动重连强依赖路径 |
