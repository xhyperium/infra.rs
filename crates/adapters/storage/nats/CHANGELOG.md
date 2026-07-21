# Changelog

## [Unreleased]

### Added

- 生产默认：`NatsPool` / `NatsEventBus`（`async-nats` Core NATS）
- 配置：`FOUNDATIONX_NATS_{URL,USER,PASSWORD}`（兼容 `FOUNDATIONX_NATSX_*`）
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

### Changed

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
