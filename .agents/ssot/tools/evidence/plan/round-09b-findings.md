# Round 09b Findings — §34 Production Audit Path E2E（v1.1 后）

| 字段 | 值 |
|------|-----|
| round | `9b` |
| role | Final Verifier · §34 E2E path · post-v1.1 |
| scope | §34 生产终态箭头 + 七条件 C1–C7 + 全 40 R-\* |
| prior | `round-09-findings.md`（v1.0 **FAIL**） |
| date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — v1.1 **基本闭合** 了 R9 的路径断点（幽灵原子性、§15.3/15.4、retention/privacy、整链替换 Task、bootstrap Durable、背压）。§34 **计划层路径** 现可逐步指到真实 Task。整轮仍 FAIL：**R-SPEC-003**（§33.4 映射 hygiene）未绿；另 C2 Subject 策略与 systemd 清单为 residual WEAK，不单独否决路径存在性。

**不得**宣称「§34 生产审计路径已闭合」或 fail_rounds=0——实现 ABSENT + 映射表仍有非 ID 单元。

---

## failed_checks

### A. 生产终态路径 10 步（计划层）

| # | 步骤 | Task 证据 | 裁定 |
|---|------|-----------|------|
| 1 | typed evidence draft | T-CORE-010 | **PASS** |
| 2 | canonical V1 | T-CORE-014..016；I-1；T-CORE-026 | **PASS** |
| 3 | domain-separated digest | T-CORE-018；T-DOM-001 | **PASS** |
| 4 | contiguous chain | T-CORE-019；CAS | **PASS** |
| 5 | idempotent linearizable append | T-MEM-002/005；T-PG-005；**T-CORE-037** IdempotencyConflict | **PASS** |
| 6 | durable persistence | T-FILE-004；T-PG-003；**T-BOOT-001** required→Durable | **PASS** |
| 7 | crash recovery | T-FILE-005/008；I-17；T-PG-006 | **PASS** |
| 8 | signed checkpoint | T-CORE-024；T-CP-001/002 | **PASS** |
| 9 | independent anchor | T-CP-005；T-ARCH-017；A8 实现可后置 | **PASS\***（合同+门禁有；真实 WORM DEFER-ANCHOR-IMPL） |
| 10 | reproducible verification | T-CORE-025；T-CLI-004；plan §8 | **PASS\***（模板有；实例无） |

→ **A 路径：计划层无断点**（相对 v1.0 的 C4/幽灵/bootstrap 洞已补）。

### B. 否定终态

| 检查 | 裁定 |
|------|------|
| 禁 `Vec+SHA256` 作生产终态 | PASS（plan §0.1） |
| 禁静默 rehash 声称 V1 连续 | PASS（Forbidden + T-LEG） |

### C. 「关键操作可审计」七条件

| # | 条件 | 映射 | 裁定 |
|---|------|------|------|
| C1 | 操作身份明确 | T-CORE-006/007；T-POL-002；I-13 | **PASS** |
| C2 | actor 和 subject 明确 | T-CORE-008/010 | **WEAK**：§7.2 Subject **规范化/版本策略**仍无 Task/DEFER |
| C3 | 稳定 canonical digest | T-CORE-018；T-DOM-001；T-ARCH-004 | **PASS** |
| C4 | 无未声明业务↔durable 窗口 | **T-ATOM-001…006**；T-DOM-005；T-ARCH-016 | **PASS**（计划层；DEFER-ATOM-004 须 accepted） |
| C5 | 无 gap/fork/dup | chain props；CAS；T-CORE-037 | **PASS** |
| C6 | 截尾 + 整链替换可检测 | T-CP-004；**T-CP-007** 存在 | **PASS\*** 任务有；**§33 映射仍写 `T-CP-005 + verify`** → 台账 hygiene 扣分进 R-SPEC-003 |
| C7 | schema/verifier/公钥保留期 | **T-PRIV-002**（六类）；T-SCH-002；T-CP-006；T-PRIV-001 artifacts | **PASS** |

```text
C: 强 5 · WEAK 1 (C2) · 映射 hygiene 污染 C6 台账
A: 10/10 有 Task
→ 路径存在性: PASS*
→ 清单 R-SPEC-003: FAIL → round FAIL
```

### 全 40 R-\* 与 §34 冲突面

