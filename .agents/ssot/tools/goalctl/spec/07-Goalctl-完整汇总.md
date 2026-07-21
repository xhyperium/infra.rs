# AI-Native 工作流总纲

## 1. 底层本质

AI-Native 不是让 AI 多写代码，而是把软件工程重构为：

```text
Human Goal / Risk Boundary
        ↓
Machine-readable Spec / Task Contract
        ↓
Isolated Agent Execution
        ↓
Independent Verification
        ↓
Evidence
        ↓
Deterministic Gate
        ↓
PR / Merge / Release
        ↓
Runtime Observation
        ↓
Eval / Harness Improvement
```

人负责意图、边界、风险接受和最终裁决；AI 负责在明确范围内执行。

## 2. 基本真理

1. AI 生成代码通常不是主要瓶颈，真正瓶颈是目标不清、上下文不完整、验收不可执行、结果无法证明。
2. Agent 只能使用它能读取的状态，聊天记录和人的脑内知识不能作为可靠项目状态。
3. 模型是非确定性的，因此必须由确定性的测试、Evidence、Gate 和审批边界约束。
4. 跨会话、跨 Agent 的状态必须外部化到 Git、Goal、Spec、Task、Matrix、Evidence 和运行记录。
5. Prompt 只能承担软约束；分支保护、路径限制、依赖门禁、Evidence 校验必须由机器执行。

## 3. 常见错误假设

- Agent 越多越高效：错误。多 Agent 会增加上下文复制、冲突和协调成本。
- 一个长 Prompt 可以驱动长期任务：错误。长任务需要 Task DAG、checkpoint、状态恢复和 worktree。
- 测试通过等于完成：错误。测试不能独立证明需求、架构、安全、发布和回滚完备。
- Writer 可以独立验收自己的工作：高风险任务中不可接受。

## 4. 推荐工作流

```text
Signal
→ Goal
→ AutoResearch
→ Spec
→ Executable Acceptance
→ Task DAG
→ Worktree
→ Writer
→ Evidence
→ Independent Verifier
→ G0-G11 Gate
→ Draft PR
→ Human Approval
→ Merge / Release
→ Observation
→ Failure Corpus
→ Eval / Skill / Harness Update
```

## 5. 推荐角色

### Orchestrator

- 解析 Goal；
- 生成 Task DAG；
- 编译上下文；
- 分配执行者；
- 汇总 Evidence；
- 不直接批准最终结果。

### Writer

- 只修改 allowed paths；
- 每次只完成一个原子修改；
- 不得修改 Task Pack、Gate 或自身权限。

### Verifier

- 独立读取 Goal、Spec、Diff 和 Evidence；
- 主动寻找反例；
- 不修改实现；
- 不接受 Writer 自述作为事实。

### Arbiter / Gate

- 汇总确定性检查；
- 输出 PASS、FAIL、BLOCKED 或允许范围内的 PASS_WITH_RISK；
- 高风险结论交给人类裁决。

## 6. infra.rs 推荐控制面

```text
docs/goal/                         方法和规则 SSOT
.agents/ssot/<layer>/<crate>/      模块交付制品 SSOT
crates/** / tools/**               唯一实现根
tools/xtask                        架构与仓库结构验证
tools/archgate                     架构门禁（monorepo-only；**非** infra.rs SSOT tools 面，本仓仅 evidence/goalctl/xtask/verifyctl）
crates/evidence                    Evidence Core
crates/adapters/evidence/file      Append-only Evidence Adapter
tools/evidence-cli                 只读 Evidence 校验 CLI
tools/goalctl                      Goal 编译、调和和执行协调
```

## 7. 成熟度推进顺序

```text
1. 外部化状态
2. 机器可读 Artifact
3. Authority Snapshot
4. State Reconciliation
5. Task Compilation
6. Evidence 双轨
7. Deterministic Harness
8. Agent Adapter
9. Independent Verifier
10. Native Goal Gate
11. 有限低风险自治
```

## 8. 北极星指标

```text
Verified Accepted Changes / Human Review Minute
```

辅助指标：

- First-pass Gate Pass Rate；
- Human Minutes / Accepted PR；
- Scope Violation Rate；
- Evidence Completeness；
- Repeated Failure Rate；
- Failure → Eval 转化率；
- Historical Replay Pass Rate。

