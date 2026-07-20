# Round 6 — §10–§13 术语 / 确定性 / API 预算 / 测试合同

| 字段 | 值 |
|------|-----|
| Scope | 计划完备性（非实现验收） |
| Spec | `SPEC-TESTKIT-002` §10–§13 |
| Plan package | `plan.md` · `tasks.md` · `spec-inventory.md` · `gap-matrix.md` · `residual-open.md` · `approval-packet.md` · `.worktree/testkit-todo.md` |
| 日期 | 2026-07-14 |

## 检查项

| # | 规范点 | 计划应覆盖内容 |
|---|--------|----------------|
| 6.1 | §10 术语 | Stub/Fake/Mock/Simulator 定义；无 interaction verification 不得名 Mock；下游 `Mock*` 须登记审计 |
| 6.2 | §11 确定性 | 十条规则 + §11.1 sleep 例外 + §11.2 seed + §11.3 env 隔离；testkit 不提供 env helper |
| 6.3 | §12 API 预算 | 仅 4 类型 re-export；禁止 prelude/宏/DSL/mock/fixture/runtime/docker 等；新增须两消费者+RFC |
| 6.4 | §13.1 ManualClock unit | 18 条 bullet 全矩阵 → Task |
| 6.5 | §13.2 Clock suite 三分 | `ClockCommonContract` / `ManualClockDeterminismContract` / `SystemClockSmokeContract` |
| 6.6 | §13.3–13.5 | property / concurrency / compile assertions → Task |
| 6.7 | §13.6 mutation | score≥90% **且** 禁存活列表 8 条枚举可勾 |
| 6.8 | §13.7–13.9 | line/branch/Miri/contract 负测 |

## PASS

1. **§10 术语定义入口存在**：`spec-inventory.md` **I-TERM** 写明 Stub/Fake/Mock/Simulator 与「无 verification 不得名 Mock」。
2. **§10 下游命名策略有门禁位**：`T-GATE-011`（NAMING-001 warning 扫描 Mock*）+ residual `NAMING-001 下游 Mock* 全量 rename` DEFER + approval **A9**。
3. **§11 确定性清单存在**：**I-DET** 覆盖无真实时间/sleep/顺序依赖/全局状态/未固定 seed/时区/env/默认端口/吞后台错误/retry 当修 flaky。
4. **§11 与门禁挂钩**：`T-CLK-018` 源码守卫；archgate **TIME-001 / IO-001**（I-AG-*）；指标 `real_time_calls_*` / `sleep_calls_*` / `flaky_retry_usage` 在 **I-METRICS**。
5. **§12 API 预算完整**：**I-API** 冻结 4 符号 + 禁止表；`T-DEL-005` + `T-GATE-004` public API snapshot；plan §4/§5 与规范一致。
6. **§13.1 有 Task**：`T-CLK-014` AC=`I-TEST-UNIT 全勾`；inventory 摘要含构造/advance/rewind/overflow/mono/fault/snapshot/失败不变/无 Default·Clone/Send Sync。
7. **§13.3 property 有 Task**：`T-CLK-015` + **I-TEST-PROP** 指向 §13.3。
8. **§13.4 concurrency 有 Task**：`T-CLK-016` + **I-TEST-CONC**。
9. **§13.5 compile assertions 有 Task**：`T-CLK-017`（!Default !Clone Send Sync；无旧宏导出）。
10. **§13.7–13.8 门槛有登记**：I-TEST-COV / I-TEST-MIRI；`T-GATE-008…010`；DEF-010 OPEN。
11. **§13.9 contract 负测有 Task**：`T-CTC-011` reference + `T-CTC-012` broken kill 100% + **I-TEST-CTC-NEG**。
12. **§4.1 测试文件布局有挂钩**：`T-CLK-001`「目录符合 §4.1」（含 `manual_clock_contract.rs` 等文件名约束方向正确）。

## FAIL

### F6-1 — §13.2 Clock suite 三分：inventory 有、Task 无

- **规范引用**：§13.2「Clock suite 必须分为 `ClockCommonContract` / `ManualClockDeterminismContract` / `SystemClockSmokeContract`」；禁止错误要求所有 Clock 实现语义完全相同。
- **缺失**：
  - `spec-inventory.md` **I-TEST-CLK-CONTRACT** 已枚举三分名称；
  - **`tasks.md` 无任何 Task 的 AC 引用 `I-TEST-CLK-CONTRACT` 或三分 suite**；
  - `T-CLK-014` 仅「§13.1 全矩阵 / I-TEST-UNIT」；
  - `T-V10-006` 笼统「§13」不足以强制三分交付；
  - gap-matrix 将 §13.1–13.5 打包为「测试矩阵」，未单列 §13.2 三分风险。
