# goalctl：生产级 Goal

> Goal ID：`GOAL-2026-GOALCTL-001`  
> 文档版本：1.0.0  
> 状态：Proposed  
> 风险等级：R3——错误编译可能削弱所有下游验证与门禁  
> 组件路径：`tools/goalctl`  
> Parent Goal：`GOAL-2026-0001-vibe-coding-self-verification`  
> 配套 Spec：`goalctl-spec.md`

---

## 0. 最终裁定

`goalctl` 必须成为 xhyper.rs 唯一的 Goal Contract 编译器，把人类或 Agent 的目标意图转换成确定、可验证、可追踪、可哈希的机器契约。

它只回答：

> 什么可观察事实全部成立时，这个 Goal 才算完成？

它不执行测试、不生成最终 Evidence、不决定是否合并，也不提供手工“完成”按钮。

## 1. 问题本质

AI 生成代码的最大不确定性不是语法，而是目标语义漂移：

- Outcome 使用“优化、完善、支持”等不可观察词语。
- 实现过程中范围扩大，但 Goal 没有变化。
- 测试覆盖了实现，却没有覆盖真实验收条件。
- 多个 Agent 对同一 Goal 有不同理解。
- PR、Spec、Evidence 与 Goal 缺少稳定标识。
- Owner 可以通过修改状态字段直接宣布完成。

因此 `goalctl` 的价值不是管理任务列表，而是把意图变成后续系统无法随意重新解释的完成契约。

## 2. 基本真理

1. 不可观察的 Outcome 不可验证。
2. 没有稳定 ID 的验收条件不可追踪。
3. 相同输入若产生不同 Contract，就不能作为自动门禁输入。
4. Goal 必须同时定义 Scope、Constraint 和 Non-goal，否则必然漂移。
5. 完成状态必须由 Evidence 和 Gate 推导，不能由作者声明。
6. 未知字段、悬空引用、依赖循环和歧义必须 fail-closed。
7. Goal 只能定义“证明什么”，具体“如何运行”属于 `verifyctl`。
8. Contract 必须绑定 Schema、Policy 和源文件摘要。

## 3. 核心 Outcome

实现生产级 Goal 控制平面，使所有非平凡变更在开始编码前具备：

- 唯一 Goal ID。
- 明确、可观察的 Outcome。
- 精确 Include/Exclude Scope。
- 稳定 AC/INV/CON/NG 标识。
- 风险等级和升级触发器。
- Evidence 类型与最低 Trust 要求。
- Goal/Spec/ADR/Owner 依赖闭包。
- 确定性编译后的 Contract digest。

## 4. 目标用户

| 用户 | 目标 |
|---|---|
| 架构师 | 固化边界、依赖、风险和系统不变量 |
| Goal Owner | 创建可执行完成定义 |
| Builder Agent | 获取无歧义实现输入 |
| Verifier Agent | 获取独立反证目标 |
| `verifyctl` | 获取结构化 proof requirements |
| `evidence` | 绑定 Goal、AC/INV 和 Contract digest |
| `gate` | 获取 Completion Policy 和最低 Trust |
| 审计者 | 重建 Goal 从创建到完成的历史 |

## 5. 范围

### 5.1 In Scope

- Goal v1 Schema。
- Goal 创建、格式化、校验和确定性编译。
- Outcome、Scope、Risk、AC、INV、Constraint、Non-goal 语义验证。
- Goal、Spec、ADR、Owner 引用解析。
- Goal 依赖 DAG。
- Canonical JSON、source digest、contract digest。
- PR Goal Binding 解析和校验。
- Goal graph、trace、diff 和派生 status。
- Schema/Contract 版本与迁移。

### 5.2 Out of Scope

- 执行 Check 或调用业务外部服务。
- 生成 Verification Plan。
- 收集或签署 Evidence。
- 输出 Merge/Release Decision。
- 替代 ADR、领域设计或 Issue 管理。
- 用 LLM 主观判断 Goal 是否合格。
- 自动修改 Goal 以绕过失败。

