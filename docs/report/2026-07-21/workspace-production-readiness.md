# Workspace 全量 Crate 生产就绪性分析

> **日期:** 2026-07-21 | **范围:** 全部 21 个 workspace members | **总 LOC:** ~14,000
> **部署:** 并行 agent team 分析（core/l1/exchange 三个视角）
> **关联:** [storage-adapters-production-readiness.md](storage-adapters-production-readiness.md)

## 1. 总览矩阵

| Crate | 层级 | LOC | 文件数 | 测试文件 | 成熟度 | 生产就绪？ |
|-------|------|----:|:------:|:--------:|--------|:--:|
| kernel | L0 | 1,698 | 4 | 6 | active | **是** |
| testkit | T0 | 575 | 2 | 6 | active | **是** (dev-dep only) |
| canonical | types | 1,177 | 3 | 3 | active | **是** (wire-committed) |
| decimalx | types | 1,475 | 1 | 6 | active | **接近** |
| bootstrap | L1 | 769 | 4 | 2 | active | **是** |
| configx | L1 | 434 | 3 | 2 | partial | **否** |
| evidence | L1 | 425 | 4 | 2 | partial | **否** |
| observex | L1 | 494 | 4 | 2 | partial | **否** |
| resiliencx | L1 | 756 | 5 | 4 | active | **接近** |
| schedulex | L1 | 434 | 4 | 2 | partial | **否** |
| transportx | L1 | 418 | 1 | 5 | active | **否** |
| contracts | contracts | 614 | 2 | 4 | active | **是** (Additive Only) |
| binancex | adapter/exch | 642 | 2 | 1 | scaffold+mock | **否** (scaffold) |
| okxx | adapter/exch | 504 | 2 | 1 | scaffold+mock | **否** (scaffold) |
| postgresx | adapter/stor | 1,693 | 9 | 1 | active | **是** |
| redisx | adapter/stor | 1,509 | 7 | 2 | active | **接近** |
| kafkax | adapter/stor | 1,041 | 10 | 1 | active | **接近** |
| taosx | adapter/stor | 755 | 4 | 1 | active | **部分** |
| ossx | adapter/stor | 693 | 5 | 1 | active | **部分** |
| natsx | adapter/stor | 666 | 6 | 1 | active | **接近** |
| clickhousex | adapter/stor | 509 | 4 | 1 | active | **部分** |

**统计：** 6 个 crate 生产就绪，5 个接近就绪，5 个部分就绪，5 个未就绪

## 2. 分层详解

### 2.1 L0 信任根

**kernel (1,698 LOC)** — 🟢 生产就绪

| 文件 | LOC | 职责 |
|------|----:|------|
| `clock.rs` | 583 | `Clock` trait, `Timestamp`, `MonotonicInstant`, `ClockDomain`, `SystemClock` |
| `error.rs` | 573 | `XError` (opaque), `ErrorKind` (9 variants), `XResult<T>` |
| `lifecycle.rs` | 520 | `ComponentState` (6-state machine), `ShutdownSignal`, `ShutdownGuard` |

**核心设计：** Clock trait 无单调性默认实现；XError 不透明私有字段含 `retry_after` 和 `source`；ShutdownSignal Clone-able 多观察者；跨域单调性 PartialOrd 跨域返回 None；突变体路径恢复 poison。6 个测试文件 + loom 模型检查 + compile-fail 负向合约。工作区中最成熟的 crate。

**testkit (575 LOC)** — 🟢 生产就绪（仅 dev-dep）

| 文件 | LOC | 职责 |
|------|----:|------|
| `clock.rs` | 561 | `ManualClock` — 确定性可控制时钟，含故障注入 |

`ManualClock` 提供 wall/monotonic 独立控制、`ManualClockFault`（BeforeUnixEpoch/Overflow/Unavailable）故障注入、单锁一致性快照及 poison 恢复。已确认仅在 dev-dependency 使用。6 个测试文。

### 2.2 Types 层

**canonical (1,177 LOC)** — 🟢 生产就绪（wire-committed）

15 个公开类型全部实现 Serialize/Deserialize，所有 committed wire types 使用 `deny_unknown_fields`。Wire commitment 分 v1/v1.1/v1.2/v1.3 四个层级，golden fixture 测试覆盖每层。非包级稳定——wire 已承诺但包稳定性待 M1 批准。

