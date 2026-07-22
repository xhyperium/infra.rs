# adapters/storage/nats — Matrix

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| S-1 | workspace member `natsx` | PASS | Cargo.toml |
| S-2 | 生产默认导出 | PASS | `NatsPool / NatsEventBus / NatsSubscription` |
| S-3 | from_env / FOUNDATIONX_* | PASS | `FOUNDATIONX_NATS_{URL,USER,PASSWORD} 或 FOUNDATIONX_NATSX_*` |
| S-4 | 离线测试 | PASS | cargo test -p natsx |
| S-5 | live ignore 入口 | PASS | `tests/live_event_bus.rs` |
| S-6 | bench 有界 | PASS | `benches/hot_path.rs（3s 有界）` |
| S-7 | crate docs | PASS | docs/usage·config·operations |
| S-8 | SSOT 11 层 + landing | PASS | 本树 |
| S-9 | package stable | OPEN | 未宣称 |
| S-10 | DEFER 能力 | OPEN | JetStream 全量 / NKey / TLS 默认开启策略 |
