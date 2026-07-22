# contracts SSOT 对齐

| 字段 | 值 |
|------|-----|
| package（`cargo -p`） | `contracts` / lib `contracts`（产品名别名 `xhyper-contracts`，不可用于 `-p`） |
| path | `crates/contracts` |
| version | `0.1.1`（**非** package stable；adapter-trait 出口面；未宣称 Production Ready） |
| Active Spec | `.agents/ssot/contracts/spec/spec.md`（若存在；以本仓源码为准） |
| 审计/跟进 | 2026-07-21 W3 + L3 子集（#172）；**defer-close 2026-07-22**：live helpers |
| 状态 | R4 trait 面 + first-batch 语义；L3 子集 KV+Instr；**LiveContractProfile** 业务 live 助手 **PASS**；**≠** first-batch 全绿 / Agent L5 |

## 结论摘要

| 问题 | 状态 |
|------|------|
| trait 出口（storage / venue / Instrumentation） | **已落地** |
| 事务可测语义 | **部分闭合**：`TxContext` + `TxRunner` + `run_tx_commit_on_ok` |
| 消息可测语义 | **部分闭合**：`BusMessage` + `MessageAck` |
| contract-testkit | **独立 crate 已落地**（Fake + suite + **Batch-2** + **BackendProfile**） |
| first-batch 语义文档 + 套件 | **部分闭合**（CT-8） |
| VenueAdapter override 门禁 | **部分闭合**（CT-10） |
| Tx/Bus/Repo/Venue **live helpers** | **PASS**：`src/live.rs` · `LiveContractProfile` + `LiveHandles` + kv/bus/tx helpers |
| 交易所 **业务** live（签名/下单） | **NO-GO 默认 CI**（exchange **生产默认 REST+WS 离线** #210+#214；live 仅 `#[ignore]` server_time） |
| bootstrap 双平面 | **已收敛**：`Bounded*` + Instrumentation re-export |

替换 `#43`/`#46`/`#53` 的 `xhyper-contracts` 草图。消费者：`observex` 实现 `Instrumentation`；`resiliencx` 消费；adapters storage 生产客户端 + **exchange binancex/okxx 生产默认 VenueAdapter**（#210+#214）为实现面（非 package stable）。

**依赖澄清**：contracts **不**直接依赖 `serde` / `thiserror`；`decimalx` 仅 `[dev-dependencies]`（用于交易所 adapter 测试）。正常依赖仅限于 `kernel`、`canonical`、`async-trait`、`bytes`、`futures-core`。

## 本仓可观察事实

```text
crates/contracts/
  live.rs                       LiveContractProfile / LiveHandles / kv_roundtrip / bus_publish / apply_ack
  TxContext / TxRunner / …
  docs/contracts/*.md           first-batch 语义
  tests/conformance_first_batch 委托 contract_testkit

crates/test-support/contracts/  package contract-testkit
  fakes/batch2.rs               Batch-2 Fake 面
  backend.rs                    BackendProfile 真后端探测（缺凭据 → Unavailable，诚实回退 Fake）
```

验证：

```bash
cargo test -p contracts --all-targets
cargo test -p contract-testkit --all-targets
cargo clippy -p contracts -p contract-testkit --all-targets -- -D warnings
```

## 条款矩阵（本仓）

| ID | 条款 | 状态 | 证据 |
|----|------|------|------|
| CT-1 | KeyValueStore / Instrumentation 可调用 | PASS | contract-testkit Fake + public_surface |
| CT-2 | Tx 可测 commit/rollback | PASS | run_tx + RecordingTxRunner |
| CT-3 | TxRunner 对象安全 | PASS | `&dyn TxRunner` |
| CT-4 | 消息带 ID；subscribe 流为 BusMessage | PASS | FakeEventBus |
| CT-5 | 失败注入至少一类 | PASS | FakeTxContext::with_commit_failure |
| CT-6 | public_surface 非空断言 | PASS | contract-testkit 驱动 |
| CT-7 | bootstrap 无静默同名冲突 | PASS | Bounded* |
| CT-8 | 全 trait 深度文档+套件 | **部分** | first-batch 11 篇；ObjectStore 等仍 OPEN |
| CT-9 | 非 scaffold 真实后端验证入口 | **加厚 PASS** | redis live KV + observex Instr + **live.rs helpers** + BackendProfile |
| CT-10 | VenueAdapter override 门禁 | **部分** | venue_override_gate |
| CT-11 | `[lints] workspace = true` | PASS | Cargo.toml |
| CT-12 | LiveContractProfile 业务 live 助手 | **PASS** | `src/live.rs` |
| CT-13 | Batch-2 Fake + backend profile | **PASS** | `contract-testkit` fakes/batch2 + backend |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| Tx/Bus/Repo/Venue 业务 live | DEFER | **PASS（helpers/profile）** | `crates/contracts/src/live.rs` |
| Batch-2 Fake | DEFER | **PASS** | `crates/test-support/contracts/src/fakes/batch2.rs` |
| 真后端 profile | DEFER | **PASS** | `crates/test-support/contracts/src/backend.rs` |

## 与 testkit / contract-testkit 的关系

- **不**在 `crates/testkit` 内放 contracts fake（testkit = ManualClock + IntegrationHarness）
- **独立** `contract-testkit`：Fake/Recording + suite + Batch-2 + BackendProfile
- `contracts` 生产依赖图 **不含** contract-testkit

## 未做（诚实边界 / OPEN）

- **全** trait 深度 conformance（ObjectStore / TimeSeries / PubSub / Analytics）
- 交易所 **业务** live（签名/下单/WS 行情）— adapters NO-GO
- VenueAdapter **强制 compile-fail** override 机控
- first-batch **全绿** L3 宣称；Agent L5

## L3 子集（infra-s9t.3 · closed）+ live 加厚

| Trait | L3 三条件 | 本仓 |
|-------|-----------|------|
| KeyValueStore | 语义 + Fake + 非 scaffold 入口 | **满足**（RedisLiveKv + live helpers） |
| Instrumentation | 同上 | **满足**（observex） |
| Tx / Bus / Repository / Venue | live **helpers** | **PASS（helpers）**；交易所业务路径仍 **NO-GO** |

## 双栏落地

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / exchange 可交易 |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：live.rs + contract-testkit Batch-2/backend PASS |
| 2026-07-22 | SSOT 同步纠偏：Fake 命名空间 `contract_testkit::` |
| 2026-07-21 | 独立 contract-testkit（#178）；L3 子集 #172 |
