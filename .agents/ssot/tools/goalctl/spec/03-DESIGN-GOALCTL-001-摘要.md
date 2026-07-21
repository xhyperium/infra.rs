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
