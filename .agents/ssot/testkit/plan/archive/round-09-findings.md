# Round 9 — §21–§23 Evidence 目录清单 / 1·7·30 天 / 指标全表

| 字段 | 值 |
|------|-----|
| Scope | 计划完备性（非实现验收） |
| Spec | `SPEC-TESTKIT-002` §21–§23 |
| 日期 | 2026-07-14 |

## 检查项

| # | 规范点 | 计划应覆盖内容 |
|---|--------|----------------|
| 9.1 | §21 目录树 | `evidence/testkit/<date>-<change-id>/` 下 15 个文件名 |
| 9.2 | §21 证明义务 | commit / 非 production graph / broken kill / 无真实时间 / 无宏回流 / SKIP≠PASS |
| 9.3 | §22 1 天 | 6 条里程碑映射 Wave/Task 或显式调度 |
| 9.4 | §22 7 天 | 6 条里程碑映射 |
| 9.5 | §22 30 天 | 7 条里程碑映射 |
| 9.6 | §23 指标 | 14 项名称+目标值全表 → I-METRICS + 验收 Task |

## PASS

1. **§21 路径约定**：plan §7 + `T-EVID-001` `evidence/testkit/<date>-…` 与规范一致。
2. **§21 文件清单（inventory）**：**I-EVID** 列出  
   manifest / commit / cargo-metadata / consumers / production-graph / public-api.diff / fmt / clippy / tests / coverage / mutants / miri / **contract-negative** / archgate / verdict —— **与 §21 15 项语义对齐**。
3. **§21 证明义务**：plan §7 证明列表含 commit、未进 production graph、broken fake 可杀、无真实时间、无宏回流、SKIP≠PASS —— 对齐规范。
4. **§21 有验收 Task**：`T-EVID-001` AC「§21 文件清单」；`T-V10-010` 交叉 Evidence；W7 绑定。
5. **§23 指标全表完整**（I-METRICS 与规范逐字对齐）：

   | 指标 | 目标 | inventory |
   |------|------|-----------|
   | testkit_production_dependents | 0 | ✓ |
   | contract_testkit_production_dependents | 0 | ✓ |
   | testkit_public_macro_count | 0 | ✓ |
   | testkit_placeholder_public_type_count | 0 | ✓ |
   | real_time_calls_in_testkit | 0 | ✓ |
   | sleep_calls_in_unit_contract_tests | 0 | ✓ |
   | manual_clock_unchecked_arithmetic_count | 0 | ✓ |
   | manual_clock_line_coverage | ≥95% | ✓ |
   | manual_clock_branch_coverage | ≥90% | ✓ |
   | manual_clock_mutation_score | ≥90% | ✓ |
   | contract_suite_broken_impl_kill_rate | 100% | ✓ |
   | hidden_macro_dependency_count | 0 | ✓ |
   | active_testkit_spec_count | 1 | ✓ |
   | flaky_retry_usage | 0 | ✓ |

6. **§23 验收 Task**：`T-V10-009`「Round9 指标 §23 全量 / I-METRICS」。
7. **Evidence 与 Campaign 边界诚实**：todo/plan 声明 plan 包 ≠ §24 Evidence；实现 ABSENT —— 正确。

## FAIL

### F9-1 — plan.md §7 Evidence 文件列表缺 `contract-negative-tests.log`

- **规范引用**：§21 目录必须含 `contract-negative-tests.log`。
- **缺失**：
  - `plan.md` §7 写：
    ```text
    manifest.json, commit.txt, cargo-metadata.json, consumers.json,
    production-graph.json, public-api.diff, fmt/clippy/tests logs,
    coverage.json, mutants.json, miri.log, archgate.json, verdict.md
    ```
    **未出现 contract-negative-tests.log**（亦未写 contract-negative）。
  - inventory **有** contract-negative；T-EVID-001 引 §21 可间接覆盖——但 **计划主文与 inventory 不一致**，十轮/执行易漏拷该文件。
- **建议补丁**：
  1. 修正 plan §7 与规范 §21 目录树逐文件一致（含扩展名 `.log`）。
  2. I-EVID 改为表格 15 行文件名，与 T-EVID-001 AC 绑定「15/15」。

### F9-2 — §22 1/7/30 天计划未映射（仅 gap N/A）

- **规范引用**：§22 完整时间表（1 天 6 项 · 7 天 6 项 · 30 天 7 项），含「重新评级 incubating / 2.5 of 5」「stable 验收」等。
- **缺失**：
  - `gap-matrix.md` §22 → **N/A**，无关闭 Wave、无 DEF；
  - inventory **无 I-SCHED / I-22**；
  - tasks/todo/plan **无 1 天/7 天/30 天里程碑表**；
  - Wave 退出条件未标注「对应 §22 哪一地平线」；
  - 「重新评级为 incubating / 2.5 of 5」**无 Task**（与当前 incubating 声明的关系未写）。
- **为何算计划遗漏（非实现 ABSENT）**：§22 是规范内时间盒验收面；标 N/A 而未说明「由 PR 栈/Wave 替代且映射如下」→ 完备性缺口。实现未开工不构成此项 FAIL 的豁免。
- **建议补丁**：
  1. 新增 plan 附录或 inventory **I-SCHED-1D / 7D / 30D** 表，每行映射现有 Wave/Task/PR。
  2. 修正 gap-matrix §22：N/A → PARTIAL/映射表 + 关闭条件。
  3. 增加 `T-RATE-001`（可选）：质量分 2.5/5 incubating 声明写入 README/registry 注释（若组织使用该评分）。

### F9-3（次要）— §22 与 W7–W9 时间盒未设日历

- **说明**：规范给相对日，计划给 Wave 无日期；可接受若 I-SCHED 用「依赖完成」代替日历。记观察，不单列主 FAIL（并入 F9-2 修复即可）。

### F9-4（次要）— 指标采集「如何量」未写命令

- **说明**：I-METRICS 有目标值，多数可靠 T-GATE/rg/llvm-cov 推断；`flaky_retry_usage` / `active_testkit_spec_count` 缺采样命令。建议在 I-METRICS 加「度量命令」列。不计入主 fail_count。

## 本轮结论：FAIL

## fail_count: 2

> 主 FAIL：F9-1、F9-2。§23 指标表本身 **PASS**。
