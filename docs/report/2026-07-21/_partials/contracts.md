# contracts（xhyper-contracts）生产就绪 partial

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| package / lib | Cargo `name = "contracts"` · lib `contracts`（文档/SSOT 常称 `xhyper-contracts`；**`-p xhyper-contracts` 会失败**） |
| 路径 | `crates/contracts` |
| 版本 | `0.1.0` · `publish = false` |
| 源码结构 | `src/lib.rs` + `src/fakes.rs`（无子模块拆分） |
| 目标层级 | **L3 Contract Ready**（生产用 trait 子集） |
| 本审计结论 | **L3 子集**（KV+Instr）已闭合（#172）；first-batch **整体未** L3；**非** Production Ready / **非** first-batch 签字 |
| 证据基线 | SSOT：`docs/ssot/contracts-ssot-alignment.md`；计划：`docs/plans/2026-07-21-core-crates-production-readiness.md` §2；core 报告：`docs/report/2026-07-21/core-crates-production-readiness.md` §4.3（**部分陈述相对 post-W3 源码已陈旧**） |
| 测试命令 | `cargo test -p contracts --all-targets`（见 §6） |

---

## 1. 结论

**`contracts` 是可用的 R4 trait 出口 + 最小 contract-testkit 入口；`KeyValueStore` 与 `Instrumentation` 满足 L3 三条件子集；first-batch 整体仍未 L3。**

依据 L3 定义（语义合同 **+** conformance suite **+** 至少一**非 scaffold** 验证入口）：

| L3 条件 | 状态 | 说明 |
|---------|------|------|
| 语义合同（trait 文档） | **部分** | first-batch **11** trait 有 `docs/contracts/*.md`；`ObjectStore` / `TimeSeriesStore` / `PubSub` / `AnalyticsSink` **无**独立语义文档 |
| conformance suite | **部分** | `tests/conformance_first_batch.rs` 覆盖 KV / Repository / Tx / EventBus / Instrumentation |
| 非 scaffold 验证入口 | **部分 PASS（#168/#172）** | **KV**：`redisx::RedisLiveKv` + `live_kv_conformance`；**Instr**：`observex`；Tx/Bus/Repo/Venue 业务 live **仍 DEFER** |

已闭合（相对早期 core 报告 §4.3 草稿 + infra-s9t）：

- `TxRunner::begin_tx` → `Box<dyn TxContext>`，**对象安全**；`run_tx_commit_on_ok` 编排 Ok→commit / Err→rollback
- `BusMessage { id, payload }` + `MessageAck`；EventBus 流项带 ID（at-most-once 能力边界显式）
- 最小 Fake/Recording 在 crate 内公开（`src/fakes.rs`）
- bootstrap 同名能力面收敛为 `Bounded*`；`Instrumentation` re-export `contracts::Instrumentation`
- Venue structured cancel/query：**中文** default Invalid + `is_default_*` + `venue_override_gate`
- **L3 子集文档**：`crates/contracts/docs/L3_FIRST_BATCH_STATUS.md`

仍阻断 **first-batch 全 L3** / 生产宣称：

1. Tx / EventBus / Repository / Venue **业务** 无真实后端入口
2. **二期 storage trait** 语义与 conformance 未闭合（CT-8 DEFER）
3. **独立 contract-testkit crate** 仍 DEFER
4. VenueAdapter additive default **无 compile-fail**（仅运行时门禁）
5. Additive Only **无 API snapshot / semver diff 机控**

**判定：L3 子集（KV+Instr）可用 · first-batch 整体未 L3 · 禁止 Production Ready 宣称。**

---

## 2. 源码结构与公开面清单

### 2.1 布局

```text
crates/contracts/
  Cargo.toml              # package name = contracts
  README.md / CHANGELOG.md / AGENTS.md
  src/
    lib.rs                # 全部 trait + 类型 + run_tx_commit_on_ok + 单元测
    fakes.rs              # 最小 contract-testkit + venue default helpers
  tests/
    public_surface.rs     # 15 trait 可达 + Fake 真路径
    conformance_first_batch.rs
    venue_override_gate.rs
  docs/contracts/         # first-batch 11 篇语义
  docs/L3_FIRST_BATCH_STATUS.md
  benches/ examples/      # example 有 fake_surface
```

