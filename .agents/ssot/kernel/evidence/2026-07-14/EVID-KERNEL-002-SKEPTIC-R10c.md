> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002-SKEPTIC-R10c — residual / 台账诚实性审计

| 字段 | 值 |
|------|-----|
| Agent | Skeptic (read-only) |
| Date | 2026-07-14 |
| Team | kernel-todo-r10 |
| Residual SSOT | `residual-open.txt` |
| Scans | T0 立即扫描 + T1 Doc-Sync 后复扫 |
| Production code | **未修改** |

## 0. Verdict（一句话）

**源码侧 L1 CLOSED 主张大体真实**（ERR-010 / CLK-010 / LC-005 可复验）；**台账未收敛**：`tasks` / `plan` / `gap-matrix-v2` / `approval-packet` / `design` / `test` / `release` 仍把已关 residual 写成 OPEN/GAP → **`RES-DOC-001` 的 CLOSED 为假 PASS**；**无 AI 自批 `Status: Approved` / 无假 §18 PASS / registry 仍 incubating**。

---

## 1. Drift table（按严重度）

| Sev | Doc | Claim | Truth | Note |
|-----|-----|-------|-------|------|
| **P0** | `tasks/tasks.md` | T-API-001…T-TEST-005 / T-VER = `pending`；OPEN 表仍含 ERR-010/CLK-010/LC-005/TEST-005/GATE-009 | residual + 源码：上述已 **CLOSED**（或 DEFER accepted）；kernel-todo 已勾 done | 任务台账与 SSOT **反向**；会触发假重做 / 假缺口 |
| **P0** | `plan/approval-packet.md` §2.2 | 人审须知 OPEN：ERR-010 / GATE-009 / LC-005 / TEST-005 / CLK-010 等 | residual：ERR-010/CLK-010/LC-005/TEST-005/GATE-009 已关；OPEN 仅 007/014–016/DOWN-006/PERF/EVID/18 | **人审包过时** → 误导 Approved 决策选项 A（“先收口 ERR-010”） |
| **P0** | `plan/gap-matrix-v2.md` | GAP：context_cow / const fn 缺 / LC-005 测缺 / GATE-009 未实现；P1 队列仍含 ERR-010 | 源码：`context_cow` 全树 0；`pub const fn from_clock_elapsed`；poison/1000/drop/!Clone 测试在；GATE-009 为 **DEFER accepted** 非已实现 | 差距矩阵仍是战役**前**快照 |
| **P1** | `plan/plan.md` §0.2–0.3 | §8 `context_cow` 超面；B/C 表仍把 ERR-010/CLK-010/LC-005/TEST-005/GATE-009 当 OPEN；18.3 trybuild OPEN | residual 与 R10b-verdict 已关/DEFER；Campaign status 行已写 L1 PASS 但章节矩阵未跟 | 计划体 **自相矛盾** |
| **P1** | `residual-open.txt` RES-DOC-001 | **CLOSED** — gate/tasks/matrix/review/goal 已对齐 | gate/matrix/review/goal **已**对齐；**tasks 未对齐**；plan/gap/approval/design/test/release 仍漂 | **假 CLOSED / 应 REOPEN** |
| **P1** | `design/design.md` | OPEN 含 RES-TEST-005；缺 PERF-001 / EVID-001 / 18-APPROVED | residual：TEST-005 CLOSED(DEFER)；OPEN 为 007/014–016/DOWN/PERF/EVID/18 | 剩余清单不完整且含假 OPEN |
| **P1** | `test/test.md` | OPEN：trybuild（RES-TEST-005） | residual：TEST-005 **CLOSED (DEFER accepted)**；`api_compile.rs` 模块文档已写 DEFER 理由 | 测试管线层假 OPEN |
| **P2** | `release/release.md` | BLOCKED 条件点名 RES-LC-004、RES-API-004、RES-GATE-\* 须全 CLOSED | LC-004/API-004/GATE-001…008 早已 CLOSED；GATE-009 DEFER；Review ID 仍 `exec-v2` 过时 | 关闭条件引用**过期 residual 集合** |
| **P2** | `plan/plan.md` / residual §3.3 | RES-API-004 CLOSED 含 trybuild 为 dev-dep；§3.3 PASS | `crates/kernel/Cargo.toml`：**无 trybuild**；仅 proptest + static_assertions；loom 在 `cfg(loom)` target dep | 与 TEST-005 DEFER 部分对冲，但 API-004 表述 **膨胀** |
| **P2** | Historical evidence（R10 / G2 / mid / CI-PR-fix） | 各文件内嵌当时 OPEN 集合 | 非 live SSOT；与 residual 冲突属预期 | 须标注 “historical / superseded by residual-open + R10b” |
| **P3** | `goal/goal.md` | AC-5 仍写 LC-005 PASS 在 partial 句内；M4 已写 ERR-010 已关 | 大体诚实；AC-5 勾选仍空正确 | 低风险措辞 |
| **OK** | `gate/gate.md` · `matrix/matrix.md` · `review/review.md` · `goal` M4–M6 · `kernel-todo.md` · `R10b-verdict` · `residual-open` OPEN 集 | L1 CLOSED + OPEN 仅 7 项 | 与源码/residual 一致（gate 明确 DEFER ≠ 机控完成） | Doc-Sync **部分完成** |

