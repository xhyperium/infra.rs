# AGENTS.md — contract-testkit

> 仓库级规则见 [`../../../AGENTS.md`](../../../AGENTS.md)。  
> 权威规范：SPEC-TESTKIT-002 §3.2 · [`.agents/ssot/testkit/spec/spec.md`](../../../.agents/ssot/testkit/spec/spec.md)

## 身份

- **T0 test-support**（非生产 runtime；`publish = false`）
- package：`contract-testkit` · lib：`contract_testkit`
- path：`crates/test-support/contracts`
- 角色：可复用 **Fake/Recording** + **per-trait conformance suite**

## 本 crate 约束

- 业务 crate **只能**通过 `[dev-dependencies]` 引用
- 允许依赖：`contracts` / `kernel` / `canonical` / `decimalx` / `async-trait` / `bytes` / `futures-*` / `tokio`
- `default = []`；禁止 feature 把本 crate 泄漏进 normal graph
- 禁止真实网络 / Docker / 凭据
- 验证：`cargo test -p contract-testkit` · `cargo clippy -p contract-testkit --all-targets -- -D warnings`

## 与 contracts / testkit 的关系

| crate | 职责 |
|-------|------|
| `contracts` | trait / type 出口（R4）；**无** Fake 实现 |
| `testkit` | ManualClock core only |
| `contract-testkit`（本 crate） | Fake + suite |

## 禁止占位

不得合并无行为 public placeholder；suite 必须驱动真实 trait 方法。
