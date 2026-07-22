# `bootstrap` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.3.2`：L1 唯一组合根；正式 storage contracts 注入已落地；**非**完整应用运行时 |
| Package / lib | `bootstrap` / `bootstrap`（`xhyper-bootstrap` 仅为历史标签） |
| Path | `crates/bootstrap` |
| Layer | L1 唯一组合根（R3.1 / ADR-016） |
| Authority | 本文件是 active current-state spec |
| Verified at | 2026-07-22 第 4 轮可复现实验 |

> “已落地”只覆盖本文列出的组合合同，不等于交易栈、跨资源事务、全量生命周期或 package stable。

## 1. 定位与边界

- ADR-016 裁定 `bootstrap` 为唯一组合根；runtime `gate` 已退役。
- 依赖通过 typed `PlatformContext`、`AppContext`、bounded contexts 与固定槽位集合暴露。
- 禁止字符串、`Any`、`TypeId` Service Locator；禁止通用 `register` / `resolve`。
- 不重导出具体 adapter 类型，也不在生产依赖中绑定 Redis、NATS 等实现。

非目标：通用 DI/插件框架、配置解析、重试/调度/传输实现、业务状态机、跨资源事务协调、泛型全局 Repository 注册、完整异步组件启动编排。

## 2. 依赖与注入面

| 依赖 / 类型 | 当前用途 |
|---|---|
| `kernel` | shutdown 与统一错误语义 |
| `contracts` | `Instrumentation`、`KeyValueStore`、`EventBus` 等正式契约 |
| `observex` | 默认 `TracingInstrumentation` |
| `evidence` | 可选或必需的 `EvidenceAppender` |
| `ContractStoreSet` | 固定可选槽位：`Arc<dyn KeyValueStore>`、`Arc<dyn EventBus>` |
| `StoreSet` / `Bounded*` | 兼容的本地有界句柄面，继续保留 |

Redis/NATS 仅为 dev-dependency，用于组合实验；这不把具体 adapter 引入 bootstrap 的生产依赖。

## 3. 公开合同

- `Bootstrap::with_contract_store_set` 注入正式 storage trait objects。
- `PlatformContext::contract_store_set` 与 `AppContext::contract_store_set` 只读访问固定集合。
- `ContractStoreSet` 提供 typed `with_*`、借用访问器与 `Arc` clone 访问器；没有运行时注册表。
- `StoreSet`、`MarketDataContext`、`ExecutionContext` 兼容面保持可用。
- `AsyncDrain` 以 LIFO 执行显式注册的关停 hook；不等于分布式编排器。
- `require_evidence` 在 debug/release 的 infallible build 路径均 fail-closed；可恢复调用使用 `try_build*`。

## 4. 可复验证据

固定摘要 Redis 与 NATS 容器实验通过：真实 `RedisClient` / `NatsEventBus` 注入
`ContractStoreSet`，测试只从 `AppContext` 的正式 trait 访问器取得依赖并完成 KV set/get 与
NATS subscribe/publish。该实验明确不证明跨资源原子事务。

```bash
cargo test -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
node scripts/storage-composition-conformance.mjs
cmp .agents/ssot/bootstrap/spec/spec.md \
  .agents/ssot/bootstrap/spec/xhyper-bootstrap-complete-spec.md
```

## 5. NO-GO / OPEN

- 具体 adapter 的生产装配策略与凭据生命周期：由应用组合根调用方负责。
- 跨资源事务、通用 Repository 槽位、动态 Service Locator：明确不提供。
- composition manifest、全量异步启动与逆序补偿、交易栈端到端：OPEN。
- package stable / Agent L5：未宣称。

追溯：`crates/bootstrap/{src,tests/storage_composition_e2e.rs}`、
`scripts/storage-composition-conformance.mjs`、`docs/ssot/bootstrap-ssot-alignment.md`。
