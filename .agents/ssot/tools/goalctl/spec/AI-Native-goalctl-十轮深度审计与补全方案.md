# AI-Native / `goalctl` 十轮深度审计与补全方案

```text
Document ID:      AUDIT-GOALCTL-001
Audit Scope:      现有 7 份 AI-Native / goalctl 汇总文档
Audit Method:     十轮独立维度交叉检查
Audit Date:       2026-07-15
Target Repo:      xhyperium/infra.rs
Verdict:          方向正确，但尚未达到可直接无歧义实施的生产级规格
Current Score:    78 / 100
Target Score:     95+ / 100
```

---

# 一、最终结论

现有方案的主方向是正确的：

```text
Authority
→ Artifact
→ Reconciliation
→ Task Compilation
→ Evidence
→ Harness
→ Agent
→ Verifier
→ Gate
```

它成功避免了最常见的错误：

- 没有把“更强模型”当作第一解决方案；
- 没有直接构建庞大多 Agent 网络；
- 没有让 Writer 自审；
- 没有把 Prompt 当作硬门禁；
- 没有把 `xhyper-gate` 错当作 Goal Gate；
- 没有让只读 `evidence-cli` 承担 Evidence 写入；
- 明确禁止重建 `.config/goal`；
- 将 Phase 1 收敛到只读能力。

但是，当前文档仍存在一组会在实现阶段引发返工的结构性遗漏：

```text
1. 规则与 Schema 自身的版本治理不足；
2. Authority Rank 可能被硬编码为第二套 SSOT；
3. 工作区运行目录与仓库既有外置 target 约束冲突；
4. Artifact Envelope 缺少迁移、签名和 canonical 规范；
5. Reconciliation 缺少“事实观察者”和时效域模型；
6. Task Pack 缺少风险、能力、资源和副作用合同；
7. Evidence 缺少隐私、保留、外部锚定和原始产物定位；
8. Harness 缺少沙箱、资源预算、取消和进程树恢复协议；
9. Agent / Verifier 缺少模型供应链、上下文污染和串谋防护；
10. Shadow → Mirror → Cutover 缺少量化晋升阈值和自动回滚条件。
```

这些不是补充性文档问题，而是决定系统是否能够安全扩展的基础合同。

---

# 二、十轮检查总览

| 轮次 | 检查维度 | 结论 | 严重度 |
|---|---|---|---|
| 1 | 范围与目标一致性 | 方向一致，但 Phase 边界仍有少量交叉 | Medium |
| 2 | 权威与 SSOT | 存在 Authority Rank 第二套 SSOT 风险 | Critical |
| 3 | Schema 与数据模型 | 缺少 Schema Registry、兼容性和迁移协议 | Critical |
| 4 | 状态机与调和 | 缺少时效域、观察事实和降级规则 | High |
| 5 | CLI / 存储 / 运行目录 | `target/goalctl` 与外置 target 约束冲突 | High |
| 6 | Evidence / 审计 | 双轨正确，但保留、隐私、锚定不完整 | High |
| 7 | 安全与权限 | 方向正确，沙箱和供应链合同不足 | Critical |
| 8 | 并发、恢复与幂等 | Writer Lease 有雏形，但恢复协议不足 | High |
| 9 | CI、发布与迁移 | 缺少量化 Cutover 阈值和自动回滚 | High |
| 10 | Self-improving / Eval / 运维 | 缺少基准治理、成本预算和运行 SLO | High |

---

# 三、第一轮：范围与目标一致性检查

## 3.1 已覆盖

当前方案已经明确区分：

```text
Phase 1 只读控制能力
Phase 2 Evidence
Phase 3 Execution Harness
Phase 4 Agent / Verifier
Phase 5 Native Gate
```

这是正确的纵向切片。

## 3.2 遗漏一：Phase 1 的真实边界仍不完全一致

部分摘要将 `0.1.0` 定义为：

```text
doctor
index
resolve
reconcile
compile
```

但 PR-1 与 PR-2 分别只做到：

```text
PR-1 doctor/index
PR-2 resolve/artifact
```

这本身没有问题，但缺少一个正式的版本晋升表：

| PR | 可用命令 | Package version | Feature maturity |
|---|---|---|---|
| PR-1 | doctor/index | 0.1.0-dev | Experimental |
| PR-2 | + resolve/artifact | 0.1.0-dev | Experimental |
| PR-3 | + reconcile | 0.1.0-rc.1 或内部 snapshot | Candidate |
| PR-4 | + compile | 0.1.0 | Phase 1 Complete |

### 修复

新增：

```text
.agents/ssot/tools/goalctl/release/version-capability-matrix.md
```

