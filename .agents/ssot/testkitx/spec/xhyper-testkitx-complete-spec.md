> **状态：Superseded（已被取代）**  
> 历史 L1/`testkitx` 路径文档。权威：[`../../testkit/testkit-complete-spec.md`](../../../testkit/testkit-complete-spec.md)  
> 计划：[`../../testkit/plan/plan.md`](../../../testkit/plan/plan.md)  
> **不得**被读成第二实现或并行 SSOT。物理包为 `testkit` @ `crates/testkit`（T0 test-support，非 L0 runtime）。

---

# testkit 实现规范

状态：当前 `0.1.0` 最小测试脚手架合同  
逻辑名（历史）：`testkitx`  
物理 package / 路径：`testkit` @ `crates/testkit`

> 历史文档可能写 `testkitx` 或 `crates/testkit/testkitx`；以根 `Cargo.toml` members 与源码为准。

## 1. 目的

L0 测试宏与契约测试脚手架（spec §4.1）。**不**进入生产运行时依赖图。

## 2. 当前依赖与实现面

| 项 | 事实 |
|----|------|
| 依赖 | `kernel`（**非**零依赖；**非** `xlib_harness`） |
| 已实现 | `ManualClock`；`xlib_test!`；`mock!`；`provider_capability_contract_tests!` |
| **未实现** | 完整 fixture builder 产品面、Docker/真实服务 harness（属 INFRA-010+ TARGET） |

## 3. 非目标

- 不替代各 crate 领域正确性证明。
- 不提供真实交易所/存储连通保证。
- 不实现 INFRA harness 的 compose/fault/evidence 管线。

## 4. 验收

- 路径与 package 名与 Cargo 一致。
- README 不得写“零依赖”。
- 与 `scripts/integration/*` TARGET stub 分工明确：testkit 是单测脚手架，不是真实集成 harness。
