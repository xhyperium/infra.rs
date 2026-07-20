# Round 01–05 Summary — 计划完备性（SPEC-TESTKIT-002）

| 字段 | 值 |
|------|-----|
| Spec | `SPEC-TESTKIT-002` · `.agent/SSOT/testkit/xhyper-testkit-complete-spec.md` |
| Plan | `PLAN-TESTKIT-002-v1-complete` |
| Scope | §0–§9 对照 plan 包映射完整性 |
| Mode | **只读 verifier** · **≠ 实现完成** · **SKIP ≠ PASS** |
| 日期 | 2026-07-14 |

---

## 1. 分轮结论

| Round | 主题 | 结论 | fail_count | Findings 文件 |
|-------|------|------|------------|---------------|
| 1 | §0–§3 定位 / 问题本质 / 裁定 / 组件 | **FAIL** | **5** | [`round-01-findings.md`](./round-01-findings.md) |
| 2 | §4–§6 目录 / 依赖 / crate 规则 | **FAIL** | **3** | [`round-02-findings.md`](./round-02-findings.md) |
| 3 | §7 ManualClock 逐 API | **FAIL** | **8** | [`round-03-findings.md`](./round-03-findings.md) |
| 4 | §8 宏退役合同 | **FAIL** | **4** | [`round-04-findings.md`](./round-04-findings.md) |
| 5 | §9 Contract Testkit | **FAIL** | **5** | [`round-05-findings.md`](./round-05-findings.md) |

```text
fail_rounds (R1–R5):  5 / 5
total fail_count:      25
PASS rounds:           0
```

**计划完备性 R1–R5：未通过（fail_rounds ≠ 0）。**  
不得将本摘要解读为实现 §24 闭合、Spec Approved 或 Campaign COMPLETE。

---

## 2. fail_count 汇总表（按可修复补丁单元）

| ID | Round | 规范锚点 | 缺失摘要 |
|----|-------|----------|----------|
| F1-1 | 1 | §0 架构图 | 正交 Test Support **图**交付物未入库 |
| F1-2 | 1 | §1 不稳定来源清单 | 13 bullet 未完整进 inventory（I-DET 子集不足） |
| F1-3 | 1 | §2.2 provider 硬编码全表 | stream/0/Pending/… 与隐藏 dep 迁移清单过粗 |
| F1-4 | 1 | §3.3 harness 职责 | OUT-OF-SCOPE 职责 bullet 未枚举 |
| F1-5 | 1 | §3.4 Fixture 所有权 | `test-support/fixtures/<schema>` + 两消费者门闩缺失 |
| F2-1 | 2 | §4.1 目录正例 | tests/*.rs 正例树未进 I-DIR |
| F2-2 | 2 | §4.1 新模块 RFC | 准入八问+RFC 无 Task/I-* |
| F2-3 | 2 | §4.2 CTC 目录 | suite_self_tests / compile_fail 未映射 |
| F3-1 | 3 | §7.3/7.4 | `non_exhaustive` 等枚举属性 |
| F3-2 | 3 | §7.7 | 禁带符号纳秒 fetch_add |
| F3-3 | 3 | §7.7 | 禁 release 模式回绕（合同句） |
| F3-4 | 3 | §7.8 | mono 失败不改状态 + 禁 signed delta |
| F3-5 | 3 | §7.9 | 禁 one-shot fault 队列 |
| F3-6 | 3 | §7.9 | scripted fault 两消费者准入 |
| F3-7 | 3 | §7.11 now | 锁失败 → Unavailable |
| F3-8 | 3 | §7.11 mono + 签名表 | poison 分项 + I-CLK-SIG 签名矩阵（本轮计为 trait 恢复不完整与签名 SSOT；详见 round-03，计数 8 含签名矩阵条） |
| F4-1 | 4 | §8.1 | external downstream=0 门槛 |
| F4-2 | 4 | §8.2 | 无替代宏 + 五条迁移矩阵 |
| F4-3 | 4 | §8.3 | 拆分时硬编码清除清单 |
| F4-4 | 4 | §8.4 | builder 命名规则 |
| F5-1 | 5 | §9.1 | 反原则（非 mock 默认值合同） |
| F5-2 | 5 | §9.2 | 禁 adapter crate 依赖专条 |
| F5-3 | 5 | §9.3 | Fake/Sandbox/Real bullet 矩阵 |
| F5-4 | 5 | §9.5 | 硬编码全表 + 最小 profile/禁 DSL |
| F5-5 | 5 | §9.6 | ContractFailure 字段 + 禁裸 unwrap |

> Round 3 在 findings 正文列为 8 条（含 §7 签名矩阵与 mono poison 分项等）；上表 F3-8 为压缩行，**以 round-03-findings.md 的 fail_count: 8 为准**。

---

## 3. 已覆盖良好的主线（不得解释为 PASS 轮次）

以下在 plan 包中**有**清晰 Wave/Task/DEF/I-* 映射（实现仍为 ABSENT）：

- T0 / test-support 身份与 layer 迁移（DEF-001，T-ARCH-*）
- ManualClock V2 主路径：Mutex 模型、checked wall/mono、fault、snapshot、无 Default/Clone（W1 / I-CLK 主干）
- 四类退役对象删除与 provider 迁出方向（W3/W4，I-DEL）
- contract-testkit 建包、按 trait 分 suite、reference/broken 负测、隐藏依赖禁止（W4）
- 依赖仅 kernel、无 feature、dev-dep only、生产图隔离门禁方向（§5，W6）
- crate `forbid`/`deny`（§6，T-CLK-019）

---

## 4. 高优先级补丁建议（关闭 R1–R5 前）

1. **扩展 `spec-inventory.md`**  
   - `I-DIR-CORE` / `I-DIR-CTC` 正例树  
   - `I-CLK-SIG` 签名表 + fault scope（禁 one-shot）+ trait 分步语义  
   - `I-CTC-HARDCODE-*` / `I-CTC-FAKE|SANDBOX|REAL-*` / `I-CTC-9` 字段  
   - `I-FIXTURE-*`、`I-1-IMPLICIT`、harness OUT-OF-SCOPE  
2. **收紧 `tasks.md` AC**  
   - 禁止只写「见 §xx」；绑定上列 I-*  
   - T-CLK-003/004/007/008/009/011；T-DEL-001/002；T-CTC-002/003/015  
3. **`residual-open.md`**  
   - harness DEFER 附职责列表 + accepted 条件  
   - external downstream 扫描 N/A 裁定  
4. **重跑 R1–R5** 直至 `fail_rounds=0`，再继续 R6–R10（§10–§25）。  
5. **禁止**：用本 summary 勾选 T-V10-PLAN 为 PASS；实现 ABSENT 不变。

---

## 5. 与计划包状态一致性

| 项 | 状态 |
|----|------|
| Campaign | PLANNING（plan.md） |
| 实现 | ABSENT / NOT STARTED |
| Spec Status | Proposed（未 Approved） |
| T-V10-PLAN | 本轮 R1–R5 **FAIL** → 整体计划 10x 尚未 fail_rounds=0 |
| §24 | 未闭合 |
| SKIP 当 PASS | **未发生**；DEFER 项保持 DEFER |

---

## 6. 计数块（机器可读）

```text
round_01_fail_count=5
round_02_fail_count=3
round_03_fail_count=8
round_04_fail_count=4
round_05_fail_count=5
total_fail_count=25
fail_rounds=5
pass_rounds=0
verdict=FAIL
implementation_claimed=false
```