**decimalx (1,475 LOC)** —  接近就绪

10 个公开类型，核心 `Decimal { mantissa: i128, scale: u8 }`，MAX_SCALE=18。5 种舍入策略带中点检测，所有算术运算均为已验证。6 个测试文件覆盖属性测试、oracle 对比、边界矩阵、adversarial serde。

差距：单文件 1,475 LOC 需模块化（拆分为 decimal.rs、money.rs、ops.rs、round.rs）。建议增加 MIRI 完备性测试。

### 2.3 L1 Core Services

**bootstrap (769 LOC)** — 🟢 生产就绪

组合根模式（`BootContext` + `StoreSet`），bounded capability traits（当前占位，仅含 `label()` 方法），bounded.rs + traits.rs + error.rs。BootContext 模式就绪；bounded wiring 需与 contracts trait 对接。

**configx (434 LOC)** — ⚠️ 未就绪

仅 `MemoryConfigStore` — 内存级字符串键值存储。无文件/数据库源、无热重载、无 schema 验证。无法处理量化交易多源配置场景。

**schedulex (434 LOC)** — ⚠️ 未就绪

仅任务 ID 登记表 — ID 分配和状态统计。无 cron/timer 支持、无执行引擎、无重试机制。

**evidence (425 LOC)** — ⚠️ 未就绪

审计证据追加面（仅 `append()`）。无查询/验证 API、无持化后端、无证据链完整校验。

**observex (494 LOC)** — ⚠️ 未就绪

本地 TracingInstrumentation 最小实现。无 OTLP 导出、无分布式上下文传播、无采样策略。

**resiliencx (756 LOC)** — 🟡🟢 接近就绪

4 个模式完备：`retry.rs` (303L, 指数退避 + jitter)、`circuit.rs` (222L, tristate closed/open/half-open)、`bulkhead.rs` (107L)、`rate_limit.rs` (100L)。L1 库就绪但未被任何 adapter 消费——需要 P1 集成工作。

**transport (418 LOC)** — ⚠️ 未就绪

单文件 `lib.rs` — HTTP/WS 传输层。需模块化拆分（http.rs、ws.rs、tls.rs），添加 deadline、body 大小限制和 TLS 强制。

### 2.4 Contracts（401 LOC，15 个 trait）— 🟢 生产就绪

Additive Only 稳定 contract layer。15 个 trait 定义完备，通过 contract-testkit 委托的集成测试覆盖。DEFER-8/CT-10 门禁在 `venue_override_gate.rs` 中强制执行。

### 2.5 Exchange Adapters — ⚠️ 均未就绪（scaffold）

**binancex (642 LOC)** / **okxx (504 LOC)**

两者均为结构完整但行为 stub 的适配器——实现全部 6 个 venue trait（VenueAdapter、MarketDataSource、InstrumentCatalog、ExecutionVenue、AccountSource、VenueTimeSource），使用 `transportx::MockHttpTransport` 注入模式，并正确覆盖 additive defaults（cancel_order_request/query_order_request）。然而：

- `connect()`/`disconnect()` 仅为 AtomicBool 翻转（无实际 WS/HTTP 握手）
- 交易操作返回虚设的 DTO
- 行情数据流返回 `stream::empty()`
- 仓位/余额/symbol_info 返回占位值
- 仅 `server_time` 有通过 `HttpDriver` 注入的真实 HTTP 路径

两者均通过 DEFER-8 运行时门禁，单元测试充分（binancex 12 个，okxx 10 个），各有 `#[ignore]` live test。binancex 稍成熟（有 venex 扩展类型 `Candle`/`Timeframe`/`fetch_candles`）。

### 2.6 Storage Adapters（7 crates，6,866 LOC）

> 详见 [storage-adapters-production-readiness.md](storage-adapters-production-readiness.md)

**汇总：** postgresx (1,693L) 生产就绪。redisx (1,509L)、kafkax (1,041L)、natsx (666L) 接近就绪。taosx (755L)、ossx (693L)、clickhousex (509L) 部分就绪。

## 3. 量化交易适用性矩阵