## 6. 组件边界

```text
goal.yaml + referenced specs/policies
  → goalctl
    → goal.lock.json + contract_digest
      → verifyctl / evidence / gate
```

边界规则：

- `goal.yaml` 是 authoring SSOT。
- `goal.lock.json` 是派生机器契约，禁止手工编辑。
- `goalctl` 不读取测试结果来改变 Contract。
- 下游不能通过自由文本重新解释 Contract。
- Goal Policy 变化必须改变绑定摘要并使旧 Decision 失效。

## 7. 不变量

| ID | 不变量 |
|---|---|
| GCTL-INV-001 | 相同语义输入生成相同 Contract bytes |
| GCTL-INV-002 | 任何授权语义变化都改变 contract digest |
| GCTL-INV-003 | Goal 作者不能手工设置 COMPLETED |
| GCTL-INV-004 | 未知字段和未知版本默认拒绝 |
| GCTL-INV-005 | Scope 不得逃逸仓库根目录 |
| GCTL-INV-006 | Goal 依赖图必须无环且引用闭合 |
| GCTL-INV-007 | 每个 required AC 必须声明 proof type |
| GCTL-INV-008 | R2/R3 Goal 必须定义不变量 |
| GCTL-INV-009 | `fmt` 不能改变 Contract 语义 |
| GCTL-INV-010 | `compile --check` 不能修改工作区 |
| GCTL-INV-011 | Contract 不依赖网络、系统时间和环境变量 |
| GCTL-INV-012 | goalctl 不能输出 Gate ALLOW |

## 8. 风险模型

`goalctl` 必须支持 R0–R3，并执行最低约束：

| 风险 | Goal 最低要求 |
|---|---|
| R0 | Outcome、Scope、AC |
| R1 | R0 + negative behavior 或失败条件 |
| R2 | R1 + 至少一个 INV + integration/property proof type |
| R3 | R2 + independent verifier、replay/rollback/runtime proof requirements |

交易、仓位、资金、账务、风控、凭据和不可逆迁移不得低于 R3。

## 9. 成功指标

| 指标 | 目标 |
|---|---:|
| 非平凡 PR Goal 绑定率 | 100% |
| required AC 稳定 ID 覆盖率 | 100% |
| AC proof type 覆盖率 | 100% |
| Contract 编译确定性 | 100% |
| Scope 漂移逃逸 | 0 |
| 悬空引用/循环依赖逃逸 | 0 |
| 手工完成绕过 | 0 |
| 单 Goal 本地编译 P95 | ≤ 300 ms |
| 全仓 Goal 校验 P95 | 版本化基线内 ≤ 2 秒目标 |

Goal 字数、文档数量和 Agent 置信分不属于成功指标。

## 10. 生产场景

### 10.1 新功能

Owner 创建 Goal，定义 Outcome、Scope、AC/INV、Risk；`goalctl validate --level ready` 通过后才允许 Builder 开工。

### 10.2 Scope 漂移

PR diff 超出 Include 且不在 Allowed Generated 中时，`goalctl`/Gate 阻断；Owner 必须修改 Goal 并使旧验证失效。

### 10.3 R3 交易变更

Goal 必须显式写出幂等、资金守恒、未知成交、回滚和 Runtime guardrail，不允许只写“优化订单逻辑”。

### 10.4 Goal/Schema 自身变更

由当前受保护 Schema 和 Gate 评估；候选 Schema 不能成为批准自身的唯一解释器。

## 11. 验收条件