禁止每个 PR 都笼统声称“目标版本 0.1.0 已实现”。

## 3.3 遗漏二：`goalctl` 的长期非目标需要分层

目前非目标混合了：

- 永久非职责；
- Phase 1 暂缓能力；
- 未来可能实现能力。

应拆分为：

```text
Permanent Non-goals
Deferred Capabilities
Explicitly Forbidden Capabilities
```

例如：

| 能力 | 分类 |
|---|---|
| 自动批准 Constitution | 永久禁止 |
| Agent Adapter | 延期 |
| GitHub Draft PR | 延期 |
| 生产交易执行 | 永久非职责 |
| Native G0-G11 | 延期且需独立 CR |

---

# 四、第二轮：Authority 与 SSOT 检查

## 4.1 结构性问题：Authority Rank 被硬编码

现有设计建议在代码中定义：

```rust
ROOT_CONSTITUTION = 0
APPROVED_RFC = 10
APPROVED_ADR = 20
...
```

这会产生新的规则源：

```text
docs/goal/00-authority-map.md
vs
tools/goalctl/src/authority/rank.rs
```

一旦两者漂移，`goalctl` 自身成为第二套 Authority SSOT，违背核心目标。

## 4.2 正确方案：Authority Policy Compiler

代码只定义稳定模型，不定义具体排序：

```rust
pub struct AuthorityPolicy {
    pub schema_version: String,
    pub classes: Vec<AuthorityClass>,
}
```

权威排序应来自机器可读政策，例如：

```text
docs/goal/schema/authority-policy.json
或
docs/goal/authority-policy.toml
```

如果当前治理不允许新增该文件，则从 `00-authority-map.md` 的唯一结构化 Control Block 编译。

### 必须满足

```text
Authority Rank Policy
= Git 中的治理事实
≠ Rust 常量
```

## 4.3 遗漏：Authority 的作用域

现有模型只有 `rank`，缺少：

```text
scope
subject
effective_from
effective_until
supersedes
conflicts_with
```

同一 ADR 可能只适用于：

- 一个 crate；
- 一个目录；
- 一个行为；
- 一个版本区间。

建议：

```json
{
  "authority_id": "ADR-012",
  "scope": {
    "modules": ["evidence"],
    "paths": ["crates/infra/evidence/**"],
    "capabilities": ["audit-storage"]
  },
  "effective_from": "commit-or-version",
  "effective_until": null,
  "supersedes": ["ADR-010:evidence-section"]
}
```

否则“高权威优先”会错误覆盖不属于同一 subject 的规则。

## 4.4 遗漏：Approval Evidence

文档写 `Status: Approved` 不应成为最终批准事实。

必须定义：

```text
ApprovalRecord
```

至少包含：

- approval_id；
- artifact_id；
- subject digest；
- approver identity；
- approval scope；
- approved commit；
- timestamp；
- expiration（适用时）；
- revocation。

否则任何人修改 Markdown 为 `Approved` 就可能晋升 Authority。

## 4.5 遗漏：Authority Revocation

需要支持：

```text
ACTIVE
SUSPENDED
REVOKED
SUPERSEDED
EXPIRED
```

`Rejected` 不等同于曾生效后被撤销。

---

# 五、第三轮：Schema 与数据模型检查

## 5.1 Critical：缺少 Schema Registry

已有多个 `schema_version`，但没有定义：

- Schema 文件放在哪里；
- 谁是 owner；
- 如何升级；
- 是否向后兼容；
- Reader 支持哪些版本；
- 未知版本如何处理；
- Migration 如何证明无损。

至少需要：

```text
.agents/ssot/tools/goalctl/schemas/
├── authority-snapshot.schema.json
├── artifact-envelope.schema.json
├── repository-index.schema.json
├── reconciliation-report.schema.json
├── task-pack.schema.json
├── validation-plan.schema.json
├── evidence-manifest.schema.json
├── verifier-report.schema.json
└── shadow-diff.schema.json
```

更推荐全仓 Schema SSOT 统一放在已批准的 schema 根，并由 `schema_codegen` 生成 Rust 类型或验证器，避免 Rust 类型与 JSON Schema 漂移。

## 5.2 缺少兼容性策略

建议采用：

```text
Major: breaking
Minor: additive
Patch: clarification / bug fix
```

Reader 合同：

```text
Reader v1.x:
- 必须读取 1.0～当前 1.x
- 遇到更高 major 必须拒绝
- 未知字段行为必须由 schema 决定
```

## 5.3 `deny_unknown_fields` 的风险

对治理输入使用 `deny_unknown_fields` 可以 fail-closed，但会阻断 minor additive 演进。

