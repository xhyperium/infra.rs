# GOAL-GOALCTL-002：goalctl 生产级 Goal Delivery OS 完整可执行目标

```text
Document ID:        GOAL-GOALCTL-002
Document Type:      Executable Goal
Status:             PROPOSED
Target Package:     xhyper-goalctl
Implementation:     tools/goalctl
Specification Root:.agents/ssot/tools/goalctl
Baseline Date:      2026-07-16
Baseline Commit:    db5eaee02662fe19dcb3b61a3d7b1390076adb70
Baseline Version:   0.1.0（Phase 1 read-only MVA）
Owners:             Platform / Tooling + Governance
Required Review:    Security（涉及 trust、approval、runner、bootstrap 时）
Supersedes:         不自动替代既有 Goal-终态目标.md；批准后再通过 CR 建立替代关系
Authority Inputs:   CONSTITUTION.md
                    docs/goal/00-authority-map.md
                    docs/goal/schema/authority-policy.yaml
                    DECISION-PACK-001
                    CR-20260716-goalctl-foundation
                    CR-20260716-goalctl-impl-phase1
```

> 本文是可执行目标，不是批准记录。任何扩大 Phase 1 只读范围、接入 Agent、写入 Evidence、启用 required CI 或 Cutover 的行为，仍须独立 CR 与审批。

---

## 0. 执行裁定

`goalctl` 的长期定位不是“又一个 CLI”，而是 **infra.rs 的 Goal Delivery OS 编译与验证内核**：

```text
Human Intent / Risk Boundary
          ↓
Authority Resolution
          ↓
Artifact Normalization
          ↓
Fact Observation + Reconciliation
          ↓
Executable Task Compilation
          ↓
Isolated Harness Execution
          ↓
Evidence Collection
          ↓
Independent Verification
          ↓
Deterministic Gate
          ↓
PR / Merge / Release
          ↓
Runtime Observation
          ↓
Failure Corpus / Eval / Harness Improvement
```

最终系统必须做到：

1. 不依赖聊天上下文，也能重建当前权威、真实状态、可执行任务和验收方法；
2. 不信任 Writer 自述，不用目录存在、文件存在或 Markdown 状态替代事实证明；
3. 所有结论绑定不可变 Git subject，任何 commit/tree/policy/evidence 变化都会使旧结论失效；
4. 所有权限、路径、能力、资源和副作用均采用 fail-closed 合同；
5. `goalctl` 不成为第二套 SSOT，不自创 G0–G11，不自证自身可信；
6. 从 Shadow 到 Cutover 必须有量化门槛、回滚演练和独立审批；
7. 每次失败进入 Failure Corpus，并持续改善 Schema、Eval、Harness、Policy 和 Agent Skill。

---

# 一、问题的底层本质

## 1.1 真正问题

AI 工程交付的瓶颈不是代码生成速度，而是：

```text
意图不可计算
+ 权威互相冲突
+ 状态声明与事实脱节
+ 任务边界不可执行
+ 验证不可重复
+ Evidence 不可追溯
+ 权限与执行环境不可控
```

因此，`goalctl` 解决的不是“如何让 AI 写得更快”，而是：

> **如何把人类目标编译成受限、可执行、可验证、可审计、可回滚的工程状态机。**

## 1.2 当前基线事实

截至基线提交，仓库已经具备：

- `xhyper-goalctl 0.1.0`；
- `version / doctor / index / resolve / artifact / reconcile / compile`；
- Authority rank 从 `docs/goal/schema/authority-policy.yaml` 加载；
- `.config/goal` fail-closed；
- `resolve` 从 committed subject 读取 policy 与 authority；
- `index` 对 dirty Cargo manifest fail-closed，并拒绝非 HEAD 的 live `cargo metadata`；
- Task Pack 基础 scope、P0/P1 验证、protected path approval-ref 存在性检查；
- canonical JSON 的稳定 key 排序；
- CLI smoke 与若干负例测试。

