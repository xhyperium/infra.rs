# Round 09 Findings — §34 Production Audit Path (E2E)

| 字段 | 值 |
|------|-----|
| Round | **9** |
| Role | Final Verifier · End-to-end production audit path |
| Source | `xhyper-evidence-complete-spec.md` **# 34. 最终裁定**（及 §1.1 生产可信度组合） |
| Plan pack | `plan.md` · `gap-matrix.md` · `tasks.md` · `approval-packet.md` · `.worktrees/evidence-todo.md` |
| Prior rounds | R3–R8 已 FAIL（不重复展开，本轮只走 §34 路径） |
| Date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — 计划在 **Core → append → durable → checkpoint** 中段有任务骨架，但 §34 终态要求的 **生产审计路径并未在每个 bullet 上可执行闭合**：

1. 「业务成功与 durable evidence 之间没有未声明窗口」被 **幽灵 `T-ATOM`** + **external Attempted/terminal 无 Task** 打断；
2. 「历史 schema、验证器、公钥在保留期内仍可用」被缩成 **`T-POL-002 + docs` / `T-CP-006 + docs`**，无 §22 retention 合同；
3. 生产可信组合中的 **retained source artifacts** 无 artifact store 任务；
4. **bootstrap 强制 production Durable adapter** 缺位 → 路径末段无法声称「生产审计可用」。

**不得**因 plan 写出了 end-state 文本串就判 PASS。§34 路径闭合 = 每个箭头都有 **真实 Task ID + 可机验 AC**（或正式 `DEFER(accepted)`）。

---

## failed_checks

### 0. 对照物：§34 终态与「关键操作可审计」条件

#### A. 生产终态路径（规范字面）

```text
typed evidence draft
→ canonical V1
→ domain-separated digest
→ contiguous chain
→ idempotent linearizable append
→ durable persistence
→ crash recovery
→ signed checkpoint
→ independent anchor
→ reproducible verification
```

§1.1 同族组合另含：**retained source artifacts**（§34 条件第 7 条与之交叉）。

#### B. 明确否定的终态

```text
Vec<Record> + SHA256(prev || fields)   ← 禁止作为生产终态
```

#### C. 「关键操作可审计」七条件

```text
1. 操作身份明确
2. actor 和 subject 明确
3. 输入与结果使用稳定 canonical digest
4. 业务成功与 durable evidence 之间没有未声明窗口
5. 链没有 gap、fork 或 duplicate
6. 可信 checkpoint 能检测尾部截断和整链替换
7. 历史 schema、验证器和公钥在保留期内仍可用
```

---

### 1. E2E 路径逐步裁定（A）

| # | 路径步骤 | 计划关闭证据 | 裁定 |
|---|----------|--------------|------|
| 1 | typed evidence draft | `T-CORE-010` EvidenceDraft+builder；`R-API-002` | **PASS**（任务存在；实现 ABSENT） |
| 2 | canonical V1 | `T-CORE-014..016` encode/digest；`R-CANON-001`；golden `T-CORE-026` | **PASS***（AC 粒度偏章节引用，见 R5；路径上有 Task） |
| 3 | domain-separated digest | `T-CORE-018` digest_canonical；禁 `hash_bytes`；`T-DOM-001` | **PASS** |
| 4 | contiguous chain | `T-CORE-019` ChainHead；sequence 从 1；mem/file CAS | **PASS** |
| 5 | idempotent linearizable append | `T-MEM-002` 幂等+CAS；`T-MEM-005` 并发；`T-PG-005` | **WEAK→FAIL\***：同 event_id **不同内容 → IdempotencyConflict** 无显式 Task/AC（R5）；「linearizable」未写进 AC |
| 6 | durable persistence | `T-FILE-004` fsync；`T-PG-003` 事务；`R-APPEND-001` | **WEAK**：`Process` 语义全缺（R4）；生产默认 Durable 仅 approval A6，**无装配 Task** |
| 7 | crash recovery | `T-FILE-005/006/008`；`T-PG-006` | **WEAK**：§20.5/§24.7 用例未列表化（R4/R5） |
| 8 | signed checkpoint | `T-CORE-024` + `T-CP-001/002` | **PASS**（合同层） |
| 9 | independent anchor | `T-CP-005`；approval A8 Defer 实现细节 | **WEAK**：仅「可插合同」；**无** EVIDENCE-ANCHOR-001 门禁 Task；真实 OSS/WORM 实现可 DEFER 但须正式登记 |
| 10 | reproducible verification | `T-CORE-025` pure verify；`T-CLI-004` vectors verify；独立 verifier 要求 | **PASS***（模板在 plan §8；实例 `evidence/system/` 不存在） |

**路径中断点**：步骤 5–7 的 AC 不可机验 + 步骤 9 无生产门禁 → **不能**声明「生产审计路径计划完备」。

