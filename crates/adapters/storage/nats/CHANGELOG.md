# Changelog

## [0.3.2] — 2026-07-22

### Added

- `JetStreamConsumerConfig` / `JetStreamConsumer` / `JetStreamDelivery` 持久 pull 消费面
- 显式 `ack` / `double_ack` / `nak` / `progress` / `term` 与稳定投递元数据
- Core 无回放、JetStream 重投/确认/背压/MaxDeliver/Term broker conformance

### Security

- `JetStreamDelivery::Debug` 不输出 payload 或底层消息
- `term` 与 `max_deliver` 明确不冒充自动 DLQ

## [0.3.1] — 2026-07-22

### Added

- JetStream 薄封装：`JetStream` / `StreamConfig` / `PullConsumerConfig` / `validate_stream_name`
- TLS 策略：`TlsPolicy { Disable, Prefer, Require }` + `NatsConfig::{tls,tls_policy,jetstream}`
- 默认 TLS：loopback → Prefer；非 loopback → Require（`require_tls(true)`）
- 环境变量：`FOUNDATIONX_NATS_{TLS,TLS_POLICY,JETSTREAM}`（兼容 `FOUNDATIONX_NATSX_*`）

## [Unreleased]

### Added

- 生产默认：`NatsPool` / `NatsEventBus`（`async-nats` Core NATS）
- 配置：`FOUNDATIONX_NATS_{URL,USER,PASSWORD}`（兼容 `FOUNDATIONX_NATSX_*`）
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

### Changed

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