依赖白名单：`kernel` + `canonical` + `async-trait` / `bytes` / `futures-core`。  
Lint：`forbid(unsafe_code)` · `deny(unreachable_pub)` · `[lints] workspace = true`；`missing_docs` 未 deny。

### 2.2 公开 trait（15）

| # | Trait | 方法摘要 | 语义文档 | Fake / 合同测 | Adapter 实现 | 就绪度 |
|---|--------|----------|----------|---------------|--------------|--------|
| 1 | `KeyValueStore` | `get` / `set(+ttl)` | ✅ | Fake + conformance + **live** | redisx mock + **`RedisLiveKv`** | **L3 子集满足** |
| 2 | `EventBus` | `publish` / `subscribe→BusMessage` | ✅ | Fake + conformance | kafkax/natsx scaffold | **部分**（at-most-once；无 ack API） |
| 3 | `Repository<T,Id>` | `find` / `save` | ✅ | Fake + conformance | postgresx scaffold | **部分**（无分页/删除/并发） |
| 4 | `TxContext` | `commit` / `rollback` | ✅ | Fake + Recording | 经 FakeTxContext | **部分**（无隔离级别；假上下文） |
| 5 | `TxRunner` | `begin_tx→Box<dyn TxContext>` | ✅ | Fake/Recording + 对象安全 | postgresx → FakeTxContext | **形状就绪；语义未真实验证** |
| 6 | `TimeSeriesStore` | `write_series` / `query_series` | ❌ | 无 Fake | taosx scaffold | **experimental / 未就绪** |
| 7 | `ObjectStore` | `put_object` / `get_object` | ❌ | 无 Fake | ossx scaffold | **experimental / 未就绪** |
| 8 | `AnalyticsSink` | `sink` | ❌ | 无 Fake | clickhousex scaffold | **experimental / 未就绪** |
| 9 | `PubSub` | `pub_message` / `sub_channel→BusMessage` | ❌ | 无 Fake | redisx scaffold | **experimental / 与 EventBus 重叠** |
| 10 | `Instrumentation` | `record_retry` / `record_circuit_*` | ✅ | Recording + conformance | observex 真实现；bootstrap/resiliencx 消费 | **L3 子集满足**（非 IO 契约） |
| 11 | `VenueAdapter` | 连接/下单/cancel·query（legacy+request）/仓位余额/行情/时间/元数据 | 迁移 facade 文档散见 | default 行为测；无完整 Fake | binancex/okxx scaffold | **迁移面；非生产推荐入口** |
| 12 | `MarketDataSource` | ticks / orderbook / trades 流 | ✅ | 无独立 Fake | 拆自 VenueAdapter | **部分**（仅文档+形状） |
| 13 | `InstrumentCatalog` | `symbol_info` | ✅ | 无独立 Fake | 拆自 VenueAdapter | **部分** |
| 14 | `ExecutionVenue` | place / cancel / query（结构化）+ `VenueId` | ✅ | scaffold 充当前端；override 门禁 | binancex/okxx | **推荐入口；仍 scaffold** |
| 15 | `AccountSource` / `VenueTimeSource` | position·balance / `server_time` | ✅ | 无独立 Fake | 拆自 VenueAdapter | **部分** |

> 计数：storage 9 + observability 1 + venue 单体 1 + 能力拆分 4 = **15 trait**（`AccountSource` 与 `VenueTimeSource` 在公开面测试中分计；SSOT「15-trait」口径一致）。

### 2.3 公开类型 / 函数 / Fake

| 符号 | 角色 |
|------|------|
| `BusMessage { id, payload: Bytes }` | 消息项（带 ID） |
| `MessageAck { Ack, Nack }` | 确认模型**类型预留**；trait **无** ack 方法 |
| `run_tx_commit_on_ok` | Ok→commit / Err→rollback 编排 helper |
| `FakeTxContext` / `with_commit_failure` | 事务参考 + commit 失败注入 |
| `FakeTxRunner` | 对象安全 begin_tx |
| `RecordingTxRunner` | 可观察 commit/rollback 标志 |
| `FakeEventBus` | 进程内 at-most-once 快照流 |
| `FakeKeyValueStore` | 内存 KV；TTL **不**自动过期 |
| `FakeRepository<T,Id>` | 内存仓储 + `id_of` 注入 |
| `RecordingInstrumentation` / `InstrEvent` | 可观测记录 |
| `VENUE_*_DEFAULT_MSG` / `is_default_*_error` | Venue additive default 机检辅助 |

