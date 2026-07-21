# Evidence Plan 10x Verdict

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-EVIDENCE-002-v1-complete` |
| Spec | `SPEC-EVIDENCE-002` · Status **Proposed**（≠ Approved） |
| Scope | 计划完备性十轮（plan §4 R-SPEC-001…R-HONEST-001 ×40）· **非** 实现 W7 |
| Baseline | `main@007ca7b5` |
| Sources | `plan.md` · `tasks.md` · `gap-matrix.md` · `spec-inventory.md` · `residual-open.md` · `approval-packet.md` · `.worktrees/evidence-todo.md` · `xhyper-evidence-complete-spec.md` · prior `round-0Nb-findings.md` |
| Pass3 date | 2026-07-14 |
| Pass3 posture | **strict** · 仅真实 Task ID · 不把 Proposed 当 Approved · 不把计划 PASS 当实现 PASS |

---

## Pass history

```text
pass1: fail_rounds=10 (round-01..10-findings)
       主洞：幽灵 T-ATOM、T-CI-002 草案桶、preimage/Error/门禁未枚举、
             residual/inventory 缺、§33 映射洞、ADR-012 未对账 等

pass2: R1b–R5b PASS; R6b–R10b FAIL (R-SPEC-003 only)
       v1.1 补 inventory I-1…I-26 + 真实 T-ATOM/T-PRIV/T-ARCH-010…019 +
       拆 CI 桶后，内容层 39/40 绿；唯一 strict FAIL：
       - §33.4 full replacement → "T-CP-005 + verify"（verify 非 Task ID）
       - §33.4 startup → T-FILE-005/T-CP-004（未挂已存在的 T-CP-008）
       - §33.1 ADR → 仅 T-DOC-004（漏 T-DOC-005 / ADR-012）

pass3: <this run> · hygiene 后复检
       fail_rounds = 0
       failed_checks = []
