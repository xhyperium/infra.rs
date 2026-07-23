# `bootstrap` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.3.3`：L1 唯一组合根；正式 storage contracts 注入 + 进程内 typed composition + 同步关停 drain，**非**完整应用运行时 |
| Package / lib | `bootstrap` / `bootstrap`（`publish = false`） |
| Path | `crates/bootstrap` |
| Layer | L1 唯一组合根（R3.1 豁免） |
| Authority | 本文件是 active current-state spec |
| Baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| Verified at | 2026-07-23 · root 串行 coverage `963 / 963`；候选已重冻，本地 reviewer 完成，verifier 技术/证据初验完成；GitHub CI artifact pending |

> `[KNOWN]` 表示 Cargo、源码或仓库治理可直接证明；`[INFERRED]` 表示由这些证据收窄出的结论。共同失败条件：相关权威、manifest 或实现发生变化。

## 1. 定位与边界

- `[KNOWN] HIGH` bootstrap 是本 workspace 的唯一组合根；runtime `gate` crate 已退役。
- `[KNOWN] HIGH` 依赖通过 typed `PlatformContext`、`AppContext`、`StoreSet` 和 bounded contexts 暴露。
- `[KNOWN] HIGH` 禁止字符串、`Any`、`TypeId` Service Locator；禁止通用 `register` / `resolve` API。
- `[KNOWN] HIGH` bootstrap 可按 R3.1 依赖其他 L1 完成装配，但不得跨层 re-export 具体 adapter 类型。

非目标：通用 DI/插件框架、配置解析、重试/调度/传输实现、业务状态机、跨资源事务、泛型全局 Repository 注册、完整 Evidence 协议、异步 runtime、分布式关停编排器或完整应用生命周期。

## 2. 当前依赖

`crates/bootstrap/Cargo.toml` 的直接普通依赖只有四个 workspace path package；固定摘要 storage 组合实验另使用集中声明的第三方 dev-dependencies 与 `natsx` / `redisx` path dev-dependencies：

| 依赖 | 当前用途 |
|---|---|
| `kernel` | `ShutdownGuard` / `ShutdownSignal`、`XError` / `ErrorKind` |
| `contracts` | `Instrumentation` trait 权威，以及 `KeyValueStore` / `EventBus` 正式 storage contracts |
| `observex` | 默认 `TracingInstrumentation` |
| `evidence` | `EvidenceAppender` 与 `InMemoryEvidenceAppender` 的 re-export 来源 |

`ContractStoreSet` 只提供固定可选的 `Arc<dyn KeyValueStore>` / `Arc<dyn EventBus>` 槽位；Redis/NATS 仅用于组合实验，不进入生产依赖。六个 `Bounded*` venue/storage trait 定义在 bootstrap 本地 `traits` 模块，不是 `contracts` 的同名 API。依赖事实不证明真实交易应用已完成装配。

## 3. 当前公开 API

| 类型 | 当前职责 |
|---|---|
| `Bootstrap` | instrumentation / evidence / store / drain / shutdown builder |
| `PlatformContext` | instrumentation、shutdown signal、可选 evidence、StoreSet 与 ContractStoreSet |
| `AppContext` | 已构建只读上下文、正式 storage contracts 只读访问器、唯一 shutdown owner、trigger/drain 入口 |
| `MarketDataContext` | market source、catalog、KV、platform |
| `ExecutionContext` | execution venue、account、venue time、platform |
| `BootstrappedApp` | `AppContext` 应用包装；委托 trigger / graceful shutdown |
| `ShutdownController` | 兼容型显式 shutdown owner；消费自身触发 guard |
| `StoreSet` | 类型化可选 adapter 句柄集合 |
| `ContractStoreSet` | 正式 `KeyValueStore` / `EventBus` 固定 typed 槽位；无动态注册 |
| `AsyncDrain` / `DrainStepResult` | 同步 hook 快照、批内 LIFO 与逐步结果 |
| `BootstrapError` | missing / invalid / unavailable 组合错误 |

当前没有 `Gate`、`Capability`、`register_capability`、`AppContext::gate` 或动态 mutation API。

## 4. 构建与错误语义

| 路径 | 返回 | 校验与所有权 |
|---|---|---|
| `build` | `AppContext` | 全构建模式强制校验；失败 panic；成功产物持唯一 guard |
| `try_build` | `Result<AppContext, BootstrapError>` | 强制校验；成功产物持唯一 guard |
| `build_app` | `BootstrappedApp` | 间接走 `build`；成功产物持唯一 guard |
| `try_build_app` | `Result<BootstrappedApp, BootstrapError>` | 间接走 `try_build`；成功产物持唯一 guard |

`require_evidence()` 在 `build` / `build_app` 缺注入时 release/debug 均 fail-closed（panic）；可恢复路径 `try_build` / `try_build_app` 返回 `BootstrapError::MissingDependency`。

`take_shutdown_guard` 是兼容逃生口：显式取走后，后续 build 产物不会凭空重建 guard。错误映射为 missing → `ErrorKind::Missing`、invalid → `Invalid`、unavailable → `Unavailable`；drain 注册锁中毒属于依赖不可用并保留下层原因。

bootstrap 自有错误文本合同：

