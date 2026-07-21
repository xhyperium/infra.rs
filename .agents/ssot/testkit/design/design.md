# DESIGN-TESTKIT-002 · Deterministic Test Support（索引式设计）

| 字段 | 值 |
|------|-----|
| Design ID | `DESIGN-TESTKIT-002` |
| Source Spec | `SPEC-TESTKIT-002`（[spec/spec.md](../spec/spec.md)） |
| 形态 | **索引式**：设计权威在 spec §1–§7、§25；本文件不重复 2198 行 spec 内容，只做设计要点的导航与不变量锚定 |
| Package | `xhyper-testkit` @ `crates/testkit`（lib `testkit`） |
| Version | `0.1.1` · Stable（`publish = false`） |
| Plane | T0 / test-support（无 production layer） |
| Spec Status | **Stable** 2026-07-14 |
| Ship | PR [#247](https://github.com/xhyperium/infra.rs/pull/247) · [#254](https://github.com/xhyperium/infra.rs/pull/254) · [#255](https://github.com/xhyperium/infra.rs/pull/255) · tag `testkit-v0.1.1` |
| Residual SSOT | [plan/residual-open.md](../plan/residual-open.md) · DEF-001…010 全 CLOSED + 1 OPTIONAL |
| Status | **Active**（反映 ManualClock V2 已 ship 事实） |

## 设计权威导航

> 与 kernel 不同（kernel design 是 1149 行独立长文），testkit 的设计已内聚在 spec 本身。
> 本文件只做导航，不另立设计 SSOT。冲突时：**spec/spec.md → 本设计 → 源码 + 测试**。

| 设计主题 | 权威位置 |
|----------|----------|
| testkit 身份（T0 test-support · 与生产图正交 · 非测试工具大全） | spec §0 / §1 / §25 |
| 当前实现裁定（保留 ManualClock · 删除 xlib_test!/mock!/FixtureBuilder） | spec §2 |
| 组件划分（testkit / contract-testkit / integration harness / Fixture 所有权） | spec §3 |
| 依赖合同（只依赖 kernel · dev-dependency only · 生产图隔离） | spec §5 / §14 |
| ManualClock 完整合同（Mutex State · checked wall/mono · Fault 三态 → ClockError · Snapshot · poison · 无 Clone/Default） | spec §7 |
| 宏退役合同 | spec §8 |
| Contract Testkit（trait-level suites · broken impl negative tests） | spec §9 |
| Mock/Fake/Stub/Simulator 术语 | spec §10 |
| 测试确定性（禁 sleep · 禁真实时间 · 禁随机性 · 环境隔离） | spec §11 |
| 完成定义（§24.1–.6） | spec §24 |

## 设计不变量（锚定）

1. **正交于生产图**：testkit → kernel 是测试图依赖；所有消费为 dev-dependency，禁止进生产 normal graph（`cargo xtl test-graph-check`）。
2. **ManualClock runtime-neutral**：不读真实时间、不 sleep、不 unchecked arithmetic、不 Clone/Default；状态 Mutex 保护 + poison 显式恢复。
3. **极小公开面**：仅 4 类型（ManualClock/Error/Fault/Snapshot）；测试抽象只有在杀死真实错误时才有价值（spec §25）。
4. **诚实分层**：Approved/Stable ≠ §24 全闭合 ≠ production ready；OPTIONAL 项诚实标注。

## 当前实现事实

`crates/testkit/src/`：`lib.rs`（14 行 · `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]` + `#![deny(unreachable_pub)]`）· `clock.rs`（482 行 · ManualClock V2）。
依赖仅 `xhyper-kernel`（dev: `proptest`）。`crates/test-support/contracts/`（contract-testkit）。

**Status: Active（ManualClock V2 已 ship）· 0.1.1 Stable · `publish = false` 保持。**
