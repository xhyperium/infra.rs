> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 04 Findings — Evidence Plan Completeness

| 字段 | 值 |
|------|-----|
| round | `4` |
| focus | Adapters §19–§21（memory/file/postgres）· crash recovery · concurrency · durability |
| result | **FAIL** |
| sources | `xhyper-evidence-complete-spec.md` · `plan/plan.md` · `plan/gap-matrix.md` · `plan/tasks.md` · `plan/approval-packet.md` · `.worktrees/evidence-todo.md` |
| checklist | `plan.md` §4 全部 40 Check ID |
| baseline | `main@007ca7b5` · SPEC-EVIDENCE-002 **Proposed** |
| verifier | Verifier · 2026-07-14 |
| cross_ref | Round 3 已 FAIL（fail-closed / §22 / T→Task）；本轮叠加 adapter 合同粒度不足 |

---

## result: FAIL

§19–§21 在 plan 中有 Wave/W2–W4 任务骨架与路径，但 **File/Postgres 合同 AC 粒度远低于 Spec**，crash recovery 未展开为可验收步骤，durability 三态中 `Process` 与 group-commit/目录 fsync 规则缺失，原子性三选一映射含幽灵任务。不得判 PASS。

---

## failed_checks

| Check ID | 判定 | 证据摘要 |
|----------|------|----------|
| **R-MEM-001** | **FAIL** | `T-MEM-004/007` 覆盖 `production_allowed=false` 与 Durable→`DurabilityFailure`。Spec **§19.3** 要求 **bootstrap、release gate、archgate** 阻止 `evidence_memory` 进入 production feature、**release binary** 与 **systemd 部署清单**。计划仅有 `T-ARCH-005`「release 图无 memory」——**bootstrap 组装合同**与 **deploy/systemd 清单门禁** 无 Task。§19.2 禁止「verify 永远成功 / 锁中毒空 Vec / **chain head 返回零摘要** / 声称 Durable」中，零摘要 head 未写入 MEM AC（仅 core `T-CORE-019`）。 |
| **R-FILE-001** | **FAIL** | 存在 `T-FILE-001…009` 且引用 §20.3，但 AC 未强制 Spec 关键不变量：`payload_len` **硬上限**；帧 **前后 length 必须相同**；**fixed commit marker** 完整才算提交；**segment_digest**；未完成尾帧不得当记录；Durable 路径 **fdatasync/fsync + 必要时同步目录元数据 + 再更新 durable head**；**group commit** 规则（请求 Durable 必须等 group fsync；fsync 前不得返回 Durable；batch 任一失败明确报告）。`T-FILE-004` 仅写「返回前 fsync」——**低于 §20.4**。`T-FILE-005` 仅「不完整尾帧 / 仅允许截断未提交」——**未覆盖 §20.5 启动 10 步**（writer lock→读可信 checkpoint→扫 segment→验 header/frame→不完整尾→sequence/digest→对照 checkpoint→建 head→才允许写）。 |
| **R-PG-001** | **FAIL** | `T-PG-002`「heads/records/outbox/checkpoints + 唯一约束」贴近表名，但 **chain head 行必含 sequence / head_digest / updated_at** 未写 AC；唯一约束应明确 `(chain_id, sequence)` 与 `(chain_id, event_id)`。`T-PG-003` FOR UPDATE 有；`T-PG-004/005` outbox 有。Spec **§21.5** 强制场景含：**多进程并发 append、同 event_id 并发重试、同 expected head 竞争、事务 rollback、连接断开、deadlock retry**。`T-PG-006` 仅「并发/死锁/回滚」——**缺连接断开**与 **同 expected_head 竞争** 明文 AC。 |
| **R-APPEND-001** | **FAIL** | Spec §13.1 三态：`Volatile` / `Process` / `Durable`；生产默认 `Durable`；`Durable` = 事务提交或 fsync 后崩溃仍可读。计划：`T-CORE-022` 枚举有；memory 最高 Volatile（`T-MEM-004`）有；file fsync 偏薄（见上）。**`Process`（已写 OS/page cache 未承诺掉电）在任何 adapter 任务中无语义/测试 AC**；生产默认 Durable 仅在 approval A6，**无 bootstrap/默认参数任务**。 |
| **R-ATOM-001** | **FAIL** | 同 Round 3：§15.1 A/B/C 三选一；`tasks.md` §33.6 写 **`T-ATOM via design`（幽灵 ID）**；C SoT 无任务；外部副作用 Attempted+terminal 非正式 DEFER。W4 不能声称原子性合同计划完备。 |
| **R-READ-001** | **FAIL** | Spec §14：`head` / `get_by_event_id` / `read_range`；`1 <= limit <= 10_000`；`start_sequence >= 1`；严格递增；**不得静默跳过损坏**；损坏必须错误。`T-MEM-003` AC 仅为「head/get/range 限制」——**数值边界与损坏语义未写**。file/pg 无独立 Reader AC（指望 conformance `T-MEM-008` 复用，但钩子本身未列出上述不变量）。 |
| **R-SPEC-003** | **FAIL** | §33.3 Adapter 闭合映射：memory/file/pg/concurrency/idempotency/crash/disk-fsync/不降级 volatile 有指针，但 **crash recovery / disk full·short write·fsync failure** 依赖 `T-FILE-008` 笼统 chaos，无用例清单对齐 §24.7/§20.5/§21.5；叠加幽灵 `T-ATOM` → 完成定义不可追踪。 |

