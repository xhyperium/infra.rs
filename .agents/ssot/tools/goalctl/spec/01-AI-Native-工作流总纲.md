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
tools/archgate                     架构门禁
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