当前仍不是生产级 Goal Delivery OS，主要原因如下。

## 1.3 当前结构性缺口

| ID | 严重度 | 缺口 | 结果风险 |
|---|---:|---|---|
| GAP-001 | P0 | `artifact` 读取 live worktree，却用 HEAD `source_commit` 标记 | 未提交内容可污染 artifact snapshot |
| GAP-002 | P0 | `reconcile` 读取 live 文件，并以目录/文件存在推导 VERIFIED、OK | 形成高风险假阳性 |
| GAP-003 | P0 | `compile --task-file` 未强制 `source_commit == bound tree` | 可产生 commit A + tree B 的伪快照 |
| GAP-004 | P0 | 默认 compile 不是从 Goal/Spec/Plan/Task 编译，而是生成通用模板 | “编译器”无法证明任务来源与完整性 |
| GAP-005 | P0 | `approval_refs` 只校验非空，不验证 ApprovalRecord 内容 | 任意字符串可绕过 protected asset 条件 |
| GAP-006 | P0 | Evidence 未做 subject、digest、freshness、producer trust 校验 | VERIFIED/RELEASED 无可信事实基础 |
| GAP-007 | P1 | CLI 合同声明 `--trust-level`，当前 CLI 未实现 | 合同与实现漂移 |
| GAP-008 | P1 | `--source-commit` 只完整覆盖部分命令 | 各命令读取视图不一致 |
| GAP-009 | P1 | Rust 手写模型/验证器与 JSON Schema 无自动一致性证明 | Schema 漂移 |
| GAP-010 | P1 | canonical JSON 仅排序 key，不是完整 canonicalization 规范 | 跨语言 digest 可能不一致 |
| GAP-011 | P1 | Repository Identity 长期为 DEGRADED | 不能作为发布 Evidence 链头 |
| GAP-012 | P1 | Artifact module filter 使用 substring 逻辑 | `goal` / `goalctl` 等边界可能误命中 |
| GAP-013 | P1 | 路径/glob 语义在多个模块重复实现 | scope、policy、artifact 语义可能不一致 |
| GAP-014 | P1 | 现有 README、Goal、Spec、Version Matrix 存在过时事实 | Agent 读取后可能采取错误动作 |
| GAP-015 | P2 | 无 Bootstrap Trust Root | `goalctl`、policy、schema 可同 PR 自我证明 |
| GAP-016 | P2 | 无 Harness / Evidence / Verifier / Shadow Diff | 无法安全进入自治与 Cutover |
| GAP-017 | P2 | 无 SLO、成本预算、Failure Corpus、历史回放 | 无法形成 Self-improving 闭环 |

---

# 二、不可再拆解的基本真理

1. **模型输出不是事实。** 事实必须来自可验证的 Git、测试、Evidence、Registry、Release 或 Runtime Observation。
2. **可变工作区不能证明不可变 subject。** 任何标记 `source_commit` 的输出，必须只读取该 commit/tree，或明确标记为 `LIVE_UNCOMMITTED` 且禁止进入 Gate。
3. **状态不是单一字符串。** Specification、Implementation、Verification、Release、Operations 必须独立表达。
4. **目录存在不等于验证通过。** `evidence/` 目录、`tests/` 目录、README、CHANGELOG 均不能独立生成 VERIFIED、RELEASED 或 OK。
5. **权限必须是机器合同。** Prompt 中的“不要修改”不是权限控制。
6. **Writer 不能扩大 Task Pack，也不能成为唯一 Verifier/Approver。**
7. **同一输入必须产生同一输出。** wall clock、随机数、绝对本机路径、未排序集合不能进入可摘要对象。
8. **Evidence 必须绑定 subject。** branch 名、PR 号、文件路径本身都不足以证明内容。
9. **批准是可验证事实，不是 Markdown 单词。**
10. **规则解释器不能成为规则源。** `goalctl` 只解析经批准的 policy/schema。
11. **Fail-open 会累积隐性债务。** 不确定时必须 `NOT_PROVEN`、`BLOCKED` 或非零退出。
12. **Cutover 是治理事件，不是代码合并的自然结果。**