| 模块 | 量化交易角色 | 状态 | 差距 |
|------|-------------|:--:|------|
| kernel | 时间源/生命周期/错误 | 🟢 | — |
| testkit | ManualClock 测试 | 🟢 | — |
| decimalx | 金额/精度/小数 | 🟡 | 需模块化 + MIRI |
| canonical | 线格式 DTO | 🟢 | — |
| contracts | Trait 定义 | 🟢 | — |
| configx | 多源配置 | ⚠️ | 需文件源 + 热重载 |
| schedulex | 定时任务 | ⚠️ | 需实际调度器 |
| bootstrap | 组合根 | 🟢 | Bounded traits 占位 |
| resiliencx | 重试/断路/限流/隔舱 | 🟡 | 需 adapter 集成 |
| observex | 分布式追踪 | ⚠️ | 需 OTLP 导出 |
| transport | HTTP/WS 传输 | ⚠️ | 需 deadline + TLS |
| binancex/okxx | 交易所连接 | ⚠️ | 真实客户端未实现 |
| postgresx | 持久化 | 🟢 | Repository trait 仅 scaffold |
| redisx | 缓存/状态 | 🟡 | 需 TLS |
| kafkax/natsx | 消息总线 | 🟡 | 需 offset/JetStream |
| taosx | 时序行情数据 | 🟡 | 需 pool + mock |
| clickhousex | 分析/审计 | 🟡 | 需批量写入 |

## 4. 优先级矩阵

### P0 — 阻塞生产

| # | Crate | 差距 |
|---|-------|------|
| 1 | configx | 无文件源 + 热重载 |
| 2 | schedulex | 无实际调度器 |
| 3 | observex | 无 OTLP 导出 |
| 4 | transport | 无 deadline + TLS |
| 5 | binancex/okxx | 真实客户端未实现 |
| 6 | taosx/ossx/clickhousex | 缺 mock + pool |

### P1 — 生产加固

| # | Crate | 差距 |
|---|-------|------|
| 7 | resiliencx × adapters | 重试/断路器集成 |
| 8 | decimalx | 单文件 → 模块化 + MIRI |
| 9 | evidence | 查询 API + 持久化 |
| 10 | natsx | JetStream persistent streams |
| 11 | kafkax | Consumer offset management |

### P2 — 优化

| # | Crate | 差距 |
|---|-------|------|
| 12 | bootstrap | Bounded trait wiring |
| 13 | All L1 | 扩展集成测试覆盖率 |

## 5. 当前可部署最小集

```text
kernel → testkit → decimalx → canonical → contracts
  ↓
bootstrap → postgresx → resiliencx
```

6 个 crate 即可在量化交易场景中建立基础持久化 + 重试 + 类型系统。

## 7. 依赖链分析（第二遍审计）

**关键发现：12 个 crate 为生产孤儿**——无任何其他 crate 消费它们。

```text
孤儿 crate (无消费方):
  bootstrap, clickhousex, configx, kafkax, natsx, ossx,
  postgresx, redisx, resiliencx, schedulex, taosx

已连线 crate (有消费):
  kernel ← 所有 crate (L0 信任根)
  contracts ← 14/21 crates (trait 定义)
  canonical ← 5 crates (wire DTOs)
  decimalx ← 4 crates (numeric types)
  testkit ← 仅 dev-dep
  observex ← bootstrap (ADR-005 注入)
  evidence ← bootstrap
```

**影响：** bootstrap 组合根仅依赖 kernel + contracts + observex + evidence。
所有 storage 和 exchange adapter 均为独立构建和测试，但从未接入实际应用。
这意味着即使 adapter 达到生产就绪，仍无法在 bootstrap 层被使用，除非完成 bounded trait wiring。

**依赖深度：** 所有 crate 距离 kernel 均为 1 跳——扁平的层次结构，无深层嵌套链。
这是良好设计的标志，但孤儿问题抵消了其优势。

## 8. 结论

21 个 crate 中 6 个生产就绪，5 个接近就绪，10 个需要补齐。L0 信任根（kernel + testkit）和 types 层（canonical + decimalx）是最成熟的区域。L1 核心服务差距显著——configx/schedulex/evidence/observex/transport 均无法满足量化交易生产需求。Exchange adapter 从 scaffold 到 production 的差距最大。Storage adapter 分为两个梯队：postgresx 就绪，redisx/kafkax/natsx 接近就绪，taosx/ossx/clickhousex 需要 mock + pool。
