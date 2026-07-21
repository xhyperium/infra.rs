# Round 10 Findings — Full Heading Re-enumeration

| 字段 | 值 |
|------|-----|
| Round | **10** |
| Role | Final Verifier · Independent heading coverage |
| Spec | `.agents/ssot/tools/evidence/xhyper-evidence-complete-spec.md` |
| Plan pack | `plan.md` · `gap-matrix.md` · `tasks.md` · `approval-packet.md` · `.worktrees/evidence-todo.md` |
| Rule | 每个 `##` / `###`：**M** = 明确提及 · **W** = 仅章节号/弱代理 · **Z** = **零提及**（omission） |
| Date | 2026-07-14 |
| **result** | **FAIL** |

---

## result

**FAIL** — 独立枚举完整规范全部 `##`/`###` 标题后：

- **多数章节** 在 gap-matrix 有 **§N 行** 或 Wave 代理（表面覆盖）；
- **子节级** 大量仅 **W**（「见 §xx」/笼统 Task），不可机验；
- **明确 Z（零提及）** 子节存在，足以否决「plan 包无遗漏」主张；
- 叠加 R9 §34 路径 FAIL + checklist **16 FAIL** → 计划完备性十轮 **不得** `fail_rounds=0`。

---

## failed_checks

### R-SPEC-002 加强检 — 子节覆盖

gap-matrix 满足 **章级** §0–§34 行（R-SPEC-002 字面 PASS），但本轮标准是：

> 每个 `##`/`###` 是否在 plan **或** gap **或** tasks **或** todo 中被 **点名或可追踪 Task** 触及。

**章级 PASS ≠ 子节完备。** 下列 **Z / 关键 W** 触发本轮 FAIL。

### R-SPEC-003 / 幽灵映射（跨轮确认）

仍成立：`T-ATOM via design` 非 ID；external Attempted 无 Task；33.5 规划桶。

### R-TODO-001 / residual

todo 有 Wave 与 DEF；**无** 按 `##` 标题的跟踪；`residual-open` **ABSENT**。

---

## 完整标题枚举

> 标记：`M` 明确 · `W` 弱 · `Z` 零提及  
> 「出处」= 首次/主要命中文件（非穷尽）。