---

# 三、被误认为真理的常见假设

| 常见假设 | 裁定 |
|---|---|
| Phase 1 命令已存在，因此系统已经“完成” | 错。0.1.0 只证明最小命令面存在 |
| `Status: Approved` 就是批准 | 错。必须验证 ApprovalRecord、subject digest、scope、状态和有效期 |
| 测试目录存在就代表 VERIFIED | 错。必须有执行记录、退出码、环境和 subject |
| CHANGELOG 存在就代表 RELEASED | 错。必须有 release manifest、tag/registry fact、provenance |
| `source_commit` 字段存在就完成绑定 | 错。所有输入必须实际从该 commit/tree 读取 |
| key 排序就是跨语言 canonical JSON | 不完整。还需数值、Unicode、转义、数组语义和版本 |
| 更强模型能弥补规格缺口 | 错。更强模型会更快放大模糊边界 |
| 多 Agent 自动提高吞吐量 | 错。没有 Task DAG、Lease、Fencing 和独立验证时只会增加冲突 |
| required CI 可直接切到新工具 | 错。必须先 Shadow、Mirror、差异解释和回滚演练 |
| 任何 approval ref 都足够 | 错。引用本身不等于有效批准 |
| live worktree 更方便，所以可用于快照 | 仅可用于开发诊断，不可用于发布级结论 |

---

# 四、可以被打破的限制

1. **“Markdown 只能给人读”**：通过唯一 Control Block、Schema Registry 和 generated types 转为机器输入。
2. **“每个 Agent 都要加载全仓上下文”**：通过 Authority Snapshot、Task Pack 和 Context Pack 最小化上下文。
3. **“CI 必须每次全量执行”**：通过确定性影响分析、Evidence 缓存和 subject digest 重用安全缩短执行。
4. **“人工 review 是主要吞吐瓶颈”**：用 Review Bundle 把人工工作收敛为风险裁决，而不是重做机器检查。
5. **“失败只能修当前 PR”**：失败进入 Failure Corpus，自动生成 regression、eval 和 policy/harness 改进。
6. **“规则只能散落在脚本中”**：把 policy/schema/diagnostic/version capability 统一治理。
7. **“工具必须一次性替换旧链路”**：Shadow → Mirror → Cutover → Sunset 渐进迁移。

不可被打破的限制：

- 不创建 `.config/goal`；
- 不让 Writer 自批；
- 不允许旧 Evidence 自动继承到新 tree；
- 不允许未批准的 protected asset 变更；
- 不允许 `goalctl` 自写 G0–G11 PASS；
- 不允许未经独立 CR 的 required CI Cutover；
- 不允许不可信 Fork 获得 Secrets、GitHub write 或特权 runner。

---

# 五、从零设计的新方案

## 5.1 终态系统分层

```text
L0 Bootstrap Trust
  - goalctl-bootstrap-verify
  - trusted keys / policy digest / schema bundle digest

L1 Deterministic Core
  - repository snapshot
  - identity
  - canonicalization
  - schema registry
  - path semantics
  - diagnostics

L2 Goal Compiler
  - authority resolver
  - artifact parser/index
  - fact observer
  - reconciler
  - task/prompt compiler

L3 Execution Control
  - capability policy
  - immutable worktree
  - sandbox
  - lease/fencing
  - budget/cancellation/recovery

L4 Proof System
  - evidence collector
  - review bundle
  - append-only audit chain
  - independent verifier
  - gate adapter

L5 Delivery Integration
  - GitHub PR adapter
  - CI shadow/mirror/cutover
  - release manifest/provenance
  - runtime observations

L6 Self-improving
  - failure corpus
  - historical replay
  - eval suites
  - policy mutation testing
  - skill/harness improvement
```

## 5.2 核心纵向闭环

