# Round 07 Findings — Migration Skeptic

| 字段 | 值 |
|------|-----|
| Round | **7** |
| Role | Verifier / Migration Skeptic |
| Scope | SPEC-EVIDENCE-002 §31 P0–P6 vs plan Waves W0–W9；legacy 非静默重编码；双包共存；cutover 删 `tools/evidence` 序 vs callers；ADR 冲突；路径真实性 |
| Baseline docs | `plan.md` · `tasks.md` · `gap-matrix.md` · `approval-packet.md` · `xhyper-evidence-complete-spec.md` §31 |
| Repo facts | `main` layout · callers · ADR-010 / ADR-012 |
| **result** | **FAIL** |

---

## failed_checks

### R-MIG-001 — P0–P6 与 Wave 对齐（FAIL / 不完整）

规范 §31.2 阶段 vs 计划波次 **无显式对照表**（plan 仅有 W0–W9 与 §6.2 P0 冻结；gap-matrix §6 用 P0–P6 叙述但未挂 Task）。隐式对照如下：

| Spec §31 | 含义 | Plan Wave / Tasks | 判定 |
|----------|------|-------------------|------|
| **P0** 冻结错误扩散 | 禁 hash_bytes / Debug-hash / 生产 InMemory；quality incubating；措辞 | **W0**：`T-FREEZE-001` `T-DOC-001` `T-POL-001` | **对齐**（任务仍 TODO，但波次正确） |
| **P1** Core V1 | `crates/evidence` + V1 + golden | **W1**：`T-CORE-*` | **对齐** |
| **P2** Compatibility bridge | Legacy read + migration manifest；不伪造 V1 连续 | **无独立 Wave**；`T-LEG-001/002` 塞在 **W3**（依赖 `T-CORE-025`） | **部分对齐**：语义有任务，阶段名/序被折叠进 Domain 波次，易被跳过或与 P3 捆死 |
| **P3** Domain migration | domain_macro typed outcome / EventId / policy | **W3**：`T-DOM-*` + gate | **对齐**（gate 属合理扩展） |
| **P4** Durable adapters | memory conformance + file + postgres + crash + durability | **W2** memory + **W4** file/pg | **拆分合理**；但 R-MIG 仍缺「P4 = W2∪W4」书面映射 |
| **P5** Checkpoint | CP + signer + rotation + anchor + tail | **W5**：`T-CP-*`（另附 CLI/obs/perf） | **对齐+扩展**（CLI 不在 §31.2 P5 字面，可接受） |
| **P6** Cutover | **bootstrap 强制 production adapter**；删 `tools/evidence`；删旧 API/mock/hash_bytes；更新架构法；registry stable | **W6**：`T-CUT-*` `T-ARCH-*` `T-REG-001` | **缺口**：见下「P6 bootstrap」 |

**P6 遗漏任务（规范字面第一项）**：

```text
§31.2 P6: bootstrap 强制 production adapter
```

`tasks.md` W6 **无** bootstrap 装配/强制 production adapter 的 Task ID。  
`T-ARCH-005`（release 图无 memory）≠ bootstrap 强制 durable adapter。

W7–W9 是计划扩展（十轮 / 人审 / §33），规范 §31 无对应阶段——**可接受**，但应在映射表中标 `plan-only`。

---

### R-MIG-002 — Legacy non-silent-reencode（PASS 文案 / 执行未落地）

| 位置 | 内容 |
|------|------|
| Spec §31.2 P2 | 不得把旧链静默重编码后声称历史连续 |
| plan Forbidden | 「旧链静默重编码为 V1」 |
| plan §6.1 | 新 genesis + migration record + 旧链只读 verifier |
| tasks | `T-LEG-001` / `T-LEG-002` **TODO** |
| gap-matrix §6 | 明确禁止静默 rehash |

**文案与禁令一致，PASS（纪律层）**。  
**实现层 ABSENT**——不得因文案存在而把 legacy 迁移标 DONE。

---

### R-MIG-003 — 双包共存风险（FAIL）

plan §5：

> 迁移期允许 **双包短暂共存**（legacy feature 或 `evidence_legacy`）

**仓库事实**：

| 项 | 现状 |
|----|------|
| 旧包路径 | `tools/evidence` |
| 旧 `package.name` | **`evidence`**（`tools/evidence/Cargo.toml`） |
| 目标 | `crates/evidence`，`name=evidence`（plan §5） |
| Workspace members | 根 `Cargo.toml` 仅 `tools/evidence` |
| 调用方 | `domain_macro`、`gate` → `path = ".../tools/evidence"` |

**致命缺口**：Cargo workspace **禁止两个 package 同名 `evidence`**。  
计划提到 `evidence_legacy` 重命名，但：

1. **无 Task**：rename 旧包 / feature-gate / path 别名 / 依赖图切换步骤缺失；
2. **无 AC**：双包窗口内 `cargo metadata` / `lint-deps` 必须绿；
3. **无截止**：共存最长 Wave / PR 边界未定义（仅风险表「双包共存过久」一笔）；
4. **API 双轨**：旧 `EvidenceSink`/`hash_bytes`/`InMemoryEvidenceSink` vs 新 `EvidenceAppender`/`digest_canonical`——下游如何在窗口内编译无逐步迁移矩阵。