---

## 2. CLOSED 主张 × 源码交叉（强制项）

| ID | residual 主张 | 源码/树证据 | Skeptic |
|----|---------------|------------|---------|
| **RES-ERR-010** | CLOSED — 删 `context_cow`；redisx→`context()` | `rg context_cow` 全树 **0**；`.architecture/api/kernel-public-api.txt` **无** `context_cow` | **真 CLOSED** |
| **RES-CLK-010** | CLOSED — `pub const fn from_clock_elapsed` | `crates/kernel/src/clock.rs:63`：`pub const fn from_clock_elapsed(elapsed: Duration)`（`#[doc(hidden)]`，故 public-api 快照可不列） | **真 CLOSED** |
| **RES-LC-005** | CLOSED — poison / 1000 / guard drop / !Clone·!Default + 生产 into_inner | `lifecycle.rs`：`poison_recovery_into_inner`、`guard_drop_does_not_trigger`；`wait`/`trigger`/`is_triggered` 均 `into_inner`；`tests/lifecycle_concurrency.rs::concurrent_regression_1000_cycles`；`api_compile.rs` `assert_not_impl_any!(ShutdownGuard: Clone)` / `MonotonicInstant: Default` | **真 CLOSED** |
| **RES-GATE-009** | CLOSED (**DEFER accepted**) — 非机控完成；临时 API-001+人审清单 | 无 KERNEL-API-002 机器实现证据；`gate.md` 明文禁止把 DEFER 计为 §12 全量机控；**未**伪称 Implemented | **DEFER 诚实**（非假 implemented） |
| **RES-TEST-005** | CLOSED (DEFER) — static_assertions 替代 | `api_compile.rs` 模块文档明确 trybuild DEFER + static 覆盖核心禁止面；**无** trybuild dep | **DEFER 诚实**（design/test 未同步） |

---

## 3. Residual 完备性

### 3.1 residual-open 中 STILL OPEN（是否应 OPEN）

| ID | residual 理由 | 应 OPEN？ | 分类 |
|----|---------------|-----------|------|
| **RES-18-APPROVED** | Spec `Status: Proposed` | **是** | 人审 / 政策 |
| **RES-API-007** | version 仍 `0.1.0`（`Cargo.toml` 确认） | **是** | 人 / release 策略 |
| **RES-TEST-014** | branch≥90% 未在 stable 测得；line 98.82% PASS | **是** | 环境（需 nightly） |
| **RES-TEST-015** | cargo-mutants ABSENT | **是** | 环境 / 工具 |
| **RES-TEST-016** | miri 组件 ABSENT | **是** | 环境 / 工具 |
| **RES-DOWN-006** | 树外 sleep 计时非正确性 | **是** | 策略 / 标注债 |
| **RES-PERF-001** | Cow::Borrowed 未做 | **是** | 可选 / 人决策 |
| **RES-EVID-001** | §17 全树 partial | **是** | 证据完备性 |

### 3.2 文档提及但 residual 缺失？

- **无** 新增 live residual ID 出现在 gate/matrix/review/goal 却不在 residual-open。
- **编号空洞**：`RES-ERR-008` 从未登记（001–007,009,010）— 非遗漏 OPEN，为 mid 编号跳号。
- **历史 mid 文件**（`EVID-KERNEL-002-R-test-gate-mid.md` 等）使用 **旧义** RES-TEST/GATE 编号，与 residual mid 冻结后的语义不同；不得当 live 台账。

### 3.3 residual-open vs kernel-todo

| 方向 | 结果 |
|------|------|
| residual OPEN → kernel-todo | **齐全**（C/D 节 7 项一致） |
| residual 本战役 CLOSED → kernel-todo B 节 | **齐全**（ERR-010/CLK-010/LC-005/TEST-005/GATE-009 已勾） |
| kernel-todo vs tasks.md | **tasks 落后**（仍 pending/OPEN） |

### 3.4 不得 CLOSED 却保持 OPEN 的工具 residual

- TEST-014/015/016 证据文件均写 **OPEN / DEFER · 不得 CLOSED** — 与 residual 一致；**无** 假 CLOSED。

---

## 4. Forbidden phrase hits

