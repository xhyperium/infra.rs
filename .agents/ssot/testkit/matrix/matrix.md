# MATRIX-TESTKIT-002 · 合同追踪矩阵

| 字段 | 值 |
|---|---|
| Active Spec | [SPEC-TESTKIT-002](../spec/spec.md) |
| package / current version | `testkit` / `0.1.3` |
| 身份 | T0 测试支持平面；L1 确定性测试原语；非 production runtime |
| 当前 Gate | **`c4604ce` 机器门禁 PASS；独立复审待裁决** |

## 1. 能力矩阵

| 能力 | 归属 | 当前合同 | 验证 | 声明边界 |
|---|---|---|---|---|
| 确定性 wall/mono | `ManualClock` | 单 Mutex、checked、失败不修改 | CLK-01…03、05、07 | crate 内测试 |
| wall fault | `ManualClock` | 三态精确映射，不影响 mono | CLK-04 | crate 内测试 |
| poison | `ManualClock` | 控制报错；now unavailable；mono 恢复原值 | CLK-06 | 不伪造零、不 panic |
| clock domain | `ManualClock` | 实例独立、跨域不可比较 | CLK-08、10 | process-lifetime 且 allocator 未耗尽 |
| 确定性 scenario | `IntegrationHarness` | 顺序、一次性、首错停止 | HAR-01…03、09 | 无网络/进程/I/O |
| panic 隔离 | `IntegrationHarness` | catch unwind → typed outcome/error | HAR-04、11 | 非通用进程隔离 |
| 生命周期 | `IntegrationHarness` | 消费型 builder；重跑/运行后追加不能编译 | HAR-02、06…08 | 不返回缓存成功、不静默丢 step |
| terminal result | `HarnessReport` / `HarnessRunError` | 成功持 clock+records；失败保留 report+source | HAR-03…05、10…12 | typed、可追溯 |
| 记录 | `StepRecord` | 私有字段 + getter + 四态 outcome + snapshot | HAR-05、10 | 无 epoch 0 sentinel |
| trait contract suites | `contract-testkit` | 独立 crate | 独立 SSOT | 不塞入 testkit core |
| 真实 external integration | tools/CI | OOS | 未在本矩阵记 PASS | 网络/进程/容器/真实服务 |

## 2. 需求到 Gate

| Spec 条款 | Test | Gate |
|---|---|---|
| §3 ManualClock | CLK-01…10 | G-02、G-05 |
| §4 runner | HAR-01…12 | G-03、G-04、G-05、G-07 |
| §5 依赖消费 | BND-01…04 | G-06、G-07 |
| §6 禁止项 | BND-04…06 | G-03、G-07 |
| §7 验收 | 全矩阵 | G-08 |
| §8 版本/residual | 版本检查、CLK-10 | G-09 + reviewer residual 裁定 |

## 3. 声明矩阵

| 声明 | 允许？ | 说明 |
|---|---|---|
| “testkit 是 T0/L1 test-support” | 是 | 当前架构身份 |
| “IntegrationHarness 是确定性 crate 内 runner” | 是 | 仅内存 step 与 ManualClock |
| “testkit 提供真实外部集成 harness” | 否 | tools/CI OOS |
| “crate 公开面仅四类型” | 否 | 四类型仅指 ManualClock 子模块 |
| “epoch 0 可作为错误 fallback” | 否 | 只允许调用方显式选择 epoch 0 |
| “历史 PASS 覆盖当前变更” | 否 | 必须固定候选并新鲜验证 |
| “testkit 是 production runtime/package stable” | 否 | `publish=false`，无生产运行时层级 |

## 4. 当前残余

- `R-CLK-DOMAIN-EXHAUSTION`：OPEN；保持 process-lifetime 边界，禁止回绕后继续宣称唯一。
- `R-EXTERNAL-HARNESS`：OOS；不与 crate 内 `IntegrationHarness` 混同。
- typed runner、`thiserror` 错误、fail-closed/并发/公开方法测试和 PATCH bump：IMPLEMENTED；前一
  `c27b7ce` 裁决已失效；`c4604ce` 全仓门禁 PASS，独立终审待裁决。