**false_pass 风险**：一旦有人同时加 `crates/evidence` 为 member 且不改旧包名 → **整仓构建直接炸**，与「短暂共存」叙述矛盾。

---

### R-MIG-004 — Cutover 删除序 vs callers（部分 PASS / 残留风险）

**Callers（已核实）**：

| Crate | 依赖 | 用法 |
|-------|------|------|
| `crates/domain/macro` | `evidence = { path = "../../../tools/evidence" }` | `hash_bytes(format!("{point:?}"))` · `EvidenceSink` · 测试 `InMemoryEvidenceSink` |
| `crates/infra/gate` | 同上 | `EvidenceSink` · `hash_bytes(name)` · 测试 InMemory |

**任务序（正确骨架）**：

```text
T-DOM-001 / T-GATE-001
  → T-CUT-001 调用方全部迁离 tools/evidence
    → T-CUT-002 删除 tools/evidence
      → T-CUT-003/004 文档 + 清旧 API
```

**问题**：

1. **`T-CUT-001` 依赖过窄**：仅 `T-DOM-001` + `T-GATE-001`，未依赖 `T-DOM-006` / `T-GATE-002` / 测试与 fuzz 全绿。可能「编译迁走但仍 Debug-hash」即删旧包。
2. **`T-ARCH-001`（EVIDENCE-PATH-001）依赖 `T-CUT-002`**：路径门禁在**删除之后**才启用 → 双包窗口 **无** PATH 机控，与 P0「冻结 tools 扩 runtime」不对齐。
3. **其他引用面未任务化**：docs、ADR-012 叙述、CI、`docs/architecture/spec.md` 路径、历史 `evidence/` 目录描述——`T-CUT-003` 只点 architecture/spec，范围偏窄。
4. **P6 bootstrap 强制 production** 无任务（上表）。

删除序方向正确，但 **入口条件与门禁时序不足 → 记 FAIL（有条件）**。

---

### R-GOV-001 / ADR 冲突（FAIL — 覆盖不全）