### 本轮其余 checklist 项

| Check ID | 判定 | 备注 |
|----------|------|------|
| R-SPEC-001 / R-SPEC-002 | PASS | Spec + gap §0–§34 |
| R-GAP-001 | PASS | DEF-001…018 全在 todo + gap（见下） |
| R-GAP-002 | PASS* | 威胁行存在；安全深度见 R3 FAIL（本轮不重复升格） |
| R-PATH-001 | PASS | `crates/infra/evidence` plan §5 |
| R-PATH-002 | PASS | memory/file/postgres 路径写明 |
| R-PATH-003 | PASS | evidence-cli + 删 tools/evidence |
| R-DEP-001/002 | PASS | |
| R-API-001…003 | PASS | W1 |
| R-CANON-001…004 | PASS | W1 |
| R-TIME-001 / R-CHAIN-001 | PASS | |
| R-APPEND-002 | PASS* | 幂等+CAS 任务有；fail-closed 全局面 R3 已 FAIL |
| R-CP-001 | PASS | W5；与 file recovery 交叉仍薄 |
| R-ERR-001 | PASS | |
| R-TEST-001 | PASS* | chaos 笼统；缺 disk full 专项 Task 名 |
| R-CLI-001 | PASS | repair-tail 依赖 file recovery |
| R-POL-001 | PASS* | 结构有；§22 见 R3 |
| R-GATE-001 | PASS | |
| R-MIG-001 | PASS | |
| R-EVID-001 | PASS | 含 `adapter-conformance.json` / `recovery-tests.json` 槽位 |
| R-DOWN-001 | PASS | |
| R-GOV-001 | PASS | |
| R-FORBID-001 | PASS | |
| R-TODO-001 | PASS | W2/W4 adapter 波次 + DEF 全覆盖 |
| R-CROSS-001 | PASS | |
| R-HONEST-001 | PASS | 未假 Approved；adapter 仍 ABSENT 诚实 |

---

## omissions

### O4-1 — Memory adapter（§19）对照

| Spec 要求 | 计划 Task | 缺口 |
|-----------|-----------|------|
| path `crates/adapters/evidence/memory` · `production_allowed: false` | T-MEM-001/007 | path/README 有；**机器可读 production_allowed 标记**（cfg/const）未规定 |
| seal / sequence / previous / event_id 幂等 / head CAS / verify / 并发线性化 | T-MEM-002/005 | verify 合同未单列 AC |
| 禁：verify 恒成功 | DEF-007 + T-CORE-034 无 mock | MEM 侧无「假 verify」回归 AC |
| 禁：lock poison → 空 Vec | T-MEM-006 | 有 |
| 禁：chain head 零摘要 | — | **缺 MEM/Reader AC** |
| 禁：声称 Durable；Durable 请求 → DurabilityFailure | T-MEM-004 | 有 |
| 最高 durability = Volatile | 隐含 | 应显式断言 max level |
| bootstrap/release/archgate/systemd 阻断 | T-ARCH-005 部分 | **bootstrap + systemd 缺** |
| conformance 钩子 | T-MEM-008 | 有；套件条目未列 |

### O4-2 — File adapter（§20）对照