```text
RepositoryIdentity
+ RepositorySnapshot(commit, tree)
+ AuthoritySnapshot
+ ArtifactIndex
+ FactSet
        ↓
ReconciliationReport
        ↓
TaskPack + ValidationPlan + CapabilityGrant
        ↓
HarnessRun
        ↓
EvidenceManifest + ReviewBundle
        ↓
VerifierReport
        ↓
GateDecision
        ↓
ShadowDiff / Cutover Decision
```

任何一个上游 digest 改变，下游对象必须重新计算或标记 STALE。

## 5.3 十二维终态能力

| 维度 | 终态结果 |
|---|---|
| G0 Bootstrap | 独立验证 binary、policy、schema 与签名 |
| G1 Identity | 仓库稳定 ID、迁移、Fork 隔离 |
| G2 Snapshot | commit/tree/submodule/LFS/sparse checkout 全绑定 |
| G3 Authority | policy 驱动、scope-aware、approval-aware、可撤销 |
| G4 Artifact | Strict/Mixed/Legacy，迁移可测，Legacy 不产 PASS |
| G5 Fact | Git/CI/Release/Runtime observer 产生结构化事实 |
| G6 Reconcile | 五维状态、freshness、冲突、降级、NOT_PROVEN |
| G7 Compile | 真实 Goal→Spec→Plan→Task 编译和 trace coverage |
| G8 Harness | 隔离、能力、资源、Lease、Fencing、恢复 |
| G9 Evidence | Review Bundle + Audit Chain，subject-bound |
| G10 Verify/Gate | 独立 verifier，适配而不重写 G0–G11 |
| G11 Improve | Failure→Eval→Policy/Harness/Skill→Replay 闭环 |

---

# 六、目标结果与完成定义

## 6.1 北极星目标

```text
Verified Accepted Changes / Human Review Minute
```

该指标不能通过降低 Gate 或弱化 Evidence 提升。

## 6.2 必须自动回答的问题

系统必须以结构化输出回答：

1. 当前仓库身份是什么，可信度是多少？
2. 本次读取的是哪个 commit/tree？
3. 当前 Authority Policy、Schema Bundle 的 digest 是什么？
4. 某模块的有效 Goal/Spec/Plan/Task 是哪些？
5. 哪些声明冲突，冲突的事实来源是什么？
6. Specification / Implementation / Verification / Release / Operations 各自状态是什么？
7. 哪些状态只是叙述，哪些是当前 subject 的事实？
8. Task 允许和禁止修改什么？
9. 每个 P0/P1 AC 由什么验证命令或 verifier 覆盖？
10. 是否触碰 protected asset，批准是否有效？
11. Evidence 是否新鲜、完整、可重放？
12. 当前结果是否允许进入 PR、Merge、Release？

## 6.3 终态 Definition of Done

同时满足下列条件才允许把 Goal 标记为 ACHIEVED：

- 所有 subject-bound 命令只读取 committed snapshot；
- 任何 live 模式输出均明确标记 `non_authoritative=true`；
- `compile` 真实读取并验证 Goal/Spec/Plan/Task，不生成虚构 trace；
- ApprovalRecord 完整验证，不接受仅非空 ref；
- Reconcile 不再使用目录存在推导 VERIFIED/RELEASED/OK；
- Schema 与 Rust 类型/输出有自动一致性测试；
- canonicalization 有跨语言 golden vectors；
- RepositoryIdentity 为 FULL，或 enforcing mode 明确拒绝 DEGRADED；
- Evidence 绑定 commit/tree/policy/schema/task/runner；
- Writer 与 Verifier 隔离；
- Shadow 与 Legacy 差异 100% 可解释；
- 回滚演练通过；
- required CI Cutover 有独立 Approved CR；
- 30 天运行无 P0 false negative；
- Failure Corpus 与 replay 自动运行；
- 旧工具 Sunset 有 owner、删除日期和历史读取策略。

---

