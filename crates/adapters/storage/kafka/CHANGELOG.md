# Changelog

## [0.3.2] — 2026-07-22

### Added

- 可复现 broker conformance：未确认重建、确认后推进、checkpoint 失败重复窗口

### Changed

- 将主能力诚实命名为 `ProduceThenCheckpointCoordinator` / `Session`；旧 `Eos*` 仅保留弃用别名
- Memory/File checkpoint 改为单调提交，拒绝 offset 溢出；文件落盘补齐文件与父目录 `sync_all`
- TLS 配置在当前未实现的传输上 fail-closed，禁止静默明文降级

## [0.3.1] — 2026-07-22

### Added

- 应用层 offset 提交：`OffsetCommitStore` / `MemoryOffsetStore` / `FileOffsetStore`
- At-least-once：`AtLeastOnceConsumer` / `KafkaAtLeastOnceBus`（显式 `ack`/`commit`）
- 历史名称 `EosCoordinator` / `EosSession`（自 0.3.2 起弃用；该能力并非 EOS）
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
