# TEST-TESTKIT-002 · 验证合同

| 字段 | 值 |
|---|---|
| Source Spec | [SPEC-TESTKIT-002](../spec/spec.md) |
| 当前 package 版本 | `0.1.3` |
| 本轮状态 | **R3 GATES PASS / INDEPENDENT REVIEW PENDING** |
| 适用范围 | T0/L1 test-support；非 production runtime |

本文列出实现必须通过的测试，不记录未经本轮执行的 PASS。旧 PR、tag、release evidence 与历史 coverage 只能作回归线索，不能替代当前候选的新鲜结果。

## 1. ManualClock 测试矩阵

| ID | 场景 | 必须断言 |
|---|---|---|
| CLK-01 | 显式构造 | wall 等于输入；mono 为显式值；无 `Default` / `Clone` |
| CLK-02 | wall advance/rewind 边界 | checked 成功值正确；overflow/underflow 返回 typed error 且 snapshot 不变 |
| CLK-03 | mono advance/regression | 只前进；overflow/回退返回 typed error 且状态不变 |
| CLK-04 | fault set/clear | 三种 fault 精确映射；wall/mono 保存值不变；clear 后恢复 |
| CLK-05 | snapshot | wall/mono/fault 来自同一锁临界区；getter 值完整 |
| CLK-06 | poison | 控制路径 `Synchronization`；`now()` 为 `Unavailable`；`monotonic()` 恢复原状态且不 panic/不伪造零 |
| CLK-07 | concurrency | 并发读取与控制保持合法组合；多控制者推进无丢失更新；无 data race/部分快照 |
| CLK-08 | domain | 活跃实例 domain 独立；同 domain 可比较；跨 domain 返回 `None` |
| CLK-09 | epoch 0 | 显式 epoch 0 可往返；任何错误路径不得产生 epoch 0 fallback |
| CLK-10 | allocator 边界 | 防回绕或 typed exhaustion；未实现时 residual 保持 OPEN，不得写 PASS |

## 2. IntegrationHarness 测试矩阵

| ID | 场景 | 必须断言 |
|---|---|---|
| HAR-01 | 多步成功 | 按登记顺序只执行一次；outcome 全成功；时钟观测正确 |
| HAR-02 | 空 scenario | `run(self)` 返回空 `HarnessReport`；原 builder 已移动，不能再追加或重跑 |
| HAR-03 | 业务失败 | `StepOutcome::Failed` + `HarnessRunError`；`Error::source()` 保留原错误；首错后停止 |
| HAR-04 | step panic | panic 被截获；形成 panic outcome/error；不穿透 runner；后续 step 不执行 |
| HAR-05 | step 前后的 clock fault / snapshot failure | `ObservationFailed` + `HarnessRunError`；不写 epoch 0 sentinel；不伪造成功记录 |
| HAR-06 | 成功后重跑 | `run(self)` 后原 builder 已移动；compile-fail，不能返回缓存成功 |
| HAR-07 | 失败后重跑 | `run(self)` 后原 builder 已移动；compile-fail，不能重复执行或改写记录 |
| HAR-08 | 运行后追加 | `run(self)` 后原 builder 已移动；`step(self)` compile-fail，不存在静默丢 step 路径 |
| HAR-09 | 首错停止 | 失败步以后所有闭包均未调用，且没有成功记录 |
| HAR-10 | record/report 封装 | `StepRecord` 字段私有；名称/outcome/前后 snapshot getter 可读；report 持最终 clock+records；直接字段访问 compile-fail |
| HAR-11 | 非文本 panic payload | 明确记录 panic 类别；不因格式化再次 panic |
| HAR-12 | report assert helper | 成功断言通过；不满足时按测试惯例 panic；helper 仅在 `HarnessReport`，不改变 report 状态 |

## 3. 边界与图测试

| ID | 检查 | 通过条件 |
|---|---|---|
| BND-01 | Cargo normal dependency | 仅 `kernel`；第三方依赖由 workspace 集中管理 |
| BND-02 | feature | `default = []`；无 async/network/I/O feature |
| BND-03 | 消费图 | `testkit` 与 `contract-testkit` 的生产 normal dependents 为零 |
| BND-04 | 源树扫描 | 无真实时间、sleep、网络、文件、环境、进程、容器能力 |
| BND-05 | 退役 API | 四个已退役宏/placeholder 不存在且不能编译 |
| BND-06 | 公开面 | 导出四个 clock 类型及五个 runner 类型；`StepRecord` 无 public field；无意外公开项 |

## 4. 建议验证命令

主实现交付时按仓库门禁执行，至少包括：

```bash
cargo fmt --all --check
cargo clippy -p testkit --all-targets --all-features -- -D warnings
cargo test -p testkit
node scripts/quality-gates/check-workspace-deps.mjs
```

并运行仓库已有的 API surface、coverage、mutation、Miri 与 production-graph 检查。命令不存在、SKIP 或工具不可用均不得记为 PASS；应记录为 BLOCKED、NOT RUN 或 residual。

## 5. Evidence 规则

每次行为变化必须记录候选 SHA、命令、退出码、工具版本与结果。历史 `2026-07-14` evidence 不覆盖当前 runner 变更。实现与定向测试已经闭合；固定候选的独立重审和全仓门禁完成前不得改写为最终 PASS。