## 9. 最终推荐

先证明系统在没有模型参与时能回答：

```text
什么是权威？
当前状态是什么？
哪些声明冲突？
任务允许修改什么？
如何验证完成？
Evidence 是否仍有效？
```

然后再接入 Agent。否则 AI 只会更快地放大控制面漂移。


---

# SPEC-GOALCTL-001 摘要

## 1. 定位

```text
Package:       xhyper-goalctl
Binary:        goalctl
Path:          tools/goalctl
Layer:         Internal Tools
Publish:       false
Initial:       0.1.0
```

`goalctl` 是：

- Authority Resolver；
- Artifact Indexer；
- State Reconciler；
- Task / Prompt Compiler；
- Execution Harness Coordinator；
- Evidence Bundle Organizer；
- Agent Adapter Host。

它不是：

- 新治理 SSOT；
- 新 G0-G11 编号体系；
- 通用 task runner；
- 自动批准系统；
- 生产 Capability Gate；
- 自动部署系统。

## 2. 强制边界

- 原生读取 `.agents/ssot/**`；
- 禁止创建或依赖 `.config/goal`；
- 保留 `docs/goal` 和 `.agents/ssot` 的权威地位；
- `target/goalctl/**` 仅为可删除运行产物；
- Legacy Markdown 不能独立证明 VERIFIED 或 RELEASED；
- Writer 不能修改 Task Pack 或扩大权限；
- 任何 Evidence 必须绑定 subject commit。

## 3. 主要问题

### 控制面路径分裂

权威路径已经是 `.agents/ssot/**`，但部分旧工具仍依赖 `.config/goal/**`。

### 状态声明冲突

README、Spec、Plan、Test、Release、Gate、Evidence 和 Registry 可能互相矛盾。

### 自然语言 Task 不可安全执行

缺少：

- allowed paths；
- prohibited paths；
- validation commands；
- stop conditions；
- approval requirements；
- source commit。

## 4. Phase 1 能力

`0.1.0` 只实现：

```text
doctor
index
resolve
reconcile
compile
```

明确不实现：

- Agent 自动执行；
- 自动 PR；
- 自动 merge；
- Native G0-G11；
- CI required-check cutover；
- 生产访问。

## 5. 核心数据模型

### AuthoritySnapshot

记录：

- authority path；
- rank；
- kind；
- status；
- Git blob SHA；
- SHA-256；
- source commit。

### ArtifactEnvelope

```json
{
  "schema_version": "1.0.0",
  "artifact_type": "spec",
  "artifact_id": "SPEC-GOALCTL-001",
  "module": "goalctl",
  "status": "PROPOSED",
  "source_commit": null,
  "updated_at": "2026-07-15",
  "evidence_ids": [],
  "gate_ids": ["G2"],
  "supersedes": null
}
```

### ModuleStatus

拆分为：

```text
Specification
Implementation
Verification
Release
Operations
```

### TaskPack

包含：

- Trace：Goal / Spec / Plan / Task；
- Objective / Non-goals；
- Source commit；
- Allowed / prohibited paths；
- Acceptance Criteria；
- Validation commands；
- Dependencies；
- Stop conditions；
- Human approval。

## 6. Artifact 解析模式

### Strict

缺 Control Block、非法 JSON、未知字段或路径不匹配立即失败。

### Mixed

优先结构化 Control Block；缺失时回退 Legacy Parser 并产生警告。

### Legacy

仅作迁移读取，不能生成正式 PASS。

## 7. 状态事实优先级

```text
Runtime / Registry Fact
> Release Fact
> Current-commit Evidence
> Current-commit Gate Verdict
> Structured Artifact
> Legacy Narrative
> Plan Expectation
```

没有足够证据时必须输出 UNKNOWN / NOT_PROVEN，而不是乐观推断。

## 8. 关键 Acceptance Criteria

- 无 `.config/goal` 仍可运行；
- Authority Snapshot 重复生成完全一致；
- 能发现状态冲突；
- 无当前 commit Evidence 不得 VERIFIED；
- P0/P1 AC 无验证方式时编译失败；
- allowed/prohibited 相交时失败；
- protected asset 无批准时要求人工审批；
- Legacy Narrative 不能独立生成 RELEASED；
- 所有输出路径必须仓库相对；
- xtask、crate-standard 通过（monorepo 历史另含 archgate；**infra.rs 不移植 archgate，不作验收**）。

