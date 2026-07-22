# Changelog

## [0.3.2] — 2026-07-22

### 新增

- `JetStreamConsumerConfig` / `JetStreamConsumer` / `JetStreamDelivery` 持久 pull 消费面
- 显式 `ack` / `double_ack` / `nak` / `progress` / `term` 与稳定投递元数据
- 可配置 `command_timeout`，约束全部确认类 broker 指令
- Core 无回放、JetStream 重投/确认/背压/MaxDeliver/Term broker conformance

### 安全

- `JetStreamDelivery::Debug` 不输出 payload 或底层消息
- `term` 与 `max_deliver` 明确不冒充自动 DLQ
- 远程 Disable/Prefer、URL userinfo 与零 deadline/capacity/reconnect 配置 fail-closed

### 变更

- Core 与 JetStream 操作增加内部 deadline；subscription/client capacity 与有限 reconnect 可配置
- stats 增加 connected/disconnected/slow-consumer 事件

### 已知问题

- 同客户端跨 broker 重启恢复连续三次实验失败（命令通道关闭）；保持 NO-GO，跟踪 `infra-2d9.3.1`

## [0.3.1] — 2026-07-22

### 新增

- JetStream 薄封装：`JetStream` / `StreamConfig` / `PullConsumerConfig` / `validate_stream_name`
- TLS 策略：`TlsPolicy { Disable, Prefer, Require }` + `NatsConfig::{tls,tls_policy,jetstream}`
- 默认 TLS：loopback → Prefer；非 loopback → Require（`require_tls(true)`）
- 环境变量：`FOUNDATIONX_NATS_{TLS,TLS_POLICY,JETSTREAM}`（兼容 `FOUNDATIONX_NATSX_*`）

## [Unreleased]

### 新增

- 生产默认：`NatsPool` / `NatsEventBus`（`async-nats` Core NATS）
- 配置：`FOUNDATIONX_NATS_{URL,USER,PASSWORD}`（兼容 `FOUNDATIONX_NATSX_*`）
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

### 变更

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
