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
- xtask、archgate、crate-standard 通过。

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