```

### pass2 → pass3 修补核对（R-SPEC-003 三处）

| 缺陷（pass2） | 当前 `tasks.md` §33 映射 | 裁定 |
|---------------|--------------------------|------|
| full replacement = `T-CP-005 + verify` | `T-CP-007` | **CLOSED** |
| startup = `T-FILE-005 T-CP-004` | `T-CP-008` | **CLOSED** |
| ADR = 仅 `T-DOC-004` | `T-DOC-004 T-DOC-005` | **CLOSED** |

任务表中 `T-CP-007` / `T-CP-008` / `T-DOC-005` 均存在且 AC 与映射语义一致。

---

## Per-round results (pass3)

每轮独立重跑同一 40 项清单；任一项 FAIL → 该轮 FAIL。本轮 **40/40 PASS** → 十轮全绿。

| Round | Result | 1-line reason |
|-------|--------|---------------|
| **R1** | **PASS** | Spec ID 存在；gap §0–§34 齐；§33 映射仅真实 Task ID（含 T-CP-007/008、T-DOC-005）；DEF/T1–T18 登记完整。 |
| **R2** | **PASS** | 目标路径 `crates/evidence` + adapters memory/file/postgres + `tools/evidence-cli` + cutover 删 `tools/evidence` 写明；core 白名单 + I-12 禁表。 |
| **R3** | **PASS** | API（Digest/Draft/Outcome/RecordV1+seal）与 canonical 25 步/genesis/边界/域分离均绑 Task+I-\*；时间分离与 ChainHead 有 Task。 |
| **R4** | **PASS** | Durability 三态 + 生产 Durable；幂等/CAS/fail-closed；Reader 合同；A/B/C/External/Memory/Rejected 原子性 T-ATOM-001…006；CP+Signed+TailTruncated。 |
| **R5** | **PASS** | Error 24+XError 映射 I-4；mem 禁伪 Durable；file frame/fsync/I-17 recovery；pg 四表不变量；golden/property/fuzz/cov/mutants/miri 非草案桶。 |
| **R6** | **PASS** | CLI I-10 全命令+退出码；policy I-13；EVIDENCE-\* 23/23（I-9→T-ARCH）；P0–P6↔Wave I-14；§32 树 plan §8。 |
| **R7** | **PASS** | domain_macro+gate 迁移任务在；approval 人审闸 A1–A13；Forbidden≡I-26 无矛盾执行步；todo 覆盖 Wave/DEF；INFRA-003 边界声明。 |
| **R8** | **PASS** | 诚实：Spec Proposed、campaign PLANNING、≠§33 闭合、≠stable；无「T-CP-005 + verify」类非 ID 令牌；无幽灵 T-ATOM。 |
| **R9** | **PASS** | 对抗复扫 §33.1–33.6 对照规范 checkbox 全集：每项可追踪到任务表存在的 ID；DEFER 候选登记于 residual，未伪 CLOSED。 |
| **R10** | **PASS** | 交叉：inventory I-1…I-26 齐；PLAN-GAP-001…009 CLOSED；pass2 唯一 FAIL 已灭；无新增 strict 遗漏使任一 R-\* 翻 FAIL。 |

```text
pass3_round_fail_count = 0
pass3_checklist_fail_count = 0
```

---

## Checklist 40

| # | Check ID | Result | Evidence (1 line) |
|---|----------|--------|-------------------|
| 1 | R-SPEC-001 | **PASS** | `xhyper-evidence-complete-spec.md` 页眉 Spec ID=`SPEC-EVIDENCE-002`；plan/gap/todo 同 ID。 |
| 2 | R-SPEC-002 | **PASS** | `gap-matrix.md` §1 行 §0–§34 齐全（34 行+主题/Wave）。 |
| 3 | R-SPEC-003 | **PASS** | `tasks.md` §33.1–33.6 映射全部为真实 Task ID；规范 checkbox 无漏项；无 `+ verify` / `via design`。 |
| 4 | R-GAP-001 | **PASS** | DEF-001…018 在 gap+residual；+DEF-019/020；todo §1 入口。 |
| 5 | R-GAP-002 | **PASS** | gap-matrix §2 T1–T18 均有目标防御与 Gap 状态。 |
| 6 | R-PATH-001 | **PASS** | plan §5 `crates/evidence/` 模块树。 |
| 7 | R-PATH-002 | **PASS** | plan §5 + T-MEM/FILE/PG-001 → `crates/adapters/evidence/{memory,file,postgres}`。 |
| 8 | R-PATH-003 | **PASS** | `tools/evidence-cli/`（T-CLI-001）+ T-CUT-002 删除 `tools/evidence`。 |
| 9 | R-DEP-001 | **PASS** | plan §5 / T-CORE-002：kernel + sha2 + thiserror only。 |
| 10 | R-DEP-002 | **PASS** | I-12 完整禁表；T-ARCH-002 AC 绑 I-12。 |
| 11 | R-API-001 | **PASS** | T-CORE-004…007 Digest32/ChainId/EventId/OperationId/EvidenceName。 |
| 12 | R-API-002 | **PASS** | T-CORE-009/010 + I-2 Outcome 六态 tag。 |
| 13 | R-API-003 | **PASS** | T-CORE-012/013 字段私有 + seal_record_v1。 |
| 14 | R-CANON-001 | **PASS** | I-1 25 步；T-CORE-014 AC「I-1 逐步 1..25」。 |
| 15 | R-CANON-002 | **PASS** | T-CORE-017 + I-3 GENESIS；禁全零。 |
| 16 | R-CANON-003 | **PASS** | T-CORE-029 `("ab","c")≠("a","bc")`。 |
| 17 | R-CANON-004 | **PASS** | T-CORE-018 digest_canonical；禁公开 hash_bytes。 |
| 18 | R-TIME-001 | **PASS** | T-CORE-011 recorded_at / event_time。 |
| 19 | R-CHAIN-001 | **PASS** | T-CORE-019 ChainHead+Option；T-MEM-010 禁零摘要哨兵。 |
| 20 | R-APPEND-001 | **PASS** | I-5 + T-CORE-022/038 三态；T-BOOT-001 生产 Durable。 |
| 21 | R-APPEND-002 | **PASS** | T-MEM-002 CAS/幂等；T-CORE-037 IdempotencyConflict；T-DOM-005 fail-closed。 |
| 22 | R-READ-001 | **PASS** | T-CORE-023 + T-MEM-003 head/get/range；limit 1..=10000。 |
| 23 | R-ATOM-001 | **PASS** | T-ATOM-001…006 + I-15（A/B/C/External/Memory/Rejected）。 |
| 24 | R-CP-001 | **PASS** | T-CORE-024 + T-CP-001…008（含 Signed/TailTruncated/anchor/全链替换/startup）。 |
| 25 | R-ERR-001 | **PASS** | I-4 24 variants + XError 映射；T-CORE-020/021；禁损坏→Invalid。 |
| 26 | R-MEM-001 | **PASS** | T-MEM-004/007/010 + T-ARCH-005 + T-BOOT-001 + T-MEM-PROD-SYS。 |
| 27 | R-FILE-001 | **PASS** | T-FILE-002…010 + I-17 恢复 10 步 + fsync/group commit。 |
| 28 | R-PG-001 | **PASS** | T-PG-002 heads/records/outbox/checkpoints + 唯一约束；T-PG-003/004 事务。 |
| 29 | R-TEST-001 | **PASS** | golden/property/fuzz/line+branch/mutants/miri 均有正式 Task（无 T-CI-002 草案桶）。 |
| 30 | R-CLI-001 | **PASS** | T-CLI-002…007 + I-10 命令/退出码/默认行为/repair AC。 |
| 31 | R-POL-001 | **PASS** | T-POL-001 骨架已落盘 `.architecture/evidence-policy.toml`；T-POL-002 + I-13。 |
| 32 | R-GATE-001 | **PASS** | I-9 共 23 EVIDENCE-\* → T-ARCH-001…006 + 010…019 全映射。 |
| 33 | R-MIG-001 | **PASS** | I-14 P0–P6↔Wave；plan §6 禁静默 rehash；T-LEG-001…003。 |
| 34 | R-EVID-001 | **PASS** | plan §8 目录树 ≡ 规范 §32。 |
| 35 | R-DOWN-001 | **PASS** | T-DOM-001…006 + T-GATE-001/002。 |
| 36 | R-GOV-001 | **PASS** | approval-packet A1–A13；Spec Approved / stable 人审-only。 |
| 37 | R-FORBID-001 | **PASS** | plan 页眉 → I-26；approval §2 ≡ inventory I-26；无矛盾执行步骤。 |
| 38 | R-TODO-001 | **PASS** | `.worktrees/evidence-todo.md` 覆盖 Wave 摘要 + DEF 入口；residual SSOT 仓内。 |
| 39 | R-CROSS-001 | **PASS** | plan §1.2 + gap §5 与 INFRA-003 边界声明。 |
| 40 | R-HONEST-001 | **PASS** | Spec/plan/gap/todo 均为 **Proposed** / PLANNING；未写 Approved/stable/§33 闭合；未宣称实现完成。 |

**Checklist summary:** PASS **40** / FAIL **0**

---

## Ghost/orphan scan

### §33 映射 → 任务表

```text
scanned: tasks.md 「§33 勾选 → Task 映射」全部单元格
phantom_ids_found: []
non_task_tokens_in_id_cells: []
  (parentheticals like 「可 DEFER accepted」「AC：branch…」are status/AC notes, not Task IDs)
