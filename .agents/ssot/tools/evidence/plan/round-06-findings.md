> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 6 Findings — CLI §25 · Policy §26 · Gates §27 · CI §28

| 字段 | 值 |
|------|-----|
| Round | **6** |
| Scope | `xhyper-evidence-complete-spec.md` §25–§28 vs `plan/*` · `tasks.md` · `.worktrees/evidence-todo.md` |
| Code skim | `tools/evidence`（现状无 CLI/policy/gates）；`tools/archgate` 存在为 member；gap 主张对照 |
| Result | **FAIL** |
| Date | 2026-07-14 |

---

## result

**FAIL** — CLI 七命令有任务槽，但 §25.2 默认行为与 repair-tail 全约束未完整落 AC；§26 字段登记清单未逐项写入；**§27 共 22 个 `EVIDENCE-*` ID 中多枚未映射到独立门禁任务**；§28 命令集与 Nightly 六项大量折叠进 `T-CI-001/002` 且 Nightly 仅「草案」——**不能声称 every command / every EVIDENCE-* ID maps to a task without silent drop。**

---

## failed_checks

### FC-R6-01 · §25.1 命令映射（表面 PASS，细节弱）

| 命令 | Task | 判定 |
|------|------|------|
| `evidence-cli verify` | `T-CLI-002` | PASS（有） |
| `inspect` | `T-CLI-002` | PASS |
| `head` | `T-CLI-002` | PASS |
| `export` | `T-CLI-002` | PASS |
| `checkpoint verify` | `T-CLI-003` | PASS |
| `vectors verify` | `T-CLI-004` | PASS |
| `repair-tail` | `T-CLI-005` | PASS（有） |

命令 **均有 Task ID**；但 `T-CLI-002` 四命令合一，AC 仅「只读默认」——见下节失败。

### FC-R6-02 · §25.2 默认行为未逐条 AC

| Spec | Task AC | 判定 |
|------|---------|------|
| 只读 / 不修改链 | `T-CLI-002`「只读默认」 | 部分 |
| human-readable + `--json` | **无** | **FAIL** |
| JSON **不是** canonical | **无** | **FAIL** |
| 错误写 stderr | **无** | **FAIL** |
| 敏感信息不输出 | **无** | **FAIL** |
| 支持 chain 与 sequence 范围 | **无** | **FAIL** |

### FC-R6-03 · §25.3 退出码

`T-CLI-006` AC = `0/2/3/4/5/6/7` 对照 §25.3 → **PASS**（列表完整）。

### FC-R6-04 · §25.4 repair-tail 约束不完整

| Spec 约束 | `T-CLI-005` | 判定 |
|-----------|-------------|------|
| 仅最后一个未提交、无 commit marker 的不完整 frame | 仅「§25.4」引用 | **弱/FAIL** |
| 执行前只读验证 | **无** | **FAIL** |
| 生成 repair plan | **无** | **FAIL** |
| 用户显式确认 | 「显式+确认」 | PASS |
| 备份原文件 | 「备份」 | PASS |
| 输出 repair evidence | **无** | **FAIL** |
| **不得跨越可信 checkpoint** | **无** | **FAIL** |

### FC-R6-05 · §26 Policy 结构与必填字段

| Spec | Task | 判定 |
|------|------|------|
| 文件 `.architecture/evidence-policy.toml` | `T-POL-001` | PASS |
| 示例 chain + operation（namespace/producer/advance/restore 等） | `T-POL-001`「schema_version + 示例」 | 弱（未钉死示例字段） |
| 每个 required operation 登记 12 项：producer, operation, subject strategy, chain strategy, actor strategy, input canonical domain, output/error canonical domain, atomicity, durability, checkpoint policy, retention, owner | `T-POL-002`「policy 完整字段」 | **弱** — **12 字段未枚举进 AC** |
| `T-DOM-004` macro chain 登记 | 有 | PASS（槽） |
| gap DEF-015 无 policy | 代码树现状无该文件 | gap **准确** |

### FC-R6-06 · §27.1 Core 门禁 ID 映射

| ID | Task | 判定 |
|----|------|------|
| EVIDENCE-PATH-001 | `T-ARCH-001` | PASS |
| EVIDENCE-DEP-001 | `T-ARCH-002` | PASS |
| EVIDENCE-DEP-002 | `T-ARCH-002` | PASS |
| EVIDENCE-ANYHOW-001 | `T-ARCH-003` | PASS |
| **EVIDENCE-CANONICAL-001** | `T-ARCH-004` 文案为 DOMAIN/DEBUG/JSON/GENESIS/PUBLIC — **未列 CANONICAL** | **FAIL** |
| EVIDENCE-DOMAIN-001 | `T-ARCH-004` | PASS |
| EVIDENCE-DEBUG-HASH-001 | `T-ARCH-004` | PASS |
| EVIDENCE-JSON-HASH-001 | `T-ARCH-004` | PASS |
| EVIDENCE-GENESIS-001 | `T-ARCH-004` | PASS |
| EVIDENCE-PUBLIC-001 | `T-ARCH-004` | PASS |

