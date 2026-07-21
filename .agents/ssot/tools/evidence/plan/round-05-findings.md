# Round 5 Findings — Testing Contract §24

| 字段 | 值 |
|------|-----|
| Round | **5** |
| Scope | `xhyper-evidence-complete-spec.md` §24.1–§24.11 vs `plan/*` · `tasks.md` · `.worktrees/evidence-todo.md` |
| Code skim | `tools/evidence/src/lib.rs` · `crates/domain/macro/src/lib.rs` · `crates/infra/gate/src/lib.rs` |
| Result | **FAIL** |
| Date | 2026-07-14 |

---

## result

**FAIL** — 计划对 §24 以「章节引用」居多，但 **mutation kill list、五个 fuzz 目标、adapter 覆盖率、checkpoint 属性全集、幂等冲突用例** 未落到可独立验收的 Task/AC；多个关键项被折叠进 `T-CI-002`「文档+workflow **草案**」，存在 **Nightly 静默降级为规划** 风险，不满足 R-TEST-001 / §33.5 的闭合强度。

---

## failed_checks

### FC-R5-01 · §24.1 Golden 内容清单未在 tasks 展开

| Spec 要求 | Tasks 映射 | 判定 |
|-----------|------------|------|
| 目录 `tests/vectors/evidence-v1/` | `T-CORE-026` AC「§24.1 列表全覆盖」；`plan.md` 写路径但注「以落地为准」 | **弱** |
| 向量：empty/genesis, attempted, committed, rejected, failed, cancelled, compensated, all optional present/absent, max name length, event_time before epoch, timestamp boundaries, multi-record chain, checkpoint | **未逐条列为 Task 或 AC 清单** | **FAIL** |
| 每向量：semantic input / canonical hex / record digest / expected chain head | **未写入 AC** | **FAIL** |

`approval-packet.md` A10 仍开放「crate 内 vs 仓库 `tests/vectors`」路径 SSOT → 与 §24.1 固定路径冲突未在 W0 闭合。

### FC-R5-02 · §24.2 Canonical properties 覆盖不全

| Spec 项 | Task | 判定 |
|---------|------|------|
| encode→decode→encode 稳定 | `T-CORE-028` | PASS（有） |
| 字段边界不同语义不同 preimage | `T-CORE-029` 仅强制 `("ab","c")≠("a","bc")` | 部分 |
| trailing bytes 拒绝 | `T-CORE-015` 提到 trailing | 部分 |
| invalid tag 拒绝 | `T-CORE-015` | 部分 |
| **invalid length 拒绝** | 无独立 AC | **FAIL** |
| **truncation 拒绝** | 无独立 AC | **FAIL** |
| **任意输入不 panic** | 无 Task | **FAIL** |

### FC-R5-03 · §24.3 Chain properties 仅章节引用

`T-CORE-030` AC =「§24.3」。未枚举：逐字段篡改 / 删中间 / 重排 / 重复 sequence / sequence gap / 混 chain_id / 错 previous / 错 record digest / event_id duplicate。  
**可验收性不足**（依赖实现者读完整 spec，易漏项）。

### FC-R5-04 · §24.4 Checkpoint properties 映射残缺

| Spec 项 | Task | 判定 |
|---------|------|------|
| 改 checkpoint 任意字段 → 签名/digest 失败 | 无 | **FAIL** |
| 本地链短于 checkpoint → TailTruncated | `T-CP-004` | PASS |
| 链头不一致 → CheckpointMismatch | 无显式 | **FAIL** |
| 旧 key 验证 | 无 | **FAIL** |
| key rotation | `T-CP-006` | PASS |
| 未知 key ID | 无 | **FAIL** |
| 无效 signature | 无 | **FAIL** |

### FC-R5-05 · §24.5 Idempotency 不完整

| Spec 项 | Task | 判定 |
|---------|------|------|
| 同 event_id + 同内容 → 同 receipt | `T-MEM-002`「idempotent」笼统 | 弱 |
| **同 event_id + 不同内容 → IdempotencyConflict** | **无** | **FAIL** |
| 并发同 event_id → 仅一条 | `T-MEM-005` 部分（fork/dup sequence） | 弱 |