---

### 2. 「关键操作可审计」七条件（C）— 本轮主检

| # | 条件 | 计划映射 | 裁定 |
|---|------|----------|------|
| C1 | 操作身份明确 | `EvidenceName`/`OperationId` `T-CORE-006/007`；`T-POL-002` required ops 登记 | **WEAK**：policy 12 字段未枚举进 AC（R6）；仅 domain_macro 示例，非全仓 required 枚举 |
| C2 | actor 和 subject 明确 | Actor `T-CORE-008`；Draft 含 subject_digest `T-CORE-010` | **FAIL 局部**：§7.2 **Subject 规范化/版本策略** 无独立 Task（R8）；subject 仅 digest 字段 ≠「明确」的领域策略闭合 |
| C3 | 输入/结果稳定 canonical digest | `T-CORE-018`；`T-DOM-001` 去 Debug-hash；`T-ARCH-004` | **PASS**（路径有；DEF-006 实现仍 OPEN） |
| C4 | 业务成功 ↔ durable evidence **无未声明窗口** | §15 A/B/C；`T-PG-004` outbox；`T-DOM-005` fail-closed；§33.6 写 **`T-ATOM via design`** | **FAIL**（阻断）：见下节 |
| C5 | 无 gap / fork / duplicate | Chain 属性 + CAS + 幂等任务 | **WEAK**：chain properties 仅 `§24.3` 章节号；IdempotencyConflict 缺 |
| C6 | checkpoint 检测截尾 + **整链替换** | `T-CP-004` TailTruncated；full replacement = `T-CP-005 + verify` | **FAIL 局部**：`+ verify` **非 Task ID**；无独立「整链替换检测」AC（R8） |
| C7 | 历史 schema / verifier / **公钥** 保留期内可用 | `T-LEG-002`；`T-CP-006 + docs`；`T-POL-002 + docs` | **FAIL**：`T-CP-006`=key **rotation 测试** ≠ retention；§22.3 六类 retention **未任务化**（R3） |

#### C4 深检（§34 最关键生产窗口）

| 子路径 | Spec | 计划 | 裁定 |
|--------|------|------|------|
| A 同事务 | §15.1 A | `T-PG-003` 直接 append 事务 | 有（adapter 内） |
| B outbox | §15.1 B / §15.2 | `T-PG-004/005` | 有 |
| C evidence 为 SoT | §15.1 C | **无 Task** | **OPEN** |
| Tier-A 证明 | §33.6 / EVIDENCE-ATOMICITY-001 | **`T-ATOM via design`** | **幽灵 ID — FAIL** |
| 外部不可逆副作用 Attempted+terminal | §15.3 / §33.6 | 「订单域后续；policy 预留」 | **无 Task · 无 DEFER(accepted) — FAIL** |
| Rejected 路径 append 成功才返回拒绝 | §15.5 | `T-DOM-002/005` 偏 outcome/fail-closed | **WEAK**（未钉「先 durable append 再业务 Rejected」序） |
| required fail-closed 全系统 | §13.7 | 仅 `T-DOM-005`（macro） | **FAIL**（gate/其他 required 无）（R3） |
| 背压下不切 Volatile | §29.3 | **无 Task** | **OPEN**（与 C4 同族空洞） |

**结论**：C4 **未闭合** → §34「关键操作可审计」**整体不得宣称计划完备**。

---

### 3. 否定路径与 Forbidden（B）

| 检查 | 结果 |
|------|------|
| plan 明确反对 `Vec+SHA256` 六字段终态 | **PASS**（plan §0.1 / Forbidden / §6.1） |
| 禁止旧链静默 rehash 为 V1 连续 | **PASS** 文案（实现 ABSENT） |
| 无签名 checkpoint+anchor 时禁「不可篡改」 | plan 有；`T-DOC-001` **TODO**（README 仍可能旧措辞） |
| §34 末句「快速 evidence 制造空洞」 | plan Forbidden 有「假 PASS」族，**未**逐字引用该句；纪律方向一致 |

---

### 4. 与 checklist R1–R40 交叉（本轮复检摘要）

> 完整 40 项见 notes §「Checklist 1–40」；此处只列与 §34 路径直接冲突的失败项。

