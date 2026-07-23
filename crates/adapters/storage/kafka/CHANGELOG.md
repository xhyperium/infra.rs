# Changelog

## [0.3.3] — 2026-07-23

### Added（R1–R3 生产路径 fail-closed 与可测性）

- 抽取 `validate_consumer_config`：空 topic / 负 partition 在 broker I/O **前**拒绝，并有离线单测
- 抽取 `generate_anonymous_group_id`：EventBus 匿名 group 嵌入 prefix/seq/pid，单测锁定格式与序号隔离
- 抽取 `is_topic_already_exists_error`：`create_topic` 幂等分类可离线覆盖，避免鉴权/网络失败误判为已存在

### Boundaries

- native group EOS、事务 exactly-once、Schema Registry **仍 NO-GO**
- 本轮未宣称 package stable / 真实集群 TLS 全量 live 闭合

## [0.3.2] — 2026-07-22

### 新增

- 可复现 broker conformance：未确认重建、确认后推进、checkpoint 失败重复窗口

### 变更

- 将主能力诚实命名为 `ProduceThenCheckpointCoordinator` / `Session`；旧 `Eos*` 仅保留弃用别名
- Memory/File checkpoint 改为单调提交，拒绝 offset 溢出；文件落盘补齐文件与父目录 `sync_all`
- TLS 接入 rskafka rustls transport；支持 webpki roots 与可选 PEM CA
- 远程明文、未知 SASL 机制与不完整凭据在连接前 fail-closed；SASL 仅支持 PLAIN
- connect/metadata/topic/partition client 增加内部 deadline
- 固定摘要 SASL_SSL 实验覆盖成功路径、错误 CA 与错误密码

## [0.3.1] — 2026-07-22

### 新增

- 应用层 offset 提交：`OffsetCommitStore` / `MemoryOffsetStore` / `FileOffsetStore`
- At-least-once：`AtLeastOnceConsumer` / `KafkaAtLeastOnceBus`（显式 `ack`/`commit`）
- 历史名称 `EosCoordinator` / `EosSession`（自 0.3.2 起弃用；该能力并非 EOS）
- `ConsumerConfig::start_offset` / `with_start_offset` / `resolve_start_offset`（`StartOffset::At`）

### 说明

- rskafka 无 group coordinator / transactional producer；本版本以应用层语义闭环 DEFER

## [Unreleased]

### 新增

- 生产默认：`KafkaPool` / `KafkaProducer` / `KafkaConsumer` / `KafkaEventBus`（`rskafka`）
- 配置：`FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}`
- live 测试 `tests/live_event_bus.rs`；bench `benches/hot_path.rs`
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`

### 变更

- 收敛到 `xhyper-contracts::EventBus`（移除本地 StorageAdapter）
