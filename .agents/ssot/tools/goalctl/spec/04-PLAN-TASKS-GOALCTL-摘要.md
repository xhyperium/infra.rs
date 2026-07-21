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
- archgate；
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
xtask / archgate / crate-standard PASS
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