建议区分：

```text
Authority / Security Policy:
  Strict unknown-field rejection

Operational Reports:
  Preserve unknown extensions or version-gated acceptance
```

否则任何增加可选字段都会导致旧 CLI 全部失败。

## 5.4 Artifact Envelope 缺少关键字段

建议补充：

```json
{
  "schema_version": "1.0.0",
  "artifact_type": "spec",
  "artifact_id": "SPEC-GOALCTL-001",
  "module": "goalctl",
  "status": "PROPOSED",
  "subject_digest": "...",
  "source_commit": null,
  "effective_commit": null,
  "created_at": "...",
  "updated_at": "...",
  "owner": "platform",
  "approval_refs": [],
  "evidence_ids": [],
  "gate_ids": ["G2"],
  "supersedes": null,
  "extensions": {}
}
```

关键遗漏：

- `subject_digest`：防止 ID 相同但内容被替换；
- `owner`；
- `approval_refs`；
- `effective_commit`；
- `created_at`；
- 扩展命名空间。

## 5.5 Canonical JSON 未定义

“稳定字段顺序 + pretty JSON”不等于跨实现 canonical。

必须明确：

- UTF-8；
- Unicode normalization；
- key ordering；
- number encoding；
- newline；
- escaping；
- no insignificant whitespace；
- hash 的输入是 canonical bytes 还是 pretty bytes。

推荐：

```text
人类输出：pretty JSON
Evidence/hash：canonical JSON bytes
```

两者不得混用。

---

# 六、第四轮：状态机与调和检查

## 6.1 已有优势

拆分五维状态是正确的：

```text
Specification
Implementation
Verification
Release
Operations
```

## 6.2 遗漏：状态不只有“值”，还需要“观察时间”

例如运行状态：

```text
HEALTHY
```

如果是三个月前观测的，不能代表当前健康。

每个派生状态应包含：

```json
{
  "value": "VERIFIED",
  "subject_commit": "...",
  "observed_at": "...",
  "valid_until": null,
  "confidence": "PROVEN",
  "basis": ["EVID-..."]
}
```

## 6.3 遗漏：事实域不同

需要区分：

```text
Commit-scoped Fact
Release-scoped Fact
Environment-scoped Fact
Time-window Fact
External-registry Fact
```

例如：

- 单测 PASS：commit-scoped；
- crates.io 发布：release/registry-scoped；
- 服务健康：environment/time-window-scoped；
- Approval：artifact digest-scoped。

不能只用 `source_commit` 一个字段承载所有事实。

## 6.4 遗漏：状态降级规则

当新 commit 出现：

```text
VERIFIED(commit A)
HEAD = commit B
```

应该：

```text
Verification = STALE
```

但 Release 状态是否降级取决于 release subject，不应一并失效。

需要明确每个维度的 invalidation rules。

## 6.5 遗漏：部分事实与证据不足

建议状态结果除 value 外，提供：

```text
PROVEN
INFERRED
DECLARED
UNKNOWN
CONFLICTED
STALE
```

避免把：

```text
UNKNOWN
```

和：

```text
BLOCKED
```

混为一谈。

## 6.6 遗漏：Reconciliation 插件接口

不同模块的 Release Fact 来源不同：

- library：Cargo version / tag / registry；
- binary：release artifact；
- service：deployment / health；
- docs-only：merged commit。

需要：

```rust
pub trait FactObserver
```

而不是把所有事实规则硬编码在核心调和器中。

---

# 七、第五轮：CLI、存储与运行目录检查

## 7.1 High：`target/goalctl` 与仓库既有约束冲突

现有 Change Request 已写明构建产物使用：

```text
../.cargo/target
```

并禁止写死 `./target/`。

文档中多次使用：

```text
target/goalctl/**
```

这会产生路径不一致。

## 7.2 正确目录模型

将运行状态与 Cargo build target 分离：

```text
.goalctl-runtime/     不建议提交，仓库根本地运行态
或
${XDG_STATE_HOME}/xhyper-goalctl/<repo-id>/
```

推荐优先级：

```text
--state-dir
GOALCTL_STATE_DIR
XDG_STATE_HOME/xhyper-goalctl/<repo-id>
<repo>/.worktrees/goalctl-state   （仅已批准时）
```

不要复用 Cargo `target` 目录，因为：

- `CARGO_TARGET_DIR` 可外置；
- CI 会清理 build target；
- goalctl 状态不等于编译产物；
- 多 worktree 需要隔离状态。

## 7.3 缺少配置优先级

需要定义：

```text
CLI flag
> environment variable
> repository config
> user config
> default
```