## 9. 安全不变量

```text
INV-001 不创建 .config/goal
INV-002 相同输入产生相同输出
INV-003 Legacy 不产生正式 PASS
INV-004 Evidence 绑定 commit
INV-005 Writer 不自批
INV-006 Task Pack 不被 Writer 修改
INV-007 Protected Asset 需审批
INV-008 Audit 失败不得成功
INV-009 不创造新 Goal Gate
INV-010 goalctl 不成为新 SSOT
```


---

# DESIGN-GOALCTL-001 摘要

## 1. 架构风格

采用：

```text
Hexagonal Architecture
+ Compiler Pipeline
+ Deterministic Serialization
+ Fail-closed Policy
```

核心逻辑不直接依赖文件系统、Shell、GitHub 或具体模型厂商。

## 2. 内部模块

### authority

- 发现和排序权威；
- 验证 Approved / Proposed / Superseded；
- 生成 commit-bound Authority Snapshot；
- 检测结构冲突。

### artifact

- 扫描十一层制品；
- 解析 Artifact Envelope；
- 兼容 Legacy Markdown；
- 构建 Artifact Index；
- 记录来源位置和 hash。

### reconcile

- 收集 StatusClaim；
- 验证 Evidence 新鲜度；
- 多维状态派生；
- 同级冲突检测；
- 输出 Reconciliation Report。

### compiler

- 解析 Goal → Spec → Plan → Task；
- 编译 AC、Validation、Scope、Stop Conditions；
- 输出 Task Pack / Prompt Pack。

### policy

- Protected Asset Policy；
- Scope Policy；
- Approval Policy；
- Stop Condition Policy；
- Environment Policy。

### execution

后续阶段负责：

- worktree；
- writer lease；
- process runner；
- timeout；
- scope guard；
- resume。

### audit

- Review Bundle；
- AuditEvent；
- File Evidence Sink；
- Chain Head Binding；
- 失败运行审计。

### verifier

- 独立验证 Traceability、Scope、负路径、Evidence、架构和安全；
- 只读，不修改实现。

### adapters

- Git；
- 文件系统；
- Cargo metadata；
- Legacy Goal Validator；
- xtask（monorepo 历史另含 archgate；infra.rs 不移植）；
- Evidence；
- Codex / Grok；
- GitHub Draft PR。

## 3. 依赖方向

```text
Domain Types
    ↑
Authority / Artifact / Reconcile / Compiler / Policy
    ↑
Application Services
    ↑
CLI / Adapters
```

禁止：

- Domain → Filesystem；
- Domain → Process；
- Compiler → Codex/Grok；
- Core → GitHub；
- 状态算法依赖模型输出。

## 4. 确定性设计

确定性对象：

- RepositoryIndex；
- AuthoritySnapshot；
- ReconciliationReport；
- TaskPack；
- ValidationPlan。

要求：

- BTreeMap；
- 显式排序；
- UTF-8；
- 固定换行；
- 仓库相对路径；
- 不含当前时间、随机 UUID、PID、机器名和临时绝对路径。

## 5. Artifact Envelope

Markdown 保留给人阅读，JSON Control Block 作为机器状态来源。

路径与类型必须一致，例如：

```text
spec/spec.md → artifact_type=spec
release/release.md → artifact_type=release
```

重复 Control Block、未知字段、module 不一致、非法 Gate ID 均失败。

## 6. Legacy Parser

只解析明确元数据字段，不做全文关键词搜索。

可解析：

```text
Status: Proposed
- **Status**: Proposed
| Status | Proposed |
```

不可误判：

```text
不得 PASS
历史状态曾为 PASS
PASS 条件如下
代码块中的示例状态
```

## 7. 状态调和

Claim 级别：

```text
RuntimeFact
RegistryFact
ReleaseFact
VerifiedEvidence
GoalGateVerdict
StructuredArtifact
LegacyNarrative
PlanExpectation
```

同级冲突必须 BLOCKED；低级相反声明记为 contradiction。

## 8. Evidence 双轨

### Review Bundle

供 PR、Matrix、Gate 阅读：

- manifest.json；
- task-pack.json；
- commands.jsonl；
- scope-report.json；
- verifier-report.json；
- gate-results.json；
- diff-summary；
- hashes。