| Check ID | 本轮裁定 | 与 §34 关系 |
|----------|----------|-------------|
| **R-SPEC-003** | **FAIL** | §33.6 = §34 系统条件的勾选面；幽灵 T-ATOM / external 无 ID |
| **R-ATOM-001** | **FAIL** | 直接对应 C4 窗口 |
| **R-APPEND-002** | **FAIL** | fail-closed 不全 → C4 |
| **R-POL-001** | **FAIL** | retention/policy 不全 → C7 |
| **R-CP-001** | **WEAK→FAIL\*** | TailTruncated 有；**整链替换**无独立 AC → C6 |
| **R-GATE-001** | **FAIL** | ≥9 个 EVIDENCE-\* 未进门禁 Task（R6）；含 ANCHOR/ATOMICITY 路径 |
| **R-TEST-001** | **FAIL** | 33.5 塞 `T-CI-002` 草案 → 路径「reproducible」不可勾完成 |
| **R-MIG-001** | **FAIL** | P6 bootstrap 强制 durable 缺 → 生产路径断（R7） |
| R-HONEST-001 | **PASS** | 未把 Proposed/§33 写成已闭合 |
| R-EVID-001 | **PARTIAL** | 模板有、实例无 |

---

## omissions

### O9-1 — §34 路径上的「空洞」清单（计划未钉死）

1. **`T-ATOM` 幽灵** + 无 SoT(C) 路径任务  
2. **§15.3 Attempted + terminal** 无 Task / 无正式 DEFER  
3. **§22 六类 retention** + artifact store 合同全缺  
4. **整链替换检测** 无独立 Task ID  
5. **startup verify** 拼凑 `T-FILE-005`+`T-CP-004`，非启动合同  
6. **bootstrap 强制 Durable / 禁 memory** 无 Task（仅 release 图 `T-ARCH-005`）  
7. **§29.3 背压**：StorageUnavailable / 不静默 Volatile / 不无限内存缓存 — **零任务**  
8. **§15.4 纯内存状态** 四步序 — **零任务**  
9. **Subject 策略版本**（§7.2）— 无 Task  
10. **IdempotencyConflict** 负例 — 无 Task  

### O9-2 — 生产路径「纸面闭环」假象

| 纸面 | 事实 |
|------|------|
| plan §0.1 画出与 §34 相同的箭头链 | 箭头 ≠ Task 覆盖 |
| gap §34 行 `PASS*` | 注释「仅文档存在；代码未满足」——**正确**；但 tasks 映射仍留洞 |
| §33.6 映射表 | 存在行 ≠ 存在可执行 Task |
| todo「DEF-012/013 生产阻断」 | 已登记 OPEN，**未**把 §34 七条件做成验收清单 |

### O9-3 — residual / 治理

- `residual-open` 文件仍 **ABSENT**（`T-RES-001` TODO）  
- ADR-012 `auditx` 与 002 路径对撞 **未** 入 plan（R7）— 影响生产路径「唯一 SSOT 路径」  

---

## false_pass_risks

| 风险 | 触发 | 后果 |
|------|------|------|
| **「plan 已写 §34 箭头」⇒ 路径闭合** | 只读 plan §0.1/§34 摘要 | 跳过 C4/C7 空洞直接进 W1 |
| **「W4+W5 做完」⇒ 可审计** | 无 Tier-A 证明 / 无 Attempted 序 / 无 retention | 对外声称「关键操作可审计」= §34 禁止的快速 evidence |
| **`T-CP-005` DONE** | 仅 trait 接口 | 勾 C6 整链替换 / independent anchor 生产条件 |
| **`T-POL-002` DONE** | 只登记 operation 名 | 勾 C7 retention |
| **anchor A8 Defer** | Defer 实现细节被当成 Defer **合同** | 无 ANCHOR 门禁仍标生产 |
| **memory 测绿** | 无 bootstrap 强制 | 生产仍装 Volatile，C4 窗口大开 |
| **R-CANON/R-API 全绿** | Core 完备幻觉 | 忽略 append/atomic/retention 后半路径 |

---

## notes

### 路径覆盖分数（计划层，非实现）

```text
A 路径 10 步:  强覆盖 5 · 弱 4 · 断点 1（整链级生产证明依赖 C4/C6/C7）
C 七条件:      强 1 (C3) · 弱 2 (C1,C5) · FAIL 4 (C2局部,C4,C6局部,C7)
B 否定终态:    文案 PASS
综合:          FAIL — 任一 C4/C7 FAIL 即否决「生产审计路径计划完备」
```

### Checklist 1–40 最终复检（本 Verifier）