# 七、范围、非目标与禁止项

## 7.1 本 Goal 范围

- `tools/goalctl/**`
- `.agents/ssot/tools/goalctl/**`
- `docs/goal/schema/**`
- goalctl 相关 CR / ADR / CI integration
- 与 `xhyper-evidence`、`tools/evidence-cli` 的只读/写入接口合同
- 必要的 `tools/xtask`、crate-standard 接线（monorepo 历史另含 archgate；**infra.rs 不移植 archgate**）
- Shadow/Mirror/Cutover 所需工作流

## 7.2 永久非职责

- 生产交易执行；
- 通用部署平台；
- 通用工作流调度器；
- 取代 `docs/goal` 或 `.agents/ssot`；
- 自创 Goal Gate 编号；
- 自动批准 Constitution；
- 将模型判断直接当 Gate 事实。

## 7.3 延期能力

- Agent Writer；
- Draft PR 自动创建；
- GitHub write；
- required CI；
- release/deploy adapter；
- 多仓库编排；
- 远程 runner attestation。

## 7.4 明确禁止

- `.config/goal`；
- Writer 修改 Task Pack、ApprovalRecord、Gate 或自身 CapabilityGrant；
- 不可信 Fork 使用 Secrets、特权 self-hosted runner、GitHub write；
- 只用 branch 名绑定 Evidence；
- 旧 commit Evidence 自动继承到新 tree；
- 用 Legacy narrative 独立产生 VERIFIED/RELEASED；
- 以“目录存在”产生验证事实；
- 同一 PR 无额外审批同时修改 bootstrap verifier、trust policy 与 signing key；
- 未经 Cutover CR 替换 `just goal-check` required 语义。

---

# 八、AI / 自动化 / 研究增强介入位置

## 8.1 AI 适合承担

- 将自然语言 Goal 草案转换为结构化候选 Artifact；
- 检测 Goal/Spec/Plan/Task trace 缺口；
- 生成候选 AC 和反例；
- 根据失败日志提出最小修复；
- 对 diff 做独立 adversarial review；
- 聚类 Failure Corpus；
- 生成新的 regression/eval；
- 发现文档陈旧、冲突和 orphan artifact。

## 8.2 AI 不得独立承担

- Authority 最终裁决；
- Approval；
- Protected Asset 放行；
- Evidence 新鲜度事实；
- Gate 最终 PASS；
- Secrets/production capability 授权；
- bootstrap trust 验证；
- required CI Cutover。

## 8.3 AutoResearch

每次重大 Spec/Policy 变更应自动执行：

```text
Repository archaeology
→ Existing contract extraction
→ Source/test/schema differential
→ Historical failure search
→ External standard comparison
→ Negative-case synthesis
→ Proposed change + evidence map
```

研究产物必须记录来源、commit、结论置信度和未解决问题。

---

# 九、可复利增长的系统架构

## 9.1 复利飞轮

```text
失败
→ 标准化 Failure Corpus
→ 归因：Goal / Spec / Policy / Code / Harness / Model / Environment
→ 生成 regression / mutation / replay
→ 改进 Schema、Policy、Harness、Skill
→ 历史回放
→ 通过后进入基线
→ 后续同类失败成本下降
```

## 9.2 Harness 复用

每个验证能力以统一 Harness Contract 暴露：

```text
input_digest
environment_digest
command
cwd
capabilities
budget
timeout
expected_artifacts
exit semantics
redaction policy
```

同一 Harness 可被本地、CI、Verifier 和历史回放复用。

## 9.3 Compound Engineering 资产

- canonical test vectors；
- path semantics corpus；
- policy mutation suite；
- approval negative corpus；
- artifact migration fixtures；
- reconciliation conflict corpus；
- TaskPack property tests；
- evidence freshness fixtures；
- runner trust fixtures；
- shadow diff history；
- failure taxonomy；
- version compatibility matrix。

这些资产必须随每次缺陷增长，而不是只修当前代码。

---

