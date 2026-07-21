# `bootstrap` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 实现合同；非生产就绪 |
| Package / lib | `xhyper-bootstrap` / `bootstrap` |
| Path | `crates/bootstrap` |
| Layer | L1 唯一组合根（R3.1） |
| Authority | 本文件是 active current-state spec |
| Candidate | [SPEC-INFRA-BOOTSTRAP-002](../../../../draft/bootstrap-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Implementation snapshot | `b0934baa`（2026-07-15） |
| Document commit | `e0b98df4` |
| Verified at | `e0b98df4`（相关实现路径未变化） |

> `[KNOWN]` 表示 Cargo、源码或 Accepted ADR 直接证明；`[INFERRED]` 表示由这些证据收窄出的结论。共同失败条件：相关权威或实现发生变化。

## 1. 定位与已批准边界

- `[KNOWN] HIGH` ADR-016 已裁定 `bootstrap` 为唯一组合根；runtime `gate` crate 已退役删除。
- `[KNOWN] HIGH` 运行时依赖通过 typed `PlatformContext`、`AppContext` 和 bounded contexts 暴露。
- `[KNOWN] HIGH` 禁止字符串、`Any`、`TypeId` Service Locator；禁止通用 `register` / `resolve` API。
- `[KNOWN] HIGH` bootstrap 可按 R3.1 依赖其他 L1 完成装配，但不得跨层重导出具体 adapter 类型。

非目标：通用 DI/插件框架、配置解析、重试/调度/传输实现、业务状态机、Evidence 核心或 adapter 实现。

## 2. 当前依赖

| 依赖 | 当前用途 |
|---|---|
| `xhyper-kernel` | `ShutdownGuard` / `ShutdownSignal`、错误分类 |
| `xhyper-contracts` | `Instrumentation` 与 5 个细粒度 venue/storage trait object |
| `xhyper-observex` | 默认 `TracingInstrumentation` |
| `xhyper-evidence` | 可选/必需模式的 `EvidenceAppender` |

dev-dependencies 含 `xhyper-binance`、`xhyper-redisx`、`xhyper-canonical`、`tokio`、`futures-util`；它们只证明测试装配，不证明生产应用已组装。

## 3. 当前公开 API

| 类型 | 当前职责 |
|---|---|
| `Bootstrap` | instrumentation/evidence/shutdown builder |
| `PlatformContext` | instrumentation、shutdown signal、可选 evidence |
| `AppContext` | 已构建只读上下文及兼容窄访问器 |
| `MarketDataContext` | market source、catalog、KV、platform |
| `ExecutionContext` | execution venue、account、venue time、platform |
| `BootstrappedApp` | `AppContext` + shutdown owner |
| `ShutdownController` | 消费自身触发 shutdown guard |
| `BootstrapError` | missing/invalid/unavailable 组合错误 |

当前没有 `Gate`、`Capability`、`register_capability`、`AppContext::gate` 或动态 mutation API。

## 4. 当前构建与错误语义

| 路径 | 返回 | 当前校验 |
|---|---|---|
| `build` | `AppContext` | 仅 `debug_assert!(validate)` |
| `try_build` | `Result<AppContext, BootstrapError>` | 强制校验 |
| `build_app` | `BootstrappedApp` | 间接走 `build` |
| `try_build_app` | `Result<BootstrappedApp, BootstrapError>` | 强制校验 |

`require_evidence()` 只在 `try_build*` 路径保证 fail closed；release 下 infallible 路径可绕过该前置条件。这是当前兼容事实与已知差距，不是批准的长期安全合同。

错误映射：missing dependency → `ErrorKind::Missing`；invalid configuration → `Invalid`；dependency unavailable → `Unavailable`。

## 5. 当前成熟度与开放项

- `[KNOWN] HIGH` workspace 非测试 Rust 消费方尚未使用 bootstrap；当前证据限于 14 个单元测试与 4 个 e2e 测试。
- `[KNOWN] HIGH` bounded contexts 已验证 typed trait object 组合，但没有真实 app 生命周期证据。
- `[KNOWN] HIGH` 当前没有通用组件启动/逆序补偿、composition manifest 或异步 drain/stop 合同。
- `[INFERRED] HIGH` 上述候选能力若触及跨层 trait/模块布局，必须先走批准流程；详情见 Candidate Draft。

反例条件：发现非测试 app/service 消费方会推翻“仅测试消费”；所有 public build 路径在 release 强制校验会推翻 require-evidence 差距。

## 6. 验收

```bash
cargo test -p xhyper-bootstrap
cargo check -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
cargo xtl lint-deps
cargo xtl no-new-gate
cargo fmt -- --check
```

通过条件：本文 API/依赖与源码一致；无 runtime gate/Service Locator 回流；R3/R3.1 通过。测试绿色不等于真实应用或生产生命周期闭合。

## 7. 追溯

- [ADR-016](../../../../../docs/architecture/adr/016-bootstrap-sole-composition-root.md)
- [PLAN-GATE-RETIRE-001](../../gate/plan/xhyper-gate-retirement-complete-plan.md)
- `crates/bootstrap/{Cargo.toml,src/,tests/e2e.rs}`