### 2.4 重复 / 重叠 trait

| 对 | 关系 | 风险 |
|----|------|------|
| `VenueAdapter` vs `ExecutionVenue` + `MarketDataSource` + `InstrumentCatalog` + `AccountSource` + `VenueTimeSource` | 能力拆分 vs 迁移 monolith | 方法面重复；生产应优先拆分 trait；adapter 常双实现 |
| `EventBus` vs `PubSub` | 均 publish/subscribe 风格，`BusMessage` 流 | 语义边界文档不足；PubSub 无 first-batch 文档 |
| bootstrap `Bounded*` vs contracts 同名概念 | **已改名前缀**，非同签名静默冲突 | 仍是**第二平面**；组合根非直接 `dyn contracts::KeyValueStore` |

---

## 3. 语义完备性评估（生产视角）

### 3.1 事务（TxContext / TxRunner）

| 维度 | 现状 |
|------|------|
| 句柄 | ✅ 显式 `TxContext`；非「裸 Future」 |
| 对象安全 | ✅ `dyn TxRunner` 合同测覆盖 |
| 编排 | ✅ `run_tx_commit_on_ok`；rollback 失败被吞、保留业务 Err |
| 隔离级别 / 只读 / savepoint | ❌ 未表达 |
| 与 Repository 原子组合 | ❌ 无 `Repository` 在同一 `TxContext` 上的标准绑定 |
| 真实后端 | ❌ postgres scaffold `begin_tx` → **`FakeTxContext`**，**不**绑定 rows 事务 |

**结论：可测形状已闭合；生产事务语义未闭合。**

### 3.2 消息（EventBus / PubSub / MessageAck）

| 维度 | 现状 |
|------|------|
| 消息 ID | ✅ `BusMessage.id` |
| 交付保证 | 文档/注释：**at-most-once**；at-least-once / 事务消息需 Additive 扩展 |
| ack/nack | 类型存在；**trait 无 handle/ack API** |
| headers / partition / offset | ❌ |
| redelivery | ❌ 明确不保证 |
| Fake | ✅ 一次性快照流；非实时 fan-out |

**结论：最小可幂等消费面；非生产消息中间件合同。**

### 3.3 KV / 仓储

| 维度 | KeyValueStore | Repository |
|------|---------------|------------|
| CRUD 最小面 | get/set | find/save |
| not-found | `Ok(None)`（文档化） | `Ok(None)` |
| TTL | 参数存在；Fake **不**过期 | N/A |
| 分页 / 扫描 / 删除 | ❌ | ❌ |
| 并发写 / CAS | ❌ | ❌ |
| 失败注入 | Fake 仅 lock 中毒路径 | 同左 |

### 3.4 对象存储 / 时序 / 分析

- trait **仅方法签名**；注释写「待新增」
- **无**语义文档、**无** Fake、**无** conformance
- adapters 为内存 HashMap scaffold
- **明确非 L3 子集**

### 3.5 Venue

- **生产推荐**：`ExecutionVenue`（无 additive default，结构化 cancel/query）
- **迁移 facade**：`VenueAdapter` 的 `cancel_order_request` / `query_order_request` default → 中文 `Invalid`
- 门禁：`tests/venue_override_gate.rs` 断言 binancex/okxx **非** default；BareVenue 仍为 default
- **非** compile-fail；真实交易所网络路径 **DEFER**

### 3.6 Instrumentation

- 同步、无 I/O；`observex::TracingInstrumentation` 为真实实现面
- resiliencx / bootstrap 消费同一 trait
- **最接近「可生产使用」的契约**，但仍属可观测注入点而非交易/存储核心

---

## 4. contract-testkit / conformance / adapter 验证入口

