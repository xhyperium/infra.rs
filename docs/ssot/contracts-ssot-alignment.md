# contracts SSOT 对齐

| 字段 | 值 |
|------|-----|
| package | `xhyper-contracts` / lib `contracts` |
| path | `crates/contracts` |
| Active Spec | `.agents/ssot/contracts/spec/spec.md`（若存在；以本仓源码为准） |
| 审计/跟进 | 2026-07-21 W3 语义深化 + Venue override gate（`infra-asa.4`） |
| 状态 | R4 trait 面已落地；first-batch 语义文档 + Fake/conformance 可运行；**非**整体 Production Ready |

## 结论摘要

| 问题 | 状态 |
|------|------|
| trait 出口（storage / venue / Instrumentation） | **已落地** |
| 事务可测语义 | **部分闭合**：`TxContext` + `TxRunner::begin_tx` + `run_tx_commit_on_ok` |
| 消息可测语义 | **部分闭合**：`BusMessage { id, payload }` + `MessageAck`；at-most-once 能力边界 |
| contract-testkit | **最小入口已落地**（本 crate 内 Fake/Recording；非独立 `test-support` crate） |
| first-batch 语义文档 + 套件 | **部分闭合**（CT-8）：见 `crates/contracts/docs/contracts/` |
| VenueAdapter override 门禁 | **部分闭合**（CT-10 / DEFER-8）：`tests/venue_override_gate.rs` |
| 全 trait 深度合同 / 真实后端 | **DEFER**（W4） |
| bootstrap 双平面 | **已收敛命名**：bootstrap 用 `Bounded*`；`Instrumentation` re-export contracts |

替换 `#43`/`#46`/`#53` 的 `xhyper-contracts` 草图。消费者：`observex` 实现 `Instrumentation`；`resiliencx` 消费；adapters 为 scaffold 实现面。

## 本仓可观察事实

```text
crates/contracts/               EXISTS
  TxContext / TxRunner          begin_tx → Box<dyn TxContext>（对象安全）
  run_tx_commit_on_ok           Ok→commit / Err→rollback 编排 helper
  FakeTxContext / FakeTxRunner  reference fake
  RecordingTxRunner             可观察 commit/rollback 标志（公开 API）
  FakeKeyValueStore             内存 KV
  FakeRepository                简单内存仓储
  RecordingInstrumentation      可观察 record_* 事件
  BusMessage / MessageAck       消息 ID + ack 模型
  FakeEventBus                  进程内 at-most-once 参考实现
  VenueAdapter                  additive default 中文 Invalid + override 辅助检测
  ExecutionVenue                推荐生产入口（无 default）
  docs/contracts/*.md           first-batch 11 trait 语义
  tests/conformance_first_batch 真实 trait 路径套件
  tests/venue_override_gate     binancex/okxx 非 default 门禁
```

验证：

```bash
cargo test -p contracts --all-targets
cargo clippy -p contracts --all-targets -- -D warnings
cargo test -p okxx -p binancex --all-targets
cargo test -p bootstrap --all-targets   # Bounded* 与 Instrumentation re-export
```

## 条款矩阵（本仓）

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| CT-1 | KeyValueStore / Instrumentation 可调用 | PASS | FakeKeyValueStore + RecordingInstrumentation + public_surface |
| CT-2 | Tx 可测 commit/rollback | PASS | `run_tx_commit_on_ok_*` + RecordingTxRunner |
| CT-3 | TxRunner 对象安全 | PASS | `&dyn TxRunner` 测 |
| CT-4 | 消息带 ID；subscribe 流为 BusMessage | PASS | FakeEventBus + EventBus trait |
| CT-5 | 失败注入至少一类 | PASS | `FakeTxContext::with_commit_failure` |
| CT-6 | public_surface 非空断言 | PASS | 驱动 FakeTx/FakeBus/KV/Instr 真路径（无 `assert_eq!(15,15)`） |
| CT-7 | bootstrap 无静默同名冲突 | PASS | bootstrap `Bounded*` 前缀；见 bootstrap 对齐文 |
| CT-8 | 全 trait 幂等/取消/分页/一致性文档+套件 | **部分** | first-batch 11 篇语义文档 + `conformance_first_batch`；ObjectStore/TimeSeries/PubSub/Analytics 仍 DEFER |
| CT-9 | 非 scaffold 真实后端验证入口 | DEFER | adapters 仍内存 scaffold（W4） |
| CT-10 | VenueAdapter additive override 编译/运行门禁 | **部分** | `is_default_*_error` + `tests/venue_override_gate.rs`（binancex/okxx）；非强制 compile-fail |
| CT-11 | `[lints] workspace = true` | PASS | `Cargo.toml` |

## 与 testkit 的关系

- **不**在 `crates/testkit` 内放 contracts fake（testkit 仅 ManualClock core + kernel）
- 最小 contract-testkit **入口**在 `crates/contracts`（Fake/Recording 公开类型，`src/fakes.rs`）
- 完整 `crates/test-support/contracts` 独立 crate **仍 DEFER**

## Venue 入口策略

| 入口 | 角色 | 备注 |
|------|------|------|
| `ExecutionVenue` | **生产推荐** | 结构化 cancel/query，无 additive default |
| `VenueAdapter` | 迁移 facade | `cancel_order_request`/`query_order_request` default → 中文 Invalid；树内必须 override |

## 未做（DEFER）

- 独立 contract-testkit crate 与 **全** trait conformance suite（含 ObjectStore 等）
- 真实 postgres/kafka/nats/交易所验证入口（非 scaffold）— W4
- VenueAdapter 能力矩阵与 **强制 compile-fail** override 机控
- Additive Only 的 API snapshot / semver diff 门禁

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版占位：15-trait 落地；contract-testkit DEFER |
| 2026-07-21 | 生产就绪：Tx/消息语义、Fake/Recording testkit、与 bootstrap Bounded* 收敛；PR #98 |
| 2026-07-21 | PR #98 合入 main |
| 2026-07-21 | W3（infra-asa.4）：first-batch 语义文档、`fakes` 模块、venue override 运行时门禁；CT-8/CT-10 部分闭合；**非** Production Ready |
