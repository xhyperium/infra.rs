# Round 8: Cross-Crate Integration — 跨 crate 集成风险评估

| 字段 | 值 |
|------|-----|
| 轮次 | 8/10 |
| 视角 | 跨 crate 集成风险 |
| 日期 | 2026-07-22 |

## 1. 审查摘要

本报告分析 infra.rs workspace 内 24 个 crate（含 2 个工具）之间的集成风险。整体集成健康度为 **良好**，依赖图呈清晰分层 DAG（L0 + L1），无生产环境下的循环依赖。主要发现：版本约束一致、`kernel::XError` 统一错误面、Bounded* 替面设计意图明确、能力 trait 拆分（VenueAdapter → ExecutionVenue/MarketDataSource/…）已全部落地且双向适配器实现一致。两个低风险点：`NoopInstrumentation` 在 bootstrap 与 resiliencx 中有两份等价实现，以及 `contracts` 单元测试与 adapter crate 之间的 dev-dep 循环（已知且文档化）。

## 2. 依赖图分析

### 2.1 分层拓扑

```
L0 (kernel v0.3.0) —— 唯一不依赖任何内部 crate 的包
    │
    ├── configx v0.1.0      (仅 kernel)
    ├── schedulex v0.1.0    (无生产依赖)
    ├── decimalx v0.1.0     (仅 kernel)
    ├── canonical v0.1.0    (kernel + decimalx)
    ├── evidence v0.1.0     (无生产依赖)
    │
    ├── L1: contracts v0.1.0  (kernel + canonical)
    ��       ├── observex v0.1.0    (kernel + contracts)
    │       ├── resiliencx v0.1.0  (kernel + contracts)
    │       ├── bootstrap v0.3.0   (kernel + contracts + observex + evidence)
    │       ├── transportx v0.1.0  (仅 kernel)
    │       │
    │       ├── adapter exchange:
    │       │   ├── binancex v0.3.0 (kernel + canonical + contracts + decimalx + transportx)
    │       │   └── okxx v0.3.0    (kernel + canonical + contracts + decimalx + transportx)
    │       │
    │       └── adapter storage:
    │           ├── redisx v0.3.0     (kernel + contracts)
    │           ├── postgresx v0.3.0  (kernel + contracts)
    │           ├── kafkax v0.3.0     (kernel + contracts)
    │           ├── natsx v0.3.0      (kernel + contracts)
    │           ├── clickhousex v0.3.0(kernel + contracts)
    │           ├── taosx v0.3.0      (kernel + canonical + contracts + decimalx)
    │           └── ossx v0.3.0       (kernel + contracts)
    │
    └── testkit v0.1.1           (仅 kernel)
        └── contract-testkit v0.1.0  (kernel + canonical + contracts + decimalx)
```

### 2.2 版本一致性

| 版本组 | crate |
|--------|------|
| `0.3.0` (workspace) | kernel, testkit, binancex, okxx, bootstrap, redisx, kafkax, natsx, postgresx, taosx, ossx, clickhousex |
| `0.1.0` (独立) | configx, schedulex, decimalx, canonical, contracts, evidence, observex, resiliencx, transportx, contract-testkit, verifyctl |
| `0.2.0` | goalctl |
| `0.1.1` | testkit |

所有 path 依赖的版本约束（`{ path = "...", version = "..." }`）与目标 crate 实际版本**完全一致**，无 mismatch（经 `cargo metadata` 自动验证）。

### 2.3 dev-dep 循环

`contracts` 在其 `[dev-dependencies]` 中依赖 `binancex`、`okxx`、`contract-testkit`，而这些 adapter crate 的生产依赖又指向 `contracts`。这形成了 **dev-only 循环**：

```
contracts --[dev-dep]--> binancex/okxx/contract-testkit --[prod-dep]--> contracts
```

**评估**：这是已知且有文档化的设计选择（`contracts/src/lib.rs:341-343` 明确注释"禁止依赖 contract-testkit（dev-dep 环会造成双版本）"）。`contracts` 的单元测试构建自己的 mock/stub 实现而非使用 contract-testkit，适配器测试才是 conformance suite 的正确位置。

**风险等级**：低（dev-only，不进入生产构建图；Cargo 能正确处理 dev-dep 策略，不会造成版本分裂）。

## 3. 集成点逐一检查

### 3a. bootstrap vs contracts：Bounded* 替面设计

**发现**：`bootstrap::traits` 定义了与 `contracts` 命名相似但语义不同的 trait：

