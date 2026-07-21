# Round 10b Findings — Full Heading Re-enumeration（v1.1 后）

| 字段 | 值 |
|------|-----|
| round | `10b` |
| role | Final Verifier · Independent heading coverage · post-v1.1 |
| rule | 每个 `##` / `###`：**M** 明确 · **W** 弱代理 · **Z** 零提及 |
| prior | `round-10-findings.md`（v1.0 **FAIL**；Z1–Z7 等） |
| date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — v1.0 的 **严格 Z 子节**（§15.3/15.4、§22.2–22.4、§23.3、§29.3）在 v1.1 均已 **升为 M**（有 Task 与/或 I-\*）。章级/子节覆盖显著改善。整轮仍 FAIL：全 40 R-\* 中 **R-SPEC-003** 未绿；另有少量 **W** 残留（Subject 策略、systemd、branch 覆盖、合桶门禁）。

**原 Z 集合清零 ≠ fail_rounds=0。**

---

## 专项：原 Z 子节覆盖（强制）

| 标题 | v1.0 | v1.1 出处 | 标记 |
|------|------|-----------|------|
| ## 15.3 外部不可逆副作用 | **Z** | I-15；T-ATOM-004；§33.6；residual DEFER-ATOM-004 | **M** |
| ## 15.4 纯内存状态 | **Z** | I-15；T-ATOM-005 | **M** |
| ## 22.2 原始 artifact | **Z** | I-16；T-PRIV-001 | **M** |
| ## 22.3 Retention | **Z** | I-16 六类；T-PRIV-002 | **M** |
| ## 22.4 删除 | **Z** | I-16；T-PRIV-003 | **M** |
| ## 23.3 算法迁移 | **Z** | I-21；T-SCH-003 | **M** |
| ## 29.3 背压 | **Z** | I-22；T-BP-001 | **M** |

→ **原 Z1–Z7 全部消除。** 另：`### 内容不同`（13.5）→ T-CORE-037 **M**。

---

## failed_checks

### R-SPEC-002 子节加强

章级 gap §0–§34：**PASS**。  
子节：原 Z 已 M；残留 **W** 不升格为 R-SPEC-002 FAIL（字面为章级齐全）。

### R-SPEC-003

**FAIL** — 见 R8b：`T-CP-005 + verify`；startup 未用 T-CP-008；ADR 映射漏 T-DOC-005。

### 全 40 R-\* 终裁（与 6b–9b 一致）

```text
FAIL:     R-SPEC-003
PASS/*:   39
result: FAIL
```

---

## 完整标题枚举（v1.1 差分重点）

> 仅列相对 R10 v1.0 **标记变化** 或 **仍 W** 的关键项。未列标题默认保持 M/W 且无新 Z。

### §0–§12

| 标题 | 标记 | 备注 |
|------|------|------|
| ## 1.3 信任边界 | **M**（原 W） | I-23 组件列表 |
| ## 2.4 非职责 | **M**（原 W） | I-24 |
| ## 4.3 Dev dependencies | **M**（原 W） | I-12 Dev 允许列表 |
| ## 7.2 Subject | **W** | 字段 T-CORE-010；**规范化策略仍无 Task** |
| ### 内容不同 (13.5) | **M**（原 Z） | T-CORE-037 |

### §13–§18

| 标题 | 标记 | 备注 |
|------|------|------|
| ## 15.1 持久化业务状态 | **M** | T-ATOM-001 + T-PG-003；含模式 C T-ATOM-003 |
| ## 15.3 | **M** | T-ATOM-004 |
| ## 15.4 | **M** | T-ATOM-005 |
| ## 15.5 Rejected | **M** | T-ATOM-006 |
| ## 16.5 频率 | **M** | T-CP-003 + T-BOOT-002 hard deadline |

### §19–§23

| 标题 | 标记 | 备注 |
|------|------|------|
| ## 19.3 生产阻断 | **W→M\*** | T-BOOT-001 / T-ARCH-005；\***systemd 清单仍 W/近 Z** |
| ## 22.1–22.4 | **M** | T-PRIV-\* / I-16 |
| ## 23.1–23.3 | **M** | T-SCH-001…003 |

### §24–§30

| 标题 | 标记 | 备注 |
|------|------|------|
| ## 24.8 Fuzz | **M** | T-FUZZ-001；I-8 |
| ## 24.9 Coverage | **M\*** | line 有；branch 弱 |
| ## 24.10 Mutation | **M** | T-MUT-001；I-7 |
| ## 24.11 Miri | **M** | T-MIRI-001 |
| ## 25.2 默认行为 | **M** | I-10 |
| ## 25.4 repair-tail | **M** | I-10 |
| ## 27.1–27.3 | **M** | I-9 23 IDs |
| # 28 CI | **M** | I-20 |
| ## 29.3 背压 | **M** | T-BP-001 |
| # 30 Observability | **M** | I-11 11 metrics；T-OBS-001 |

