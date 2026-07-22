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

- 同客户端跨 broker 重启、原 Core subscription 恢复与慢消费者观测已连续 3/3 通过；`infra-2d9.3.1` 已关闭
- 超过有限 `max_reconnects` 后命令通道仍会关闭，调用方必须重建 client；Core 断线窗口无回放，Cluster/HA 仍为 NO-GO

## [0.3.1] — 2026-07-22

### 新增

- JetStream 薄封装：`JetStream` / `StreamConfig` / `PullConsumerConfig` / `validate_stream_name`
- TLS 策略：`TlsPolicy { Disable, Prefer, Require }` + `NatsConfig::{tls,tls_policy,jetstream}`
- 默认 TLS：loopback → Prefer；非 loopback → Require（`require_tls(true)`）
- 环境变量：`FOUNDATIONX_NATS_{TLS,TLS_POLICY,JETSTREAM}`（兼容 `FOUNDATIONX_NATSX_*`）

## [Unreleased]

### 修复

- 修正首次 `Connected` 事件的重复计数
- 重连 conformance 保持动态 host ingress 不变，连续验证 broker 进程重启后的发布与原 Core subscription 恢复
- pool 保留并等待订阅转发任务；任务 panic 不再被静默丢弃

### 新增

- 生产默认：`NatsPool` / `NatsEventBus`（`async-nats` Core NATS）
- 配置：`FOUNDATIONX_NATS_{URL,USER,PASSWORD}`（兼容 `FOUNDATIONX_NATSX_*`）
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

### 变更

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