| ID | Outcome 验收条件 |
|---|---|
| GCTL-GOAL-AC-001 | 合法 v1 Goal 能确定编译为 Contract |
| GCTL-GOAL-AC-002 | 不可观察 Outcome 不能 READY |
| GCTL-GOAL-AC-003 | 缺失、重复或复用 AC/INV ID 被拒绝 |
| GCTL-GOAL-AC-004 | 未知字段和重复 YAML key 被拒绝 |
| GCTL-GOAL-AC-005 | 仓库外路径、`..` 和危险符号链接被拒绝 |
| GCTL-GOAL-AC-006 | Scope diff 可检测未声明变更 |
| GCTL-GOAL-AC-007 | Goal/Spec/ADR/Owner 悬空引用被拒绝 |
| GCTL-GOAL-AC-008 | Goal 依赖循环被拒绝并展示环路 |
| GCTL-GOAL-AC-009 | 相同语义输入跨环境输出相同 Contract |
| GCTL-GOAL-AC-010 | 合法格式变化不改变 contract digest |
| GCTL-GOAL-AC-011 | 授权语义变化必然改变 contract digest |
| GCTL-GOAL-AC-012 | `fmt` 幂等且不改变语义 |
| GCTL-GOAL-AC-013 | `compile --check` 无文件写入 |
| GCTL-GOAL-AC-014 | 派生 Contract 漂移导致非零退出 |
| GCTL-GOAL-AC-015 | R2/R3 缺少不变量时拒绝 READY |
| GCTL-GOAL-AC-016 | R3 缺少独立/Replay/Rollback proof 要求时拒绝 |
| GCTL-GOAL-AC-017 | Goal 无 `mark-complete` 或等价后门 |
| GCTL-GOAL-AC-018 | PR Binding 缺失、重复或摘要不符时阻断 |
| GCTL-GOAL-AC-019 | 下游可只依赖 Contract 制定验证计划 |
| GCTL-GOAL-AC-020 | Evidence 可绑定每个 AC/INV |
| GCTL-GOAL-AC-021 | Gate 可从 Contract 读取完成政策和 Trust |
| GCTL-GOAL-AC-022 | 编译过程不访问网络或执行 Goal 内容 |
| GCTL-GOAL-AC-023 | 结构化诊断包含 code、location、field、remediation |
| GCTL-GOAL-AC-024 | 性能满足批准基线且不删除语义检查 |

## 12. MVA

首个版本只实现：

1. Goal v1 Schema。
2. `fmt / validate / compile / show / doctor`。
3. AC/INV/CON/NG ID 检查。
4. Scope、引用和依赖 DAG。
5. Canonical JSON 和 SHA-256 digest。
6. PR Goal Binding。
7. `compile --check`。
8. 结构化 JSON 诊断。

## 13. 1 天、7 天、30 天

### 1 天

- 冻结 v1 Schema、ID 和 canonicalization rules。
- 创建 library-first crate 骨架。
- 编译一个真实 xhyper.rs Goal。

### 7 天

- 完成 Scope、引用、DAG、Risk 和 Binding。
- 完成 golden/property/negative tests。
- 接通 `verifyctl plan`。
- Fast CI 启用 `compile --all --check`。

### 30 天

- 完成 Schema 迁移、Evidence/Gate 端到端追踪。
- 迁移全部活跃 Goal，删除旧双轨。
- 发布 v1.0.0。

## 14. Definition of Done

- 全部 `GCTL-GOAL-AC-*` 和 `GCTL-INV-*` 有自动化 Evidence。
- Schema、CLI、错误码、Canonicalization Profile 已版本化。
- 与 `verifyctl/evidence/gate` 完成端到端契约测试。
- 无静态 PASS、手工完成、占位实现或自由文本授权路径。
- 性能满足批准基线。
- 发布包含 CHANGELOG、Provenance、迁移和回滚说明。
- 通过 worktree + PR + Squash Merge 合入，main 为 GREEN。

## 15. 最终推荐路径

```text
Schema v1
→ Semantic validation
→ Deterministic Contract
→ PR Binding
→ verifyctl consumption
→ Evidence traceability
→ Gate-derived completion
```

`goalctl` 完成的标志不是“能解析 YAML”，而是任何 Agent 都无法在不改变 Contract digest 的前提下重新解释完成条件。