### §31–§34

| 标题 | 标记 | 备注 |
|------|------|------|
| ## 31.2 / P0–P6 | **M** | I-14 |
| ### P6 Cutover | **M** | T-BOOT + T-CUT |
| ## 33.4 | **W** | 映射表 hygiene（任务有） |
| ## 33.5 | **M** | 非草案桶 |
| ## 33.6 | **M** | 真实 T-ATOM-\*；无幽灵 |
| # 34 | **M** | 路径任务齐；实现 ABSENT |

### 统计（约）

```text
原严格 Z (7+):     0 remaining
新/残留 W 高危:    §7.2 Subject 策略 · systemd 清单 · branch≥90% · §33.4 映射单元格
H1/## 章级:        无新 Z
```

---

## omissions

1. **§7.2 Subject 规范化/版本策略** — 仍 W（领域职责未任务化）。  
2. **§19.3 systemd 部署清单** — 近 Z。  
3. **§33.4 映射单元格** — 非 ID `verify`；未挂 T-CP-007/008。  
4. **branch coverage ≥90%** — 无硬 AC。  
5. **T-ARCH-004/006 合桶** — 多 ID 单任务。  
6. **approval 签字表 A1–A10 表头** vs A13。  
7. plan 标题仍写「v1」、changelog 为 v1.1（版本标签漂移，非规范 Z）。

---

## false_pass_risks

| 风险 | 机制 |
|------|------|
| **「Z 清零」⇒ 十轮通过** | 忽略 R-SPEC-003 与实现 OPEN |
| **R-SPEC-002 章级绿** | 掩盖 §7.2 / systemd W |
| **合桶 T-ARCH DONE** | 5 门禁只做 1 |
| **heading M = 可勾 §33** | M 只证明「被提及」，不等于证据闭合 |

---

## notes

### 与用户强制项对照

| 强制项 | 本轮结论 |
|--------|----------|
| 23 EVIDENCE-\* 均有 Task | **YES**（I-9） |
| 无 T-ATOM via design | **YES** |
| ADR-012 A11 | **YES** |
| §33 仅真实 Task ID | **NO**（一处 `+ verify`） |
| I-26 Forbidden 统一 | **YES** |
| 原 Z：15.3/15.4/22.2–22.4/23.3/29.3 | **全部 M** |

### 第二轮计划 10x 状态（6b–10b）

| Round | result | 主失败 |
|-------|--------|--------|
| 6b | FAIL | R-SPEC-003 |
| 7b | FAIL | R-SPEC-003（迁移主题已 PASS） |
| 8b | FAIL | R-SPEC-003（幽灵已灭） |
| 9b | FAIL | R-SPEC-003（§34 路径任务齐） |
| 10b | FAIL | R-SPEC-003（原 Z 清零） |

```text
pass2 fail_rounds ≥ 5  （本 Verifier 负责 6b–10b 全 FAIL）
禁止 evidence-plan-10x-verdict.md 写 fail_rounds=0
```

### 最小闭合 R-SPEC-003 补丁（建议 Planner）

1. `tasks.md` §33.4：`full replacement` → `T-CP-007`；`startup verify` → `T-CP-008`。  
2. §33.1 ADR → `T-DOC-004 T-DOC-005`。  
3. 可选：§33.5 branch → 独立 AC 或 DEFER(accepted)。  
4. 可选：§7.2 / systemd 各加一行 Task 或正式 residual DEFER。  
5. 重跑 R1b–R10b；目标 fail_rounds=0。

### 诚实声明

- Spec **Proposed** · Campaign **PLANNING** · **≠ Stable** · **≠ §33 闭合**。  
- v1.1 修补 **有效**；第二轮仍因 **映射 hygiene** 不能宣称十轮通过。

```yaml
round: 10b
result: FAIL
failed_checks: [R-SPEC-003]
omissions:
  - §7.2 Subject strategy still W
  - systemd deploy list near-Z
  - §33.4 mapping cell non-Task-ID
  - branch≥90% soft
  - plan title v1 vs changelog v1.1
false_pass_risks:
  - Z-clearance mistaken for fail_rounds=0
  - chapter-level R-SPEC-002 masking W cells
notes: |
  Previously Z sections §15.3, §15.4, §22.2-22.4, §23.3, §29.3 all M after v1.1.
  23/23 EVIDENCE-* mapped. No live T-ATOM via design. A11 present. I-26 unified.
  §33 real-Task-IDs-only still FAIL on "T-CP-005 + verify".
```
