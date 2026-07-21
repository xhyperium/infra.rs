> **infra.rs 说明**：本文为控制面生产 Goal/Spec，落在 `.agents/ssot/tools/`。  
> 组件期望路径：`tools/{goalctl,verifyctl}`；**本仓尚未创建对应 crate**。  
> 对齐：[docs/ssot/tools-ssot-alignment.md](../../../../../docs/ssot/tools-ssot-alignment.md)

---

# goalctl：生产实施 Spec

> Spec ID：`SPEC-2026-GOALCTL-001`  
> 对应 Goal：`GOAL-2026-GOALCTL-001`  
> 文档版本：1.0.0  
> 风险等级：R3  
> 组件路径：`tools/goalctl`  
> Parent Spec：`SPEC-2026-0001-vibe-coding-self-verification`

---

## 0. 实施裁定

`goalctl` 必须采用 library-first、无网络、确定性编译器架构。Authoring YAML 经过结构验证、语义验证、引用闭合和规范化后生成只读 Goal Contract；下游只消费 Contract，不重新解释自由文本。

## 1. 目录与 SSOT

```text
.agent/
├── goals/<goal-id>/
│   ├── goal.yaml
│   ├── rationale.md
│   └── links.yaml
├── compiled/goals/<goal-id>.lock.json
├── schemas/
│   ├── goal-v1.schema.json
│   ├── goal-contract-v1.schema.json
│   └── goal-binding-v1.schema.json
└── specs/control-plane/goalctl/
    ├── goal.md
    └── spec.md

tools/goalctl/
├── Cargo.toml
├── src/
├── schemas/
├── tests/
└── README.md
```

- `goal.yaml`：唯一可编辑 Goal SSOT。
- `goal.lock.json`：确定性派生文件，禁止手改。
- `main`：正式 Goal 状态 SSOT。
- `.github/`：只调用 CLI，不复制 Goal 语义。

## 2. Goal Source Schema

```yaml
apiVersion: agent.xhyperium.dev/v1
kind: Goal

metadata:
  id: GOAL-2026-0042-change-aware-verification
  title: 建立变更感知验证
  owners: [platform]
  created_at: 2026-07-20
  labels: [ai-native, ci]

spec:
  outcome:
    statement: >-
      PR 只执行足以证明当前变更安全的最小验证闭包，未知影响自动升级全量验证。
    beneficiary: developers
    observable: true

  rationale:
    problem: 重复全量编译导致反馈过慢。
    baseline_ref: evidence://ci/baseline-2026-07

  scope:
    include:
      - tools/verifyctl/**
      - .agent/verification/**
    exclude:
      - crates/domain/**
    allowed_generated:
      - .agent/compiled/goals/**

  risk:
    level: R2
    reasons:
      - changes merge verification selection
    escalation_triggers:
      - unknown-impact
      - policy-change

  acceptance:
    - id: AC-001
      statement: 未知影响必须升级为全 workspace 验证。
      proof:
        types: [impact-analysis-test]
        minimum_trust: ci-trusted

  invariants:
    - id: INV-001
      statement: 缓存命中不能降低最低 Trust。

  constraints:
    - id: CON-001
      statement: 禁止直接在 main 开发。

  non_goals:
    - id: NG-001
      statement: 不实现测试框架本身。

  dependencies:
    goals: []
    specs: []
    adrs: []

  completion:
    require_all_acceptance: true
    require_all_invariants: true
    gate_entrypoint: goal-complete
```

## 3. 字段规则

### 3.1 Metadata

- `id/title/owners/created_at` 必填。
- ID 格式：`GOAL-YYYY-NNNN-<slug>` 或仓库批准的等价稳定格式。
- ID 创建后不得修改或复用。
- Owner 必须可由受保护 Owner Registry 解析。
- 日期必须为 ISO 8601 完整日期。

### 3.2 Outcome

- 描述外部可观察结果，而不是实现动作。
- 禁止仅使用“优化、完善、支持、重构”。
- 必须明确 beneficiary 和 observable。
- `observable=false` 不能进入 READY。

### 3.3 Scope

- `include` 至少一项。
- 路径相对仓库根并使用 `/`。
- 拒绝绝对路径、`..`、环境变量、未解析 glob 和仓库外符号链接。
- Rename 同时检查 old/new path。
- 实际 diff 超出 Scope 时返回范围漂移错误。

### 3.4 AC/INV/CON/NG