| ID | 裁定 | 与 §34 |
|----|------|--------|
| R-SPEC-003 | **FAIL** | 完成定义映射是 §34 勾选面 |
| R-ATOM-001 | PASS | C4 |
| R-APPEND-001/002 | PASS/* | 路径 5–6 |
| R-CP-001 | PASS | C6 任务层 |
| R-GATE-001 | PASS | ANCHOR/ATOMICITY 等已挂 |
| R-TEST-001 | PASS* | 可复现验证 |
| R-MIG-001 | PASS | bootstrap 生产 |
| R-POL-001 | PASS | C1/C7 policy 键 |
| R-MEM-001 | PASS* | 生产禁 memory；\*systemd 清单仍缺 |
| R-HONEST-001 | PASS | 未伪闭合 §34 |
| 其余 | 同 R6b | — |

```text
failed_checks: [R-SPEC-003]
result: FAIL
```

---

## omissions

1. **§7.2 Subject 规范化/版本策略**无 Task 或正式「由 domain 负责」DEFER 登记。  
2. **§19.3 systemd 部署清单** 阻断无 Task（bootstrap/release/archgate 有）。  
3. **§33.4 映射** 未指向 T-CP-007/008（任务有、勾选表脏）。  
4. **T-ATOM-004** / **DEFER-ANCHOR-IMPL** 尚未 `DEFER(accepted)` 人签。  
5. fail-closed 全系统 required ops 仍主要靠 T-DOM-005 + EVIDENCE-COVERAGE 合桶。  
6. §34 实例 Evidence 包目录仍不存在（计划模板有）。

---

## false_pass_risks

| 风险 | 触发 | 后果 |
|------|------|------|
| **「10 步都有 Task」⇒ §34 闭合** | 忽略实现 OPEN + 人审 | 对外「可审计」= 规范禁止的快速 evidence |
| **T-CP-005 DONE** | 仅 anchor 接口 | 误勾 C6 整链替换（须 T-CP-007 证据） |
| **T-PRIV-002 DONE** | 只写 retention 字段名 | 无保留期强制执行仍勾 C7 |
| **A8 Defer 实现** | 当 Defer 合同 | 无独立 anchor 仍标生产 |
| **映射表未改仍称 v1.1 全绿** | 只读 residual PLAN-GAP CLOSED | 掩盖 R-SPEC-003 |

---

## notes

### 相对 v1.0 R9 断点对照

| v1.0 断点 | v1.1 | |
|-----------|------|---|
| 幽灵 T-ATOM | T-ATOM-001…006 | FIXED |
| SoT(C) 无 Task | T-ATOM-003 | FIXED |
| §15.3 无 Task/DEFER | T-ATOM-004 + residual DEFER | FIXED（待 accepted） |
| §15.4 纯内存 | T-ATOM-005 | FIXED |
| retention/artifact | T-PRIV-001…003；I-16 | FIXED |
| 整链替换无 Task | T-CP-007 | FIXED（映射未跟） |
| startup verify | T-CP-008 | FIXED（映射未跟） |
| bootstrap Durable | T-BOOT-001 | FIXED |
| §29.3 背压 | T-BP-001；I-22 | FIXED |
| IdempotencyConflict | T-CORE-037 | FIXED |
| ADR-012 | A11 + T-DOC-005 | FIXED |
| EVIDENCE-ANCHOR-001 | T-ARCH-017 | FIXED |

### 诚实声明

- 战役：**PLANNING** · Spec：**Proposed** · **§33/§34 未闭合** · 实现：**ABSENT**。  
- 计划层 §34 箭头 **可追踪**；清单 **R-SPEC-003** 仍否决 fail_rounds=0。

```yaml
round: 9b
result: FAIL
failed_checks: [R-SPEC-003]
omissions:
  - Subject §7.2 normalization strategy untasked
  - systemd deploy list untasked
  - §33.4 map not pointing at T-CP-007/008
  - DEFER-ATOM-004 / DEFER-ANCHOR-IMPL not yet accepted
false_pass_risks:
  - path-has-tasks mistaken for §34 closed
  - T-CP-005 as replacement proxy
  - retention field-only C7 fake close
notes: |
  A-path 10 steps plan-level PASS after v1.1.
  C1–C7 plan-level: C2 WEAK; others PASS/PASS*.
  Round FAIL from R-SPEC-003 mapping hygiene only (strict checklist).
```