# 十、最小可行行动（MVA）

本 Goal 的首个 MVA 不是接 Agent，而是完成 **Phase 1.1 Truth Hardening**：

```text
MVA = 所有 read-only 输出都与同一 commit/tree 一致，
      且 reconcile/compile 不产生可证明的假阳性。
```

MVA 必须完成：

1. 新建统一 `RepositoryView`：
   - `CommittedView { commit, tree_id }`
   - `LiveView { dirty_digest, non_authoritative=true }`
2. `artifact / reconcile / compile` 支持 `--source-commit` 并使用 `CommittedView`；
3. compile 强制 `request.source_commit == repository_snapshot.commit`；
4. compile 强制 tree 与 commit 的实际 tree 一致；
5. 删除目录存在 → VERIFIED/OK 的推导；
6. ApprovalRecord 解析与 digest/scope/status 校验；
7. 修复 `--trust-level` 合同漂移；
8. 更新所有陈旧文档；
9. 为上述行为增加负例、property 和 golden tests。

MVA 完成后建议版本：`0.1.1`，仍然是 read-only，不宣称 Harness/Agent/Cutover。

---

# 十一、1 天、7 天、30 天行动计划

## 11.1 1 天

### 目标

冻结事实基线，阻止继续在漂移合同上扩展。

### 行动

- 创建 `CR-goalctl-phase1-hardening`；
- 将本 Goal 与配套 SPEC 放入 `.agents/ssot/tools/goalctl/`；
- 修正 README、Goal、SPEC、Version Matrix 的“尚未实现”陈述；
- 建立 `CURRENT-STATE.md`，列出：
  - 已实现；
  - 未实现；
  - 已知风险；
  - 当前版本；
  - baseline commit；
- 建立 P0 issue：
  - committed view；
  - reconcile false-positive；
  - compile commit/tree mismatch；
  - ApprovalRecord validation；
- 加入禁止性回归测试：
  - dirty artifact 不得盖 HEAD；
  - dirty reconcile 不得盖 HEAD；
  - commit A + tree B 必须失败；
  - 任意 approval ref 必须失败。

### 1 天验收

```bash
test ! -d .config/goal
rg -n "tools/goalctl.*不存在|尚未存在|尚无 resolve|实现未授权" \
  .agents/ssot/tools/goalctl tools/goalctl docs/goal/change-requests
cargo test -p xhyper-goalctl
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
just goal-check
```

## 11.2 7 天

### 目标

完成 Phase 1.1 Truth Hardening。

### 行动

- 实现 `RepositoryView` 端口；
- 把 `resolve/artifact/reconcile/compile/index` 统一迁移到该端口；
- 实现 committed artifact index；
- 实现结构化 Fact Observation；
- Reconcile 只接受 FactSet，不直接猜目录；
- 实现 ApprovalRecord registry 与 validator；
- 统一 PathSpec 语义；
- 增加 RFC 8785/JCS 或明确的 Xhyper Canonical JSON v1；
- 实现 CLI contract conformance test；
- 所有 JSON 输出运行 schema validation；
- 增加 100+ negative/property/fuzz cases；
- 形成 PR-1…PR-N 小步合并，每个 PR 独立 Evidence。

### 7 天验收

- 所有 subject-bound 命令在 dirty worktree 下结果不变；
- 非 HEAD commit 的 artifact/resolve/reconcile 可重放；
- Task Pack 的 commit/tree 必须为真实 Git 对；
- 无有效 ApprovalRecord 时 protected path compile 失败；
- 任何目录存在都不能独立生成 VERIFIED；
- Rust 输出全部通过 Schema Registry；
- 两次运行字节一致；
- Windows/Linux path fixture 一致。

## 11.3 30 天

### 目标

完成 Phase 2 Proof System，并进入 Shadow。

### 行动

