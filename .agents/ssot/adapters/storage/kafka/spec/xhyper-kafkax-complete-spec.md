# kafkax 实现规范

状态：当前 `0.3.2` 实现合同（纯 Rust `rskafka 0.6` 默认真实路径；broker conformance 为显式运行的 `#[ignore]` 测试）。**未宣称 package stable。**

## 0. 权威与范围

裁定顺序为 Constitution → 组织 Rust 规范 → 本文 → 代码与可复验证据。规格中的“完成”只描述本文列出的合同，不等于 workspace Production Ready。

`kafkax` 位于 `crates/adapters/storage/kafka`。默认构建始终包含真实 `KafkaPool`、producer 和手动分区 consumer；旧内存实现仅在 `scaffold` feature 下导出。不存在 `real` feature，也不依赖 `rdkafka`。

## 1. 交付语义矩阵

| 入口 | 当前语义 | ack/checkpoint 所有权 | 明确非目标 |
|---|---|---|---|
| `KafkaEventBus` / `contracts::EventBus` | at-most-once facade | 无 ack handle；调用方不能要求重投 | 不得称 ALO/EOS |
| `KafkaConsumer` | 手动分区、进程内流 | 无 broker group commit | group coordinator、rebalance、自动恢复 |
| `AtLeastOnceConsumer` | 应用级、单 owner、单分区 ALO building block | pending 成功处理后显式 `ack`，应用 store 保存 next-to-read | multi-owner、lease/fencing、Kafka group offset |
| `ProduceThenCheckpointCoordinator` | 非原子 produce → checkpoint | produce 成功后才推进应用 checkpoint | Kafka transaction、幂等 producer、exactly-once |

`ProduceThenCheckpointCoordinator` 的关键失败窗口是：produce 已被 broker 接受，但 checkpoint 写入失败。重启/重试会再次 produce，形成可观察重复；调用方必须使用稳定幂等键。旧 `EosCoordinator` / `EosSession` 仅为 deprecated 源码兼容别名，名称不构成 EOS 能力声明。

## 2. Checkpoint 合同

- `OffsetCommitStore::commit(topic, partition, delivered_offset)` 保存 `delivered_offset + 1`。
- 内置 Memory/File store 在同一临界区取 `max(old, next)`：旧值重放幂等，checkpoint 不回退。
- 负 offset 与 `i64::MAX` 溢出明确返回 `Invalid`，不得使用饱和运算伪报成功。
- `AtLeastOnceConsumer` 同一时刻只持有一条 pending；未 ack 即销毁时 store 不前进，重建从旧 checkpoint 重投。
- 任意 offset 的 `AtLeastOnceConsumer::commit` 与手工 produce-result 标记是 deprecated 高级兼容面，普通生产路径分别使用 `ack` 与 `produce_then_commit`。

`FileOffsetStore` 采用同目录临时文件 `write_all → sync_all → rename → 父目录 sync_all`。该合同只覆盖单进程、单实例以及受测文件系统的 crash-consistency；不承诺多进程原子、跨文件系统 rename、掉电绝对持久、lease 或 fencing。

## 3. 安全与运维边界

- TLS 已接入 `rskafka` rustls transport；使用 webpki roots，并可通过 PEM `tls_ca_file` 追加 CA。
- SASL 仅批准 PLAIN；机制、用户名或密码不完整时在连接前 `Invalid` fail-closed，密码 `Debug` 脱敏。
- 远程 broker 必须 TLS；明文只允许 loopback，禁止 `tls=false` 静默连接远程地址。
- connect、metadata、topic 创建与 partition client 获取均有内部非零 deadline。
- topic/partition 生命周期由部署方管理；库的测试仅创建唯一单分区 topic。
- consumer fetch 错误会结束当前流；本版不承诺自动重连、rebalance、poison/DLQ、schema registry、HA 或 multi-owner。
- 单节点容器测试不能作为上述能力的证据。

## 4. 验证与证据

离线门禁：

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
node scripts/kafka-tls-sasl-conformance.mjs
```

可复现 broker conformance：

```bash
node scripts/broker-conformance.mjs
```

Kafka 场景必须证明：

1. AMO EventBus 不回放订阅建立前的消息；
2. 未 ack 后重建读取相同 offset；
3. ack 后重建读取下一 offset；
4. produce 成功但 checkpoint 失败后重试可产生重复；
5. 唯一 topic、cargo 外层硬超时、日志与容器清理。

固定摘要 Kafka 的 SASL_SSL 实验已证明：受信 CA + PLAIN 凭据可发布，错误 CA 与错误密码
均 fail-closed。该证据只覆盖 TLS/CA/SASL-PLAIN，不得升级为 group/rebalance/HA/native EOS 结论。

受控外部环境仍可运行 `tests/live_event_bus.rs`；ignored 或单节点 PASS 不得升级为未列出的能力结论。

追溯：`crates/adapters/storage/kafka/{Cargo.toml,src,tests/broker_conformance.rs,tests/tls_sasl_conformance.rs}`、
`scripts/kafka-tls-sasl-conformance.mjs`、`docs/ssot/kafkax-ssot-alignment.md`。