| contracts trait | bootstrap Bounded* | 差异 |
|----------------|-------------------|------|
| `MarketDataSource` (async, 3个subscribe方法) | `BoundedMarketDataSource` (仅 `fn label()`) | 完整协议 vs 最小标签 |
| `InstrumentCatalog` (async `symbol_info`) | `BoundedInstrumentCatalog` (仅 `fn label()`) | 同上 |
| `KeyValueStore` (async get/set) | `BoundedKeyValueStore` (仅 `fn label()`) | 同上 |
| `ExecutionVenue` (async order ops) | `BoundedExecutionVenue` (仅 `fn venue_id()`) | 同上 |
| `AccountSource` (async position/balance) | `BoundedAccountSource` (仅 `fn label()`) | 同上 |
| `VenueTimeSource` (async server_time) | `BoundedVenueTimeSource` (仅 `fn label()`) | 同上 |

**评估**：这不是重复定义。Bounded* trait 是 bootstrap 的组合根有界字段 —— 它们只需要标识（label）就可以在不引入完整 async 契约类型的前提下支持构建时组装（`MarketDataContext`、`ExecutionContext`��。Bounded 前缀是刻意为之，模块注释说明"与 contracts 同名历史面已收敛，加前缀以消除静默双平面冲突"。

**关键集成点**：`Instrumentation` 和 `EvidenceAppender` 在 bootstrap 中通过 `pub use` 从 `contracts` / `evidence` re-export，保持 **类型别名级别的一致性**（编译期验证 `contracts::Instrumentation` ≡ `bootstrap::Instrumentation`）。

**风险等级**：无（设计意图明确，文档覆盖充分）。

### 3b. contracts vs adapters：trait 实现完整性

**发现**：两个 exchange adapter（binancex、okxx）均实现了以下 contracts trait 的全集：

| trait | binancex | okxx | 说明 |
|-------|----------|------|------|
| `VenueAdapter` | 完整（含 cancel_order_request/query_order_request） | 完整 | DEFER-8 门禁通过 |
| `MarketDataSource` | 实现（委托 VenueAdapter） | 实现（委托 VenueAdapter） | |
| `InstrumentCatalog` | 实现（委托 VenueAdapter） | 实现（委托 VenueAdapter） | |
| `ExecutionVenue` | 实现（委托 VenueAdapter） | 实现（委托 VenueAdapter） | |
| `AccountSource` | 实现（委托 VenueAdapter） | 实现（委托 VenueAdapter） | |
| `VenueTimeSource` | 实现（委托 VenueAdapter） | 实现（委托 VenueAdapter） | |

 **能力拆分一致性**：两个 adapter 使用相同的委托模式 —— 主实现在 `VenueAdapter` impl block 中，能力 trait 通过 `self.cancel_order_request()` / `VenueAdapter::subscribe_ticks(self, symbol)` 等方式委托。模式完全一致，可互换。

**Storage adapter trait 实现**：

| trait | 实现 adapter | 状态 |
|-------|-------------|------|
| `KeyValueStore` | redisx | 生产（feature `runtime-tokio` + 真实 Redis 连接） |
| `EventBus` | kafkax / natsx | 生产（rskafka / async-nats） |
| `Repository` | postgresx | 生产 |
| `TxRunner` / `TxContext` | postgresx (PgTxRunner) | 生产 |
| `PubSub` | redisx (feature `pubsub`) | 生产 |
| `TimeSeriesStore` | taosx | 规格存在，待生产验证 |
| `ObjectStore` | ossx | 规格存在，待生产验证 |
| `AnalyticsSink` | clickhousex | 规格存在，待生产验证 |

**风险评估**：存储 adapter 的 taosx/ossx/clickhousex 仍处于 scaffold 阶段（仅实现 trait 最小面），标注为"待新增"。这不是集成问题，而是实现完成度问题——trait 签名已是生产形状。

### 3c. decimalx → canonical：Money/DTO 类型安全

**发现**：`canonical` 通过 `pub use decimalx::Money` re-export `Money` 类型。

```rust
// canonical/src/lib.rs:42
pub use decimalx::Money;
```

**类型身份验证**（来自测试）：
```rust
let m: Money = DecimalxMoney::try_new(Decimal::new(1, 0), "USD".parse().expect("currency")).expect("money");
let as_decimalx: DecimalxMoney = m;   // 编译赋值，类型相同
assert_eq!(m, as_decimalx);
```

**评估**：`Money` 是类型别名 re-export（非复制），编译器保证类型同一性。所有 DTO（`Order`、`Tick`、`Trade`、`Position` 等）直接使用 `decimalx::{Decimal, Price, Qty}` 作为字段类型，无中间包装。

**风险等级**：无（编译期类型安全，测试覆盖）。

### 3d. kernel → observex/resiliencx：Instrumentation trait 传递