- ID 在 Goal 内唯一、稳定、不可重新编号复用。
- 每个条目只表达一个可独立判断的事实。
- AC 必须声明 proof types 和 minimum trust。
- `minimum_trust` 只允许 `untrusted`、`developer-local`、`ci-trusted`、`release-trusted`，严格偏序为 `untrusted < developer-local < ci-trusted < release-trusted`。
- Proof type 必须存在于受保护 Proof Registry；未知类型失败，禁止自由文本类型静默通过。
- 下游只可提高最低 Trust，不得降低 Goal Contract 声明的 Trust。
- R2/R3 至少一个 INV。
- R3 必须声明独立验证、Replay、Rollback/Runtime proof requirements。

### 3.5 Dependencies

- 引用必须存在且摘要可解析。
- Goal 图必须为 DAG。
- 未完成依赖允许 Goal READY，但禁止 Goal COMPLETED。
- 外部依赖使用固定版本/digest，禁止 `latest`。

## 4. PR Goal Binding

严格解析 PR 描述中的唯一绑定：

```text
<!-- xhyper-goal-binding:v1 -->
goal_id: GOAL-2026-0042-change-aware-verification
contract_digest: sha256:...
```

约束：

- 只解析两个 allowlisted 字段。
- 不执行或展开 PR 文本。
- 缺失、重复、未知字段、摘要不符均失败。
- Binding 改变使旧 review、Evidence 和 Decision 失效。

`resolve-binding` 输出版本化对象，而不是把 PR 文本直接交给下游：

```json
{
  "apiVersion": "agent.xhyperium.dev/v1",
  "kind": "GoalBinding",
  "goal_id": "GOAL-2026-0042-change-aware-verification",
  "goal_contract_digest": "sha256:...",
  "subject": {
    "repository": "infra.rs",
    "commit_sha": "9f6d...c81",
    "tree_sha": "41a2...e10",
    "source_digest": "sha256:..."
  },
  "source": {
    "provider": "github",
    "event": "merge_group",
    "event_digest": "sha256:..."
  },
  "binding_digest": "sha256:..."
}
```

计算 `binding_digest` 时排除自身字段；branch、PR number 和可移动 ref 只能作为非权威 qualifier，不能替代精确 Subject。

## 5. 编译流水线

```text
read bounded input
→ parse YAML and reject duplicate keys
→ validate apiVersion/kind
→ JSON Schema validation
→ semantic validation
→ resolve references/owners
→ build dependency DAG
→ normalize paths/dates/sets
→ canonical JSON
→ source/schema/contract digests
→ atomic lock write
```

禁止：

- 网络访问。
- Shell 或 Goal 内命令执行。
- 环境变量展开形成授权语义。
- 当前时间/随机数进入 Contract。
- 静默丢弃未知字段。
- 自动改写歧义 AC。

## 6. Contract Schema

```json
{
  "apiVersion": "agent.xhyperium.dev/v1",
  "kind": "GoalContract",
  "compiler": {"name": "goalctl", "version": "1.0.0"},
  "goal_id": "GOAL-2026-0042-change-aware-verification",
  "source_digest": "sha256:...",
  "schema_digest": "sha256:...",
  "contract_digest": "sha256:...",
  "risk": "R2",
  "scope": {},
  "acceptance": [],
  "invariants": [],
  "constraints": [],
  "non_goals": [],
  "dependencies": {},
  "completion": {}
}
```

Contract 要求：

- UTF-8、稳定字段顺序、禁止重复键。
- 有序数组保序；语义集合稳定排序。
- 时间统一格式。
- 摘要采用 domain separation；计算 `contract_digest` 时排除该字段自身，禁止递归摘要。
- Trust 枚举及偏序与共享控制面契约完全一致。
- 写入使用临时文件和原子 rename。
- `--check` 比较预期 bytes，不写文件。

## 7. 生命周期推导

| 状态 | 推导事实 |
|---|---|
| DRAFT | Source 存在但 readiness validation 未通过 |
| READY | Contract 编译成功且 proof requirements 完整 |
| ACTIVE | 存在绑定 Goal 的有效 PR/worktree 事件 |
| VERIFYING | 当前 Subject 已有 Verification Plan/Run |
| VERIFIED | required Evidence Closure 完整 |
| COMPLETED | Gate 通过、合入 main、Provenance 完整 |
| BLOCKED | 依赖、Evidence 或 Policy 前置缺失 |
| ABORTED | 有受控终止事件和原因 |

状态只能聚合读取，不能通过编辑 Goal Source设置。

## 8. CLI 契约