| Spec 小节 | 要求 | 计划 | 缺口 |
|-----------|------|------|------|
| §20.1 | 单机 append-only / 独立于业务 DB / durable buffer | T-FILE-001 路径 | 定位文档可接受 |
| §20.2 | 每 ChainId **单进程独占 writer lock**；多 reader；第二 writer 失败；**不得仅进程内 Mutex** | T-FILE-003 | 有；锁类型（flock/fcntl）未指定可接受 |
| §20.3 header | magic, format_version, chain_id, first_sequence, previous_segment_head, created_at | T-FILE-002「§20.3」 | **字段清单未进 AC**——执行易漏 previous_segment_head |
| §20.3 frames | u32_be len · canonical bytes · u32_be len · commit marker | 同上 | **双 length + commit marker** 未写 |
| §20.3 footer | final_sequence, final_head, segment_digest | 同上 | **segment_digest** 未写 |
| §20.3 规则 | payload_len 硬上限；前后 length 同；commit 后才提交；digest 检损；尾帧未完成不算记录 | 部分隐含于 005 | **硬上限数值策略任务缺失** |
| §20.4 Durable | write full frame → fsync data → **目录元数据** → 更新 durable head → receipt | T-FILE-004 | **目录 fsync / head 更新顺序 / group commit / batch 失败** 缺 |
| §20.5 recovery | 10 步 + 仅允许截断无 commit 的最后不完整 frame 且高于最新 committed 且不违 checkpoint；已提交损坏 → quarantine 禁写 | T-FILE-005/006 | **10 步与 checkpoint 对照未进 AC**；quarantine 有 |
| §20.6 rotation | 64MiB 或 1e6 先到；seal footer；fsync；新 header 引用旧 final head；跨 segment 不断链 | T-FILE-007 | 阈值有；**seal+fsync+header 引用** 未逐条 AC |
| chaos | crash/fault | T-FILE-008 | 有名无用例矩阵（短写/disk full/fsync fail 分列更好） |

### O4-3 — Postgres / Outbox（§21）对照

| Spec 小节 | 要求 | 计划 | 缺口 |
|-----------|------|------|------|
| §21.1 | 资金/持仓/账本/订单与业务同事务所 | 定位隐含 | 无「Tier-A 链」登记与 PG 强制绑定 Task |
| §21.2 表 | evidence_chain_heads / records / outbox / checkpoints | T-PG-002 | 表名有（略称 heads） |
| §21.2 唯一 | (chain_id, sequence) (chain_id, event_id) | 「唯一约束」 | **应写死两对** |
| §21.2 head 行 | chain_id, sequence, head_digest, updated_at | — | **缺** |
| §21.3 直接 append | BEGIN…SELECT head FOR UPDATE…check event_id…expected head…allocate seq…seal…INSERT…UPDATE head…COMMIT；失败整事务回滚 | T-PG-003 | 骨架有；**seal 在事务内**与回滚 AC 可更硬 |
| §21.4 outbox | 业务同事务 insert outbox；dispatcher claim→idempotent append→mark；**durable 前禁删 outbox** | T-PG-004/005 | 有；**claim 协议/租约** 未写 |
| §21.5 并发 | 六类场景；**不允许 sequence gap 或 fork** | T-PG-006 偏窄 | **连接断开、expected_head 竞争** 明文缺 |
| §15.1 C | evidence log 作为 SoT | — | **无** |

### O4-4 — Crash recovery（跨 §20.5 / §21.4 / §33.3）

| 场景 | 计划 | 评估 |
|------|------|------|
| File 半条 frame 崩溃 | T-FILE-005/008 | 方向对；步骤/checkpoint 约束不全 → **计划不完备** |
| File 已提交损坏 | T-FILE-006 quarantine | 有 |
| PG dispatcher 崩溃重试 | T-PG-005 | 有幂等 |
| PG 事务中连接断开 | — | **缺** |
| 恢复后与 signed checkpoint 冲突 | T-CP-004 + T-FILE-005 弱耦合 | **无联合 AC**（§20.5 步骤 2/8） |
| repair-tail CLI | T-CLI-005 | 有；依赖 file recovery 完备 |

### O4-5 — Concurrency

| Adapter | Spec | 计划 | 评估 |
|---------|------|------|------|
| memory | 并发线性化 | T-MEM-005 | 充分方向 |
| file | 单写者；非多写者并发 append | T-FILE-003 | 模型正确；应明确「多 writer 测试 = 第二失败」而非线性化套用 mem |
| postgres | 多进程 append / 幂等重试 / head 竞争 / deadlock / disconnect | T-PG-006 | **不完整** |
| conformance | 三适配器同套 | T-MEM-008 + FILE/PG-009 | 钩子有；**用例列表未冻结** |

### O4-6 — Durability

| 要求 | 计划 | 评估 |
|------|------|------|
| 三态枚举 | T-CORE-022 | 有 |
| 生产默认 Durable | approval A6 | **无实现 Task** |
| memory 拒 Durable | T-MEM-004 | 有 |
| file Durable = fsync 合同完整 | T-FILE-004 | **不完整**（见 O4-2） |
| pg Durable = 事务提交 | T-PG-003 | 隐含 |
| 不得降级 volatile | T-MEM-004 T-ARCH-005 §33.3 | 部分；缺运行时配置防降级测试 |
| Process 语义 | — | **全缺** |
| 性能不得靠降 durability | T-PERF-002 | 有文档契约 |