- `MissingDependency` Display / XError context：`缺少必需依赖：{name}`；
- `InvalidConfiguration` Display / XError context：`bootstrap 配置无效：{name}`；
- `DependencyUnavailable` Display 与 XError context 均为 `依赖不可用：{name}`；任意下层 source
  不内插进顶层用户文本，只通过 `#[source]` 链结构化保留；
- drain 自有文本使用 `关停钩子锁中毒`、`排空步骤失败`、`未知错误`。

re-export 类型、下层 source 与 hook 返回的 opaque context 由定义方负责；bootstrap 不翻译或吞掉下层原因。

## 5. Graceful shutdown 合同

- `trigger_shutdown(self)` 只触发 signal，不执行 drain。
- `graceful_shutdown(self)` 返回 `Result<Vec<DrainStepResult>, BootstrapError>`，固定先触发 signal，再执行 `AsyncDrain::drain`。
- drain 对本次快照按 LIFO 执行；单步 `Err` 记录为 `DrainStepResult` 并继续，返回所有正常返回步骤的结果。
- ownerless context 只有在 signal 已由外部 owner 触发时才允许 graceful drain；否则返回 `MissingDependency { name: "shutdown_guard" }` 且不执行 hook。
- graceful 的 ownerless 检查以 `ShutdownSignal::is_triggered()` 为线性化点；并发外部触发只有发生在检查前才能使本次成功。
- graceful 消费 `self`；Missing 返回时 context 与未执行 hook 一并丢弃，不能重试。
- `BootstrappedApp::into_parts` 把唯一 guard 显式转移给 `ShutdownController`；须先触发 controller，返回的 `AppContext` 才可 graceful drain。
- `run_drain(&self)` 只 drain，不触发 signal；需要完整顺序时使用 `graceful_shutdown`。

## 6. Drain 并发、阻塞与 panic 边界

`AsyncDrain` 沿用历史类型名，但当前 hook 是同步 `FnOnce() -> XResult<()>`，不是异步执行器。

- `register` 与 drain 快照在同一 mutex 上线性化；
- 快照前完成的注册进入本批，快照后的注册进入下一批；
- hook 在锁外执行；不同批 hook 可以并发，不承诺跨批全局 LIFO 或全程串行；
- hook 可以永久阻塞；本 crate 不提供 deadline 或取消；
- hook panic 不隔离，会中断当前 drain，未执行快照项随栈展开丢弃。

调用方必须在 hook 内自行建立有界等待、合作式取消、join 和 panic 策略。本文不宣称 timeout / cancellation safety。

## 7. 当前成熟度与测试事实

- `[KNOWN] HIGH` 测试分布在模块内单元测试、`tests/public_api.rs`、`tests/public_api_surface.rs` 与 `tests/storage_composition_e2e.rs`；不存在名为 `tests/e2e.rs` 的测试文件。
- `[KNOWN] HIGH` 公开测试覆盖四条 build 路径、关停所有权、signal-before-drain、ownerless fail-closed、批内 LIFO、错误后继续、兼容转移与 register/drain 快照。
- `[KNOWN] HIGH` 单元测试真实触发 drain mutex poison，覆盖低层 `Internal` 与 builder 层 `DependencyUnavailable` 映射。
- `[KNOWN] HIGH` 三类 `BootstrapError` 的 Display、XError context、kind 与 source 有精确中文合同测试。
- `[KNOWN] HIGH` 固定摘要 Redis/NATS 容器实验从 `AppContext` 的正式 trait 访问器完成 KV set/get 与 NATS subscribe/publish；只证明两个 contract 可调用，不证明跨资源事务。
- `[KNOWN] HIGH` `examples/minimal.rs` 是库外示例，不是生产 app 证据。
- `[INFERRED] HIGH` StoreSet、ContractStoreSet 与同步 drain 的进程内合同可验证，但不等于 package stable、完整异步生命周期、生产关停 SLA 或交易栈端到端装配闭合。

## 8. 验收

```bash
cargo fmt -p bootstrap -- --check
cargo test -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
node scripts/storage-composition-conformance.mjs
node scripts/quality-gates/cov-gate-100.mjs -p bootstrap --filter crates/bootstrap/src
cmp .agents/ssot/bootstrap/spec/spec.md \
    .agents/ssot/bootstrap/spec/xhyper-bootstrap-complete-spec.md
```

通过条件：API、依赖、测试路径与源码一致；正式 KV/EventBus 固定槽位和组合实验保持可复现；四条成功 build 路径不丢 guard；ownerless graceful fail-closed 且不运行 hook；graceful shutdown 顺序和结果合同有测；行覆盖率 100%；无 runtime gate / Service Locator 回流；双规格 `cmp` 同构。测试绿色不等于真实应用或生产生命周期闭合。

## 9. 追溯

- 仓库治理：`AGENTS.md` 与 `crates/bootstrap/AGENTS.md`
- Gate 退役计划：`.agents/ssot/gate/plan/xhyper-gate-retirement-complete-plan.md`
- 实现：`crates/bootstrap/{Cargo.toml,src/,tests/,examples/}`
- 对齐矩阵：`docs/ssot/bootstrap-ssot-alignment.md`