- 接入 `xhyper-evidence` 写入端；
- 定义 EvidenceManifest、ReviewBundle、AuditChain；
- 实现 Harness 的 sandbox、budget、timeout、cancellation；
- 实现 VerifierReport；
- 实现 Gate Adapter，只映射既有 G0–G11；
- 接入 Shadow CI，不作为 required；
- 收集至少 3 个真实模块、30 个 PR 样本；
- 自动比较 legacy 与 goalctl；
- 建立 Failure Corpus、historical replay、policy mutation tests；
- 完成 rollback drill；
- 制定 Cutover CR，但不自动批准。

### 30 天验收

- Shadow 样本 N ≥ 30 PR、模块 N ≥ 3；
- P0 false negative = 0；
- 所有差异有机器分类和 owner；
- Evidence completeness ≥ 95%；
- rollback drill 100% 通过；
- 运行成本和延迟达到目标 SLO；
- 无 Secrets 泄露；
- 不可信 Fork 路径通过安全演练。

---

# 十二、分阶段路线图

| Phase | 建议版本 | 核心结果 | 是否可 required |
|---|---|---|---|
| Phase 1.1 | 0.1.1 | committed-view、真实 reconcile、approval validation | 否 |
| Phase 1.2 | 0.2.0 | Schema/Path/Canonical/Identity/Trust 合同统一 | 否 |
| Phase 2 | 0.3.0 | Evidence + Harness 基础 | 否 |
| Phase 3 | 0.4.0 | Independent Verifier + Shadow | 否 |
| Phase 4 | 0.5.0 | Mirror + rollback drill | 否 |
| Phase 5 | 1.0.0 | Approved Cutover + production support model | 是，需独立 CR |
| Phase 6 | 1.x | Agent Adapter + bounded autonomy | 仅低风险范围 |

版本不是完成证明；每个版本必须由 capability matrix、Evidence 和已批准 CR 支撑。

---

# 十三、衡量指标

## 13.1 质量

| 指标 | 目标 |
|---|---:|
| P0 false negative | 0 |
| False VERIFIED / RELEASED | 0 |
| Scope violation rate | 0 |
| Stale Evidence acceptance | 0 |
| Contract drift incidents | 0 |
| Historical replay pass rate | ≥ 99% |
| Schema conformance | 100% |
| Deterministic replay | 100% |

## 13.2 效率

| 指标 | 目标 |
|---|---:|
| First-pass Gate pass rate | ≥ 80% |
| Human minutes / accepted PR | 下降 50% |
| Median compile latency | < 2 s（不含执行） |
| Median reconcile latency | < 5 s / repo |
| Cache hit rate | ≥ 70%，且 subject-safe |
| Failure→Regression 转化率 | ≥ 90% |

## 13.3 安全

| 指标 | 目标 |
|---|---:|
| Untrusted run with Secrets | 0 |
| Protected asset without valid approval | 0 |
| Writer self-approval | 0 |
| Unsigned enforcing binary | 0 |
| Break-glass without expiry | 0 |
| Audit chain verification | 100% |

---

# 十四、迭代优化机制

## 14.1 每次失败必须分类

```text
GOAL_GAP
SPEC_GAP
AUTHORITY_CONFLICT
SCHEMA_DRIFT
SNAPSHOT_TOCTOU
POLICY_BUG
IMPLEMENTATION_BUG
HARNESS_FLAKE
ENVIRONMENT_DRIFT
EVIDENCE_STALE
VERIFIER_MISS
MODEL_BEHAVIOR
COST_OR_BUDGET
```

## 14.2 每次修复必须产出至少一项复利资产

- regression test；
- property/fuzz test；
- mutation case；
- schema rule；
- policy rule；
- diagnostic；
- historical replay fixture；
- agent skill；
- runbook；
- SLO alert。

## 14.3 周期

- 每 PR：contract + negative case + Evidence；
- 每周：Failure Corpus triage；
- 每月：历史回放、policy mutation、dependency/security review；
- 每季度：Cutover/rollback drill、schema compatibility、key rotation；
- 每次重大事故：retrospective + eval + owner + deadline。