### O4-7 — DEF 与 adapter 相关

| DEF | todo | 关闭路径 |
|-----|------|----------|
| DEF-007 mock verify | 有 | W2/W6 · MEM+ARCH |
| DEF-008 lock poison | 有 | T-MEM-006 |
| DEF-012 无 durable adapter | 有 | W4 FILE/PG——但 FILE 合同粒度不足，**计划层未真正可关闭** |
| DEF-013 checkpoint/anchor | 有 | W5（非本轮深检） |

**DEF-001…018 全部出现在 todo**（与 R3 一致）→ R-GAP-001 PASS。

---

## false_pass_risks

1. **「T-FILE-002 §20.3」引用式 AC**：执行者勾 DONE 只需「看起来像 segment」，门禁无法核对 magic/双 length/commit marker/segment_digest → **假 PASS 高风险**。
2. **「T-FILE-004 返回前 fsync」**：未要求目录元数据与 head 更新顺序时，掉电后丢最后多条仍可能测绿 → **假 Durable**。
3. **「T-FILE-005 截断未提交」无 checkpoint 约束**：可能截掉已在可信 checkpoint 之后错误判定的帧，或放过违 checkpoint 的自动截断 → **恢复假安全**。
4. **「T-PG-002 四表 + 唯一约束」**：缺 head 列与双唯一键明文时，schema 漂移仍能勾 PASS。
5. **conformance 钩子空心**：`T-MEM-008` DONE 但无冻结用例（含 limit 边界、损坏不静默、Durable 拒绝）→ FILE/PG「同套 PASS」传染假绿。
6. **R-MEM-001 仅 README false**：文档写 production_allowed=false 但 bootstrap 仍装配 memory → 与 §19.3 冲突却可能 R-MEM 字面过。
7. **§33.3「crash recovery 通过」映射 T-FILE-005/008**：chaos 日志有输出 ≠ 覆盖 short write / disk full / fsync errno / 连接断开。
8. **幽灵 `T-ATOM`**：评审者以为原子性有主任务，实际无 ID 可勾 → 台账假完整。

---

## notes

### 本轮 failed_checks 汇总

```text
R-MEM-001
R-FILE-001
R-PG-001
R-APPEND-001
R-ATOM-001
R-READ-001
R-SPEC-003
```

### DEF-001…018

- `.worktrees/evidence-todo.md` §1：**18/18 全登记**（与 gap-matrix §4 一致）。
- 与 adapter 强相关未闭合：DEF-007/008/012（+ 生产向 DEF-013 在 W5）。

### 与 Round 3 交叉

| 主题 | R3 | R4 |
|------|----|----|
| fail-closed | FAIL R-APPEND-002 | 本轮不重复；adapter 成功返回语义仍依赖业务侧 |
| §22 privacy | FAIL 主因 | artifact 与 adapter 正交；不免除 R4 |
| R-ATOM-001 | FAIL | **再次 FAIL**（同一根因） |
| 诚实性 | PASS | PASS |

### 关闭本轮 FAIL 的最低补强（建议）

1. **展开 T-FILE-002/004/005 AC** 为 §20.3–20.5 检查表（字段、双 length、commit marker、目录 fsync、10 步恢复、checkpoint 不变量）。
2. **展开 T-PG-002/006 AC**：head 列、双唯一键、§21.5 六场景（含连接断开与 expected_head 竞争）。
3. **T-MEM**：零摘要 head 禁止；verify 不恒成功；production 阻断含 bootstrap + systemd/archgate。
4. **Durability**：`Process` 语义+测试；生产默认 Durable 的装配任务；禁止配置降级用例。
5. **消灭 T-ATOM via design**：建 `T-ATOM-001`（A/B/C 选择与证明）或拆到 PG + SoT 文档化 DEFER。
6. **冻结 conformance 用例清单** 作为 T-MEM-008 AC，供 FILE/PG 复用。
7. Reader 数值与损坏语义写入 T-MEM-003 并纳入 conformance。

### 诚实声明

- 本轮检查 **计划 vs Spec**，**不**声称 adapters 已实现（现状 ABSENT/WRONG，与 gap-matrix 一致）。
- Spec 仍为 **Proposed**；本轮 FAIL 不授权 stable / §33 勾选。
- 未将「任务行存在」等同于「合同完备」。
