> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 6–10 Summary — SPEC-TESTKIT-002 计划完备性

| 字段 | 值 |
|------|-----|
| Spec | `SPEC-TESTKIT-002`（Proposed） |
| Plan | `PLAN-TESTKIT-002-v1-complete` v1.0.0 |
| Verifier | 只读；实现 ABSENT 不计 FAIL |
| 日期 | 2026-07-14 |
| Findings | `round-06-findings.md` … `round-10-findings.md` |

---

## 1. 分轮结论

| Round | 主题 | 结论 | fail_count | Findings |
|------:|------|------|------------|----------|
| 6 | §10–§13 术语 / 确定性 / API / 测试合同 | **FAIL** | **3** | [round-06-findings.md](./round-06-findings.md) |
| 7 | §14–§16 图隔离 / Archgate / CI | **FAIL** | **3** | [round-07-findings.md](./round-07-findings.md) |
| 8 | §17–§20 文档 / 版本 / 迁移 / PR | **FAIL** | **2** | [round-08-findings.md](./round-08-findings.md) |
| 9 | §21–§23 Evidence / 1·7·30 / 指标 | **FAIL** | **2** | [round-09-findings.md](./round-09-findings.md) |
| 10 | §24–§25 + DEF + consumers 实测 | **FAIL** | **3** | [round-10-findings.md](./round-10-findings.md) |

```text
fail_rounds (R6–R10) = 5
total_fail_items     = 13
plan_10x (R6–R10)    = NOT PASS
```

> R1–R5 不在本批范围；全量计划 10x 须合并 R1–R10 后写 `testkit-plan-10x-verdict.md`。

---

## 2. 主 FAIL 一览（须补丁后重跑）

| ID | Round | 摘要 | 建议落点 |
|----|------:|------|----------|
| **F6-1** | 6 | §13.2 Clock suite 三分仅有 I-TEST-CLK-CONTRACT，**无 Task** | 新增 T-CLK-021 或扩展 T-CLK-014 |
| **F6-2** | 6 | §13.6 mutation **8 条禁存活**未枚举；T-GATE-009 只看 ≥90% | I-TEST-MUT-1…8 + AC 绑定 |
| **F6-3** | 6 | §10 Mock* **逐项审计表**未冻结 | I-TERM-AUDIT + T-GATE-011 AC |
| **F7-1** | 7 | §16.2 contract-testkit **CI 三条**无接线 Task（T-GATE-007 仅 §16.1） | T-GATE-007 扩展 / T-GATE-013 |
| **F7-2** | 7 | §16.4 Nightly **五项**无 Task/映射 | I-CI-NIGHTLY-1…5 + T-GATE-014 |
| **F7-3** | 7 | §14.4 宏 **cfg(test)/fixture/无 production symbols** 未进 T-CTC-010 | 扩展 T-CTC-010 或 T-CTC-018 |
| **F8-1** | 8 | §19.6 终态 path **`spec/spec.md`** 被 T-ARCH-007 放宽为「或 complete-spec」 | 强制终态 path + T-DOC-004 |
| **F8-2** | 8 | §18.2 删除须 **Approved RFC** 未任务化/等价声明 | A11 或 T-DOC-RFC-DEL |
| **F9-1** | 9 | plan §7 Evidence **漏 `contract-negative-tests.log`**（inventory 有） | 对齐 §21 15 文件 |
| **F9-2** | 9 | §22 **1/7/30 天**仅 gap N/A，无 Wave 映射 | I-SCHED + gap 改 PARTIAL |
| **F10-1** | 10 | T-GATE-008 **无 branch≥90%** AC（§24.3 勾） | AC = line∧branch |
| **F10-2** | 10 | T-24-003 依赖 **漏 T-CLK-014…017** | 扩大依赖 DAG |
| **F10-3** | 10 | T-24-006 依赖 **仅 T-EVID-001**，漏 ADR/snapshot/archgate/… | 扩大依赖或 I-DONE 证据链 |

次要（不计入上表 count，修复时建议顺手）：F6-4/5、F7-4、F8-3/4、F9-3/4。

---

## 3. 本批 PASS 亮点（勿回退）

| 区域 | 评价 |
|------|------|
| §12 API 预算 | I-API + T-DEL-005 + T-GATE-004 完整 |
| §14.1 GRAPH-001…005 | ID + T-GATE-001/002 + 输出列 |
| §15 Archgate 11 规则 | plan §6.2 + I-AG-* 全表 |
| §16.1 Core CI 命令 | plan §6.3 与规范对齐 |
| §17–§18 主体 | README/AGENTS/CHANGELOG/publish/incubating Task 齐 |
| §19 Phase0–6 ↔ W0–W6 | 映射清晰 |
| §20 PR-1…6 | 与规范切分一致 |
| §23 14 指标 | I-METRICS 全表正确 |
| §24 I-DONE 结构 | 40 勾语义覆盖 |
| DEF-001…010 | gap ≡ residual ≡ todo |
| Consumers | binance/okx provider；其余外部 0；dev-dep only —— **rg 实测一致** |
| §25 / Forbidden | 终态与十条禁令交叉一致；Campaign 诚实 |

---

## 4. Consumers 实测快照（Round 10）

| 符号/依赖 | 计划 | 实测 |
|-----------|------|------|
| provider 宏 | binance, okx | ✓ |
| xlib_test!/mock!/FixtureBuilder 外部 | 0 | ✓ |
| ManualClock 外部 | 0 | ✓ |
| testkit dev-dep | binance, okx | ✓ |
| testkit normal-dep | 0 | ✓ |

---

## 5. 建议修复顺序（计划包最小 diff）

```text
1. inventory 补丁（阻塞勾选精度）
   - I-TEST-MUT-1…8
   - I-TEST-CLK-CONTRACT → 绑定 Task
   - I-TERM-AUDIT（Mock* 表）
   - I-CI-CTC / I-CI-NIGHTLY 展开
   - I-SCHED-1D/7D/30D
   - I-EVID 15 行文件名表

2. tasks.md 补丁
   - T-CLK-021（§13.2 三分）
   - T-GATE-008 branch
   - T-GATE-009 绑禁存活
   - T-GATE-007/013/014（§16.2/16.4）
   - T-CTC-010/018（§14.4）
   - T-ARCH-007 终态 path；T-DOC-004
   - T-24-003 / T-24-006 依赖 DAG

3. plan.md §7 Evidence 与 §22 映射附录

4. residual / todo 同步新 Task ID 与 §22 状态

5. 重跑 R6–R10 → 目标 fail_rounds=0 → 再写全量 10x verdict
```

---

## 6. 明确不在本批评判内

- ManualClock / contract-testkit / archgate **实现是否存在**（ABSENT = 预期）
- Spec Status → Approved（人审）
- 实现 10x（T-V10-001…010）
- R1–R5 结论

---

## 7. 一句话裁决

**R6–R10 计划包尚未达到 fail_rounds=0**：核心骨架（Wave/PR/DEF/指标/GRAPH/Archgate ID/§24 结构/消费者扫描）扎实，但 **§13.2 三分、mutation 禁存活枚举、§16.2/16.4 CI、§19.6 终态 path、§22 时间表、Evidence 主文漏项、§24 闭合 Task 依赖与 branch AC** 共 13 项计划遗漏须先补齐，再宣称计划 10x PASS。