同时限制哪些配置可由用户覆盖，安全策略不能被低权威配置放宽。

## 7.4 缺少 stdout/stderr 合同

建议：

```text
stdout:
  机器输出或请求结果

stderr:
  diagnostics / warnings

--json:
  stdout 只输出单个合法 JSON
  stderr 不得混入 JSON
```

否则 CI 难以可靠解析。

## 7.5 缺少稳定 CLI 兼容政策

需要定义：

- 命令何时可重命名；
- exit code 稳定性；
- JSON 字段兼容性；
- deprecated option 的窗口；
- `--format json|jsonl|text`；
- `--quiet` / `--no-color`；
- TTY 与非 TTY 行为。

## 7.6 缺少大仓库性能预算

至少定义：

```text
doctor p95 < 1s
index p95 < 3s
resolve p95 < 2s
reconcile(module) p95 < 3s
compile(task) p95 < 2s
```

并限制：

- 单文件最大字节数；
- Control Block 最大长度；
- Artifact 数量；
- Git 命令次数；
- Cargo metadata 调用次数。

---

# 八、第六轮：Evidence 与审计检查

## 8.1 已有优势

Review Bundle + Audit Chain 的双轨设计正确。

## 8.2 遗漏：隐私与敏感数据策略

Evidence 不能直接保存：

- Secret；
- token；
-环境变量；
-用户路径；
-源数据原文；
-模型完整上下文；
-可能含商业秘密的 stdout。

需要：

```text
Redaction Policy
Artifact Classification
Retention Policy
```

Command Evidence 应支持：

```json
{
  "stdout_ref": "artifact://sha256/...",
  "stdout_sha256": "...",
  "stdout_redaction": "REDACTED|NONE|PARTIAL"
}
```

而不是默认把所有 stdout 原文写进 Git。

## 8.3 遗漏：原始产物定位

仅有 SHA-256 不足以重新取得原始产物。

必须定义：

```text
Artifact Locator
```

例如：

```text
git:path@commit
ci-artifact:run-id/name
object-store:bucket/key
content-addressed:sha256
```

## 8.4 遗漏：Retention

不同 Evidence 类型应有不同保留期限：

| 类型 | 建议 |
|---|---|
| PR 验证日志 | 90～180 天 |
| Release Evidence | 与支持周期一致 |
| Approval / Security | 长期或法规要求 |
| Agent trace | 默认短期，脱敏后保留 |
| 失败 fixture | 长期保留最小复现 |

## 8.5 遗漏：外部锚定策略

Evidence 规范已经指出单独哈希链不能证明整链替换或尾部截断。

`goalctl` 需要明确：

- 何时签名 checkpoint；
- 谁签名；
- 外部锚点在哪里；
- CI / release 是否要求外部 anchor；
- 本地开发是否只需 unsigned chain。

## 8.6 遗漏：Evidence 原子性

需要定义：

```text
业务/执行状态更新
与
Audit Event append
```

之间的原子关系。

至少：

```text
Audit append fail
→ Run 不能进入 COMPLETE
```

但还要规定状态文件与 Evidence append 的提交顺序和崩溃恢复。

## 8.7 遗漏：Evidence Chain 分片

Chain ID 建议按：

```text
repository + workflow + run
```

或：

```text
repository + module
```

必须裁定，否则全仓单链会造成锁竞争和无限增长，多链又会增加审计复杂度。

---

# 九、第七轮：安全与权限检查

## 9.1 Critical：缺少真实沙箱合同

“network deny / secrets deny”目前只是政策描述。

必须定义具体执行机制：

```text
Linux namespace / container / sandbox
seccomp / AppArmor（适用时）
read-only mounts
writable allowlist
network namespace
environment allowlist
process limits
```

如果第一阶段只支持本地命令，也要明确：

```text
LocalCommandAdapter 不是安全沙箱
只能用于受信任命令
```

不能让“结构化 argv”被误认为完整安全隔离。

## 9.2 缺少 Capability Token

Task Pack 应包含不可放宽的能力合同：

```json
{
  "capabilities": {
    "filesystem_read": ["..."],
    "filesystem_write": ["..."],
    "network": [],
    "secrets": [],
    "git_commit": true,
    "git_push": false,
    "github_draft_pr": false,
    "production": false
  }
}
```

allowed_paths 只能描述文件写入范围，不能完整表达网络、进程、GitHub 和 Secret 权限。

## 9.3 缺少资源预算

防止失控任务：

```text
max_wall_time
max_cpu_time
max_memory
max_disk_write
max_processes
max_output_bytes
max_tool_calls
max_model_tokens
max_cost
```

超限必须 STOP，而不是自动续费或无限重试。