| Phrase | Hit? | 位置 / 判定 |
|--------|------|-------------|
| `§18 PASS`（假全闭合） | **否** 作为宣称 | gate 仅在禁止条出现；无 “Test Gate §18 PASS” |
| `registry stable` 作为当前状态 | **否** | `.architecture/workspace.toml` kernel=`incubating`；各 live 文档禁止 stable |
| AI 写入 `Status: Approved` | **否** | `spec.md` 仍 `Status: Proposed` |
| `3/3` 作为完成 | **否** | 仅禁止/否定语境 |
| `L1 PASS` | **是（诚实）** | gate/review/R10b/plan header — 指战役层，非 §18 |

---

## 5. Line coverage 98.82%

| 项 | 状态 |
|----|------|
| Claim | residual + R10b-verdict + gate OPEN 表：line **98.82%** ≥95% PASS |
| Evidence file | `EVID-KERNEL-002-TEST-014-branch.md`（line PASS / branch OPEN） |
| Raw log | `/tmp/kernel-line-cov.txt` **存在**；TOTAL Lines Cover **98.82%**（423 lines, 5 missed）；含 LC-005 测试运行 |
| Branch | **未测**（stable 无 `-Z coverage-options=branch`）→ RES-TEST-014 保持 OPEN 正确 |
| Re-run needed? | 证据新鲜（含 poison/1000 测试）；**非必须**仅为审计重跑；正式归档可 `cp` 进 evidence 树（属 RES-EVID-001） |

---

## 6. False PASS 清单（核心）

1. **RES-DOC-001 CLOSED** — 在 tasks/plan/gap/approval/design/test/release 仍漂移时标 CLOSED → **假 PASS**（建议 REOPEN 或限定 closed-by 范围并补齐）。
2. **approval-packet 仍列已关 P1/P2 为 OPEN** — 对人审的 **假缺口**（反向 false FAIL，但决策风险同级）。
3. **RES-API-004 CLOSED 文案含 trybuild** — 轻微 **假 PASS 表述**；实质由 RES-TEST-005 DEFER 覆盖 trybuild UI。
4. **RES-GATE-009 / RES-TEST-005 CLOSED(DEFER)** — **非**假 implemented（gate/api_compile 诚实）；勿升格为 “§12/§11.4 全量机控 PASS”。
5. **代码 CLOSED（ERR-010/CLK-010/LC-005）** — **非** false PASS。

---

## 7. Recommended fix list for Doc-Sync（按序）

1. **REOPEN 或收紧 RES-DOC-001**：直到 `tasks.md` + `approval-packet.md` + `gap-matrix-v2.md` + `plan.md` §0.3 与 residual 一致。
2. **`tasks/tasks.md`**：T-API-001/002、T-GATE-001、T-TEST-001…005 → `done`（注明 DEFER where applicable）；T-VER-001…010 → `done`（R10b）；OPEN 表仅留 007/014–016/DOWN-006/PERF-001/EVID-001/18-APPROVED。
3. **`plan/approval-packet.md`**：§2.2 只列真实 OPEN；选项 A 删除 “先收口 ERR-010”；§4 去掉已关的 LC-005/trybuild-as-OPEN（trybuild 写 DEFER accepted）。
4. **`plan/gap-matrix-v2.md`**：ERR-010/CLK-010/LC-005 → PASS/CLOSED；GATE-009 → DEFER accepted；TEST-005 → DEFER；刷新优先级队列。
5. **`plan/plan.md`**：§0.2 去掉 context_cow 星号；§0.3 将已关 ID 移入 A；B/C 只留真实 OPEN；18.3 trybuild → DEFER accepted。
6. **`design/design.md` / `test/test.md`**：TEST-005 移出 OPEN；补全 OPEN 七项。
7. **`release/release.md`**：BLOCKED 条件改为 residual 当前 OPEN + Spec Approved；更新 Review ID。
8. **历史 evidence**：在 README 或各 mid 文件头加 `HISTORICAL — do not use as live residual`。
9. **RES-API-004 文案**：改为 “loom(cfg)+proptest+static present；trybuild 见 RES-TEST-005 DEFER”，避免暗示 trybuild dep。
10. **可选**：把 `/tmp/kernel-line-cov.txt` 摘要拷入 `evidence/2026-07-14/` 以收 RES-EVID-001 一部分。

---

## 8. OPEN residual 正确性总结

全部 8 个 residual-open OPEN 项（含 RES-18-APPROVED）在源码/环境/政策维度 **均应保持 OPEN**；无发现应 CLOSED 却 OPEN 的代码项；发现 **应 OPEN 却标 CLOSED 的文档项（RES-DOC-001）**。

---

## 9. Scan protocol

| Scan | Observation |
|------|-------------|
| T0 | residual/R10b/kernel-todo 已 L1 CLOSED；gate/matrix/review 当时仍 OPEN → 严重漂移 |
| T1 | Doc-Sync 已修 gate/matrix/review（+goal 部分）；**tasks/plan/gap/approval/design/test/release 仍漂** |

---

## 10. Artifacts touched by Skeptic

- 本文件
- `.omc/handoffs/skeptic.md`

**未**修改生产代码或 residual-open 状态行。