### Tamper-Evident Audit Chain

使用：

```text
xhyper-evidence
+ xhyper-evidence-file
```

记录执行事件和顺序。`evidence-cli` 只做独立只读校验。

## 9. 迁移策略

```text
Shadow
→ Mirror
→ Cutover
```

未满足真实样本、差异解释、回滚演练和审批前，Native Gate 不得取代旧工具。


---

# PLAN / TASKS：goalctl 实施摘要

## 1. 总体路线

采用纵向能力切片，不一次性建立空壳平台：

```text
Governance
→ Workspace Skeleton
→ Doctor / Index
→ Authority / Artifact
→ Reconciliation
→ Task Compiler
→ Evidence
→ Execution Harness
→ Agent Adapter
→ Independent Verifier
→ Gate Shadow / Cutover
```

## 2. PR 波次

### PR-0：治理制品

只落盘：

- CR；
- Goal；
- Spec；
- Design；
- Plan；
- Tasks；
- Prompt；
- Test；
- Review；
- Release；
- Retrospective；
- Matrix；
- Gate；
- Evidence Template。

不写实现代码。

### PR-1：Skeleton + Doctor + Index

实现：

- workspace 注册；
- architecture tools 分类；
- crate skeleton；
- `goalctl doctor`；
- `goalctl index`；
- deterministic Repository Index。

### PR-2：Authority + Artifact

实现：

- AuthorityEntry；
- AuthoritySnapshot；
- ArtifactEnvelope；
- Strict / Mixed / Legacy Parser；
- Artifact Index；
- `resolve`；
- `artifact inspect/index`。

### PR-3：Reconciliation

实现：

- StatusClaim；
- ClaimStrength；
- Evidence freshness；
- ModuleStatus；
- contradiction detection；
- `goalctl reconcile`。

### PR-4：Task Compiler

实现：

- Trace Resolver；
- AC Compiler；
- Validation Compiler；
- Scope Compiler；
- Protected Asset Policy；
- Task Pack；
- Prompt Pack；
- `goalctl compile`。

### PR-5：Evidence 双轨

实现 Review Bundle 与 tamper-evident Audit Chain。

### PR-6：Deterministic Harness

实现 preflight、claim、worktree、命令执行、timeout、scope guard、resume。

### PR-7：Agent Adapter

接入 Codex、Grok 等，但保持厂商无关合同。

### PR-8：Independent Verifier

Writer 与 Verifier 独立。

### PR-9：Gate Shadow + Draft PR

新旧校验并行，仅 advisory。

### PR-10：Mirror / Cutover

需要真实样本、完整差异解释、回滚演练和重新审批。

## 3. 核心 Task Wave

### W0 Governance

- CR、Goal、Spec、Design、Plan、Tasks；
- Platform / Governance / Architecture 审批。

### W1 Workspace

- worktree；
- Cargo member；
- crate-standard 文件；
- architecture registry；
- xtask classification；
- STRUCTURE 更新。

### W2 CLI

- CLI；
- exit codes；
- deterministic JSON；
- README / AGENTS / CHANGELOG。

### W3 Repository Index

- Cargo metadata；
- architecture registry parser；
- specs scanner；
- package/module mapping；
- deterministic index。

### W4 Authority

- AuthorityEntry；
- rank；
- Approved 状态；
- SHA-256；
- Git blob；
- Snapshot；
- conflict handling。

### W5 Artifact

- enums；
- Envelope；
- extractor；
- Strict/Mixed/Legacy parser；
- Artifact Index；
- negative tests。

### W6 Reconciliation

- five-dimensional status；
- StatusClaim；
- freshness；
- conflict；
- kernel fixture；
- report。

### W7 Compiler

- trace；
- AC；
- validation；
- protected assets；
- path normalization；
- Task Pack；
- Prompt Pack；
- deterministic serialization。

### W8 Phase 1 QA

- fmt；
- clippy；
- tests；
- lint-deps；
- archgate（monorepo-only；infra.rs 不移植、不作验收）；
- crate-standard；
- rule-drift；
- Evidence；
- Independent Review。

## 4. Phase 1 完成定义

