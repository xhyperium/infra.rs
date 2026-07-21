> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Pass3 Closure Matrix — SPEC-TESTKIT-002

| 字段 | 值 |
|------|-----|
| Spec | `SPEC-TESTKIT-002` · `xhyper-testkit-complete-spec.md` |
| Plan | `PLAN-TESTKIT-002-v1-complete` **v1.2.0** |
| Mode | 只读 verifier · 部分覆盖 = OPEN · 实现 ABSENT 不计 FAIL |
| 日期 | 2026-07-14 |
| 对照 | pass2 15 OPEN · `spec-inventory.md` **I-PATCH-v1.2** · `tasks.md` · `plan.md` · `gap-matrix.md` |

---

## 1. 十五行闭合矩阵

| ID | Round | 状态 | 证据（全文展开标题 / 表 / 命令 / 树） |
|----|------:|------|--------------------------------------|
| **F1-3** | 1 | **CLOSED** | `spec-inventory.md` **I-CTC-HC-TABLE**：`I-CTC-HC-1…6`（stream 空 / `server_time==0` / position 空 / balance 空 / `query_order==Pending` / invalid cancel 失败）+ **I-DEL-HC** 绑 `T-DEL-009` / `T-CTC-005…009` |
| **F2-1** | 2 | **CLOSED** | **I-DIR-CORE 正例树原文**：`crates/testkit/` 全文树（Cargo/README/AGENTS/CHANGELOG · `src/{lib,clock}.rs` · 四 tests 文件）+ 禁新增模块清单；`tasks.md` **T-CLK-001** AC=`**I-DIR-CORE 正例树** 完全一致` |
| **F2-3** | 2 | **CLOSED** | **I-DIR-CTC 正例树原文**：`crates/test-support/contracts/` 全文树（六 suite 源文件 + `tests/suite_self_tests.rs` / `compile_fail.rs`）；**T-CTC-001** / **T-CTC-023** AC 绑 `I-DIR-CTC` |
| **F3-1** | 3 | **CLOSED** | **I-CLK-DERIVE**：Fault=`non_exhaustive`+`Debug,Clone,Copy,PartialEq,Eq`；Error=`non_exhaustive`+`Debug`+手写 `Display+Error`；Snapshot=`Debug,Clone,Copy,PartialEq,Eq`；**T-CLK-003/004** AC 含 `I-CLK-DERIVE` |
| **F3-5** | 3 | **CLOSED** | **I-CLK-SCRIPTED Task 绑定**：首稳定版禁 scripted/one-shot；≥2 消费者经八问+RFC；Task **`T-CLK-023`**（负向无 one-shot API）· **`T-GATE-015`**（未来准入） |
| **F3-7** | 3 | **CLOSED** | **I-CLK-POISON 五条**原文：①不持锁执行调用方 ②poison 恢复 inner ③不伪造零 ④不 panic ⑤文档恢复语义；**T-CLK-024** AC 必须逐条 |
| **F3-8** | 3 | **CLOSED** | **I-CLK-SIG 完整签名矩阵**：16 行方法签名（含 `advance_wall → Result<Timestamp, ManualClockError>`、fault 三 API 均 `Result`、`Clock::now/monotonic`、Snapshot getters）；**T-CLK-025** |
| **F4-2** | 4 | **CLOSED** | **I-DEL-MOCK-PATHS 五路径展开** #1–5（手写 fake / trait impl / contract-testkit / `Arc<Mutex<Vec<Call>>>` / 复杂 expectation 先证明）+ 禁空壳替代宏；**T-DEL-008** |
| **F4-3** | 4 | **CLOSED** | 同 F1-3 SSOT：**I-CTC-HC-1…6** + **I-DEL-HC**（拆分时必须删除 core 宏字面断言）；**T-DEL-009** AC=`I-DEL-HC` |
| **F5-3** | 5 | **CLOSED** | **I-CTC-LAYER-MATRIX**：Fake 6 项（trait object/配置 I/O/错误注入/生命周期/调用记录/无外部 IO）；Sandbox 5 项；Real 6 项；三类禁混用默认断言；**T-CTC-020** |
| **F5-4** | 5 | **CLOSED** | 硬编码半支 = **I-CTC-HC-1…6**；最小 profile 半支 = **I-CTC-MIN-PROFILE** + **T-CTC-022**（v1.1 已半闭，v1.2 补全表） |
| **F6-3** | 6 | **CLOSED** | **I-TERM-AUDIT 冻结表**（2026-07-14）：`MockBinance*` / `MockKv*`·`Mock*Store` / `testkit::mock!` 三行 + 基线 `rg` 命令 + NAMING-001；产出 Task **T-GATE-018** |
| **F7-1** | 7 | **CLOSED** | **I-CI-CTC 三条命令字面**（clippy / test / `--test negative_implementations`）；`plan.md` **§6.3b** 同字面；**T-GATE-013** AC=`I-CI-CTC` |
| **F7-2** | 7 | **CLOSED** | **I-CI-NIGHTLY 五项字面**：①full mutation ②Miri ③property extended ④broken-impl matrix ⑤workspace production graph audit；`plan.md` **§6.3c**；**T-GATE-014** |
| **F9-2** | 9 | **CLOSED** | **I-SCHED 里程碑展开**：1 天 / 7 天 / 30 天规范交付 bullet → W0 / W1–W4 / W5–W9；`gap-matrix.md` **§22 = PARTIAL**（非 N/A）；`plan.md` 附录 §22↔Wave |

```text
pass3_open_count   = 0
pass3_closed_count = 15
```

---

## 2. R1–R10 快速回归（新 FAIL 扫描）

| Round | 主题 | 结论 | 说明 |
|------:|------|------|------|
| 1 | §0–§3 | **PASS** | F1-1/2/4/5 仍 CLOSED；F1-3 本 pass 关闭 |
| 2 | §4–§6 | **PASS** | F2-2 仍 CLOSED；F2-1/3 本 pass 关闭；deps/crate 映射仍在 I-2/I-3 |
| 3 | §7 ManualClock | **PASS** | F3-2/3/4/6 仍 CLOSED；F3-1/5/7/8 本 pass 关闭 |
| 4 | §8 宏退役 | **PASS** | F4-1/4 仍 CLOSED；F4-2/3 本 pass 关闭 |
| 5 | §9 Contract | **PASS** | F5-1/2/5 仍 CLOSED；F5-3/4 本 pass 关闭 |
| 6 | §10–§13 | **PASS** | F6-1/2 仍 CLOSED；F6-3 本 pass 关闭 |
| 7 | §14–§16 | **PASS** | F7-3 仍 CLOSED；F7-1/2 本 pass 关闭 |
| 8 | §17–§20 | **PASS** | pass2 已 0 FAIL；无回归 |
| 9 | §21–§23 | **PASS** | F9-1 仍 CLOSED；F9-2 本 pass 关闭；I-METRICS 仍在 |
| 10 | §24–§25 + DEF | **PASS** | pass2 已 0 FAIL；实现 ABSENT 不计；DEF 表仍可追踪 |

**新 FAIL：无**（未发现规范 bullet 完全无映射；不新增吹毛求疵项）。

---

## 3. 机器可读

```text
total_fail_count = 0
fail_rounds      = 0
pass_rounds      = 10
plan_version     = v1.2.0
pass             = pass3
verdict          = PASS
implementation_claimed = false
```