## 9.4 缺少 Prompt Injection 防护

Agent 读取仓库文件时，README、Issue、源码注释都可能包含诱导指令。

需要区分：

```text
Authority Content
Trusted Task Context
Untrusted Repository Content
External Research Content
```

编译 Prompt 时必须标注信任级别，并规定：

```text
Untrusted content cannot override system/task policy
```

## 9.5 缺少供应链安全

需要验证：

- Agent CLI binary hash/version；
-模型 provider / model ID；
-工具版本；
-容器镜像 digest；
-依赖锁；
-Runner identity；
-操作系统和 Rust toolchain。

Evidence Environment 应记录这些信息。

## 9.6 缺少审批策略矩阵

不是所有 protected asset 都同风险。

建议：

| 风险类 | 示例 | 审批 |
|---|---|---|
| R0 | 文档拼写 | 无额外审批 |
| R1 | 模块内部代码 | Reviewer |
| R2 | Public API / dependency | Owner + Reviewer |
| R3 | CI / architecture / Gate | Platform + Governance |
| R4 | Secret / production /资金 | 多人审批 + 人工执行 |

---

# 十、 第八轮：并发、恢复与幂等检查

## 10.1 Writer Lease 不应只用过期时间

需要：

```text
lease generation
owner identity
heartbeat
process identity
worktree identity
base commit
task pack digest
recovery token
```

## 10.2 缺少 fencing token

仅靠 lock 文件和 expiry 会出现旧 Writer 恢复后继续写入。

需要单调递增：

```text
fencing_token
```

所有状态写入和 Audit Event 必须携带当前 token，旧 token 写入被拒绝。

## 10.3 缺少幂等命令语义

`resume` 不能简单从 `next_step` 继续。

每一步需要：

```text
input digest
idempotency key
side-effect class
replay policy
```

分类：

```text
PURE
IDEMPOTENT
AT_MOST_ONCE
EXTERNAL_IRREVERSIBLE
```

外部不可逆操作必须单独批准，Phase 1/2 应禁止。

## 10.4 缺少取消协议

用户或 Arbiter 需要：

```text
goalctl cancel --run ...
```

取消必须：

- 标记 cancellation requested；
-终止进程树；
-保存日志和 Evidence；
-释放 lease；
-保持 worktree；
-输出可恢复状态。

## 10.5 缺少崩溃一致性

状态文件写入必须：

```text
write temp
fsync
atomic rename
fsync parent
```

否则断电可能造成半写 JSON。

## 10.6 缺少多 Worktree 隔离

运行目录必须按：

```text
repo-id / worktree-id / run-id
```

隔离，不能所有 worktree 共用一个可覆盖的 `target/goalctl/runs`。

---

# 十一、第九轮：CI、发布与迁移检查

## 11.1 Shadow → Mirror → Cutover 缺少量化阈值

需要明确：

### Shadow 进入 Mirror

```text
真实样本 >= 20
blocking false negative = 0
结果一致率 >= 95%
所有差异均有分类
```

### Mirror 进入 Cutover

```text
真实样本 >= 100 或两个发布周期
blocking false negative = 0
blocking false positive < 1%
deterministic failure = 0
rollback drill PASS
owner approval complete
```

具体数字可调整，但不能只写“稳定后”。

## 11.2 缺少自动降级条件

Native Gate 上线后，出现以下任一应自动回到 Legacy / advisory：

```text
schema parse systemic failure
false negative
deterministic mismatch
Evidence store unavailable
unexpected blocker spike
performance SLO breach
```

## 11.3 缺少 CI 并发控制

同一 PR 的旧 run 可能晚于新 run 完成并覆盖状态。

必须使用：

```text
concurrency group
cancel-in-progress
subject commit check
```

所有 Gate verdict 必须验证 subject commit 等于 PR 当前 HEAD。

## 11.4 缺少 Release Provenance

Release 应记录：

- source commit；
- builder identity；
- workflow run；
- toolchain；
- binary digest；
- SBOM；
- dependency audit；
- provenance statement；
- Evidence bundle；
- rollback artifact。

## 11.5 缺少 Feature Flag

Native 功能应有：

```text
goalctl.mode = advisory|shadow|mirror|enforcing
```

该模式必须来自受保护政策，不能由普通 Task 或 Agent 覆盖。

---

# 十二、第十轮：Self-improving、Eval 与运维检查

## 12.1 缺少 Eval 分层

至少需要：

```text
Unit Fixture Evals
Historical Regression Evals
Adversarial Evals
Real-task Replay Evals
Performance Evals
Cost Evals
Security Evals
```

## 12.2 缺少基准冻结