former_ghosts_closed:
  - "T-ATOM via design" → T-ATOM-001/002 + T-ARCH-016
  - "T-CP-005 + verify" → T-CP-007
  - startup weak FILE/CP-004 → T-CP-008
  - ADR only T-DOC-004 → + T-DOC-005
  - T-CI-002 草案桶 → T-FUZZ-001 / T-MUT-001 / T-MIRI-001 / T-CI-NIGHTLY-001 / T-CI-003
```

### 规范 §33 checkbox 覆盖

| 节 | Spec 勾选项数 | 映射行数 | 孤儿勾选 |
|----|---------------|----------|----------|
| 33.1 | 7 | 7 | 0 |
| 33.2 | 12 | 12 | 0 |
| 33.3 | 8 | 8 | 0 |
| 33.4 | 6 | 6 | 0 |
| 33.5 | 9 | 9 | 0 |
| 33.6 | 7 | 7 | 0 |

### 活文档幽灵串（排除 findings 历史叙述）

| 串 | plan/tasks/gap/approval/residual/inventory |
|----|--------------------------------------------|
| `T-ATOM via design` | **ABSENT**（仅 changelog/findings 历史） |
| `T-CP-005 + verify` | **ABSENT** |
| `T-CI-002` 作为活任务 | **ABSENT**（inventory 仅叙「从 T-CI-002 拆出」） |

### 编号缺口（非幽灵）

- `T-ARCH-007…009`：有意跳号（I-9 已映射 001–006 + 010–019），**非** orphan 引用。
- `T-V10-R01 … T-V10-R10`：W7 范围记号；不在 §33 映射单元格。

### Residual 诚实 OPEN（不构成 R-\* FAIL）

| 项 | 状态 | 说明 |
|----|------|------|
| DEF-001…020 | OPEN | 实现/治理缺口；计划已登记 |
| DEFER-ATOM-004 | candidate | 须人 `accepted` 才不算战役漏洞；Task 已存在 |
| DEFER-ANCHOR-IMPL | candidate | 合同接口必须；真实 WORM 可后置 |
| DEFER-STABLE | candidate | 人审后 |
| `tools/evidence/AGENTS.md` 仍含「不可篡改」 | residual | DEF-010 仍 OPEN；README 已修；不伪 CLOSED |
| plan 页眉 campaign 文案仍偏 pass1 | stale narrative | 不否决清单；建议 W0 改「pass3 fail_rounds=0」 |
| approval 签字表头写 A1–A10 | cosmetic | 正文已 A1–A13 |
| T-ARCH-004/006 合桶 | residual risk | 执行期防假 DONE；ID 层已覆盖 |

---

## §33 Task ID integrity

### 逐节映射（pass3 实读）

#### 33.1 规格闭合 — PASS

| Spec 项 | Task ID | 任务表存在 |
|---------|---------|------------|
| SPEC Approved | T-HUM-001 | Y |
| 旧 spec superseded | T-DOC-002 T-HUM-002 | Y（T-DOC-002 DONE；正式废止仍 T-HUM-002） |
| ADR 冲突修订 | T-DOC-004 T-DOC-005 | Y |
| 路径 package 对齐 | T-CUT-002 T-CUT-003 T-LEG-003 | Y |
| architecture registry | T-REG-001 T-REG-002 | Y |
| evidence-policy.toml | T-POL-001 T-POL-002 | Y |
| 无未登记 Unknown | T-RES-001 T-SKEP-001 | Y |

#### 33.2 Core 闭合 — PASS

全部映射至 `T-CORE-*` / `T-MEM-002` / `T-DOM-001` / `T-ARCH-003/004` / `T-CUT-004` 等真实 ID；范围记号 `T-CORE-014..027`、`T-CORE-005/006` 端点均在任务表。

#### 33.3 Adapter 闭合 — PASS

memory/file/pg/conformance/idempotency/crash/fsync/禁 volatile 均挂 `T-MEM-*` / `T-FILE-*` / `T-PG-*` / `T-ARCH-005`。

#### 33.4 Checkpoint 闭合 — PASS（pass2 主修点）

| Spec 项 | Task | 备注 |
|---------|------|------|
| signed checkpoint | T-CP-002 | |
| key rotation | T-CP-006 | |
| independent anchor | T-CP-005 | 合同接口；实现 DEFER-ANCHOR-IMPL |
| tail truncation | T-CP-004 | |
| full chain replacement | **T-CP-007** | 非 T-CP-005+散文 |
| startup verify | **T-CP-008** | 非 FILE/CP-004 拼凑 |

#### 33.5 测试闭合 — PASS

| Spec 项 | Task |
|---------|------|
| golden | T-CORE-026/027 T-CLI-004 |
| property | T-CORE-028/030 |
| fuzz | T-FUZZ-001 |
| line≥95% | T-CORE-033 T-CI-003 |
| branch≥90% | T-CI-003 T-CI-NIGHTLY-001（T-CI-003 AC 显式 line+branch 双阈值） |
| mutants≥90% | T-MUT-001 + I-7 |
| Miri | T-MIRI-001 |
| adapter chaos | T-FILE-008 T-PG-006 |
| historical schema | T-LEG-002 T-SCH-002 T-CI-NIGHTLY-001 |

#### 33.6 系统闭合 — PASS

| Spec 项 | Task |
|---------|------|
| required ops 登记 | T-POL-002 |
| fail-closed | T-DOM-005 |
| Tier-A 原子性 | T-ATOM-001 T-ATOM-002 T-ARCH-016 |
| external Attempted+terminal | T-ATOM-004（可 DEFER accepted；residual 登记） |
| source artifacts retention | T-PRIV-001 T-PRIV-002 |
| verifier/schema/keys 保留 | T-PRIV-002 T-CP-006 T-SCH-002 |
| CI Evidence 可追溯 | T-EVID-SYS T-CI-001 |

**Integrity verdict:** §33 仅真实 Task ID · 无幽灵 · 无孤儿勾选 · **PASS**

---

## fail_rounds

| Pass | fail_rounds | 说明 |
|------|-------------|------|
| pass1 (v1.0) | **10** | R1–R10 全 FAIL |
| pass2 (v1.1) | **5** | R6b–R10b FAIL（均仅 R-SPEC-003）；R1b–R5b PASS |
| **pass3 (hygiene)** | **0** | R1–R10 全 PASS · checklist 40/40 PASS |

```text
plan_completeness_10x: PASS
fail_rounds = 0
failed_checks = []
TOP_fixes = []   # no remaining strict omissions for plan completeness
```

### 非阻断建议（可选 hygiene，不进 fail_rounds）

1. `plan.md` 变更日志补 v1.1.1 行 + campaign status 改写为「计划 10x pass3 fail_rounds=0」。  
2. `tools/evidence/AGENTS.md` 去掉「不可篡改」措辞（DEF-010 仍 OPEN 直至全树一致）。  
3. `tasks.md` 统计表 W0 DONE 计数与 approval 表头 A1–A13 对齐。  
4. 人审将 DEFER-\* 标 `accepted` 或排进实现波次。  
5. `.worktrees/evidence-todo.md` §4.2 填入 pass2/pass3 结果（本 verdict 为 SSOT）。

---

## Allowed claims / Forbidden claims

### Allowed claims（本 verdict 授权）

```text
✓ 计划完备性十轮（pass3）fail_rounds = 0
✓ 40 项 R-* 清单在 plan/tasks/gap/inventory/residual/approval/todo 层可追踪
✓ §33.1–33.6 映射仅使用真实 Task ID（T-CP-007/008、T-DOC-005 已就位）
✓ 可进入 feature 分支实现 W0 剩余 / W1 scaffold（仍须非 main 纪律）
✓ Spec 仍为 Proposed；approval-packet 可提交人审（不自动 Approved）
```

### Forbidden claims（本 verdict **明确禁止**）

```text
✗ Spec Status = Approved / Target Stable 已达成
✗ §33.1–33.6 已闭合 / 可勾完成定义
✗ registry quality = stable
✗ Core V1 / adapters / CLI / 门禁「已实现」或「production-ready」
✓ 不可篡改 / 绝对可信 / 永久证明（无签名检查点+独立锚点时）
✗ 实现十轮（W7）fail_rounds=0（W7 NOT STARTED）
✗ 旧六字段链与 V1 字节连续 / 静默 rehash 合法
✗ SKIP / 手写 PASS / AI 独断关闭人审闸门
✗ 双 package 同名 evidence 无隔离可合入
✗ 把本「计划 10x PASS」等同「战役完成」或「0.1.1 可发」
```

### 战役状态（诚实一行）

```text
PLANNING · plan completeness 10x PASS (pass3) · Spec Proposed ·
implementation ABSENT (crates/evidence 未落地) · §33 OPEN · ≠ stable
```

---

## Sign-off

| Role | Action | Status |
|------|--------|--------|
| Verifier (pass3) | 独立复检 40 R-\* + §33 ID integrity + ghost scan | **DONE · fail_rounds=0** |
| Skeptic | 建议对照本文件反证假 PASS（尤其禁止把计划 PASS 当实现 PASS） | 待执行（可选） |
| Human Owner | A1 Spec Approved 等 | **未签** |

**文件 SSOT：** `.agents/ssot/tools/evidence/plan/evidence-plan-10x-verdict.md`  
**T-V10-PLAN AC：** fail_rounds=0 且本 verdict 落盘 → 满足计划完备性退出条件（**仅计划层**）。
