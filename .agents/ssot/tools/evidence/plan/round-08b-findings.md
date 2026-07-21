# Round 08b Findings — Completion Skeptic / §33（v1.1 后）

| 字段 | 值 |
|------|-----|
| round | `8b` |
| role | Verifier / Completion Skeptic · post-v1.1 |
| scope | §33.1–33.6 勾选 ↔ Task ID；§32；Forbidden；诚实状态；假完成扫描；全 40 R-\* |
| prior | `round-08-findings.md`（v1.0 **FAIL**） |
| date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — 幽灵 `T-ATOM via design` **已消灭**；33.5 草案桶 **已拆**；33.6 external/Tier-A **有真实 ID**；I-26 Forbidden **统一**。但 §33 映射 **仍未满足「仅真实 Task ID」**：`full replacement 检测 → T-CP-005 + verify` 含自由文本 `verify`。R-SPEC-003 / R-FORBID 路径上 R-SPEC-003 失败 → 轮次 FAIL。

---

## 专项确认

| 项 | 裁定 |
|----|------|
| 无 `T-ATOM via design` 幽灵 | **CONFIRMED**（活 plan/tasks/gap/approval/residual 无该映射；仅 v1.1 changelog 叙述） |
| §33 映射仅真实 Task ID | **FAIL**（见 33.4） |
| I-26 Forbidden 统一 | **PASS**（inventory / approval≡I-26 / plan 页眉引用） |
| 未把 Proposed 写成 Approved | **PASS** |
| 未宣称 §33 闭合 / stable | **PASS** |

---

## failed_checks

### R-SPEC-003 — §33.1–33.6 逐条（v1.1）

#### 33.1 规格闭合

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| SPEC Approved | T-HUM-001 | OK |
| 旧 spec superseded | T-DOC-002 T-HUM-002 | OK |
| ADR 冲突已修订 | **T-DOC-004 only** | **弱/FAIL\***：ADR-012 应对 `T-DOC-005`，表未列 |
| 路径 package 对齐 | T-CUT-002 T-CUT-003 | OK（依赖 A11） |
| architecture registry | T-REG-001 T-REG-002 | OK |
| evidence-policy.toml | T-POL-001 T-POL-002 | OK |
| 无未登记 Unknown | T-RES-001 T-SKEP-001 | OK（residual 已存在） |

#### 33.2 Core 闭合

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| crates/evidence…字段私有 | T-CORE-\* 表 | **OK**（全部真实 ID） |

\*Subject 规范化（§7.2 领域策略）仍无独立 Task——字段有、策略无；记 omission，不单独否决 33.2 字面勾选（勾选写 actor/subject 完整，依赖 T-CORE-008/010）。

#### 33.3 Adapter 闭合

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| memory/file/pg/conformance/idempotency/crash/disk/fsync/volatile | 真实 T-MEM/FILE/PG/ARCH | **OK** |
| disk/fsync | T-FILE-008（含 disk full/short write/fsync err） | OK |

#### 33.4 Checkpoint 闭合 — **本轮主 FAIL 点**

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| signed checkpoint | T-CP-002 | OK |
| key rotation | T-CP-006 | OK |
| independent anchor | T-CP-005 | OK |
| tail truncation | T-CP-004 | OK |
| **full chain replacement** | **`T-CP-005 + verify`** | **FAIL**：`verify` 非 Task ID；**应 `T-CP-007`**（任务表已有） |
| **startup verify** | `T-FILE-005 T-CP-004` | **弱**：拼凑恢复+截尾 ≠ 启动合同；**应 `T-CP-008`**（已存在） |

#### 33.5 测试闭合

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| golden | T-CORE-026/027 T-CLI-004 | OK |
| property | T-CORE-028/030 | OK |
| fuzz | **T-FUZZ-001**（非 T-CI-002） | OK |
| line≥95% | T-CORE-033 T-CI-003 | OK |
| branch≥90% | **T-CI-NIGHTLY-001** | **弱**：Nightly AC 未写 branch fail-under |
| mutants≥90% | T-MUT-001 | OK |
| Miri | T-MIRI-001 | OK |
| adapter chaos | T-FILE-008 T-PG-006 | OK |
| historical schema | T-LEG-002 T-SCH-002 T-CI-NIGHTLY-001 | OK |

#### 33.6 系统闭合