```bash
goalctl init --id <goal-id> --title <title>
goalctl fmt <path|--all> [--check]
goalctl validate <path|--all> [--level draft|ready]
goalctl compile <path|--all> [--check]
goalctl show <goal-id> [--format human|json]
goalctl graph [<goal-id>] [--format json|dot|mermaid]
goalctl trace <goal-id> [--format human|json]
goalctl diff <old-ref> <new-ref>
goalctl status <goal-id> [--format human|json]
goalctl resolve-binding --event <event.json> --repository <path> --output <binding.json>
goalctl schema [--version v1]
goalctl doctor
```

全局要求：

- `--repository <path>` 默认当前目录。
- 默认非交互。
- JSON stdout 不混入日志。
- `fmt` 只改表示。
- `status` 只读聚合。
- 批量运行输出每个 Goal 结果和汇总。

## 9. Rust 内部架构

```text
tools/goalctl/src/
├── lib.rs
├── main.rs
├── cli.rs
├── model.rs
├── parser.rs
├── schema.rs
├── semantic.rs
├── scope.rs
├── reference.rs
├── graph.rs
├── binding.rs
├── canonical.rs
├── compiler.rs
├── digest.rs
├── status.rs
├── diagnostic.rs
└── error.rs
```

边界：

- `main.rs` 只做 CLI 与退出码。
- Parser 不做语义推断。
- Compiler 不读 GitHub/网络/CI 状态。
- Binding 只解析 allowlisted metadata。
- Canonical 模块无时钟、随机和 I/O。
- Status 不修改 Goal/Evidence。

建议核心接口：

```rust
pub fn validate_goal(
    source: &[u8],
    context: &ValidationContext,
) -> Result<ValidatedGoal, GoalError>;

pub fn compile_goal(
    goal: &ValidatedGoal,
    context: &CompileContext,
) -> Result<GoalContract, GoalError>;
```

## 10. 错误与退出码

| Exit | 含义 |
|---:|---|
| 0 | 成功 |
| 2 | CLI 用法错误 |
| 10 | Schema 错误 |
| 11 | 语义错误 |
| 12 | 引用或 DAG 错误 |
| 13 | Scope/Binding 错误 |
| 14 | Contract 漂移 |
| 20 | I/O 错误 |
| 21 | 不支持版本 |
| 30 | 内部不变量失败 |

错误对象：

```json
{
  "code": "GCTL-E1103",
  "severity": "error",
  "goal_id": "GOAL-2026-0042-change-aware-verification",
  "path": ".agent/goals/.../goal.yaml",
  "location": {"line": 42, "column": 7},
  "field": "spec.acceptance[0].proof",
  "message": "required acceptance criterion has no proof type",
  "remediation": "declare at least one registered proof type"
}
```

## 11. 安全

- 默认无网络。
- 输入文件大小、YAML alias、节点、引用深度和图规模有限制。
- 拒绝路径穿越和仓库外符号链接。
- 不执行 Goal、模板、Artifact 或报告。
- 诊断不得泄露 Secret、环境变量或用户绝对路径。
- 输出原子写；中断不留下可误用 lock。
- PR Binding 防止命令/模板注入。
- Unknown Schema fail-closed。

## 12. 性能预算

| 场景 | 目标 |
|---|---:|
| 单 Goal validate | P95 ≤ 200 ms |
| 单 Goal compile | P95 ≤ 300 ms |
| 1,000 Goal 校验 | P95 ≤ 2 s |
| 常驻内存 | 目标 ≤ 100 MiB |
| 网络请求 | 0 |

基准必须记录硬件/Runner、cold/warm、Schema 与输入 corpus digest。

## 13. 与其他组件的契约

### verifyctl

输出 risk、scope、AC/INV、proof types、trust、budgets 和 escalation triggers。`verifyctl` 禁止从 rationale 自行增加/删除语义。

### evidence

Evidence 必须绑定 `goal_id + contract_digest + criterion/invariant IDs`。

### gate

Gate 消费 completion policy；`goalctl` 不输出 `merge_allowed`。

### GitHub

Workflow 只运行标准 CLI；Goal 语义不复制到 YAML expressions。

## 14. 测试策略

### Unit

- Schema、ID、Scope、Risk、引用、DAG、Binding、digest。

### Golden

- 合法 Source → 固定 Contract。
- 非法 Source → 固定结构化诊断。

### Property

- `fmt(fmt(x)) == fmt(x)`。
- 格式变化不改变 contract digest。
- 授权字段变化改变 digest。
- 合法路径不能逃逸仓库。
- 增加依赖不会漏掉环路。

