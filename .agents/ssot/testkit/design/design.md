# DESIGN-TESTKIT-002 · 确定性测试支持设计

| 字段 | 值 |
|---|---|
| Source Spec | [SPEC-TESTKIT-002](../spec/spec.md) |
| 状态 | Active |
| package / version | `testkit` / `0.1.3` |
| 架构身份 | T0 测试支持平面；L1 确定性测试原语；非 production runtime |

本文只解释当前设计取舍。冲突时以 spec 的可执行合同为准，实现与测试必须同时满足合同。

## 1. 模块边界

```text
testkit
├── clock
│   ├── ManualClock
│   ├── ManualClockError
│   ├── ManualClockFault
│   └── ManualClockSnapshot
└── harness
    ├── IntegrationHarness
    ├── HarnessReport
    ├── HarnessRunError
    ├── StepOutcome
    └── StepRecord
```

`clock` 提供确定性时间；`harness` 只编排内存 step。业务能力只依赖 `kernel`；crate 专用错误统一由
workspace `thiserror` 派生，避免手写 `Error` 合同漂移。

`IntegrationHarness` 的“Integration”指多个 crate 内动作组成的 scenario，不表示它拥有真实外部系统。外部 harness 的进程、容器、网络、凭据、真实端口和 evidence 生命周期归 tools/CI，保持 OOS。

## 2. ManualClock 设计

### 2.1 单锁状态机

墙钟、单调流逝与墙钟 fault 放入同一私有 `Mutex<State>`。这样一次 snapshot 对应一个明确线性化点，写入失败也能在提交前完成 checked 计算，保持全状态不变。

`domain` 是构造后不变的身份，放在锁外。每个实例分配独立 domain，使不同模拟时间线不能被误比较。

### 2.2 错误策略

- 控制 API 有错误通道：poison 映射为 `ManualClockError::Synchronization`。
- `Clock::now()` 有错误通道：fault 精确映射，poison 映射为 `ClockError::Unavailable`。
- `Clock::monotonic()` 没有错误通道：恢复 poisoned guard 中的真实状态，不创造默认值。
- 算术先 checked、后写入；失败不产生部分提交。

显式 epoch 0 可以是测试输入，但任何内部失败都不能回落到 epoch 0。`Default` 会模糊“调用方真的选择 epoch”与“忘记初始化”，因此禁止。

### 2.3 Domain allocator

domain 唯一性是 process-lifetime 范围内的有限资源合同。当前实现若仍用递增 `u64`，必须防止回绕；在 typed exhaustion 尚未闭合前，规格保留 residual，不扩张为跨进程或无限生命周期保证。

## 3. Runner 设计

### 3.1 所有权状态机

```text
IntegrationHarness
  ├── step(self) → IntegrationHarness
  └── run(self)  → HarnessReport | HarnessRunError
```

runner 是消费型 builder。`step(self) -> Self` 与 `run(self)` 让 Rust 所有权在编译期排除运行后追加和重跑；不存在可被误解为再次执行的缓存返回路径。step 业务错误是泛型 `E: Error + Send + Sync + 'static`，runner 保存 source，而不是提前抹平成字符串。

### 3.2 执行顺序

runner 顺序取出登记的 `FnOnce(&ManualClock)`。每次只执行一个 step，不隐式创建线程或 async runtime。首个失败、panic 或 runner/clock 错误结束 scenario；余下 step 保持未执行状态。

调用方闭包永远在 ManualClock 锁外执行。runner 用 `catch_unwind` 把 step panic 转换为 `StepOutcome::Panicked` 和 terminal `HarnessRunError`，避免整个测试进程被 runner 控制路径意外展开。

### 3.3 双层结果

- `StepOutcome::{Passed, Failed, Panicked, ObservationFailed}` 描述单步终态。
- `HarnessRunError` 描述 terminal 执行失败，保留 report 与原始 source chain。
- `StepRecord` 保存名称、outcome 和显式可用的前后 snapshot；字段私有，只读 getter 对外。
- `HarnessReport` 持有最终 `ManualClock` 与 records；时钟/记录访问及断言 helper 只放在 report。

外层 `run()` 不再用 `Err(&[StepRecord])` 充当错误类型。成功调用方读取 `HarnessReport`；失败调用方从 `HarnessRunError` 读取 terminal report、records 与 source。

### 3.4 缺失观测

runner 用 `ManualClock::snapshot` 记录 step 前后状态。snapshot 同步失败或其中存在 wall fault 时形成 `ObservationFailed`；不可得的 snapshot 以显式缺失表示，不能写入 `0`。这既避免 epoch sentinel，也避免把实际 epoch 0 测试误判为错误。

## 4. 依赖与安全

- crate normal dependency 仅 `kernel` + `thiserror`；runner 不引入第三方 runtime。
- 不执行网络、进程、文件、环境或真实时间操作。
- 无 `unsafe`；不以析构器检查 expectation；不在 unwind 中二次 panic。
- test assertion helper 可以 panic，但生命周期和执行控制返回 typed error。

## 5. 版本设计

当前交付版本为 `0.1.3`。runner typed error、记录封装和 fail-closed 行为已执行一次 PATCH bump，并须同步所有版本与对齐材料。

## 6. 非目标

- 不提供通用 mock 框架、fixture DSL 或测试宏。
- 不提供 async scheduler、retry、timeout 或并行 runner。
- 不提供真实 external integration harness。
- 不宣称 production runtime、crate publication 或无限 domain 唯一性。
