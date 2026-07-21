# kafkax 实现规范

状态：当前 `0.1.0` 实现合同（默认 Mock；`real` feature 提供 rdkafka 驱动；真测 `#[ignore]`，未达 M3）

## 0. 权威与范围

裁定顺序为 Constitution → XLib spec → 已批准 ADR → 本文 → 代码。**Evidence** 是上述材料或代码
直接事实；**Inference** 是最低验收收窄；**Unknown** 需另行评审。本文不批准新 API 或依赖。

`kafkax` 位于 `crates/adapters/storage/kafka`，实现 `contracts::EventBus`：
- 默认构建：`MockEventBus`（不依赖 rdkafka）；
- `feature = "real"`：`KafkaEventBus`（`rdkafka`）。

非目标：在本版本承诺 consumer group 运维合同、offset 管理 API、schema registry、exactly-once，
或把 ignored 真测当作 CI 生产证据。

## 1. Cargo 与版本

当前版本 `0.1.0`。

| 项目 | 当前事实 |
| --- | --- |
| 普通依赖 | `kernel`、`contracts`、`anyhow`、`async-trait`、`bytes`、`futures-core`、`futures-util`、`futures-channel`、`tokio`；`rdkafka` 为 optional |
| features | `default = []`；`real = ["dep:rdkafka"]` |
| dev-dependency | `tokio` |

依赖符合 R2 且无同层适配器依赖。更新必须且只能为 `x.y.z → x.y.(z+1)`。

## 2. 当前公开 API 与行为

### 2.1 MockEventBus

`MockEventBus` 为 `Debug + Default`，`new()` 创建空 `RwLock<HashMap<String, Vec<Bytes>>>`。
`publish(topic, payload)` 按 topic 追加并返回 `Ok(())`；`subscribe(topic)` 克隆订阅时已缓冲
消息并返回有限 `BoxStream<'static, Bytes>`：保序、缺失 topic 为空、**订阅后的 publish 不可见**。
锁中毒会因 `unwrap()` panic。无后台任务、broker 连接或 shutdown。

### 2.2 KafkaEventBus（feature = "real"）

- `KafkaEventBus::new(brokers: &str) -> XResult<Self>`：创建 `FutureProducer`，记录 broker 列表。
- `publish`：`FutureProducer::send`，超时 5s；错误映射为 `XError::Transient`。
- `subscribe`：每次用自增 `group.id` 创建独立 `StreamConsumer`（`auto.offset.reset=earliest`），
  经 `tokio::spawn` + mpsc 桥接为 `'static` 持续流；接收端丢弃时 task 退出。

## 3. 架构差距与边界

- **证据**：真实驱动在 `real` feature 下已引入 rdkafka；默认 CI 路径不编译、不链接原生依赖。
- **证据**：真实测试 `kafka_publish_and_subscribe` 需 `feature = "real"` 且 `#[ignore]`；
  **不得**当作 CI 默认通过或 M3 生产证据。
- **未知**：broker/TLS/SASL 生产配置、topic 配置、key/headers、分区与顺序合同、consumer group
  运维、offset/ack API、背压、重投/死信、幂等与交付保证。
- **未知**：implementation-plan 的 `Envelope`、publisher/subscriber 拆分和
  `AckableSubscriber` 是待评审提案，不能当作现有合同。
- topic 与 payload 属于信任边界；生产实现须校验名称/大小并脱敏认证信息。

## 4. 测试、验收与追溯

Mock 内联测试覆盖发布/快照订阅、缺失 topic、后续消息不可见、topic 隔离和 trait object。
真实测试在 `real` feature 下，`#[ignore]`。运行：

```text
cargo test -p kafkax
cargo build -p kafkax --features real
cargo test -p kafkax --features real -- --ignored   # 需 Kafka；非 CI 默认
cargo clippy -p kafkax --all-targets -- -D warnings
cargo fmt -- --check
cargo run -p xtask -- lint-deps
```

验收要求：如实保留 mock 快照流语义；不声称 mock 是 Kafka；不把 ignored 真测宣称为生产就绪；
任何交付保证或 trait 修订先评审；Cargo/API/测试及 patch 版本规则一致。

追溯：XLib spec §§2 R2/R6、4.3、4.5、5；
`crates/adapters/storage/kafka/{Cargo.toml,src/lib.rs,README.md}`。