- **建议补丁**：
  1. 新增 `T-CLK-021`（或扩展 `T-CLK-014`）：交付三分 suite 文件/模块；AC 必须 `I-TEST-CLK-CONTRACT` 全勾。
  2. 明确 `SystemClockSmokeContract` 归属（kernel 侧 smoke vs testkit 文档边界），避免实现时把「值不变」误套到 SystemClock。
  3. `manual_clock_contract.rs` 的 AC 写清：仅 ManualClockDeterminism + ClockCommon 中适用 ManualClock 的子集。

### F6-2 — §13.6 mutation 禁存活列表未枚举为可勾 I-*

- **规范引用**：§13.6「不得存活」8 条：
  1. `advance_wall checked_add → wrapping_add`
  2. `rewind_wall checked_sub → wrapping_sub`
  3. monotonic regression 判断反转
  4. fault 被忽略
  5. fault 清除无效
  6. snapshot 字段错配
  7. monotonic 与 wall 共用状态
  8. 失败后仍修改状态  
  目标 mutation score ≥ 90%。
- **缺失**：
  - **I-TEST-MUT** 仅写「§13.6 mutation ≥90%；禁存活列表」——**未逐条展开**；
  - `T-GATE-009` AC 仅「mutants ≥90% 或 residual DEFER」——**不绑定 8 条禁存活**；
  - 实现 10x / Evidence 无法对照「哪条 mutant 仍存活」勾选。
- **建议补丁**：
  1. 在 `spec-inventory.md` 增加 **I-TEST-MUT-1…8**（或 I-MUT-SURVIVE 表）逐条列出规范原文。
  2. `T-GATE-009` AC 改为：`mutation_score≥90%` **且** `I-TEST-MUT-*` 零存活（`mutants.json` / Evidence）。
  3. residual 若 DEFER「mutation 进 required PR CI」，仍须保留 **本地/nightly 对 8 条的显式验收**，不得 DEFER 掉列表本身。

### F6-3 — §10 下游 Mock*「逐项审计」无冻结清单

- **规范引用**：§10「当前 `MockBinanceAdapter` / `MockKvStore` 必须逐项审计…该改名属于下游任务，不阻塞 testkit spec，**但必须登记**」。
- **缺失**：
  - 仅有 `T-GATE-011`「扫描 + 登记下游」与 DEFER rename；
  - **计划包未冻结审计表**（符号 / crate / 是否含 expectation / 建议 Fake* 名 / 状态）；
  - 2026-07-14 仓库实测至少存在（非完整）：`MockBinanceAdapter`、`MockKvStore`、`MockObjectStore`、`MockTimeSeriesStore`、`MockRepository`/`MockTxRunner`、`MockNatsBus`、`MockHttpTransport` 等；计划 consumers 节只覆盖 testkit 宏/ManualClock，**未含 §10 命名审计 inventory**。
- **建议补丁**：
  1. 新增 `I-TERM-AUDIT` 表（或 residual「Mock* 审计清单」）冻结扫描结果 + OPEN/DEFER 状态。
  2. `T-GATE-011` AC 绑定该表行数与 `rg 'struct Mock'`  diff 策略。
  3. 与 approval **A9** 联动：首期 warning 的范围=表内行。

### F6-4（次要）— §13.1 / §13.3 细项依赖「见 §xx」摘要，未完全原子化

- **规范引用**：§13.1 共 18 bullet；§13.3 共 5 property 场景。
- **缺失**：I-TEST-UNIT / I-TEST-PROP 为摘要枚举，未像 I-CLK 那样逐符号 ID；与 plan §1.3「不得只写见 §xx」精神略冲突（仍写了「全部 bullet」）。
- **建议补丁**：可选将 §13.1 18 条、§13.3 5 条升为 I-TEST-UNIT-01… / I-TEST-PROP-01…，便于 T-CLK-014/015 逐勾。**不阻塞**若坚持「I-TEST-UNIT 全勾=规范 §13.1 全文」。

### F6-5（次要）— §11.2 seed 可重放 / §11.3 env 隔离无独立 AC

- **规范引用**：§11.2 失败输出 seed、可重放、禁 thread_rng 唯一输入；§11.3 串行/恢复/优先显式 config；testkit 不提供全局 env helper。
- **缺失**：I-DET 一行覆盖；无 Task 要求 property 失败打印 seed；无显式「禁止 testkit 导出 env helper」AC（靠 I-API/I-3.4 间接）。
- **建议补丁**：`T-CLK-015` AC 增加「失败输出 seed / 可重放」；I-API 或 I-3.4 加一条「无 env mutation helper」明示。

## 本轮结论：FAIL

## fail_count: 3

> 计主 FAIL：F6-1、F6-2、F6-3。F6-4/F6-5 记为次要建议，不计入 fail_count。  
> 实现 ABSENT（ManualClock V2 等未写）**不**计本轮 FAIL。