```text
CR / Spec / Design Approved
doctor/index/resolve/reconcile/compile 可运行
kernel fixture 通过
不可验证 P0/P1 AC 被阻断
Protected Asset 无批准被阻断
无 .config/goal
所有确定性输出一致
xtask / crate-standard PASS
# monorepo 历史：archgate PASS（infra.rs 不移植 archgate）
Independent Review PASS
Evidence 完整
```

## 5. Phase 1 禁止事项

- 不启动 Agent Writer；
- 不自动修改代码；
- 不自动应用 patch；
- 不产生正式 Native G0-G11；
- 不自动 PR / push / merge；
- 不修改 required CI。


---

# goalctl 治理控制制品摘要

## 1. Change Request

CR 批准内容：

- 新增 `tools/goalctl`；
- 引入 Artifact Envelope；
- 引入多维状态调和；
- 引入 Task Pack；
- 引入 Evidence 双轨；
- 采用 Shadow → Mirror → Cutover；
- Agent 默认无生产权限。

CR 不批准：

- 自动修改 Constitution；
- 自动批准 Spec / ADR / CR；
- 自动生产部署；
- 自动合并高风险 PR；
- 新建 G12+；
- 立即替换旧 Goal 工具。

## 2. Goal

目标是让系统确定性回答：

```text
当前权威是什么？
模块真实状态是什么？
哪些声明冲突？
哪些 Evidence 陈旧？
Task 是否可执行？
允许修改什么？
如何验证？
哪些操作需要人工批准？
```

Phase 1 仅交付只读 MVA。

## 3. Matrix

追踪关系：

```text
Goal → Spec
Spec → Design
Design → Plan
Plan → Task
AC → Task
Task → Code
AC → Test
Test → Evidence
Risk → Gate
```

状态：

```text
Unmapped
Mapped
Linked
Verified
Dropped
Drifted
Stale
Blocked
Changed
```

规则：

- Verified 必须有 evidence_id；
- Dropped 必须有 drop_reason；
- P0 AC 不允许 orphan；
- 源变化后 Verified → Stale。

## 4. Gate

G0-G11 用于治理和交付裁决：

- G0 Context；
- G1 Goal；
- G2 Spec；
- G3 Design；
- G4 Plan；
- G5 Task；
- G6 Code / Scope；
- G7 Test；
- G8 Evidence；
- G9 Review；
- G10 Release；
- G11 Retrospective。

G6 和 G10 不允许 PASS_WITH_RISK。

禁止先写 PASS，再补 Evidence。

## 5. Prompt

Prompt 必须包含：

- Task identity；
- Objective；
- Non-goals；
- Authority summary；
- Relevant requirements；
- Design constraints；
- Allowed / prohibited paths；
- AC；
- Validation commands；
- Stop conditions；
- Required output format。

Prompt 不得覆盖高等级权威。

## 6. Test

必须覆盖：

- 无 `.config/goal`；
- Authority deterministic；
- 状态冲突；
- stale Evidence；
- unverifiable P0 AC；
- scope intersection；
- protected asset；
- Legacy false release；
- repo-relative path；
- repository gates；
- symlink escape；
- path traversal；
- dynamic shell；
- secret leakage。

## 7. Review

Reviewer 必须独立于 Writer，并检查：

- Traceability；
- Scope；
- Architecture；
- Correctness；
- Security；
- Evidence integrity。

必须主动构造至少三个 Writer 未覆盖的反例。

## 8. Release

`0.1.0` 仅代表只读 MVA：

```text
doctor
index
resolve
reconcile
compile
```

不代表完整 Agent Runtime 或生产自治。

## 9. Retrospective

必须评估：

- 是否消除控制面 Split-Brain；
- 是否降低人工判断成本；
- 是否存在误报/漏报；
- Artifact Envelope 是否必要；
- 多维状态是否必要；
- 是否应该进入 Phase 2。

最终裁定只能是：

```text
ADVANCE_TO_PHASE_2
CONTINUE_PHASE_1_HARDENING
LIMIT_TO_READ_ONLY_DIAGNOSTICS
ROLLBACK
```


---

# PR-0 ～ PR-2 执行包摘要

## PR-0：治理落盘

### 目标

只提交治理文档，不写实现。

### 允许范围

```text
docs/goal/change-requests/**
docs/goal/CHANGELOG.md
.agents/ssot/tools/goalctl/**
CHANGELOG.md
```