### FC-R5-06 · §24.6 并发 conformance 阈值缺失

Spec 要求 `1000+ concurrent append`、多 chain 并行、无 lost append。  
`T-MEM-005` / `T-FILE-009` / `T-PG-007` 未写入 **1000+** 或 **lost append** AC。

### FC-R5-07 · §24.7 Fault 清单未展开

- File：`T-FILE-008` 仅「§24.7」；十类 fault（kill-9 四阶段、disk full、permission、short write、fsync error、corrupt final/middle）**未列表化**。
- Postgres：`T-PG-006` 偏并发/死锁/回滚；**connection drop / serialization failure / dispatcher crash / outbox replay / commit response lost** 无独立 AC。

### FC-R5-08 · §24.8 Fuzz 五目标无实现 Task

Spec 必须 fuzz：

1. canonical decoder  
2. record verifier  
3. segment parser  
4. checkpoint parser  
5. CLI import  

计划仅有：

- `T-CI-002`：nightly mutants/miri/**fuzz 任务规划**（AC=`文档+workflow 草案`）
- `plan.md` 提及 `fuzz_regressions.rs` 目录意图

**无** `T-FUZZ-*` 实现 + corpus 沉淀 + 每个目标的回归用例。  
§33.5 映射把 **fuzz → T-CI-002**，把「闭合」误绑到「规划」。

### FC-R5-09 · §24.9 Coverage 半边缺失

| Spec | Task | 判定 |
|------|------|------|
| core line ≥ 95% | `T-CORE-033` | PASS |
| core **branch ≥ 90%** | 仅 `T-CI-002` 映射表，无实现 AC | **FAIL** |
| production adapters **line ≥ 90%** | **无** | **FAIL** |
| critical recovery paths **= 100% scenario coverage** | **无**（`T-FILE-008` 不等价） | **FAIL** |

### FC-R5-10 · §24.10 Mutation score + **kill list 未入 tasks**

- Core mutation score ≥ 90%：仅 `T-CI-002` 规划 + §33.5 映射。  
- Spec 明确 **不得存活** 的突变（字段顺序 / 长度前缀删除 / 字节序反转 / domain tag 删除 / previous 不参与 hash / sequence 不参与 hash / event_id 幂等删除 / verify 比较反转 / checkpoint 对比删除 / 错误映射降级）：

  → **tasks.md / plan.md / todo 均无 kill list 任务或 AC。**

### FC-R5-11 · §24.11 Miri 范围缩水

Spec：Core **和 memory adapter** 定期 `cargo miri test`。  
计划：`T-CI-002` 笼统 Miri；**无** `evidence_memory` miri 专项；且仍是「草案」非必跑 CI。

---

## omissions

1. **§24.1** 十四类 golden 向量名 + 四元组内容未写入 `T-CORE-026` AC 正文。  
2. **§24.2** invalid length / truncation / no-panic。  
3. **§24.3** 九条 chain property 未枚举（仅章节号）。  
4. **§24.4** CheckpointMismatch、字段篡改、旧 key、未知 key ID、无效签名。  
5. **§24.5** `IdempotencyConflict` 路径。  
6. **§24.6** `1000+` 并发与 no-lost-append 阈值。  
7. **§24.7** File 10 / PG 7 故障场景列表。  
8. **§24.8** 五个 fuzz 目标 + crash→corpus 规则。  
9. **§24.9** adapter line≥90%、recovery scenario 100%、branch 强制门禁。  
10. **§24.10** 完整 mutation kill list。  
11. **§24.11** memory adapter Miri。  
12. `T-CORE-028` 未声明覆盖 trailing/tag/length/truncation/panic 全套 property（与 24.2 脱节）。  
13. `evidence-todo.md` W1 仅摘要「golden · properties」；W5/W6 未展开 §24 细项跟踪。

---

## false_pass_risks

| 风险 | 机制 | 严重度 |
|------|------|--------|
| **章节引用假覆盖** | `T-CORE-026/030`、`T-FILE-008` AC 只写「§24.x」→ DONE 时可能只做子集 | **P0** |
| **Nightly 静默降级** | mutants/miri/fuzz/branch 全进 `T-CI-002`「草案」→ §33.5 可被勾选而机器未跑 | **P0** |
| **Kill list 零跟踪** | mutation score 数字过线但存活关键突变（如 previous 不参与 hash）无人拦 | **P0** |
| **Golden 路径 SSOT 未决** | A10 开放 + `T-CORE-026` 无固定路径 → CLI vectors verify 与仓库路径漂移 | **P1** |
| **Conformance 无统一 AC** | `T-MEM-008` 钩子存在，但 24.5/24.6 具体用例未进 hook 合同 | **P1** |
| **现状代码误读为已测** | 当前 `tools/evidence` 仅有弱 unit（两记录 verify、空链、内容差异）；**无** golden/property/fuzz — gap-matrix §24=PARTIAL/WRONG **准确**，但若把现状 unit 计入「已有测试合同」会假 PASS | **P1** |

---

## notes

### 代码对照（gap 主张准确性）

| Gap / DEF | 代码事实 | 结论 |
|-----------|----------|------|
| DEF-014 无 golden | 仓库无 `tests/vectors/evidence-v1/`；`tools/evidence` 仅 lib 内 unit | **准确** |
| DEF-003 `hash_bytes` | `tools/evidence/src/lib.rs` 公开 `hash_bytes` | **准确** |
| DEF-004 全零 genesis | `chain_head` / 空链 `prev` 使用 `[0u8; 32]` | **准确** |
| DEF-005 公开可变字段 | `EvidenceRecord` 字段全 `pub` | **准确** |
| DEF-007 Mock 恒成功 | `feature = "mock"` 的 `verify_chain` 直接 `Ok(())` | **准确** |
| DEF-002 anyhow | `Cargo.toml` 依赖 `anyhow`；poison 路径使用 | **准确** |
| DEF-006 domain_macro Debug-hash | `format!("{point:?}")` / `{state:?}` → `hash_bytes` | **准确** |
| gate 裸 `hash_bytes(name)` | `gate` `hash_bytes(name.as_bytes())` | **准确**（W3 任务覆盖） |
| DEF-008 lock poison 静默空 | `len`/`chain_head`/`records` 在 poison 时 `unwrap_or(0|default)`；写路径返回 error | **准确**（读路径静默） |

### 已映射尚可保留的项（不构成 PASS）

- Golden 有任务槽：`T-CORE-026/027` + `T-CLI-004`  
- 强制边界回归：`T-CORE-029`  
- Chain 篡改有槽：`T-CORE-030`  
- Crash 有槽：`T-FILE-008`、`T-PG-006`  
- Key rotation：`T-CP-006`；TailTruncated：`T-CP-004`  
- line≥95% 槽：`T-CORE-033`  
- plan 目录意图含 `golden_vectors.rs` / `canonical_properties.rs` / `chain_properties.rs` / `fuzz_regressions.rs`

### 修复建议（仅计划层，本轮不改 tasks）

1. 将 §24.1 十四向量 + 四元组写入 `T-CORE-026` AC 列表；固定 `tests/vectors/evidence-v1/` 或关闭 A10。  
2. 拆分 `T-PROP-001..` / 扩展 `T-CORE-028` 覆盖 24.2 全句。  
3. 拆分 `T-CP-PROP-*` 覆盖 24.4 全集。  
4. 新增 `T-MEM-IDEM` 含 IdempotencyConflict；conformance AC 写 1000+。  
5. 新增 `T-FUZZ-001..005` + corpus；`T-MUT-KILL` 枚举 kill list；`T-MIR-MEM`。  
6. `T-CI-002` 拆成 **实现必跑** vs **nightly 扩量**，禁止「草案」映射 §33.5。  
7. Adapter coverage / recovery 100% 进 `T-FILE-*`/`T-PG-*` 或 `T-COV-ADP`。

### 对照 R-TEST-001

`plan.md` R-TEST-001 = golden/property/fuzz/coverage/mutants/miri → tasks。  
**现状：有名字映射，无完整合同 AC → Round 5 不能 PASS。**
