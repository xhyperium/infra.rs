# Round 07b Findings — Migration Skeptic（v1.1 后）

| 字段 | 值 |
|------|-----|
| round | `7b` |
| role | Verifier / Migration Skeptic · post-v1.1 |
| scope | §31 P0–P6 ↔ Waves；双包；cutover 序；ADR-010/012；全 40 R-\* |
| prior | `round-07-findings.md`（v1.0 **FAIL**） |
| date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — v1.1 关闭了 R7 的 **主阻断**（P0–P6 表、bootstrap 强制、双包 rename、ADR-012 对账），迁移路径在计划层 **基本可执行**。整轮仍 FAIL：§4 清单 **R-SPEC-003** 未绿（§33 映射 hygiene，跨轮残留）。迁移专项自身可记 **PASS\***。

---

## failed_checks

### 迁移专项（相对 v1.0 R7）

| 检查 | v1.0 | v1.1 | 裁定 |
|------|------|------|------|
| R-MIG-001 P0–P6↔Wave 显式表 | FAIL（隐式） | **I-14** + gap 备注 | **PASS** |
| P6 bootstrap 强制 production adapter | FAIL（缺 Task） | **T-BOOT-001** / I-19 | **PASS** |
| 双包同名 `evidence` | FAIL（无 Task） | **T-LEG-003** + I-18 + A12 + DEF-020 | **PASS** |
| 旧链非静默 rehash | PASS 文案 | 仍 PASS；T-LEG-001/002 | **PASS**（实现 ABSENT） |
| ADR-010 人审 | 有 T-DOC-004 | 保留 | **PASS** |
| ADR-012 auditx 冲突 | FAIL（全缺） | **A11** + **T-DOC-005** + plan §1.3 + DEF-019 | **PASS** |
| cutover 序：调用方迁离 → 删包 | 弱 | T-CUT-001 → T-CUT-002 → PATH-001 | **PASS** |
| callers 迁移矩阵 | 弱 | gap §3 domain_macro/gate | **PASS\*** |

### 全 40 R-\*（与 R6b 一致终裁）

```text
FAIL:     R-SPEC-003
PASS/*:   其余 39
```

R-SPEC-003 细节：

1. `tasks.md` §33.4：`full replacement 检测 | T-CP-005 + verify` — **`verify` 不是 Task ID**（`T-CP-007` 已存在却未入表）。  
2. `startup verify | T-FILE-005 T-CP-004` — 真实合同任务是 **`T-CP-008`**，映射未用。  
3. §33.1「ADR 冲突修订」仅 `T-DOC-004`（ADR-010），**未列 `T-DOC-005`/ADR-012**，与 R7 已补的对账 Task 脱节。

→ 迁移本体已修，**完成定义映射仍不干净** → 轮次 FAIL。

---

## omissions

1. §33.1 未把 ADR-012 对账 Task 写入勾选映射（对账任务存在、映射漏）。  
2. I-14 将 P2 bridge 折叠进 W3——可接受，但执行时易被 DOM 波次挤掉；无「P2 exit criteria」独立勾。  
3. 双包窗口 **最长 Wave/PR 边界** 仍仅风险表一笔，无 hard deadline Task。  
4. cutover 后 `lint-deps` / architecture registry 与 A11 路径选择的 **条件依赖**（若人审选 auditx）未写成分支 Task。  
5. systemd 部署清单（§19.3）仍无 Task（R-MEM 残留）。

---

## false_pass_risks

| 风险 | 机制 |
|------|------|
| **「I-14 有表」⇒ 迁移完成** | 表是计划层；W0–W6 任务几乎全 TODO |
| **T-LEG-003 DONE 前建 crates/infra/evidence** | 未 rename 旧包 → cargo 双 `name=evidence` 构建炸；须先/同 PR 隔离 |
| **A11 未签仍按 002 路径改 architecture** | plan 默认 002，但 ADR-012 仍 OPEN GOVERNANCE |
| **T-BOOT-001 与 T-ARCH-011 循环依赖表述** | tasks 互指；实现时需明确接线序 |

---

## notes

### P0–P6 ↔ Wave（I-14 核验）

| Spec | Wave | 主前缀 | v1.1 |
|------|------|--------|------|
| P0 | W0 | T-FREEZE / T-DOC | OK |
| P1 | W1 | T-CORE | OK |
| P2 | W3 | T-LEG | OK（折叠标注） |
| P3 | W3 | T-DOM / T-GATE | OK |
| P4 | W4（+W2 mem） | T-FILE / T-PG | OK |
| P5 | W5 | T-CP / T-CLI | OK |
| P6 | W6 | T-CUT / T-ARCH / **T-BOOT** | OK |
| plan-only | W7–W9 | T-V10 / T-HUM / T-33 | 可接受 |

### 双包策略（I-18）

```text
旧: evidence_legacy @ tools/evidence（rename）
新: evidence @ crates/infra/evidence
切后: 仅 crates/infra/evidence
Task: T-LEG-003；Forbidden I-26 #10；A12 Approve 策略
```

### 诚实声明

- Spec **Proposed** · DEF-019/020 **OPEN** · 无 stable/§33 宣称。  
- 迁移专项：**计划可执行**；**不得**因 R7 主题转绿而写 fail_rounds=0。

```yaml
round: 7b
result: FAIL
failed_checks: [R-SPEC-003]
omissions:
  - §33.1 ADR map missing T-DOC-005
  - dual-package window hard deadline absent
  - P2 exit criteria not separate checkbox
  - systemd deploy list still untasked
false_pass_risks:
  - build crates/infra/evidence before T-LEG-003 rename
  - A11 unsigned path assumption
notes: |
  R-MIG-001 PASS. Dual-package T-LEG-003 + I-18 PASS.
  ADR-012 A11 + T-DOC-005 PASS. Bootstrap T-BOOT-001 PASS.
  Round FAIL solely from R-SPEC-003 mapping hygiene.
```