每次 Harness、Prompt、Model、Tool 更新前：

```text
baseline id
candidate id
dataset digest
environment digest
```

必须固定，否则改进结果不可比较。

## 12.3 缺少防过拟合机制

不能只修复当前 fixture。

需要：

- holdout dataset；
-随机化但固定 seed；
-对抗样本；
-跨模块样本；
-不允许 candidate 读取 expected answer。

## 12.4 缺少成本治理

Agent 阶段必须记录：

```text
tokens
tool calls
wall time
runner minutes
API cost
retries
human review minutes
```

否则“自动化提高效率”无法证实。

## 12.5 缺少运行可观测性

建议指标：

```text
goalctl_command_duration_seconds
goalctl_diagnostic_total
goalctl_reconcile_conflict_total
goalctl_schema_failure_total
goalctl_scope_violation_total
goalctl_evidence_append_failure_total
goalctl_run_resume_total
goalctl_gate_shadow_difference_total
```

日志、指标、Trace 中不能包含 Secret 或完整 Prompt 原文。

## 12.6 缺少 SLO

内部工具也应定义：

```text
deterministic output = 100%
blocking false negative = 0
schema corruption loss = 0
audit append success >= 99.99%（执行阶段）
reconcile p95 < 3s/module
compile p95 < 2s/task
```

---

# 十三、文档之间的具体矛盾与不一致

## 13.1 运行目录矛盾

当前文档：

```text
target/goalctl/**
```

既有仓库约束：

```text
../.cargo/target
```

### 裁定

`goalctl` 运行状态不得放入 Cargo target。

建议：

```text
${XDG_STATE_HOME}/xhyper-goalctl/<repo-id>/
```

并提供 `--state-dir`。

## 13.2 Authority 排序矛盾

文档强调 `docs/goal` 是规则 SSOT，但又建议在 Rust 中硬编码 rank。

### 裁定

Rust 只实现解析和验证，具体 policy 必须由 Git 中机器可读 Authority Policy 提供。

## 13.3 G0-G11 与 Diagnostic 的边界仍需更强

虽然已规定 `GC-*` 不是 Gate，但一些文档写“PR 完成后可以更新 G6/G7”。

需要明确：

```text
goalctl 输出 Diagnostic
→ 独立 Gate Adapter 消费
→ Arbiter / Human 裁决 G0-G11
```

不能由 `goalctl` 命令自身写 Gate 文档。

## 13.4 `ArtifactStatus::Complete` 语义模糊

不同 Artifact 的 Complete 含义不同：

- Plan Complete；
- Code Complete；
- Review Complete；
- Retrospective Complete。

建议 Artifact 状态仅表示该文档生命周期：

```text
DRAFT / PROPOSED / APPROVED / SUPERSEDED / RETIRED
```

执行完成状态放在专门字段或阶段模型中。

否则 `COMPLETE` 会继续制造状态歧义。

## 13.5 `updated_at` 破坏确定性

Artifact Envelope 有 `updated_at`，Authority/Artifact Index 又要求相同 commit 确定性。

这不冲突，只要它来自提交内容；但工具不得自动每次运行更新该字段。

需要明确：

```text
updated_at 是作者提交的制品字段
不是 goalctl 运行时间
```

---

# 十四、必须新增的核心合同

## 14.1 Authority Policy Schema

```json
{
  "schema_version": "1.0.0",
  "classes": [
    {
      "kind": "CONSTITUTION",
      "rank": 0,
      "scope": "repository",
      "approval_required": true
    }
  ]
}
```

## 14.2 Capability Policy

```json
{
  "filesystem_read": [],
  "filesystem_write": [],
  "network": [],
  "secrets": [],
  "process_allowlist": [],
  "git": {
    "commit": false,
    "push": false
  },
  "github": {
    "draft_pr": false,
    "merge": false
  },
  "production": false
}
```

## 14.3 Resource Budget

```json
{
  "max_wall_time_seconds": 1800,
  "max_memory_bytes": 2147483648,
  "max_disk_write_bytes": 1073741824,
  "max_processes": 64,
  "max_output_bytes": 104857600,
  "max_tool_calls": 100,
  "max_model_tokens": 200000,
  "max_cost_usd": 10
}
```

## 14.4 Fact Observation

```json
{
  "fact_id": "FACT-...",
  "fact_type": "COMMIT_VERIFICATION",
  "subject": {
    "commit": "...",
    "module": "kernel"
  },
  "value": "PASS",
  "observed_at": "...",
  "valid_until": null,
  "observer": "cargo-test-adapter",
  "evidence_id": "EVID-..."
}
```

