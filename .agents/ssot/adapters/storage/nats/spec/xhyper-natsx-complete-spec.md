# natsx 实现规范

状态：当前 `0.3.1` 实现合同（Mock + async-nats 真实驱动已落地；真测 `#[ignore]`，未达 M3）。**未宣称 package stable。**

## 0. 权威、职责与非目标

权威顺序：Constitution → XLib spec → 已批准 ADR → 本文 → 代码。**Evidence** 为直接事实，
**Inference** 为不新增架构的最低要求，**Unknown** 必须评审。

`natsx` 位于 `crates/adapters/storage/nats`，实现 `contracts::EventBus`：
- `MockNatsBus`：与 kafkax mock 独立的内存总线；
- `NatsEventBus`：基于 `async-nats` 的真实驱动（始终编译，非 feature 门控）。

非目标：在本版承诺 JetStream 管理 API、queue group 运维合同、ack、持久化/重放或 request/reply，
或把 ignored 真测当作 CI 生产证据。

## 1. Cargo、API 与版本

版本 `0.3.1`（package `natsx`）。

| 项目 | 当前事实 |
| --- | --- |
| 普通依赖 | `kernel`、`contracts`、`async-trait`、`bytes`、`futures-core`、`futures-util`、`async-nats`、`anyhow` |
| features | 无 |
| dev-dependency | `tokio` |

依赖符合 R2。版本更新仅允许 `x.y.z → x.y.(z+1)`。

### 1.1 MockNatsBus

`MockNatsBus` 为 `Debug + Default`，`new()` 创建空 `RwLock<HashMap<String, Vec<Bytes>>>`。
`publish(subject, payload)` 按 subject 追加；`subscribe(subject)` 返回订阅时缓冲消息的克隆、
有限快照流：保序，缺失 subject 为空，**后续发布不可见**。锁中毒会 panic。

### 1.2 NatsEventBus

- `NatsEventBus::new(url: &str) -> XResult<Self>`：异步 `async_nats::connect`。
- `publish` / `subscribe`：subject 转为 owned `String`（async-nats 对 `'static` 的要求）。
- `subscribe` 返回的 `Subscriber` 为 owned Stream，直接 `.map(|msg| msg.payload).boxed()`。
- 错误统一映射为 `XError::Transient` 等。

## 2. 差距、并发与信任边界

- **证据**：`async-nats` 依赖与真实实现已存在。
- **证据**：`nats_publish_and_subscribe` 为 `#[ignore]`；Core NATS 不回放历史，测试先 subscribe 再 publish；
  **不得**当作 CI 默认通过或 M3 生产证据。
- **未知**：Core NATS 与 JetStream 选择、TLS/认证生产合同、subject 规则、queue/durable consumer、
  ack、重投、背压、交付保证、连接恢复和关闭。
- **未知**：通用 `Envelope`/收发拆分仍需 contracts 评审。
- subject/payload 是不可信输入；生产实现须限制名称与大小，并避免泄漏凭据或消息内容。

## 3. 测试、验收与追溯

Mock 内联测试覆盖发布/订阅、缺失 subject、快照边界、subject 隔离和 trait object。
真实测试 `#[ignore]`。运行：

```text
cargo test -p natsx
cargo test -p natsx -- --ignored   # 需 NATS；非 CI 默认
cargo clippy -p natsx --all-targets -- -D warnings
cargo fmt -- --check
cargo run -p xtask -- lint-deps
```

验收要求：当前快照语义与代码一致；不得把 mock 描述为完整 NATS；不把 ignored 真测宣称为生产就绪；
新 JetStream/API 先评审；依赖与精确 patch 版本规则通过门禁。

追溯：XLib spec §§2 R2/R6、4.3、4.5、5；
`crates/adapters/storage/nats/{Cargo.toml,src/lib.rs,README.md}`。