### Mutation

必须杀死：忽略未知字段、漏掉 Cargo/Scope 路径、忽略依赖循环、允许重复 AC、摘要漏字段、允许 status completed。

### End-to-end

- Goal Source → Contract → verifyctl Plan。
- PR Binding → exact Contract。
- Evidence → Gate-derived completion。

## 15. 实施任务

| Task | 交付 | 验证 |
|---|---|---|
| GCTL-T01 | Source/Contract/Binding v1 Schema 与 canonical corpus | golden tests |
| GCTL-T02 | Parser 与 duplicate-key rejection | negative tests |
| GCTL-T03 | Semantic validator | unit/property |
| GCTL-T04 | Scope/path engine | security tests |
| GCTL-T05 | Reference/Owner/DAG | graph tests |
| GCTL-T06 | Canonical compiler/digest | determinism tests |
| GCTL-T07 | Atomic lock/check mode | filesystem tests |
| GCTL-T08 | PR Binding | adversarial tests |
| GCTL-T09 | CLI/diagnostics | CLI golden tests |
| GCTL-T10 | verifyctl/evidence/gate integration | E2E |
| GCTL-T11 | CI required invocation | workflow test |
| GCTL-T12 | Benchmark/release | Evidence |

## 16. Spec 验收条件

| ID | 验收条件 |
|---|---|
| GCTL-SPEC-AC-001 | Source/Contract/Binding Schema 均版本化 |
| GCTL-SPEC-AC-002 | YAML duplicate key 被拒绝 |
| GCTL-SPEC-AC-003 | 未知字段/枚举/版本被拒绝 |
| GCTL-SPEC-AC-004 | 不可观察 Outcome 被拒绝 |
| GCTL-SPEC-AC-005 | AC/INV ID 缺失、重复、复用被拒绝 |
| GCTL-SPEC-AC-006 | R2/R3 最低要求自动检查 |
| GCTL-SPEC-AC-007 | Scope 路径穿越和危险 symlink 被拒绝 |
| GCTL-SPEC-AC-008 | Rename old/new path 均校验 |
| GCTL-SPEC-AC-009 | Goal/Spec/ADR/Owner 引用闭合 |
| GCTL-SPEC-AC-010 | 依赖循环展示完整环路 |
| GCTL-SPEC-AC-011 | 相同输入输出逐字节一致 Contract |
| GCTL-SPEC-AC-012 | fmt 幂等且摘要不变 |
| GCTL-SPEC-AC-013 | 授权语义变化改变摘要 |
| GCTL-SPEC-AC-014 | compile --check 零写入 |
| GCTL-SPEC-AC-015 | Atomic write 中断无有效半成品 |
| GCTL-SPEC-AC-016 | 编译零网络、零 shell、零环境语义 |
| GCTL-SPEC-AC-017 | Binding 严格解析且防注入 |
| GCTL-SPEC-AC-018 | Binding 摘要不符阻断 |
| GCTL-SPEC-AC-019 | status 由事实推导且只读 |
| GCTL-SPEC-AC-020 | 无 mark-complete 后门 |
| GCTL-SPEC-AC-021 | JSON stdout 稳定可解析 |
| GCTL-SPEC-AC-022 | 错误含位置、字段和 remediation |
| GCTL-SPEC-AC-023 | verifyctl 只依赖 Contract 可制定 Plan |
| GCTL-SPEC-AC-024 | Evidence 可绑定每个 AC/INV |
| GCTL-SPEC-AC-025 | Gate 可读取完成政策 |
| GCTL-SPEC-AC-026 | Mutation tests 杀死关键语义绕过 |
| GCTL-SPEC-AC-027 | 1,000 Goal 基准满足预算 |
| GCTL-SPEC-AC-028 | v1 迁移器不静默降低要求 |
| GCTL-SPEC-AC-029 | Workflow 仅调用 CLI、不复制规则 |
| GCTL-SPEC-AC-030 | 完整 Goal→Plan→Evidence→Gate E2E 通过 |

## 17. Definition of Done

- 30 条 Spec 验收条件和 Goal 验收/不变量全部通过。
- Schema、CLI、错误码和 canonical profile 冻结。
- Unit/Golden/Property/Mutation/Security/E2E 全覆盖核心路径。
- 无占位编译器、静态 Contract、静默兼容和完成后门。
- Fast CI 启用 `goalctl compile --all --check`。
- 发布 v1.0.0、CHANGELOG、Provenance、迁移和回滚说明。
