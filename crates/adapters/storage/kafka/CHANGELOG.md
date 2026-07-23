# Changelog

## [0.4.0] — 2026-07-23

### Milestone（0.3.x → 0.4.0 里程碑闭合）

kafkax 自 0.3.0 初始生产默认落地以来累积 9 次 PATCH 迭代，可交付面 100% 闭合：

- 十轮 draft 矩阵（R1–R10）全部收敛；不可交付面 100% 显式 NO-GO/OOS
- 完整的生产功能面：Producer pool、分区消费、headers/key、stats
- 可靠应用层语义：ALO（AtLeastOnceConsumer）、PTC（ProduceThenCheckpointCoordinator）
- 安全边界：TLS+CA、SASL/PLAIN、远程明文 fail-closed、错误脱敏、凭据 Debug 屏蔽
- 门禁与证据：110+ 直接测试（lib + offline + conformance + live + e2e + prod 矩阵 + benchmarks）
- gap 清零（0.3.7）：headers/key 公共面、stats 扩展、全量 API 测试
- G-STATS-01 严格计数（0.3.9）：produce cancel/timeout 统计
- skeptic 行为测试（0.3.8）：ALO/with_group/with_config/stats

### Boundary

- NO-GO 不变：group / rebalance / 自动重连 / native EOS / schema registry / SCRAM / OAuth / mTLS / package stable
- OOS：draft Part2 量化栈（embedded broker / io-uring / µs 热路径）
- package stable 未宣称

## [0.3.9] — 2026-07-23

### Fixed（G-STATS-01 严格：produce cancel/timeout 计数）

- 抽出 shipped `limited_produce_await` / `apply_limited_produce_outcome`，`publish_record` 共用
- 单测严格断言：timeout 臂 → `publish_timeouts+`；cancel 臂 → `publish_cancelled+`（禁止 OR published/failed）
- 集成：`stats_cancel_or_timeout_increment_on_close_during_publish` 改为严格 cancelled|timeouts；matrix 注入 `KAFKAX_DOCKER_CONTAINER` 支持 pause 超时路径

## [0.3.8] — 2026-07-23

### Fixed（skeptic：pub API 行为测试与 stats 路径）

- AtLeastOnceConsumer：nack 保留 pending、drop pending 并终止会话、is_terminated；离线 unit + live
- `KafkaEventBus::with_group` 构造与 live publish 路径
- `KafkaValidator::with_config` Basic skip 行为（离线 + live）
- `publish_timeouts` / `publish_cancelled` 与 `record_*` 同源单测 + 关闭竞态集成
- 完整 `kafkax-api-coverage` 映射（禁止 type_name 剧场）

### Boundary

- NO-GO 不变；package stable 未宣称

## [0.3.7] — 2026-07-23

### Added（gap 清零：headers / key / stats）

- `PublishRecord` + `KafkaProducer::publish_record` / `publish_with_key`
- `KafkaMessage::headers` + `header()`；消费路径透传 rskafka headers
- `partition_for_key`：应用层稳定 key→partition 辅助
- `KafkaPoolStats`：`publish_timeouts` / `publish_cancelled` / `topics_ensured` / `topics_deleted`
- 测试：`tests/api_surface_offline.rs`、`tests/e2e_api_roundtrip.rs`；selfcheck 走公共 produce 路径
- FileOffset e2e 使用 `/home/workspace/data/kafkax-gap-zero-*`

### Closed

- G-API-01/02 headers+key 公共面 → **PASS**
- G-OBS-01 stats 扩展 → **PASS**
- G-SELF-01 ordering_headers 同源 + header 透传 → **PASS**
- NO-GO 仍 CLOSED（group/rebalance/EOS/SCRAM/HA/stable…）

### Evidence

- `cargo test -p kafkax --all-targets` 离线全绿
- 隔离 broker：`e2e_api_roundtrip` + `live_selfcheck` Full

[0.3.6] — 2026-07-23

### Added（LIB-SELFCHECK-SPEC §6.2 库内自验证）

- `kafkax::selfcheck`：`CheckLevel` / `CheckStatus` / `CheckItem` / `ValidationReport` / `KafkaValidator`
- catalog 9 项：`metadata` / `produce_consume` / Full 子集 + `group_lag`/`isr_health` **Skipped(NO-GO)**
- `KafkaPool::delete_topic`：Admin 删 topic（自检清理 / Full DDL）
- `KafkaValidator::connect_and_run`：连接失败合成报告（不 panic；Basic Failed → RW/Full 短路）
- 离线单测 + `tests/live_selfcheck.rs`（live `#[ignore]`）

### Boundary

- **库内自验证 ≠ `tools/verifyctl` Goal Contract CLI**
- 不宣称 multi-module `SelfValidator` / package stable
- headers 公共面 partial；offset 为应用层语义

### Evidence

- `cargo test -p kafkax --lib selfcheck` + offline `connect_and_run` 全绿
- 默认 CI 无 broker 时 live 诚实 ignore；有 broker 时 `--ignored` 跑 RW/Full

## [0.3.5] — 2026-07-23

### Added（生产发布测试矩阵）

- `tests/prod_offline.rs`：离线功能/安全 fail-closed/不可达 broker/NO-GO 锚定
- `tests/prod_reliability.rs`：集成可靠性（顺序、checksum、1MiB、突发并发、ALO 恢复、close、stats、可选 soak、故障连接）
- `scripts/kafka-prod-matrix.mjs`：隔离 broker runner + `--fault-restart` / `--soak`
- `benches/hot_path.rs`：100B/1KiB/1MiB produce 与 p50/p95/p99 摘要
- `docs/测试矩阵-生产发布.md`：清单对照（PASS/NO-GO/OOS）

### Evidence

- 默认 CI：`prod_offline` + lib 全绿
- 隔离：`node scripts/kafka-prod-matrix.mjs --fault-restart` 主场景 + 停机/重建 PASS
- 24h soak **非**默认门禁（`KAFKAX_SOAK_SECONDS` 可选）
- group/rebalance/native EOS/HA/package stable **仍 NO-GO**

## [0.3.4] — 2026-07-23

### Added

- `KafkaConfigBuilder`：链式配置 + `build()` 校验（对齐 draft 公共对象）
- `KafkaMessage::timestamp`：消费侧透传 record 时间戳
- 公共 API 行为路径测试（builder / offset / PTC session / connect 拒绝）
- 十轮 draft 对照矩阵：`.agents/ssot/adapters/storage/kafka/evidence/kafkax-10pass-matrix.md`

### Evidence

- 真 secrets live：`live_event_bus` **3/3 PASS**
- 隔离 broker conformance **3/3 PASS**
- Part2 量化栈显式 **OOS**；group/rebalance/native EOS 仍 **NO-GO**
- **未**宣称 package stable

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