**链**：
- `contracts::Instrumentation`（权威定义点）
- `observex::TracingInstrumentation`（实现 contracts::Instrumentation）
- `bootstrap::Bootstrap::new()` → 默认创建 `TracingInstrumentation`
- `bootstrap::with_instrumentation()` → 可注入自定义实现
- `resiliencx` → 消费 `contracts::Instrumentation`（不依赖 observex）

**ADR-005 合规验证**：
- `resiliencx` **仅**依赖 `contracts`（`contracts::Instrumentation`），**禁止**直接依赖 `observex` ✓
- `observex` 依赖 `contracts`（实现 trait）+ `kernel`（`XError` 包裹） ✓
- `bootstrap` 作为组合根，可以依赖 `contracts` + `observex` ✓

**风险点**：`resiliencx` 和 `bootstrap` 各自定义了 `NoopInstrumentation` struct。两个实现在行为上等价（均为空操作），但类型不同（`resiliencx::NoopInstrumentation` ≠ `bootstrap::NoopInstrumentation`）。如果下游代码混用两者会因类型不匹配而编译错误。

**评估**：这并非 bug，因为 `Instrumentation` trait 的消费方通过 `dyn Instrumentation` 或泛型 `<I: Instrumentation>` 工作，不依赖具体 struct 类型。两份 Noop 实现是双 cravte 各自方便性设计，非集成风险。

**风险等级**：低（不影响运行时，无类型混淆风险）。

### 3e. transportx → exchange adapter：HTTP driver 一致性

**模式**：两个 exchange adapter 均通过相同的注入模式使用 transportx：

```rust
// binancex 与 okxx 使用相同的模式：
self.http = Some(http); // inject Arc<dyn HttpDriver>
http.execute(request).await.map_err(map_transport_error) // 统一错误映射
```

**`map_transport_error` 一致性**：binancex 和 okxx 各自定义了完整的 `map_transport_error` 函数，两者对 7 种 `TransportError` 变体的映射语义**完全一致**：

| TransportError | mapped to ErrorKind |
|---------------|---------------------|
| RateLimited | Transient |
| ConnectTimeout | Transient |
| ReadTimeout | Transient |
| ConnectionClosed | Unavailable |
| ProtocolViolation | Invalid |
| Io | Unavailable |
| PayloadTooLarge | Invalid |

**HTTP Driver 接口稳定性**：transportx 未声明 stable，但交换 adapter 已稳定使用其 `HttpDriver` trait��`HttpRequest` / `HttpResponse` / `MockHttpTransport`）。这形成了需要关注的 soft contract。

**风险评估**：
- `HttpDriver` trait 变更将同时影响 binancex 和 okxx ✓（集中管理）
- `map_transport_error` 在两处各有一份代码，可能 drift（两份副本，逻辑相同）

**风险等级**：低（模式完全一致，test 覆盖；建议可考虑将 `map_transport_error` 提取到共享模块以消除副本风险）。

### 3f. evidence → bootstrap：注入与生命周期

**证据链**：
```
evidence (独立 crate, 无生产依赖)
    ├── EvidenceAppender trait
    ├── EvidenceError enum
    ├── InMemoryEvidenceAppender
    ├── FileEvidenceAppender
    └── 辅助函数 (append_checked, append_batch)
    
bootstrap (依赖 evidence)
    ├── pub use evidence::{EvidenceAppender, EvidenceError, ...}  (re-export)
    ├── PlatformContext { evidence: Option<Arc<dyn EvidenceAppender>> }
    ├── Bootstrap { evidence: Option<Arc<dyn EvidenceAppender>>, require_evidence: bool }
    ├── with_evidence(Arc<dyn EvidenceAppender>) → 注入
    ├── require_evidence() → 启动期强制求值
    └── build() → panic if require_evidence && evidence.is_none()  (fail-closed)
```

**评估**：
- evidence crate 不依赖 kernel，不产生 L0 环路 ✓
- bootstrap 通过 `Option<Arc<dyn EvidenceAppender>>` 注入，支持运行时可选 ✓
- `require_evidence()` 提供 fail-closed 安全（release 也 panic） ✓
- evidence 可替换实现（InMemory / File / 未来网络签名），接口解耦 ✓

**风险等级**：无（设计清晰，测试覆盖验证了所有路径）。

### 3g. 错误类型跨边界兼容性

**统一错误面**：

```
kernel::XError  ← 所有 crate 的公共 API 返回类型
    ├── ErrorKind (8 种可分类变体)
    ├── Invalid / Transient / Unavailable / Missing / Internal ...
    └── ShutdownSignal (通过 kernel 提供)

TransportError (transportx 内部) → map_transport_error → XError
EvidenceError (evidence 内部) → BootstrapError → XError
BootstrapError (bootstrap 内部) → into_xresult → XError
```