## 14.5 Approval Record

```json
{
  "approval_id": "APPR-...",
  "subject_id": "SPEC-GOALCTL-001",
  "subject_digest": "...",
  "scope": ["IMPLEMENT_PHASE_1"],
  "approvers": [],
  "approved_commit": "...",
  "status": "ACTIVE",
  "revoked_by": null
}
```

## 14.6 Migration Manifest

```json
{
  "migration_id": "MIG-ARTIFACT-V0-V1",
  "from_schema": "legacy",
  "to_schema": "artifact-envelope/1.0.0",
  "subject_commit": "...",
  "input_digest": "...",
  "output_digest": "...",
  "lossy": false,
  "verifier": "..."
}
```

---

# 十五、重新设计后的组件边界

```text
Goal Governance SSOT
├── Authority Policy
├── Approval Records
├── Artifact Schemas
└── G0-G11 Rules

goalctl-core
├── immutable domain models
├── canonical serialization
├── reconciliation engine
└── task compiler

goalctl-adapters
├── git
├── cargo metadata
├── artifact filesystem
├── fact observers
├── evidence
└── agent providers

goalctl-runtime
├── state store
├── lease/fencing
├── process supervisor
├── cancellation
├── resource budget
└── recovery

goalctl-cli
└── stable command surface
```

是否拆成多个 crate，应在代码量和复用需求真实出现后决定。Phase 1 仍可使用单 crate 内模块，避免过早物理拆分。

---

# 十六、修复优先级

## P0：进入 PR-1 前必须补齐

1. 裁定 `goalctl` state directory，不使用 `target/goalctl`；
2. Authority Rank 改为政策输入，不硬编码为事实源；
3. 建立 Schema Registry 和版本兼容原则；
4. 定义 stdout/stderr、exit code 和 JSON compatibility；
5. 明确 ArtifactStatus 与 ModuleStatus 的语义隔离；
6. 定义 ApprovalRecord，不允许仅靠 `Status: Approved`；
7. 定义 protected asset 风险分级；
8. 定义 deterministic canonical bytes。

## P1：进入 PR-3/PR-4 前补齐

1. FactObserver；
2. 状态时效域和 invalidation；
3. Capability Policy；
4. Resource Budget；
5. Path glob 精确定义；
6. Schema migration；
7. Module 类型与 Release Fact Adapter；
8. performance limits。

## P2：进入 Harness 前补齐

1. 沙箱实现；
2. writer fencing token；
3. atomic state writes；
4. cancellation；
5. idempotency / side-effect classes；
6. process-tree termination；
7. state-store isolation；
8. Evidence redaction / retention / locator。

## P3：进入 Agent / Native Gate 前补齐

1. Prompt injection trust labels；
2. model/tool supply-chain provenance；
3. Eval registry；
4. holdout / adversarial datasets；
5. cost budgets；
6. Shadow / Mirror quantitative thresholds；
7. automatic rollback criteria；
8. native gate feature flag。

---

# 十七、建议新增 PR 波次

现有 PR-0～PR-10 应增加两个前置或插入波次。

## PR-0A：Schema 与 Policy Foundation

```text
Authority Policy
Schema Registry
Approval Record
Canonical serialization
Runtime directory policy
CLI compatibility contract
```

这应在 PR-1 大量写代码之前完成。

## PR-2A：Fact Model 与 Reconciliation Contract

位于 Artifact 后、Reconciliation 前：

```text
FactObservation
FactObserver
Validity scope
Invalidation rules
Module kinds
Release fact adapters
```

## PR-5A：Runtime Security Foundation

位于 Evidence 后、Harness 前：

```text
Capability Policy
Resource Budget
Sandbox contract
Lease fencing
Atomic state store
Cancellation
```

修订路线：

```text
PR-0 Governance
→ PR-0A Schema/Policy
→ PR-1 Skeleton
→ PR-2 Authority/Artifact
→ PR-2A Fact Model
→ PR-3 Reconciliation
→ PR-4 Compiler
→ PR-5 Evidence
→ PR-5A Runtime Security
→ PR-6 Harness
→ PR-7 Agent
→ PR-8 Verifier
→ PR-9 Shadow
→ PR-10 Cutover
```

---

# 十八、修订后的 Task Pack 最小合同