| Checkbox | 映射 | 裁定 |
|----------|------|------|
| required ops 登记 | T-POL-002 | OK |
| fail-closed | T-DOM-005 | 弱（偏 macro） |
| Tier-A 原子性 | **T-ATOM-001 T-ATOM-002 T-ARCH-016** | **OK**（幽灵已灭） |
| external Attempted+terminal | **T-ATOM-004**（可 DEFER accepted） | **OK** |
| source artifacts retention | T-PRIV-001 T-PRIV-002 | OK |
| verifier/schema/keys | T-PRIV-002 T-CP-006 T-SCH-002 | OK |
| CI Evidence | T-EVID-SYS T-CI-001 | OK |

### 其他 R-\*

| ID | 裁定 |
|----|------|
| R-FORBID-001 | **PASS** — I-26 统一；无与执行步骤矛盾（§5 双包短暂共存 + I-18 隔离策略一致） |
| R-HONEST-001 | **PASS** |
| R-EVID-001 | **PASS** — plan §8 模板 |
| R-GOV-001 | **PASS** — 人审闸 + A11 |
| R-TODO-001 | **PASS\*** |
| 其余见 R6b 表 | 同 R6b；**唯一 strict FAIL = R-SPEC-003** |

```text
failed_checks: [R-SPEC-003]
result: FAIL
```

---

## omissions

1. §33.4 映射未同步 v1.1 新增 `T-CP-007` / `T-CP-008`。  
2. §33.1 未列 `T-DOC-005`。  
3. branch≥90% 无硬 AC。  
4. fail-closed 未扩到全部 required ops（仅 T-DOM-005）。  
5. §7.2 Subject 规范化/版本策略无 Task（领域层职责，未登记 DEFER）。  
6. approval 签字区仍 `A1–A10` 表头。

---

## false_pass_risks

| 风险 | 机制 | 严重度 |
|------|------|--------|
| **「映射表有行」⇒ §33 可勾** | 含非 ID 文本仍当可追踪 | **P0** |
| **T-CP-005 DONE ⇒ 整链替换可测** | 接口任务冒充 replacement 检测 | **P0** |
| **T-ATOM-004 DEFER 未 accepted** | residual 写「可 DEFER」≠ 已 accepted | **P1** |
| **T-33-001 抢跑** | 人审前勾 33.x | **P0**（纪律已写，靠执行） |
| **I-26 页眉摘要 ≠ 全文** | 依赖「见 I-26」跳转；本地拷贝漂移时失效 | P2 |

---

## notes

### Forbidden 三处一致性

| 位置 | 形态 | 一致？ |
|------|------|--------|
| inventory **I-26** | 十条全文 | SSOT |
| approval §2 | `≡ I-26` 同十条 | YES |
| plan 页眉 | 指向 I-26 + 短摘要 | YES（引用式） |
| plan §1 原则 | 十条执行纪律（略不同措辞） | 不矛盾 |

### 相对 v1.0 R8 的改进

- 幽灵 T-ATOM：**GONE**  
- external 无 ID：**FIXED**（T-ATOM-004）  
- 33.5 规划桶：**FIXED**（拆 FUZZ/MUT/MIRI/NIGHTLY）  
- residual-open：**PRESENT**  
- ADR-012：**A11 存在**（但 §33.1 映射漏挂）

### 诚实声明

- **§33 未闭合** · Spec **Proposed** · 禁止 T-33-001 提前 DONE。  
- 修补映射表 3 行即可把 R-SPEC-003 推近 PASS；仍须独立 verifier 重跑。

```yaml
round: 8b
result: FAIL
failed_checks: [R-SPEC-003]
omissions:
  - §33.4 "T-CP-005 + verify" non-Task-ID token
  - startup not mapped to T-CP-008
  - §33.1 ADR omits T-DOC-005
  - branch≥90% soft map
  - Subject §7.2 strategy untasked
false_pass_risks:
  - T-CP-005 proxy for full-chain replacement
  - DEFER(accepted) not yet accepted for T-ATOM-004
  - premature T-33-001
notes: |
  Ghost T-ATOM via design CONFIRMED ABSENT from live mappings.
  I-26 Forbidden unified PASS. Honest status PASS.
  §33 real-Task-ID-only: FAIL on one cell (+ weak startup/ADR cells).
```