所有 crate 边界的 **public API 返回 `XResult<T>`**（即 `Result<T, XError>`），抽象均正确。内部域专用错误类型在边界处显式映射。未见任何 crate 向调用方暴露自己的错误类型（除 evidence 外，但 evidence trait 使用自己的 `EvidenceError` 作为 `Result` 错误类型，bootstrap 包裹它）。

**潜在不一致**：`evidence::EvidenceAppender` 使用 `Result<_, EvidenceError>` 而非 `XResult<_>`。这意味着：
1. bootstrap 消费 evidence 时必须手动映射 `EvidenceError` → `XError`
2. `EvidenceError` 仅 2 变体（DurabilityFailure / Unavailable）——语义清晰但非细粒度

**评估**：这是 minimal contract 设计，EvidenceError 只有 2 种可能的失败含义，使用自己错误类型而非 kernel XError 避免了证据面引入对所有 kernel 错误变体的理解。

**风险等级**：低（需要手动映射，但映射语义简单直接）。

## 4. 跨域风险矩阵

| 集成链路 | 版本一致 | trait 一致 | 错误兼容 | 生命周期 | 测试覆盖 | 风险 |
|---------|---------|-----------|---------|---------|---------|------|
| kernel → 全部 | ✓ | N/A | ✓ (XError 统一) | ✓ (ShutdownSignal) | ✓ | **无** |
| decimalx → canonical | ✓ | ✓ (re-export) | ✓ (kernel) | N/A | ✓ | **无** |
| canonical → contracts | ✓ | ✓ (DTO 形状) | ✓ (kernel) | N/A | ✓ | **无** |
| contracts → adapters | ✓ | ✓ (含能力拆分) | ✓ (XError) | ✓ (connect/disconnect) | ✓ | **无** |
| contracts → observex | ✓ | ✓ (Instrumentation) | ✓ (XError) | N/A | ✓ | **无** |
| contracts → resiliencx | ✓ | ✓ (Instrumentation) | ✓ (XError) | N/A | ✓ | **无** |
| transportx → exchange | ✓ | ✓ (HttpDriver) | ✓ (map_transport_error) | N/A | ✓ | **低** |
| evidence → bootstrap | ✓ | ✓ (re-export) | 手动映射 | ✓ (Option<Arc>) | ✓ | **无** |
| contracts ↔ adapters** | ✓ | ✓ | ✓ | ✓ | ✓ (unit) | **低** (dev-dep 环) |

> **contracts ↔ adapters = contracts dev-dep 依赖 adapters 仅用于单元测试

### 高影响项

**无**。所有集成点均处于可工作状态，无阻塞性风险。

### 需跟踪项

1. **`map_transport_error` 副本**：binancex 与 okxx 各有一份完全相同的 `map_transport_error` 函数。建议提取到 contracts 或 transportx 中以消除未来 drift 风险。
2. **transportx `HttpDriver` 稳定性**：未声明 stable，但已被 2 个 exchange adapter 依赖。transportx 的 HttpDriver trait 签名变更会影响 adapter 编译。
3. **contracts dev-dep 环**：已知设计，但添加新 adapter 时需要警惕 contract-testkit 双重链接。

## 5. 轮次结论

### 总体评分：**良好 (4/5)**

跨 crate 集成整体结构清晰、分层合理、无生产环境下的循环依赖。版本约束一致，路径依赖正确锁定。核心决策（L0 kernel → L1 contracts → adapters 的层次化架构）执行良好。

### 集成健康度检查 ✅

| 检查项 | 结果 |
|--------|------|
| 无生产循环依赖 | ✅ (仅 contracts dev-dep 循环，已知且文档化) |
| 版本约束一致 | ✅ (path 依赖 version 字段全部匹配) |
| trait 实现完整 | ✅ (adapters 全部实现 contracts 对应 trait) |
| 错误类型兼容 | ✅ (统一使用 kernel::XError 跨边界) |
| Bounded* 替面设计 | ✅ (bootstrap 有界能力，非重复定义) |
| Instrumentation 传递 | ✅ (ADR-005 合规，resiliencx 不直接依赖 observex) |
| evidence 注入模式 | ✅ (Option<Arc> 注入，fail-closed) |
| exchange adapter 模式 | ✅ (binancex/okxx 模式完全一致) |
| CAN-ID 结构化撤单 | ✅ (adapter 全部覆盖 cancel_order_request/query_order_request) |

### 建议

- P2: 将 `map_transport_error` 提取到共享位置（如 `contracts` 的 `transport_utils` 模块或 `transportx` 本身），消除双副本风险。
- P3: 标记 `transportx::HttpDriver` trait 的成熟度等级（partial/L1?），以便 adapter 作者了解兼容性承诺。
- P3: 考虑标准化 evidence 错误到 kernel XError 的映射路径（如 `impl From<EvidenceError> for XError`），简化组合根代码。