### FC-R6-07 · §27.2 Adapter 门禁 ID 映射

| ID | Task | 判定 |
|----|------|------|
| **EVIDENCE-DURABILITY-001** | 无 T-ARCH；仅实现侧 `T-MEM-004` 等 | **FAIL（门禁未机控）** |
| EVIDENCE-MEMORY-PROD-001 | `T-ARCH-005` | PASS |
| **EVIDENCE-IDEMPOTENCY-001** | 无 T-ARCH；实现 `T-MEM-002` | **FAIL** |
| **EVIDENCE-CONCURRENCY-001** | 无 T-ARCH；实现 `T-MEM-005` | **FAIL** |
| **EVIDENCE-RECOVERY-001** | 无 T-ARCH；实现 `T-FILE-005/008` | **FAIL** |
| **EVIDENCE-FSYNC-001** | 无 T-ARCH；实现 `T-FILE-004` | **FAIL** |

> 实现任务 ≠ 机器门禁任务。§27 要求 **fail 规则**；tasks 把 adapter 五条压成实现测试，**未进 `T-ARCH-*` / archgate 规则清单**。

### FC-R6-08 · §27.3 系统门禁 ID 映射

| ID | Task | 判定 |
|----|------|------|
| EVIDENCE-POLICY-001 | `T-ARCH-006`（POLICY…） | PASS |
| EVIDENCE-COVERAGE-001 | `T-ARCH-006`（注：运维路径覆盖 ≠ line coverage） | PASS（名） |
| EVIDENCE-ATOMICITY-001 | `T-ARCH-006`；§33.6 另有「T-ATOM via design」幽灵引用 | **弱** |
| EVIDENCE-CHECKPOINT-001 | `T-ARCH-006` | PASS |
| **EVIDENCE-ANCHOR-001** | `T-ARCH-006` 文案 **未列 ANCHOR**；`T-CP-005` 仅合同接口 | **FAIL** |
| **EVIDENCE-SCHEMA-001** | **无** | **FAIL** |
| **EVIDENCE-VECTOR-001** | **无**（golden 有实现任务，无「漂移须 RFC」门禁） | **FAIL** |

**§27 未映射/弱映射门禁 ID 合计：CANONICAL, DURABILITY, IDEMPOTENCY, CONCURRENCY, RECOVERY, FSYNC, ANCHOR, SCHEMA, VECTOR（≥9）。**

### FC-R6-09 · §28 Core CI 命令

| 命令 | 任务映射 | 判定 |
|------|----------|------|
| `cargo fmt -- --check` | 无 evidence 专属；依赖全局 CI 习惯 | **弱** |
| `cargo clippy -p evidence --all-targets -- -D warnings` | `T-CORE-031` 部分；非 `T-CI-001` 枚举 | **弱** |
| `cargo test -p evidence` | `T-CI-001`「§28」笼统 | 弱 PASS |
| `cargo llvm-cov -p evidence --fail-under-lines 95` | `T-CORE-033` | PASS（有） |
| `cargo mutants -p evidence` | `T-CI-002` **规划草案** | **FAIL（非必跑实现）** |
| `cargo miri test -p evidence` | `T-CI-002` 草案 | **FAIL** |
| `cargo run -p archgate -- --json` | `T-ARCH-*` 间接；**无显式 CI 命令 Task** | **FAIL** |
| `cargo run -p xtask -- lint-deps` | plan W7 示例有；`T-CI-001` 未写 | **弱** |
| `cargo run -p xtask -- crate-standard --check` | **tasks/plan 验证命令块均未列入** | **FAIL** |

### FC-R6-10 · §28 Adapter / 工具命令

| 命令 | 映射 | 判定 |
|------|------|------|
| `cargo test -p evidence_memory` | `T-CI-001` 笼统 | 弱 |
| `cargo test -p evidence_file` | 同上 | 弱 |
| `cargo test -p evidence_postgres` | 同上 | 弱 |
| `cargo test -p evidence_file --test crash_recovery` | **无**（仅 `T-FILE-008` 实现测，非 CI 入口名） | **FAIL** |
| `cargo test -p evidence_postgres --test concurrency` | **无** | **FAIL** |
| `cargo run -p evidence-cli -- vectors verify` | `T-CLI-004` | PASS |
| `cargo run -p evidence-cli -- verify <fixture>` | `T-CLI-002` | 弱 |