| 能力 | 状态 | 证据 |
|------|------|------|
| 最小 contract-testkit（本 crate） | **已落地** | `src/fakes.rs` 公开 Fake/Recording |
| 独立 `test-support/contracts` crate | **DEFER** | SSOT CT / DEFER 列表 |
| first-batch conformance | **部分** | `conformance_first_batch.rs`（5 测） |
| public_surface | **通过** | 15 trait 可达 + 真路径（已非空 `assert_eq!(15,15)` 占位） |
| venue override 运行时门禁 | **部分（CT-10）** | `venue_override_gate.rs` + binancex/okxx |
| 失败注入 | **一类** | `FakeTxContext::with_commit_failure`（Transient） |
| 真实后端验证入口 | **DEFER（W4 / CT-9）** | adapters scaffold only |
| mock-first adapter 入口 | **有（非 L3 替代）** | adapters SSOT：first-batch 进程内 mock；**≠** 非 scaffold |

---

## 5. 与 adapters / bootstrap / 下游关系

```text
contracts (trait 出口)
    ├── observex     → Instrumentation 实现（Tracing）
    ├── resiliencx   → Instrumentation 消费
    ├── bootstrap    → re-export Instrumentation；Bounded* 第二平面
    ├── binancex/okxx → VenueAdapter + ExecutionVenue（内存 scaffold）
    ├── redisx       → KeyValueStore + PubSub
    ├── kafkax/natsx → EventBus
    ├── postgresx    → Repository + TxRunner(FakeTxContext)
    ├── ossx         → ObjectStore
    ├── taosx        → TimeSeriesStore
    └── clickhousex  → AnalyticsSink
```

- adapters：**9 包 scaffold**；实现 trait 以通过编译/单测，**不**宣称 package stable / Production Ready
- contracts 的 `dev-dependencies` 拉 `binancex`/`okxx` 做 override 门禁——**验证入口绑定 scaffold**，不是真实 venue
- bootstrap 已消除「静默同名不同签名」；**Bounded*** 仍与 contracts 能力概念平行

---

## 6. 测试证据（本会话执行）

命令与说明：

```bash
# 用户任务写法（会失败）：
cargo test -p xhyper-contracts --all-targets
# error: package ID specification `xhyper-contracts` did not match any packages

# 正确 package 名：
cargo test -p contracts --all-targets
```

结果摘要（`tail` 等价汇总）：

```text
lib unit/fakes:     22 passed
conformance_first:   5 passed
public_surface:      5 passed
venue_override_gate: 4 passed
总计: 36 passed; 0 failed
```

**说明**：测试全绿证明 trait 形状、Fake 路径、override 常量与树内 adapter 非 default；**不能**证明真实事务原子性、消息 redelivery、对象存储一致性或交易所协议正确性。

---

## 7. 对照 core-crates 报告 §4.3（漂移）

早期 core 报告（`core-crates-production-readiness.md` §4.3）若干断言**相对当前 main 源码已过时**，审计时以源码 + `contracts-ssot-alignment.md` 为准：

| 报告旧述 | 当前源码 |
|----------|----------|
| `TxRunner::run_tx` 收 Future，无句柄 | `begin_tx` + `TxContext` + `run_tx_commit_on_ok` |
| TxRunner 非 dyn-compatible | **对象安全**，有测 |
| EventBus stream 仅 `Bytes` | `BusMessage { id, payload }` |
| contract-testkit 未落地 | 本 crate 最小 Fake/Recording **已落地** |
| bootstrap 同名冲突 | `Bounded*` + Instrumentation re-export **已收敛** |
| default 错误英文 | **中文** `VENUE_*_DEFAULT_MSG` |
| public_surface `assert_eq!(15,15)` | 已改为 Fake 真路径断言 |

**未过时且仍成立（相对 first-batch 全 L3）**：无完整幂等/取消/分页合同套件；Tx/Bus/Repo/Venue **业务**无真入口；Venue 与拆分 trait 重复；ObjectStore 等语义空洞；独立 testkit crate DEFER；Additive Only 无机控。  
**已过时（#168/#172）**：「无任何非 scaffold 入口」— 现有 redis live KV + observex Instrumentation。

---

## 8. 阻断项（达 L3 前）

### P0（L3 签字阻断）