| 权威 | 与 SPEC-002 冲突点 | 计划覆盖 |
|------|-------------------|----------|
| **ADR-010**（Accepted 回填） | 六字段公开 `EvidenceRecord`；`hash_bytes`；core **`mock` feature** + `MockEvidenceSink`；Article IX 解释 | `T-DOC-004` + approval **A2/A5** — **有** |
| **ADR-012**（Accepted） | runtime → **`crates/infra/auditx`**；CI 报告**留** `tools/evidence`；与 002 的 **`crates/evidence` + 删除 tools/evidence`** 正面对撞 | **完全未出现**于 plan / tasks / approval |
| Constitution Article IX | trait 须 mock feature | A5 要求 ADR 解释 — 有，但依赖 ADR-010 修订包是否同时处理 012 |
| architecture `spec.md` R 路径 | 适配器仅 `adapters/storage/*` 与 `adapters/exchange/*`；evidence 仍写 tools 时代路径 | `T-DOC-003`/`T-CUT-003` 部分；**未**裁定 `adapters/evidence/*` 新顶栏 |

approval A3 批准 `crates/adapters/evidence/*`，但 **未点名 ADR-012 auditx 废止/修订**。  
人审包缺 **A-ADR-012** 类闸门 → 执行期会「按 002 建 crates/evidence」同时「ADR-012 仍要求 auditx」，双事实源。

---

### R-PATH — `crates/adapters/evidence` vs 当前树（FAIL / 结构漂移）

| 计划/规范目标 | 仓库当前事实 |
|---------------|--------------|
| `crates/evidence` | **不存在** |
| `crates/adapters/evidence/{memory,file,postgres}` | **不存在**；`crates/adapters/` 仅有 `exchange/` + `storage/` |
| `tools/evidence-cli` | **不存在** |
| `tools/evidence` | **存在**（runtime 六字段链） |
| `crates/adapters/storage/postgres` | **存在**（`postgresx`，业务存储适配器） |

风险：

1. 新树 `adapters/evidence/postgres` 与既有 `adapters/storage/postgres` **并列**，命名/分层易混（审计持久化 ≠ 业务 Postgres 适配器）。
2. architecture SSOT 与 lint-deps 规则是否承认 **第三类** `adapters/evidence` **未**写入 tasks 的 arch 更新子项（仅 A3 人审意向）。
3. 若有人按历史记忆写 `crates/adapters/storage/evidence` 或 `crates/infra/auditx`，与 002 路径分叉 — 计划未列「禁止 auditx 路径」负向检查。

---

### R-GAP residual-open（FAIL）

| 引用 | 状态 |
|------|------|
| `T-RES-001` residual-open 初始化 | **TODO** |
| 文件 `.agents/ssot/tools/evidence/plan/residual-open.md`（或等价） | **不存在** |
| DEF-001…018 | 仅在 gap-matrix + `.worktree/evidence-todo.md`（**gitignore**） |

R-GAP-001 要求 residual/todo 有 ID：todo 有 DEF，但 **residual-open 契约文件缺失**。  
任何「DEF 已进 residual」的口头完成 = **false completeness**（且 worktree 不可作 clone 证据）。

---

## omissions

1. **无 P0–P6 ↔ W0–W9 正式映射表**（R-MIG-001 机器/人审对照点）。
2. **无双包 rename Task**（`evidence` → `evidence_legacy` 或 path package 策略）。
3. **无 P6 bootstrap 强制 production adapter Task**。
4. **无 ADR-012 修订/废止 Task 或 approval 项**。
5. **无 adapters/evidence 目录与 R2/架构图同步的独立 Task**（超出 `T-CUT-003` 一行）。
6. **无 residual-open 落盘物**（`T-RES-001` 未做）。
7. **P2 无独立退出条件**（bridge 可测、manifest 格式、只读 verifier 门禁）——仅挂在 W3 列表。
8. **Cutover 前 callers 全量扫描 Task**（`rg EvidenceSink|hash_bytes|tools/evidence` 作为 `T-CUT-001` AC）未写死命令。
9. **quality=incubating 变更**（P0）无明确改 `Cargo.toml`/registry 字段的 Task（`T-FREEZE-001` 文案含糊：「residual + AGENTS/CLAUDE」）。

---

## false_pass_risks

| 风险 | 为何危险 |
|------|----------|
| 把 plan §6.1 文案当 P2 DONE | 代码仍为六字段；`T-LEG-*` TODO |
| 把 W3 开始当 P2+P3 完成 | LEG 可被 DOM 掩盖未做 |
| 同时 member `crates/evidence` + `tools/evidence` 同名 | 构建红；「共存」叙事破产 |
| `T-CUT-001` 仅 DOM-001/GATE-001 | 删包后 fuzz/测试/文档仍引用旧 API |
| 忽略 ADR-012 | 合并后治理回滚或第二套 auditx 分叉 |
| residual 只写在 `.worktree/` | fresh clone 看起来「无 OPEN DEF」 |
| `adapters/evidence` 未建却勾 PATH 对齐 | 路径 SSOT 假绿 |

---

## notes

### 映射建议（供 Planner 补丁，本轮不改 plan）

```text
P0 → W0
P1 → W1
P2 → W2.5 或 W1 退出后并行 LEG（勿绑死 T-DOM）
P3 → W3
P4 → W2(memory conformance) + W4(file/pg)
P5 → W5 (CP 子集；CLI 标 plan-extra)
P6 → W6 + 新增 T-BOOT-001 + rename 旧包 Task + ADR-012 修订
plan-only → W7 十轮 / W8 人审 / W9 §33
```

### 非静默重编码（正面记录）

Forbidden + §6.1 + gap-matrix 禁止语 **一致且正确**；本轮 **不** 在该子项上找假绿，只强调 **未实现**。

### 诚实状态（本轮）

- Campaign：**PLANNING**；Spec：**Proposed**；**≠ Approved / ≠ stable**（plan 页眉诚实，保留）。
- 实现：仍 `tools/evidence` 0.1.0 六字段 + `hash_bytes` + 文档「不可篡改」措辞未改。
- Callers：**2** 个 workspace 依赖未迁。

### 证据指针（绝对路径）

- Spec §31：`/home/workspace/infra.rs/.agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md`（# 31）
- Plan waves：`.../plan/plan.md` §2、§5–§6
- Tasks cutover：`.../plan/tasks.md` W3/W6
- Callers：`/home/workspace/infra.rs/crates/domain/macro/Cargo.toml`、`.../crates/infra/gate/Cargo.toml`
- ADR-010：`/home/workspace/infra.rs/docs/architecture/adr/010-l0-implementation-reconciliation.md`
- ADR-012：`/home/workspace/infra.rs/docs/architecture/adr/012-control-plane-migration.md`
- Adapters 树：`/home/workspace/infra.rs/crates/adapters/{exchange,storage}/`（无 `evidence/`）

---

## verdict_summary

```text
round: 7
result: FAIL
failed_checks:
  - R-MIG-001 (P0-P6↔Wave 无显式表；P6 bootstrap 无 Task；P2 折叠进 W3)
  - R-MIG-003 (双包同名 evidence 共存方案无 Task)
  - R-MIG-004 (cutover 依赖过窄；PATH 门禁晚于删除；bootstrap 缺)
  - R-GOV-001 (ADR-012 auditx 冲突未入计划)
  - R-PATH (crates/adapters/evidence 与 storage 布局/架构法未对齐)
  - R-GAP residual-open 文件缺失
passed_subchecks:
  - legacy non-silent-reencode 文案/Forbidden 一致
  - cutover 依赖方向 callers→delete 骨架正确
  - 未宣称 migration 已完成 / stable
omissions: [见上节]
false_pass_risks: [见上节]
```
