# AGENTS.md — testkit

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md)。本 crate 的 active SSOT 是 [SPEC-TESTKIT-002](../../.agents/ssot/testkit/spec/spec.md)。`.agents/ssot/testkit` 是 infra.rs 本仓可维护的域规格 SSOT，不是 xhyper.rs 上游只读镜像。

## 身份

- package / lib：`testkit` / `testkit`
- 当前版本：`0.1.3`；相对 `0.1.2` 的行为变化已执行一次 PATCH bump
- 架构：T0 测试支持平面；L1 确定性测试原语；`publish = false`
- 不是 production runtime，不宣称 package stable
- 业务 crate 只能通过 `[dev-dependencies]` 引用

## 公开能力

`ManualClock` 子模块公开四类型：

- `ManualClock`
- `ManualClockError`
- `ManualClockFault`
- `ManualClockSnapshot`

runner 子模块公开：

- `IntegrationHarness`
- `HarnessReport`
- `HarnessRunError`
- `StepOutcome`
- `StepRecord`

“ManualClock 族四类型”不是整个 crate 的公开面上限。

## ManualClock 约束

- wall、mono、fault 使用单个 `Mutex<State>`；snapshot 同锁读取
- 所有控制算术 checked；失败返回 typed error 且不修改状态
- fault 只影响 wall；poison 时控制路径报错、`now()` unavailable、`monotonic()` 恢复原状态
- 不实现 `Default` / `Clone`；共享使用 `Arc`
- 每实例独立 domain；唯一性仅覆盖单进程生命周期内 allocator 未耗尽
- 禁止把 epoch 0 用作错误、缺失或 panic sentinel；调用方显式传入 epoch 0 仍是合法测试数据

## IntegrationHarness 约束

- 它是 crate 内确定性 scenario runner，只顺序执行内存闭包
- 不创建 runtime，不访问网络、进程、容器、文件、环境或真实时间
- 真实 external integration harness 归 tools/CI，仍为 OOS
- `step<F,E>(self, name, f) -> Self`，其中 `E: Error + Send + Sync + 'static`；保留原始 source
- `step_advance_wall/monotonic(self, ...) -> Self`；`run(self) -> Result<HarnessReport, HarnessRunError>`
- 消费型 builder 使成功/失败后重跑与运行后追加不能编译；不得退回运行时缓存语义
- step 失败、panic、clock fault 必须 fail-closed；panic 与 observation failure 返回 terminal `HarnessRunError`
- 首错停止；后续 step 不执行，不生成成功记录
- `StepOutcome::{Passed, Failed, Panicked, ObservationFailed}`；禁止退回 `bool + String`
- `HarnessReport` 持有最终 `ManualClock` 与 records；断言 helper 只属于 report
- `StepRecord` 字段私有，通过 getter 读取名称、outcome 与前后 snapshot；任何错误路径不得 `unwrap_or(0)`

## 依赖与禁止项

- normal dependency 仅 `kernel` + workspace `thiserror`；`default = []`
- 第三方依赖统一从 workspace 引用
- 禁止 `xlib_test!`、`mock!`、`FixtureBuilder`、`provider_capability_contract_tests!`
- 禁止 unsafe、占位 public API、unchecked 回绕、真实 sleep/时间、网络、文件 I/O、环境变量和全局可变状态

## 验证与证据

实现交付至少运行：

```bash
cargo fmt --all --check
cargo clippy -p testkit --all-targets --all-features -- -D warnings
cargo test -p testkit
node scripts/quality-gates/check-workspace-deps.mjs
```

并按 [test/test.md](../../.agents/ssot/testkit/test/test.md) 覆盖 clock fault、panic、重跑、运行后追加、private getters、domain 与 epoch sentinel。历史 evidence 不得直接作为当前候选 PASS。