1. **非 scaffold 验证入口（CT-9 / W4）**  
   至少一个生产子集 trait（建议 `ExecutionVenue` 或 `KeyValueStore`+`TxRunner`）对接真实或可证明语义的后端 harness。
2. **生产子集裁定 + 标签**  
   明确 L3 子集 vs `experimental`（ObjectStore / TimeSeries / PubSub / Analytics 至少标 experimental 或补齐文档+套件）。
3. **事务语义与存储绑定**  
   `TxRunner` 不得在「生产路径」上仅返回 `FakeTxContext`；需可观察 commit 影响 Repository/后端状态的合同测。

### P1

1. 扩展 conformance：ObjectStore / PubSub / TimeSeries / Analytics / venue 能力拆分  
2. EventBus/PubSub：ack 边界裁定（扩展 trait 或文档冻结「无 ack」）  
3. VenueAdapter override：**compile-fail 或 CI lint** 强制树内实现  
4. 独立 contract-testkit crate（从 `fakes` 迁出或冻结本 crate 为 SSOT 入口）

### P2

1. Additive Only API snapshot / semver diff  
2. `deny(missing_docs)` 与 trait 方法文档债  
3. 收敛 bootstrap `Bounded*` 与 contracts 的长期关系（文档化或统一）

---

## 9. L3 Contract Ready checklist

| # | 检查项 | 结果 |
|---|--------|------|
| 1 | 生产子集 trait 列表已冻结并文档化 | ⚠️ first-batch 11 有文档；全集 15 未分层 experimental |
| 2 | 每个子集 trait 有语义合同（幂等/失败/取消/资源） | ⚠️ 11/15；深度不等 |
| 3 | 通用 conformance suite 驱动真实 trait 方法 | ⚠️ first-batch 5 场景；非全 trait |
| 4 | 失败注入至少覆盖主失败类 | ⚠️ 仅 Tx commit Transient |
| 5 | 最小 Fake/Recording 可运行 | ✅ |
| 6 | 至少一非 scaffold 验证入口 | ❌ |
| 7 | Venue 生产入口无静默 default 风险 | ⚠️ ExecutionVenue 无 default；VenueAdapter 仅运行时门禁 |
| 8 | 与 bootstrap 无静默双平面 | ✅ Bounded* / re-export |
| 9 | 对象安全（需要 dyn 的入口） | ✅ TxRunner 等已测 |
| 10 | Additive Only 机控 | ❌ |
| 11 | CI：fmt/clippy/test 持续绿 | ✅ 本会话 test 绿 |
| 12 | 可宣称 L3 / Production Ready | ❌ **否** |

---

## 10. 建议优先级（contracts 轨）

1. **裁定 L3 子集**（建议：KeyValueStore、Tx*、EventBus、Instrumentation、ExecutionVenue + 必要拆分）并把其余标 experimental  
2. **W4 非 scaffold 入口**（哪怕单一 backend 的集成 harness）  
3. **Tx 与 Repository 原子合同**（commit 后可见 / rollback 不可见）  
4. 补 ObjectStore 等二期文档 **或** 移出「15 全部生产」叙事  
5. Venue override 编译期门禁 + API snapshot  

---

## 11. 参考路径

| 资源 | 路径 |
|------|------|
| 源码 | `/home/workspace/infra.rs/crates/contracts/src/lib.rs` |
| Fake | `/home/workspace/infra.rs/crates/contracts/src/fakes.rs` |
| SSOT 对齐 | `/home/workspace/infra.rs/docs/ssot/contracts-ssot-alignment.md` |
| 计划 L3 定义 | `/home/workspace/infra.rs/docs/plans/2026-07-21-core-crates-production-readiness.md` |
| core 报告 | `/home/workspace/infra.rs/.worktrees/docs/infra-status-modules-prod-audit/docs/report/2026-07-21/core-crates-production-readiness.md`（§4.3 部分陈旧） |
| 语义文档 | `/home/workspace/infra.rs/crates/contracts/docs/contracts/` |
| 本 partial | `/home/workspace/infra.rs/.worktrees/docs/infra-status-modules-prod-audit/docs/report/2026-07-21/_partials/contracts.md` |

---

*只读审计 · 未修改 `crates/contracts` 源码 · 2026-07-21*
