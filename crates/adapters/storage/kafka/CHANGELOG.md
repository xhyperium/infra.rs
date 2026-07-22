# Changelog

## [0.3.1] — 2026-07-22

### Added

- 应用层 offset 提交：`OffsetCommitStore` / `MemoryOffsetStore` / `FileOffsetStore`
- At-least-once：`AtLeastOnceConsumer` / `KafkaAtLeastOnceBus`（显式 `ack`/`commit`）
- 应用级 EOS：`EosCoordinator` / `EosSession`（produce 成功后才允许 commit；fail-closed）
- `ConsumerConfig::start_offset` / `with_start_offset` / `resolve_start_offset`（`StartOffset::At`）

### Notes

- rskafka 无 group coordinator / transactional producer；本版本以应用层语义闭环 DEFER

## [Unreleased]

### Added

- 生产默认：`KafkaPool` / `KafkaProducer` / `KafkaConsumer` / `KafkaEventBus`（`rskafka`）
- 配置：`FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`

### Changed

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