### FC-R6-11 · §28 Nightly — **不可静默丢弃**

| Nightly 项 | 映射 | 判定 |
|------------|------|------|
| full mutation | `T-CI-002` 草案 | **风险：仅规划** |
| full fuzz corpus | `T-CI-002` 草案 | **同上** |
| Miri | `T-CI-002` 草案 | **同上** |
| adapter chaos | `T-FILE-008`/`T-PG-006` 测试意图；**无 nightly job** | **FAIL** |
| checkpoint key rotation test | `T-CP-006` 有测试任务；CI nightly 未接 | **弱** |
| historical schema compatibility test | `T-LEG-002` + `T-CI-002` | **弱** |

`plan.md` 实现后命令块写「`# + coverage / mutants / miri per §28`」——注释级，**非 Task AC**。  
`evidence-todo.md` W6 仅 `T-CI-001…002`，**未声明 Nightly 为 DEFER 或必做** → **静默丢弃风险成立**。

---

## omissions

### CLI §25

1. `--json` / JSON≠canonical / stderr / 敏感信息 / range 过滤。  
2. repair-tail：只读验证、repair plan、repair evidence、禁止跨越 checkpoint、仅未提交尾帧。  
3. 各子命令独立验收（四命令挤在 `T-CLI-002`）。

### Policy §26

4. required operation **12 字段** 未写入 `T-POL-002` AC。  
5. 示例 TOML 中 `checkpoint_max_records/seconds`、`criticality`、`writer` 等未要求落盘校验。  
6. `EVIDENCE-POLICY-001` 与 policy 文件 schema 校验的机控方式未写（archgate 规则？）。

### Gates §27

7. **EVIDENCE-CANONICAL-001** 从 `T-ARCH-004` 名单脱落。  
8. Adapter：**DURABILITY / IDEMPOTENCY / CONCURRENCY / RECOVERY / FSYNC** 无 arch 任务。  
9. System：**ANCHOR / SCHEMA / VECTOR** 无 arch 任务。  
10. `T-ARCH-006` 把四条揉在一起且漏 ANCHOR；`T-ATOM via design` 为 **幽灵 Task ID**（tasks 表无 `T-ATOM`）。

### CI §28

11. `crate-standard --check` 完全缺失。  
12. `archgate --json` 无 CI 接线 Task。  
13. `evidence_file --test crash_recovery` / `evidence_postgres --test concurrency` 入口名未登记。  
14. Nightly 六项无 workflow 必跑定义；`T-CI-002` AC 停在「草案」。  
15. `cargo fmt` / `lint-deps` 未进入 `T-CI-001` 显式命令列表。  
16. `plan.md` §4.2 命令集与 §28 不完全对齐（缺 llvm-cov/mutants/miri/archgate/crate-standard/crash_recovery/concurrency）。

---

## false_pass_risks

| 风险 | 机制 | 严重度 |
|------|------|--------|
| **「§28」单点引用** | `T-CI-001` AC 只写「§28」→ 可勾 DONE 而漏 half 命令 | **P0** |
| **Nightly=草案** | mutants/miri/full fuzz 永不进 CI 仍可通过规划 Task | **P0** |
| **实现测冒充门禁** | IDEMPOTENCY 等有 adapter 测但无 `EVIDENCE-*-001` 机控 → 回归时规则不 fail | **P0** |
| **CANONICAL 门禁漏名** | 规范有 ID、tasks 未列 → cutover 后仍可用非 V1 hash | **P0** |
| **VECTOR/SCHEMA 无门** | golden 漂移或 schema 原地改可无 RFC 合并 | **P1** |
| **T-ATOM via design** | §33.6 映射指向不存在 Task → 原子性闭合假完成 | **P1** |
| **repair-tail 过宽** | 未禁越 checkpoint / 未限未提交帧 → 危险写路径被当「已合同」 | **P1** |
| **PATH-001 依赖 cutover 顺序** | `T-ARCH-001` 依赖 `T-CUT-002`：cutover 前规则无法绿，易被跳过登记 | 信息性 |

---

## notes

### 每个 EVIDENCE-* → Task 总表（审计用）

