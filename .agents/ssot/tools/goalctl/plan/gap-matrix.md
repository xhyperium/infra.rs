# Gap Matrix — GOAL-GOALCTL-002 / SPEC-GOALCTL-002 vs 现状

| 字段 | 值 |
|------|-----|
| Matrix ID | `GAP-GOALCTL-002-v1` |
| Source | `20260716` Goal + Spec |
| Baseline package | `xhyper-goalctl` **0.1.0** |
| Target agent-safe | **0.1.1** Phase 1.1 Truth Hardening |
| Status enum | `ABSENT` · `WRONG` · `PARTIAL` · `PASS` · `DEFER` · `GOVERNANCE` |

---

## 1. GAP-001 … GAP-017

| ID | 严重度 | 缺口 | 基线状态 | 目标 | 关闭 Wave | 处置 |
|----|--------|------|----------|------|-----------|------|
| GAP-001 | P0 | artifact 读 live，标 HEAD `source_commit` | WRONG | committed subject 读取；`--source-commit` | W1 | **SAFE** |
| GAP-002 | P0 | reconcile 目录/文件存在 → VERIFIED/OK | WRONG | 无事实 → NOT_PROVEN；禁假 VERIFIED/OK | W1 | **SAFE** |
| GAP-003 | P0 | compile task-file 未强制 commit==bound tree | WRONG | commit/tree 真实绑定；mismatch 失败 | W1 | **SAFE** |
| GAP-004 | P0 | 默认 compile 为通用模板 | WRONG | 全量 Goal→Task DEFER；本战役 template 标注 + 禁虚构 ID = SAFE 最小 | W1 最小 / later 全量 | **SAFE 最小 / DEFERRED 全量** |
| GAP-005 | P0 | approval_refs 只校验非空 | WRONG | ApprovalRecord 真字段校验 | W1 | **SAFE** |
| GAP-006 | P0 | Evidence subject/digest/freshness/trust | ABSENT | Phase 2 Proof | later | **DEFERRED** |
| GAP-007 | P1 | `--trust-level` 合同未实现 | ABSENT→**PASS** | CLI 实现 + 未知值 USAGE + 输出 trust_level | W1 | **SAFE / PASS** |
| GAP-008 | P1 | `--source-commit` 未全命令覆盖 | PARTIAL | 全 subject-bound 命令一致 | W1（artifact/reconcile/compile） | **SAFE** 子集 |
| GAP-009 | P1 | Rust 模型 vs JSON Schema 无自动证明 | ABSENT | schema conformance | later | **DEFERRED** |
| GAP-010 | P1 | canonical JSON 非完整 JCS | PARTIAL | Canonical v1 | later | **DEFERRED** |
| GAP-011 | P1 | Repository Identity DEGRADED | PARTIAL | 发布链头就绪 | later | **DEFERRED** |
| GAP-012 | P1 | artifact module filter substring | WRONG | exact segment | later | **DEFERRED**（resolve 已 exact） |
| GAP-013 | P1 | 路径/glob 语义重复 | PARTIAL | PathSpec 统一 | later | **DEFERRED** |
| GAP-014 | P1 | README/Goal/Spec 过时事实 | WRONG | 对齐 CURRENT-STATE | W2 | **SAFE** |
| GAP-015 | P2 | 无 Bootstrap Trust Root | ABSENT | L0 trust | later | **DEFERRED** |
| GAP-016 | P2 | 无 Harness/Evidence/Verifier/Shadow | ABSENT | Phase 2–4 | later | **DEFERRED** / **POLICY** |
| GAP-017 | P2 | 无 SLO/Failure Corpus/回放 | ABSENT | Phase 3+ | later | **DEFERRED** |

---

## 2. AC-P0 映射

| AC ID | 要求 | 覆盖 GAP | 关闭条件（机器） | 处置 |
|-------|------|----------|------------------|------|
| AC-P0-SNAPSHOT | artifact/reconcile/compile 对 `--source-commit A` 只读 A；dirty 不改输出；commit/tree mismatch 非零；输出真实 tree | GAP-001,003,008 | 负例 + CLI 冒烟 | **SAFE** |
| AC-P0-RECONCILE | 仅 Evidence/Fact→VERIFIED；无 README→OK；无事实→NOT_PROVEN | GAP-002 | unit + reconcile 负例 | **SAFE** |
| AC-P0-COMPILE | invalid approval 失败；commit/tree 绑定；scope 完整 | GAP-003,005（004 DEFER） | unit + task-file 负例 | **SAFE**（004 除外） |

---

## 3. 命令面对照

| 命令 | 0.1.0 | Phase 1.1 目标 | 备注 |
|------|-------|----------------|------|
| version | PASS | PASS | — |
| doctor | PASS | PASS | 允许 live 诊断 |
| index | PARTIAL | PARTIAL | HEAD-only cargo metadata（既有 fail-closed） |
| resolve | PASS | PASS | 已 committed |
| artifact | WRONG | PASS | W1 修复 |
| reconcile | WRONG | PASS | W1 修复 |
| compile | PARTIAL | PASS（bound 校验） | W1 修复；真编译 DEFER |
| evidence/harness/verify/gate | ABSENT | ABSENT | 未开放 capability |

---

## 4. 治理项

| ID | 项 | 处置 |
|----|-----|------|
| GOV-001 | GOAL-GOALCTL-002 Status | **HUMAN_ONLY** PROPOSED |
| GOV-002 | SPEC-GOALCTL-002 Status | **HUMAN_ONLY** PROPOSED |
| GOV-003 | Cutover / required CI | **POLICY** |
| GOV-004 | `@liukongqiang5` approve on final head | **HUMAN_ONLY** |
| GOV-005 | `LIUKONGQIANG5_APPROVE_TOKEN` | **HUMAN_ONLY**（env only） |
