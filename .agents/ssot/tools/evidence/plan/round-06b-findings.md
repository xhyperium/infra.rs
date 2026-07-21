# Round 06b Findings — CLI §25 · Policy §26 · Gates §27 · CI §28（v1.1 后）

| 字段 | 值 |
|------|-----|
| round | `6b` |
| role | Verifier · post-v1.1 remediation |
| scope | plan.md §4 全 40 R-\* + §25–§28 深检 |
| baseline | plan pack **v1.1** · inventory I-1…I-26 · residual-open · tasks §33 映射 |
| prior | `round-06-findings.md`（v1.0 **FAIL**） |
| date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — v1.1 已显著修补 R6 主洞（CLI 默认/repair AC、policy 12 键、23 个 EVIDENCE-\* 映射、CI 拆桶），但 **R-SPEC-003** 仍因 §33.4 映射含非 Task ID 文本（`T-CP-005 + verify`）失败；另有 bundling 假 DONE 与 branch 覆盖缺口等 residual。任一项 FAIL → 整轮 FAIL。

---

## 专项确认（本轮强制）

| 项 | 裁定 | 证据 |
|----|------|------|
| I-9 全部 23 个 EVIDENCE-\* 有 Task ID | **YES** | `spec-inventory.md` I-9：10 Core + 6 Adapter + 7 System 均挂 `T-ARCH-*` |
| 无 `T-ATOM via design` 幽灵 | **YES**（活文档） | 仅 changelog 提及消灭；映射表用 `T-ATOM-001…006` |
| ADR-012 A11 存在 | **YES** | `approval-packet.md` A11；`plan.md` §1.3；`T-DOC-005` |
| §33 映射仅真实 Task ID | **NO** | `full replacement 检测` → `T-CP-005 + verify`（`verify` 非 ID；应用 `T-CP-007`） |
| I-26 Forbidden 统一 | **YES** | inventory I-26 ≡ approval §2；plan 页眉指向 I-26 |
| 原 Z 子节覆盖（本轮非主） | 见 R10b | §15.3/15.4/22.2–22.4/23.3/29.3 已有 Task/I-\* |

### I-9 × Task 核验表（23/23）

| EVIDENCE-\* | Task | 备注 |
|-------------|------|------|
| PATH-001 | T-ARCH-001 | OK |
| DEP-001 | T-ARCH-002 | 与 DEP-002 合桶 |
| DEP-002 | T-ARCH-002 | 合桶 |
| ANYHOW-001 | T-ARCH-003 | OK |
| CANONICAL-001 | T-ARCH-010 | v1.1 补 |
| DOMAIN-001 | T-ARCH-004 | 五门禁合桶 |
| DEBUG-HASH-001 | T-ARCH-004 | 合桶 |
| JSON-HASH-001 | T-ARCH-004 | 合桶 |
| GENESIS-001 | T-ARCH-004 | 合桶 |
| PUBLIC-001 | T-ARCH-004 | 合桶 |
| DURABILITY-001 | T-ARCH-011 | v1.1 补 |
| MEMORY-PROD-001 | T-ARCH-005 | OK |
| IDEMPOTENCY-001 | T-ARCH-012 | v1.1 补 |
| CONCURRENCY-001 | T-ARCH-013 | v1.1 补 |
| RECOVERY-001 | T-ARCH-014 | v1.1 补 |
| FSYNC-001 | T-ARCH-015 | v1.1 补 |
| POLICY-001 | T-ARCH-006 | 三门禁合桶 |
| COVERAGE-001 | T-ARCH-006 | 合桶 |
| ATOMICITY-001 | T-ARCH-016 | v1.1 独立 |
| CHECKPOINT-001 | T-ARCH-006 | 合桶 |
| ANCHOR-001 | T-ARCH-017 | v1.1 补 |
| SCHEMA-001 | T-ARCH-018 | v1.1 补 |
| VECTOR-001 | T-ARCH-019 | v1.1 补 |