| # | ID | 裁定 | 一句理由 |
|---|-----|------|----------|
| 1 | R-SPEC-001 | PASS | Spec 文件 + ID=SPEC-EVIDENCE-002 |
| 2 | R-SPEC-002 | PASS | gap-matrix §0–§34 行齐全 |
| 3 | R-SPEC-003 | **FAIL** | 幽灵 T-ATOM；external 无 ID；33.5 规划桶 |
| 4 | R-GAP-001 | PASS* | DEF-001…018 在 gap+todo；\*residual-open 文件缺 |
| 5 | R-GAP-002 | **FAIL** | T1–T18 无 Task 绑定表（R3） |
| 6 | R-PATH-001 | PASS | plan §5 crates/infra/evidence |
| 7 | R-PATH-002 | PASS | adapters 三路径写明 |
| 8 | R-PATH-003 | PASS | CLI + 删 tools/evidence |
| 9 | R-DEP-001 | PASS | kernel+sha2+thiserror |
| 10 | R-DEP-002 | PASS | 禁 anyhow/serde/tokio 列表 |
| 11 | R-API-001 | PASS | Digest32/ChainId/EventId/OperationId/Name |
| 12 | R-API-002 | PASS | Draft + Outcome 六态 |
| 13 | R-API-003 | PASS | RecordV1 私有 + seal |
| 14 | R-CANON-001 | PASS* | 25 步任务有；\*向量名未展开（R5） |
| 15 | R-CANON-002 | PASS | genesis 非全零任务 |
| 16 | R-CANON-003 | PASS | ("ab","c") 边界任务 |
| 17 | R-CANON-004 | PASS | digest_canonical；禁 hash_bytes |
| 18 | R-TIME-001 | PASS | recorded_at/event_time |
| 19 | R-CHAIN-001 | PASS | sequence≥1；ChainHead |
| 20 | R-APPEND-001 | **FAIL** | Process 语义缺；生产默认 Durable 无装配 Task（R4） |
| 21 | R-APPEND-002 | **FAIL** | fail-closed 仅 macro；IdempotencyConflict 负例弱 |
| 22 | R-READ-001 | PASS* | trait+mem；\*limit/损坏不静默 AC 薄 |
| 23 | R-ATOM-001 | **FAIL** | 幽灵 T-ATOM；SoT(C) 缺；§15.3 无 Task |
| 24 | R-CP-001 | **FAIL\*** | Signed/Tail 有；整链替换无独立 AC |
| 25 | R-ERR-001 | PASS | EvidenceError + XError 映射任务 |
| 26 | R-MEM-001 | **FAIL** | bootstrap/systemd 阻断缺（R4） |
| 27 | R-FILE-001 | PASS* | 骨架有；\*frame/fsync 细节弱（R4） |
| 28 | R-PG-001 | PASS* | 四表任务有；\*不变量细节弱 |
| 29 | R-TEST-001 | **FAIL** | fuzz/mutants/miri/branch → T-CI-002 草案 |
| 30 | R-CLI-001 | PASS* | 七命令有槽；\*§25.2/25.4 AC 不全（R6） |
| 31 | R-POL-001 | **FAIL** | 12 字段 + 六类 retention 未进 AC |
| 32 | R-GATE-001 | **FAIL** | ≥9 EVIDENCE-\* 无门禁 Task（R6） |
| 33 | R-MIG-001 | **FAIL** | P0–P6 无显式表；bootstrap/双包 rename 缺（R7） |
| 34 | R-EVID-001 | PASS* | plan §8 模板完整；\*无实例/非每波 |
| 35 | R-DOWN-001 | PASS | domain_macro + gate W3 |
| 36 | R-GOV-001 | **FAIL** | ADR-010 有提案；**ADR-012 全缺**（R7） |
| 37 | R-FORBID-001 | **FAIL** | 页眉/§1/approval 三套 Forbidden 不等价（R8） |
| 38 | R-TODO-001 | PASS* | Wave+DEF 覆盖；\*worktree gitignore |
| 39 | R-CROSS-001 | PASS | plan §1.2 INFRA-003 |
| 40 | R-HONEST-001 | PASS | Proposed≠Approved；≠§33 闭合 |

```text
FAIL count (strict):  16
PASS / PASS*:         24
round: 9
result: FAIL
```

### 诚实声明

- Spec：**Proposed** · Campaign：**PLANNING** · **§33 / §34 生产审计路径：未闭合**  
- 本轮 **不是** 实现验收；实现仍为 `tools/evidence` 六字段原型  
- 不把 R3–R8 的 FAIL 洗成 PASS；本轮独立重走 §34，结论一致 **FAIL**

### 证据指针

- Spec §34：`/home/workspace/.agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md` L2869+  
- Spec §15 / §22 / §29.3：同文件  
- tasks §33.6：`.../plan/tasks.md` L267–277（幽灵 T-ATOM、external 备注）  
- plan §0.1 路径图：`.../plan/plan.md` L32–43  

---

## verdict_summary

```text
round: 9
result: FAIL
failed_checks:
  - §34 path C4 (undeclared window / T-ATOM ghost / §15.3)
  - §34 path C6 (full chain replacement not a Task)
  - §34 path C7 (retention not taskified)
  - R-SPEC-003 R-ATOM-001 R-APPEND-001/002 R-POL-001
  - R-CP-001* R-GATE-001 R-TEST-001 R-MIG-001 R-GOV-001 R-FORBID-001 R-MEM-001
omissions: [O9-1 … O9-3]
false_pass_risks: [plan arrows ≠ closed path; W4+W5 green ≠ auditable]
```