```json
{
  "schema_version": "1.0.0",
  "task_pack_id": "TP-...",
  "run_id": "RUN-...",
  "source_commit": "...",
  "authority_snapshot_digest": "...",
  "approval_refs": [],
  "trace": {
    "goal_id": "...",
    "spec_id": "...",
    "plan_id": "...",
    "task_id": "..."
  },
  "objective": "...",
  "non_goals": [],
  "acceptance_criteria": [],
  "validation": [],
  "scope": {
    "allowed_paths": [],
    "prohibited_paths": [],
    "path_semantics_version": "1.0.0"
  },
  "capabilities": {
    "filesystem_read": [],
    "filesystem_write": [],
    "network": [],
    "secrets": [],
    "process_allowlist": [],
    "git_commit": false,
    "git_push": false,
    "github_draft_pr": false,
    "production": false
  },
  "resource_budget": {
    "max_wall_time_seconds": 1800,
    "max_memory_bytes": 2147483648,
    "max_output_bytes": 104857600,
    "max_tool_calls": 100,
    "max_cost_usd": 10
  },
  "side_effect_class": "PURE|IDEMPOTENT|AT_MOST_ONCE|IRREVERSIBLE",
  "stop_conditions": [],
  "human_approval": {
    "required": false,
    "approval_policy_id": null,
    "approval_refs": []
  }
}
```

---

# 十九、修订后的 Evidence Manifest 最小合同

```json
{
  "schema_version": "1.0.0",
  "evidence_id": "EVID-...",
  "run_id": "RUN-...",
  "subject": {
    "repository": "xhyperium/infra.rs",
    "commit": "...",
    "module": "goalctl",
    "environment": null
  },
  "authority_snapshot_digest": "...",
  "task_pack_digest": "...",
  "environment_digest": "...",
  "toolchain_digest": "...",
  "commands_ref": "artifact://...",
  "scope_report_ref": "artifact://...",
  "verifier_report_ref": "artifact://...",
  "gate_results_ref": "artifact://...",
  "audit_chain": {
    "chain_id": "...",
    "head_sequence": 0,
    "head_digest": "...",
    "checkpoint_ref": null,
    "external_anchor_ref": null
  },
  "privacy": {
    "classification": "INTERNAL",
    "redaction_policy": "RED-001",
    "contains_secrets": false
  },
  "retention_policy": "RET-RELEASE-001",
  "complete": false
}
```

---

# 二十、修订后的指标体系

## 正确性

```text
authority_resolution_false_negative = 0
state_blocking_false_negative = 0
stale_evidence_detection = 100%
scope_escape_detection = 100% on test corpus
deterministic_output = 100%
```

## 性能

```text
doctor p95 < 1s
index p95 < 3s
resolve p95 < 2s
reconcile p95 < 3s/module
compile p95 < 2s/task
```

## 安全

```text
secret_leak = 0
unauthorized_network = 0
protected_asset_bypass = 0
stale_writer_write = 0
audit_success_without_append = 0
```

## 自动化价值

```text
human_state_resolution_time -70%
prompt_preparation_time -60%
verified_changes / human_minute
repeated_failure_rate
failure_to_eval_conversion
```

## 成本

```text
tokens / accepted task
runner minutes / accepted task
API cost / accepted task
retries / accepted task
```

---

# 二十一、最终评分

| 维度 | 当前 | 补全后目标 |
|---|---:|---:|
| 第一性原理与总体方向 | 95 | 98 |
| 角色与职责边界 | 90 | 96 |
| Authority / SSOT | 68 | 96 |
| Schema / Compatibility | 55 | 95 |
| State Reconciliation | 75 | 95 |
| Evidence / Audit | 80 | 96 |
| Security / Sandbox | 62 | 95 |
| Recovery / Concurrency | 60 | 94 |
| CI / Migration | 72 | 95 |
| Eval / Self-improving | 70 | 95 |
| **综合** | **78** | **95+** |

---

# 二十二、最终推荐路径

不建议立即按原 PR-1 直接编码。

先补一个小型 **PR-0A：Schema / Policy Foundation**，裁定六件事：

```text
1. Authority Policy 的机器 SSOT；
2. Schema Registry 和兼容性；
3. Approval Record；
4. Canonical serialization；
5. goalctl state directory；
6. CLI / JSON / exit-code 稳定合同。
```

然后执行：

```text
PR-1 Doctor / Index
→ PR-2 Authority / Artifact
→ PR-2A Fact Model
→ PR-3 Reconciliation
→ PR-4 Task Compiler
```

在 Harness 前再补：

```text
Capability Policy
Resource Budget
Sandbox Contract
Lease Fencing
Atomic State Store
Cancellation
Evidence Privacy / Retention
```

最终裁定：

> 当前方案不存在方向性失败，但存在“治理规则已完整、运行合同仍不完整”的结构性落差。先补 Schema、Authority Policy、Approval、运行目录和安全资源合同，可以显著降低后续实现返工，并防止 `goalctl` 自身成为新的规则漂移源。