| EVIDENCE-* | 有实现/相关 Task？ | 有门禁 Task？ | Round6 |
|------------|-------------------|---------------|--------|
| PATH-001 | T-CUT-002 | T-ARCH-001 | OK |
| DEP-001/002 | T-CORE-002 | T-ARCH-002 | OK |
| ANYHOW-001 | T-CORE-002 | T-ARCH-003 | OK |
| CANONICAL-001 | T-CORE-014..016 | **缺名** | **FAIL** |
| DOMAIN-001 | T-CORE-018 T-CUT-004 | T-ARCH-004 | OK |
| DEBUG-HASH-001 | T-DOM-001 | T-ARCH-004 | OK |
| JSON-HASH-001 | （依赖实现纪律） | T-ARCH-004 | OK |
| GENESIS-001 | T-CORE-017 | T-ARCH-004 | OK |
| PUBLIC-001 | T-CORE-012/035 | T-ARCH-004 | OK |
| DURABILITY-001 | T-MEM-004 等 | **无** | **FAIL** |
| MEMORY-PROD-001 | T-MEM-007 | T-ARCH-005 | OK |
| IDEMPOTENCY-001 | T-MEM-002 | **无** | **FAIL** |
| CONCURRENCY-001 | T-MEM-005 | **无** | **FAIL** |
| RECOVERY-001 | T-FILE-005/008 | **无** | **FAIL** |
| FSYNC-001 | T-FILE-004 | **无** | **FAIL** |
| POLICY-001 | T-POL-002 | T-ARCH-006 | OK |
| COVERAGE-001 | T-DOM-006 | T-ARCH-006 | OK |
| ATOMICITY-001 | T-PG-004 | T-ARCH-006 / 幽灵 T-ATOM | 弱 |
| CHECKPOINT-001 | T-CP-003 | T-ARCH-006 | OK |
| ANCHOR-001 | T-CP-005 | **无** | **FAIL** |
| SCHEMA-001 | T-LEG-* 弱 | **无** | **FAIL** |
| VECTOR-001 | T-CORE-026/027 | **无** | **FAIL** |

### 代码 / 仓库现状（gap 准确性）

| 主张 | 证据 | 结论 |
|------|------|------|
| 无 evidence-cli | 无 `tools/evidence-cli` member（仅 plan 目标） | gap §25 ABSENT **准确** |
| 无 evidence-policy.toml | `.architecture/` 未在本轮见该文件；T-POL-001 TODO | gap §26 **准确** |
| 无 EVIDENCE-* 机控 | tasks 全 TODO；archgate 存在但非 evidence 规则完成证据 | gap §27 **准确** |
| CI 无 002 专用门禁 | 现状 workspace CI 测旧 `tools/evidence` unit | gap §28 PARTIAL/WRONG **准确** |
| archgate 包存在 | 根 `Cargo.toml` members 含 `tools/archgate` | plan 依赖路径 **合理** |
| crate-standard | §28 要求；tasks 未跟踪 | **计划缺口**（非代码 bug） |

### 修复建议（计划层）

1. `T-CLI-002` 拆分或扩展 AC：§25.2 六条 + 每命令 smoke。  
2. `T-CLI-005` 枚举 §25.4 全部前置条件与禁越 checkpoint。  
3. `T-POL-002` AC 粘贴 12 字段清单。  
4. `T-ARCH-004` **显式加入 CANONICAL-001**；新增 `T-ARCH-007`（adapter 五门禁）+ `T-ARCH-008`（ANCHOR/SCHEMA/VECTOR）。  
5. `T-CI-001` AC = §28 命令逐条 checklist（含 crash_recovery / concurrency / archgate / lint-deps / crate-standard / fmt / clippy / llvm-cov）。  
6. `T-CI-002` 改为「Nightly workflow **落地**」：full mutation / fuzz corpus / miri / adapter chaos / key rotation / historical schema；若环境不可用必须 **正式 DEFER ID**，禁止用「草案」顶 §33.5。  
7. 删除或创建真实 `T-ATOM-*`，去掉「via design」幽灵引用。

### 与 Round 5 交叉

- Nightly mutants/miri/fuzz 在 R5/R6 **双重 FAIL**（测试合同 + CI 接线）。  
- `EVIDENCE-VECTOR-001` 依赖 R5 golden 路径 SSOT；两端都开则双重假 PASS。  
- `EVIDENCE-COVERAGE-001`（业务路径）勿与 §24.9 line coverage 混淆——计划未澄清命名碰撞。

### Round 6 总判

**every command mapped?** 七 CLI 命令有 Task；§28 shell 命令 **否**。  
**every EVIDENCE-* mapped to gate task?** **否（≥9 缺口）**。  
**Nightly not silently dropped?** **否 — 现态为草案折叠，等同静默降级。**  
→ **result: FAIL**