### §0–§5

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 0. 文档定位 | M | gap §0；plan 页眉 Supersedes |
| ## 1.1 Evidence 的准确定位 | M | plan §0.1；T-DOC-001；Forbidden 措辞 |
| ## 1.2 威胁模型 | M | gap §2 T1–T18；todo §3 |
| ## 1.3 信任边界 | W | R3 批注级；**无** bootstrap/signer/anchor 边界表 Task |
| ## 2.1 Core 职责 | M | plan §0.2 准入；W1 |
| ## 2.2 Adapter 职责 | M | W2/W4 路径 |
| ## 2.3 CLI 职责 | M | W5 T-CLI-* |
| ## 2.4 非职责 | W | 间接 via Forbidden；**未**逐条「不承担」清单任务 |
| ## 4.1 Core 内部依赖 | M | T-CORE-002；R-DEP-001 |
| ## 4.2 Core 外部依赖白名单 | M | 同上 + EVIDENCE-DEP |
| ## 4.3 Dev dependencies | W | proptest 仅 T-CORE-028 暗示；trybuild/static_assertions/cargo-fuzz **未点名 §4.3** |
| ## 4.4 Features | M | T-CORE-034 default=[]；禁 mock |
| *(# 3 目标目录 — 仅 H1)* | M | plan §5；gap §3 |
| *(# 5 Crate 级规则 — 仅 H1)* | M | T-CORE-003 forbid/deny |

### §6–§12

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 6.1 Digest32 | M | T-CORE-004 |
| ## 6.2 ChainId | M | T-CORE-005 |
| ## 6.3 EventId | M | T-CORE-006 |
| ## 6.4 OperationId | M | T-CORE-006 |
| ## 6.5 EvidenceName | M | T-CORE-007 |
| ## 7.1 EvidenceActor | M | T-CORE-008 |
| ## 7.2 Subject | W | 仅 subject_digest 字段；**无** 规范化/版本策略 Task |
| ## 8.1 公开模型 | M | T-CORE-010 |
| ## 8.2 EvidenceOutcome | M | T-CORE-009 |
| ## 8.3 构造 API | W | builder 在 T-CORE-010；API 细则未展开 |
| ## 9.1 公开模型 | M | T-CORE-012 |
| ## 9.2 不变量 | W | 隐含 seal/verify；无独立 invariant 清单 Task |
| ## 9.3 封链函数 | M | T-CORE-013 |
| ## 10.1 recorded_at | M | T-CORE-011 |
| ## 10.2 event_time | M | T-CORE-011 |
| ## 11.1 总体规则 | M | T-CORE-014 BE/presence |
| ## 11.2 Record preimage | M | T-CORE-014 25 步 |
| ## 11.3 record_digest | M | T-CORE-016 |
| ## 11.4 Genesis digest | M | T-CORE-017 |
| ## 11.5 内容摘要 | M | T-CORE-018；DEF-003 |
| ## 11.6 编码边界 | M | T-CORE-029 |
| ## 11.7 Decoder | M | T-CORE-015 |
| ## 12.1 ChainHead | M | T-CORE-019 |
| ## 12.2 顺序 | M | sequence 任务 |
| ## 12.3 单链一致性 | W | T-CORE-030「§24.3」代理 |
| ## 12.4 分叉 | W | CAS/ForkDetected 间接 |
| ## 12.5 链划分 | W | policy 示例；**无** 按对象划分策略矩阵 Task |

### §13–§18

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 13.1 Durability | M | T-CORE-022；\*Process 弱 |
| ## 13.2 AppendRequest | M | T-CORE-022 |
| ## 13.3 AppendReceipt | M | T-CORE-022 |
| ## 13.4 EvidenceAppender | M | T-CORE-023 |
| ## 13.5 幂等 | W | T-MEM-002 笼统 |
| ### 内容完全相同 | W | 隐含「idempotent」；**未**点名 |
| ### 内容不同 | **Z→W** | **无** IdempotencyConflict 显式 Task（R5 记 FAIL） |
| ## 13.6 Head CAS | M | T-MEM-002 |
| ## 13.7 Fail-closed | M | T-DOM-005；\*范围过窄 |
| *(# 14 Reader — 仅 H1)* | M | T-CORE-023 T-MEM-003；\*limit AC 薄 |
| ## 15.1 持久化业务状态 | W | T-PG-003/004；SoT **Z** |
| ## 15.2 Transactional outbox | M | T-PG-004/005 |
| ## 15.3 外部不可逆副作用 | **Z** | 仅 §33.6 括号「订单域后续」— **非正式 DEFER、无标题级提及** |
| ## 15.4 纯内存状态 | **Z** | plan/gap/tasks/todo **均无** 四步序 |
| ## 15.5 Rejected 路径 | W | T-DOM-002 outcome；**未**钉 append-then-reject 序 |
| ## 16.1 CheckpointV1 | M | T-CORE-024 |
| ## 16.2 SignedCheckpointV1 | M | T-CP-002 |
| ## 16.3 CheckpointSigner | M | T-CP-001 |
| ## 16.4 CheckpointVerifier | M | T-CP-001 |
| ## 16.5 频率 | M | T-CP-003 10k/60s |
| ## 16.6 独立锚点 | M | T-CP-005；\*门禁弱 |
| ## 16.7 尾部截断检测 | M | T-CP-004 |
| ## 17.1 VerificationReport | M | T-CORE-025 |
| ## 17.2 验证内容 | W | 章节引用 |
| ## 17.3 验证输入 | W | 同上 |
| ## 17.4 验证失败 | W | 错误映射间接 |
| ## 18.1 错误集合 | M | T-CORE-020 |
| ## 18.2 到 XError 的映射 | M | T-CORE-021 |

### §19–§23

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 19.1 定位 | M | T-MEM-001 |
| ## 19.2 能力 | W | 幂等/CAS 有；零摘要 head 等未全列 |
| ## 19.3 生产阻断 | W | T-ARCH-005 release 图；**bootstrap/systemd Z** |
| ## 20.1 定位 | M | T-FILE-001 |
| ## 20.2 单写者 | M | T-FILE-003 |
| ## 20.3 Segment 格式 | M | T-FILE-002 |
| ## 20.4 Durability | M | T-FILE-004 |
| ## 20.5 恢复 | M | T-FILE-005/006 |
| ## 20.6 Segment rotation | M | T-FILE-007 |
| ## 21.1 定位 | M | T-PG-001 |
| ## 21.2 表级不变量 | M | T-PG-002 |
| ## 21.3 Append 事务 | M | T-PG-003 |
| ## 21.4 Outbox 事务 | M | T-PG-004 |
| ## 21.5 并发 | M | T-PG-006 |
| ## 22.1 Evidence record | W | 模型隐含禁 payload；**无** 显式「禁敏感原文」AC |
| ## 22.2 原始 artifact | **Z** | 仅「T-POL-002 + docs」；**无** store 路径/trait/Object Lock |
| ## 22.3 Retention | **Z** | 六类 retention **未**写入 policy AC |
| ## 22.4 删除 | **Z** | erasure/deletion evidence **无 Task** |
| ## 23.1 V1 冻结 | M | golden + 禁改规则 via W1 |
| ## 23.2 Reader 兼容性 | W | T-LEG-002 弱代理 |
| ## 23.3 算法迁移 | **Z** | 双链锚定迁移 **无** 独立 Task（仅 legacy 文案） |

### §24–§30

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 24.1 Golden vectors | M | T-CORE-026；\*向量名未展开 |
| ## 24.2 Canonical properties | W | T-CORE-028/029；invalid length/truncation/no-panic 缺 |
| ## 24.3 Chain properties | W | T-CORE-030 仅章节号 |
| ## 24.4 Checkpoint properties | W | 仅 rotation/tail；多属性缺 |
| ## 24.5 Idempotency | W | 见 13.5 |
| ## 24.6 并发 | W | 无 1000+ AC |
| ## 24.7 Crash / Fault Injection | W | T-FILE-008「§24.7」 |
| ## 24.8 Fuzz | W | T-CI-002 **草案** |
| ## 24.9 Coverage | W | T-CORE-033「或记录」 |
| ## 24.10 Mutation testing | W | T-CI-002 草案 |
| ## 24.11 Miri | W | T-CI-002 草案 |
| ## 25.1 命令 | M | T-CLI-002..005 |
| ## 25.2 默认行为 | W | `--json`/stderr/敏感 **Z** 细节 |
| ## 25.3 退出码 | M | T-CLI-006 |
| ## 25.4 repair-tail | W | 缺越 checkpoint 等 |
| *(# 26 政策 — 仅 H1)* | M | T-POL-001/002；\*12 字段弱 |
| ## 27.1 Core 门禁 | W | 多 ID 映射；**CANONICAL 脱落** |
| ## 27.2 Adapter 门禁 | W | 5 条无 T-ARCH |
| ## 27.3 系统门禁 | W | ANCHOR/SCHEMA/VECTOR 缺 |
| *(# 28 CI — 仅 H1)* | W | T-CI-001「§28」；crate-standard **Z** |
| ## 29.1 Core | W | T-PERF-001 骨架 |
| ## 29.2 Adapter | W | T-PERF-002 文档 |
| ## 29.3 背压 | **Z** | plan/gap/tasks/todo **均无** StorageUnavailable 背压合同 |
| *(# 30 Observability — 仅 H1)* | W | T-OBS-001「metrics 名称清单」；**未**枚举规范 11 个 metric 名 |

### §31–§34

| 标题 | 标记 | 出处 / 备注 |
|------|------|-------------|
| ## 31.1 当前问题 | M | gap 总览；DEF 台账 |
| ## 31.2 迁移阶段 | W | 无 P↔W 表（R7） |
| ### P0：冻结错误扩散 | M | T-FREEZE-001；W0 |
| ### P1：Core V1 | M | W1 |
| ### P2：Compatibility bridge | W | T-LEG 塞 W3；阶段折叠 |
| ### P3：Domain migration | M | W3 T-DOM |
| ### P4：Durable adapters | M | W2+W4 |
| ### P5：Checkpoint | M | W5 T-CP |
| ### P6：Cutover | W | T-CUT；**bootstrap 强制缺** |
| *(# 32 自身 Evidence — 仅 H1)* | M | plan §8；T-EVID-SYS |
| ## 33.1 规格闭合 | M | tasks 映射 + T-HUM |
| ## 33.2 Core 闭合 | M | 映射表 |
| ## 33.3 Adapter 闭合 | M | 映射表 |
| ## 33.4 Checkpoint 闭合 | W | full replacement / startup 弱 |
| ## 33.5 测试闭合 | W | T-CI-002 草案桶 |
| ## 33.6 系统闭合 | **W→FAIL** | 幽灵 T-ATOM；external Z |
| *(# 34 最终裁定 — 仅 H1)* | M | plan §0.1 同文；**路径闭合见 R9 FAIL** |

---

## omissions

### 严格 Z（零提及或非正式括号备注，计 omission）

| ID | 标题 | 影响 |
|----|------|------|
| **Z1** | ## 15.3 外部不可逆副作用 | §34 C4 / §33.6 生产窗口 |
| **Z2** | ## 15.4 纯内存状态 | 领域错误模式无护栏 |
| **Z3** | ## 22.2 原始 artifact | 保留源证据；§1.1 retained artifacts |
| **Z4** | ## 22.3 Retention | §34 C7 |
| **Z5** | ## 22.4 删除 | 法律删除 vs 审计不可变 |
| **Z6** | ## 23.3 算法迁移 | 未来 hash 迁移双锚定 |
| **Z7** | ## 29.3 背压 | 生产高压下 fail-closed / 禁切 Volatile |
| **Z8** | ### 内容不同（13.5） | IdempotencyConflict 合同 |

> 「订单域后续」**不算** M：无 Task ID、无 DEFER(accepted)、todo 无勾选项。

### 高危 W（易假 PASS，升格 omission 风险）

| 标题 | 问题 |
|------|------|
| ## 7.2 Subject | 字段有、策略无 |
| ## 12.5 链划分 | 审计对象分区未任务化 |
| ## 19.3 生产阻断 | 缺 bootstrap/systemd |
| ## 24.8–24.11 | Nightly 草案桶 |
| ## 27.* / #28 | 门禁 ID / CI 命令不全 |
| #30 Observability | metric 名未钉死 |
| ## 33.6 | 幽灵映射 |

### 统计

```text
## 标题（约 120）:  M≈70 · W≈40 · Z≈8（上表 Z1–Z7 + 严格 13.5 子 ###）
### 标题（9）:      M≈6 (P0–P6 多数) · W≈2 (P2,P6) · Z/W≈2 (内容相同/不同)
H1-only 章 (3,5,14,26,28,30,32,34): 均有章级 M/W；28/30 偏 W
```

**任一 Z 落在 §15/§22/§29/§34 关键路径 → 计划不得宣称「无章节遗漏」。**

---

## false_pass_risks

1. **R-SPEC-002 字面绿**：gap 有 §0–§34 行 → 误以为 **子节** 全覆盖。  
2. **「W = 有章节号引用」当 M**：`T-CORE-030` AC=`§24.3` 掩盖九条 chain property 未枚举。  
3. **括号 DEFER**：`（订单域后续）` 被当成已管理 residual。  
4. **docs 代理**：`T-POL-002 + docs` 覆盖 §22.2–22.4 假象。  
5. **H1 有、## 无**：#30 Observability 有 T-OBS-001，但 11 个 metric 名零枚举。  
6. **P0–P6 名在 ### 出现** ≠ 映射表与 bootstrap Task 存在（R7）。  
7. **本轮若只扫 ## 标题含「Checkpoint」** 会漏 Z 的 §15.3/§22/§29.3。

---

## notes

### 与 R9 衔接

R10 的 **Z1–Z5、Z7** 直接支撑 R9 对 §34 **C4/C7** 的 FAIL。  
R10 **不** 单独重判实现代码；以 plan 包文本检索为准。

### Checklist 1–40（与 R9 一致，终裁）

```text
FAIL: R-SPEC-003, R-GAP-002, R-APPEND-001, R-APPEND-002, R-ATOM-001,
      R-CP-001*, R-MEM-001, R-TEST-001, R-POL-001, R-GATE-001,
      R-MIG-001, R-GOV-001, R-FORBID-001
      (+ R-CP-001 本 verifier 严格记 FAIL*)
PASS/PASS*: 其余
fail_count ≥ 16 → round-10 对「计划完备」= FAIL
```

### 检索方法（可复现）

对每个标题关键词在下列路径 `rg`：

```text
.agents/ssot/tools/evidence/plan/{plan,gap-matrix,tasks,approval-packet}.md
.worktrees/evidence-todo.md
```

Z 判定：无标题编号、无规范专名、无对应 Task/DEF 行。

### 诚实声明

- 本轮 **FAIL**；**禁止** 因「gap 有 35 行」写 `fail_rounds=0`。  
- Spec 仍 **Proposed**；§33/stable **未** 授权。  
- 未修改 plan/tasks/gap（仅本 findings 文件）。

---

## verdict_summary

```text
round: 10
result: FAIL
failed_checks:
  - subheading coverage (Z1–Z8)
  - R-SPEC-002 chapter-only false completeness
  - R-SPEC-003 / §33.6 ghost mapping (confirm)
  - checklist aggregate ≥16 FAIL (with R9)
omissions:
  - §15.3 Attempted+terminal
  - §15.4 in-memory state recipe
  - §22.2–22.4 artifact/retention/erasure
  - §23.3 algorithm migration
  - §29.3 backpressure
  - ### 13.5 内容不同 / IdempotencyConflict
false_pass_risks:
  - chapter rows hide subsection holes
  - "§N" AC and parenthetical defer
```

---

# TOP FIXES（计划包 — 本 Verifier 不直接改 plan）

> 综合 **R9 + R10**（并与 R3–R8 一致）的 **关键补丁优先级**。Planner 应改 `tasks.md` / `plan.md` / `gap-matrix.md` / `approval-packet.md`，**不是** 改 findings。

| Prio | Fix | 关闭什么 |
|------|-----|----------|
| **P0** | 消灭 **`T-ATOM via design`**：新建 `T-ATOM-001`（A/B/C 选择+证明）或拆真实 Task；SoT(C) 显式 Task 或 `DEFER(accepted)` | R-ATOM-001 · §34 C4 · §15.1 · §33.6 |
| **P0** | **`T-EXT-001`** Attempted+terminal（§15.3）**或** residual `DEFER(accepted)` 正式行 + todo 勾选 | §34 C4 · Z1 · §33.6 |
| **P0** | **§22 任务簇**：`T-ART-001` artifact store 合同；`T-RET-001` 六类 retention 进 `evidence-policy.toml` AC；`T-ERA-001` 删除/erasure evidence | Z3–Z5 · §34 C7 · R-POL-001 |
| **P0** | **fail-closed 矩阵**：所有 `required` ops（含 gate）测试 + 门禁；不只 `T-DOM-005` | R-APPEND-002 · DEF-017 |
| **P0** | **`T-BOOT-001`** bootstrap 强制 production Durable；禁 memory 进 binary/**systemd** | §19.3 · P6 · §34 生产默认 |
| **P0** | **双包 rename Task**（`evidence`→`evidence_legacy`）+ 共存 AC | R7 构建致命缺口 |
| **P1** | **门禁 ID 全表**：`EVIDENCE-CANONICAL/DURABILITY/IDEMPOTENCY/CONCURRENCY/RECOVERY/FSYNC/ANCHOR/SCHEMA/VECTOR` 各有 T-ARCH 或清单行 | R-GATE-001 · R6 |
| **P1** | **拆 `T-CI-002`**：fuzz / branch≥90 / mutants≥90 / miri **分 Task**，AC=门槛非「草案」；Nightly 六项显式必做或 DEFER | R-TEST-001 · §24.8–11 · §33.5 |
| **P1** | **IdempotencyConflict** 负例 Task；chain properties **九条枚举** AC | Z8 · §24.3/24.5 |
| **P1** | **整链替换 + startup verify** 独立 `T-CP-00x` AC（禁 `+ verify` 散文） | §34 C6 · §33.4 |
| **P1** | **`T-BP-001` §29.3 背压**：StorageUnavailable、禁无限缓存、禁静默 Volatile | Z7 |
| **P1** | **ADR-012 / auditx** 修订或废止进 approval + `T-DOC-00x` | R-GOV-001 |
| **P1** | **Forbidden SSOT**：页眉 ⋃ approval §2 并集；执行违反 → Wave FAIL | R-FORBID-001 |
| **P1** | **落盘 `residual-open.md`**（仓内，非 gitignore worktree）；T-RES-001 DONE 条件=文件存在+DEF 状态机 | residual · §33.1 Unknown |
| **P2** | P0–P6 ↔ W0–W9 **显式映射表**；Subject 策略 Task；§15.4 文档+反模式测试；§23.3 算法迁移骨架；metric 11 名钉死；§28 命令逐条（含 `crate-standard`） | R-MIG · R10 W/Z 清扫 |
| **P2** | 降级 **`T-PLAN-003` AC**（不得称「§33 全部」直至幽灵清零）；golden 路径 A10 闭合 | R8 假完成 |

### 十轮汇总（供 verdict 文件）

```text
R3 FAIL · R4 FAIL · R5 FAIL · R6 FAIL · R7 FAIL · R8 FAIL · R9 FAIL · R10 FAIL
（R1/R2 若未落盘不得默认 PASS）
fail_rounds ≥ 8  （已知）  ⇒  禁止 evidence-plan-10x-verdict.md 写 fail_rounds=0
禁止：Spec Approved / registry stable / §33 全勾 / 「生产可审计」宣称
```