→ **R-GATE-001 = PASS**（全列表可追踪；合桶记 residual 风险，不因缺 ID 判 FAIL）。

---

## failed_checks

### 全 40 R-\* 复检（v1.1）

| # | Check ID | 裁定 | 一句理由 |
|---|----------|------|----------|
| 1 | R-SPEC-001 | PASS | Spec ID=SPEC-EVIDENCE-002 |
| 2 | R-SPEC-002 | PASS | gap-matrix §0–§34 齐全 |
| 3 | R-SPEC-003 | **FAIL** | §33.4 `T-CP-005 + verify` 非纯 Task ID；startup 未映射已存在的 `T-CP-008`；33.1 ADR 仅 `T-DOC-004` 漏 ADR-012/`T-DOC-005` |
| 4 | R-GAP-001 | PASS | DEF-001…020 在 residual；todo 有 DEF 入口 |
| 5 | R-GAP-002 | PASS | gap §2 T1–T18 有目标防御 |
| 6 | R-PATH-001 | PASS | plan §5 |
| 7 | R-PATH-002 | PASS | memory/file/postgres |
| 8 | R-PATH-003 | PASS | CLI + cutover 删 tools/evidence |
| 9 | R-DEP-001 | PASS | kernel+sha2+thiserror |
| 10 | R-DEP-002 | PASS | I-12 完整禁表 |
| 11 | R-API-001 | PASS | W1 IDs |
| 12 | R-API-002 | PASS | Draft+六态 |
| 13 | R-API-003 | PASS | 私有+seal |
| 14 | R-CANON-001 | PASS | I-1 + T-CORE-014 |
| 15 | R-CANON-002 | PASS | T-CORE-017 |
| 16 | R-CANON-003 | PASS | T-CORE-029 |
| 17 | R-CANON-004 | PASS | T-CORE-018 |
| 18 | R-TIME-001 | PASS | T-CORE-011 |
| 19 | R-CHAIN-001 | PASS | T-CORE-019 + mem |
| 20 | R-APPEND-001 | PASS | I-5 + T-CORE-022/038 + T-BOOT-001 |
| 21 | R-APPEND-002 | PASS* | T-CORE-037 + T-DOM-005；\*系统级 required fail-closed 仍偏 domain |
| 22 | R-READ-001 | PASS | T-CORE-023 |
| 23 | R-ATOM-001 | PASS | T-ATOM-001…006 + I-15 |
| 24 | R-CP-001 | PASS | T-CORE-024 + T-CP-002/004/**007** |
| 25 | R-ERR-001 | PASS | I-4 + T-CORE-020/021 |
| 26 | R-MEM-001 | PASS* | T-MEM-004/007 + T-BOOT-001；\*systemd 部署清单无独立 Task |
| 27 | R-FILE-001 | PASS | I-17 + T-FILE-002…008 |
| 28 | R-PG-001 | PASS | T-PG-002… |
| 29 | R-TEST-001 | PASS* | golden/property/fuzz/cov/mutants/miri 均有 Task；\*branch≥90% 仅挂 Nightly 桶、无显式 AC |
| 30 | R-CLI-001 | **PASS** | I-10 命令+退出码+默认行为+repair AC；T-CLI-002…007 |
| 31 | R-POL-001 | **PASS** | I-13 12 键；T-POL-002 schema 校验 |
| 32 | R-GATE-001 | **PASS** | I-9 23/23 有 Task（见上表） |
| 33 | R-MIG-001 | PASS | I-14 P0–P6↔Wave；禁静默 rehash |
| 34 | R-EVID-001 | PASS | plan §8 ≡ §32 树 |
| 35 | R-DOWN-001 | PASS | T-DOM-\* + T-GATE-\* |
| 36 | R-GOV-001 | PASS | A1–A13；人审闸；A11 ADR-012 |
| 37 | R-FORBID-001 | PASS | I-26 三处一致（页眉引用 / inventory / approval） |
| 38 | R-TODO-001 | PASS* | Wave+DEF；\*W1+ 仍为范围摘要 |
| 39 | R-CROSS-001 | PASS | plan §1.2 INFRA-003 |
| 40 | R-HONEST-001 | PASS | Proposed≠Approved；≠§33 闭合；campaign PLANNING |

```text
strict FAIL: R-SPEC-003
PASS / PASS*: 39
result: FAIL
```

### 相对 v1.0 R6 主失败项（已修复）

| 原 FC | v1.1 | 裁定 |
|-------|------|------|
| CLI §25.2 默认行为 | I-10 + T-CLI-002 | PASS |
| repair-tail 全约束 | I-10 repair AC + T-CLI-005 | PASS |
| policy 12 字段 | I-13 + T-POL-002 | PASS |
| ≥9 EVIDENCE-\* 无门禁 | I-9 + T-ARCH-010…019 | PASS |
| CI 草案桶 / Nightly | T-MUT/MIRI/FUZZ/CI-NIGHTLY + I-20 | PASS（任务级） |
| crate-standard | T-CI-004 | PASS |
| T-ATOM via design | 已消灭 | PASS |

---

## omissions

1. **§33.4 映射未引用 `T-CP-007` / `T-CP-008`**：任务已存在，映射表仍写 `T-CP-005 + verify` 与 `T-FILE-005 T-CP-004`。  
2. **§33.1 ADR 行未含 `T-DOC-005`（ADR-012）**。  
3. **branch coverage ≥90%** 无独立 AC（line 有 `T-CORE-033`/`T-CI-003`）。  
4. **systemd 部署清单**（§19.3）无 Task。  
5. **T-ARCH-004 / T-ARCH-006 合桶**：多 ID 单 DONE 风险（见 false_pass）。  
6. approval 签字表表头仍写 `A1–A10`，实际已扩到 A13。

---

## false_pass_risks

| 风险 | 机制 | 严重度 |
|------|------|--------|
| **映射表假闭合** | 评审只读 §33 表勾「有字符串」而忽略非 ID `verify` | **P0** |
| **合桶门禁假 DONE** | `T-ARCH-004` 做 1/5 ID 即 DONE | **P0** |
| **Nightly=BLOCKED 当 PASS** | `T-CI-NIGHTLY-001` 纪律依赖执行诚实 | **P1** |
| **I-9 有表 = 门禁已接线** | 规则 Task 仍 TODO；实现未落地 | 信息性 |

---

## notes

- 本轮是 **计划完备性** 复检，非实现验收；代码仍为 `tools/evidence` 0.1.0 原型。  
- v1.1 对 R6 主题的修补质量高：CLI/Policy/Gates/CI **任务层** 基本闭环。  
- **不得** `fail_rounds=0`：R-SPEC-003 仍 FAIL。  
- 最小修补建议：  
  1. §33.4：`full replacement` → `T-CP-007`；`startup verify` → `T-CP-008`  
  2. §33.1 ADR → `T-DOC-004 T-DOC-005`  
  3. 33.5 branch → 显式 AC 或正式 DEFER(accepted)  
  4. T-ARCH-004/006 AC 要求「所列每个 EVIDENCE-\* 独立证据」  

```yaml
round: 6b
result: FAIL
failed_checks: [R-SPEC-003]
omissions:
  - §33.4 maps "T-CP-005 + verify" (non-Task-ID); T-CP-007 unused in map
  - startup map omits T-CP-008
  - §33.1 ADR omits T-DOC-005 / ADR-012
  - branch≥90% no dedicated AC
  - systemd deploy list untasked
false_pass_risks:
  - slash-bundled T-ARCH-004/006 partial DONE
  - mapping-table checkbox without ID hygiene
notes: |
  23/23 EVIDENCE-* Task IDs present (I-9).
  Ghost T-ATOM gone. A11 present. I-26 unified.
  R-CLI/R-POL/R-GATE/R-TEST plan-level PASS after v1.1.
```