---

# 十五、风险与对策

| 风险 | 对策 |
|---|---|
| goalctl 自证可信 | 独立 bootstrap verifier、签名、双人审批 |
| 工作区污染快照 | committed RepositoryView、tree/blob 校验 |
| schema 与代码漂移 | generated types 或双向 conformance tests |
| approval 绕过 | subject digest、scope、status、expiry、revocation |
| Evidence 伪新鲜 | commit/tree/policy/task/runner 全绑定 |
| 多 Agent 冲突 | 单 Writer Lease、Fencing、Task DAG |
| CI 成本失控 | impact analysis、subject-safe cache、budget |
| Fork 攻击 runner | untrusted static stage、无 secrets、无 write |
| 文档过时误导 Agent | CURRENT-STATE 机器检查、doc freshness gate |
| Legacy 永不退出 | Sunset owner、deadline、consumer scan |

---

# 十六、最终推荐路径

```text
第一优先：
  Phase 1.1 Truth Hardening
  不接 Agent，不做 Cutover，不扩命令面。

第二优先：
  把 Schema、Path、Canonical、Approval、Identity、Trust 收敛成统一内核。

第三优先：
  接 Evidence + Harness + Independent Verifier，并只做 Shadow。

第四优先：
  在真实样本、差异解释、回滚演练和独立审批全部满足后进入 Mirror/Cutover。

第五优先：
  最后才接 Agent Writer，并只开放可逆、低风险、可证明任务。
```

最终裁定：

> **先建立可信的“真相编译器”，再建立执行器；先证明系统不会错误放行，再追求自动化吞吐量。**

---

# 十七、Goal 验收清单

- [ ] 本 Goal 已通过独立 CR 批准；
- [ ] 既有 Goal/Spec 的 supersedes 关系明确；
- [ ] 当前事实文档无过时状态；
- [ ] GAP-001…GAP-006 全部关闭；
- [ ] subject-bound command 统一 RepositoryView；
- [ ] ApprovalRecord 有完整验证；
- [ ] Reconcile 无目录存在假阳性；
- [ ] Compile 真实编译 Artifact trace；
- [ ] JSON Schema 与实现一致；
- [ ] canonicalization 跨语言一致；
- [ ] RepositoryIdentity FULL；
- [ ] Evidence/Harness/Verifier 完整；
- [ ] Shadow/Mirror 指标达标；
- [ ] Rollback drill 通过；
- [ ] Cutover CR Approved；
- [ ] Failure Corpus / Eval / Replay 运行；
- [ ] Support Model / On-call / Incident Runbook 完成；
- [ ] Legacy Sunset 完成。

---

# 十八、参考事实路径

```text
tools/goalctl/Cargo.toml
tools/goalctl/src/main.rs
tools/goalctl/src/index.rs
tools/goalctl/src/resolve.rs
tools/goalctl/src/artifact.rs
tools/goalctl/src/reconcile.rs
tools/goalctl/src/compile.rs
tools/goalctl/src/canonical.rs
tools/goalctl/src/identity.rs
tools/goalctl/tests/cli_smoke.rs

.agents/ssot/tools/goalctl/README.md
.agents/ssot/tools/goalctl/goal/Goal-终态目标.md
.agents/ssot/tools/goalctl/SPEC-终态规范.md
.agents/ssot/tools/goalctl/decisions/DECISION-PACK-001.md
.agents/ssot/tools/goalctl/contracts/CLI-CONTRACT.md
.agents/ssot/tools/goalctl/contracts/RUNTIME-STATE.md
.agents/ssot/tools/goalctl/contracts/VERSION-CAPABILITY-MATRIX.md
.agents/ssot/tools/goalctl/schemas/*.schema.json

docs/goal/schema/authority-policy.yaml
docs/goal/change-requests/CR-20260716-goalctl-foundation.md
docs/goal/change-requests/CR-20260716-goalctl-impl-phase1.md
```