### 禁止范围

```text
Cargo.toml
Cargo.lock
tools/goalctl/**
tools/xtask/**
.architecture/**              # monorepo 历史路径；infra.rs 不维护
docs/architecture/spec.md
.github/workflows/**
crates/**
.config/goal/**
```

### 提交拆分

1. CR + Goal；
2. Spec + Design；
3. Plan + Tasks + Prompt；
4. Test + Review + Release + Retro；
5. Matrix + Gate + Evidence Template；
6. Changelog。

### 合并条件

- ID 一致；
- 状态无提前晋升；
- Matrix 覆盖所有 P0 AC；
- Evidence `complete=false`；
- 无实现代码；
- Platform / Governance / Architecture Approval。

---

## PR-1：Skeleton + Doctor + Index

### 目标

```text
goalctl --version
goalctl doctor
goalctl index
```

### 关键修改

- 根 Cargo 增加 `tools/goalctl`；
- monorepo 历史：`.architecture/workspace.toml` 登记 tools（**infra.rs 不维护 `.architecture`**）；
- xtask classify 支持 goalctl；
- 创建 crate-standard 骨架；
- 实现 repository root、doctor 和 deterministic index。

### `doctor`

检查：

- repo root；
- Cargo metadata；
- `.agents/ssot`；
- authority map；
- `.config/goal`；
- branch；
- dirty tree；
- architecture registry；
- internal tools。

### `index`

输出：

- package name；
- manifest path；
- package path；
- targets；
- publish；
- layer；
- module；
- implementation root；
- spec root；
- artifact path existence；
- source commit。

### PR-1 非目标

- resolve；
- reconcile；
- compile；
- Evidence Chain；
- Agent；
- Native Gate。

### 验收

```bash
test ! -d .config/goal
cargo fmt --all --check
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
cargo test -p xhyper-goalctl
cargo run -p xhyper-goalctl -- doctor --json
cargo run -p xhyper-goalctl -- index --module kernel
cargo run -p xhyper-goalctl -- index --module goalctl
cargo run -p xtask -- lint-deps
# monorepo-only（infra.rs 不移植 archgate，不作为本仓 CI 硬门禁）:
# cargo run -p archgate -- --json
cargo run -p xtask -- crate-standard --check
```

---

## PR-2：Authority + Artifact

### 目标

```text
goalctl resolve
goalctl artifact inspect
goalctl artifact index
```

### Authority

- 明确候选权威；
- 从 Git HEAD 读取；
- 生成 blob SHA 和 SHA-256；
- 按 rank 排序；
- 检测 required missing、untracked、same-rank structural conflict；
- 不让 dirty working tree 污染 Snapshot。

### Artifact Envelope

结构化 JSON Control Block：

- schema version；
- artifact type；
- artifact id；
- module；
- status；
- source commit；
- evidence ids；
- gate ids；
- supersedes。

### 解析模式

- Strict：缺失或非法即失败；
- Mixed：结构化优先，Legacy 回退并警告；
- Legacy：仅迁移读取，不产生正式 PASS。

### Path Contract

路径与类型必须一致：

```text
goal/goal.md → goal
spec/spec.md → spec
release/release.md → release
...
```

### Source Commit

状态为 COMPLETE / VERIFIED / RELEASED 时必须有 40 位 Git SHA。

### PR-2 非目标

- ModuleStatus；
- Reconciliation；
- Evidence freshness derivation；
- Task Compiler；
- Agent；
- Native Gate。

### 验收

```bash
cargo test -p xhyper-goalctl
cargo run -p xhyper-goalctl -- resolve --module kernel
cargo run -p xhyper-goalctl -- resolve --module goalctl
cargo run -p xhyper-goalctl -- artifact index --module kernel --mode mixed
cargo run -p xhyper-goalctl -- artifact index --module goalctl --mode mixed
cargo run -p xtask -- lint-deps
# monorepo-only（infra.rs 不移植 archgate，不作为本仓 CI 硬门禁）:
# cargo run -p archgate -- --json
cargo run -p xtask -- crate-standard --check
```

## 下一步

PR-3 固定实现：

```text
StatusClaim
ClaimStrength
Evidence freshness
ModuleStatus
Contradiction detection
Reconciliation Report
goalctl reconcile
```
