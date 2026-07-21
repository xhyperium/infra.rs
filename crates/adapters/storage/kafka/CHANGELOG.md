# Changelog

## [Unreleased]

### Added

- 生产默认：`KafkaPool` / `KafkaProducer` / `KafkaConsumer` / `KafkaEventBus`（`rdkafka`）
- 配置：`FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`

### Changed

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
