> **infra.rs 说明**：本文为控制面生产 Goal/Spec，落在 `.agents/ssot/tools/`。  
> 组件期望路径：`tools/{goalctl,verifyctl}`；**本仓尚未创建对应 crate**。  
> 对齐：[docs/ssot/tools-ssot-alignment.md](../../../../../docs/ssot/tools-ssot-alignment.md)

---

# verifyctl：生产实施 Spec

> Spec ID：`SPEC-2026-VERIFYCTL-001`  
> 对应 Goal：`GOAL-2026-VERIFYCTL-001`  
> 文档版本：1.0.0  
> 状态：Proposed  
> 风险等级：R3  
> 组件路径：`tools/verifyctl`  
> Parent Spec：`SPEC-2026-0001-vibe-coding-self-verification`

---

## 0. 实施裁定

`verifyctl` 采用 library-first 的 Rust 架构，将规划与执行严格分离：

- Planner 是确定性、无网络、无副作用的纯逻辑层。
- Executor 是最小权限、资源受限、可取消的运行层。
- Aggregator 只汇总事实，不派生 Merge/Release 授权。
- 下游 `evidence` 只消费 Schema 合法、绑定完整的 Run Result。

任何 CI YAML 中复制的影响判断、检查选择或通过逻辑都不构成正式实现。

## 1. 目录与单一事实源

```text
.agent/
├── verification/
│   ├── policy.yaml
│   ├── profiles/
│   │   ├── local.yaml
│   │   ├── fast.yaml
│   │   ├── depth.yaml
│   │   └── release.yaml
│   ├── checks/
│   │   ├── rust.yaml
│   │   ├── architecture.yaml
│   │   ├── security.yaml
│   │   └── integration.yaml
│   └── fixtures/
├── schemas/
│   ├── verification-policy-v1.schema.json
│   ├── verification-check-v1.schema.json
│   ├── verification-plan-v1.schema.json
│   └── verification-run-v1.schema.json
└── compiled/verification/
    └── policy.lock.json

tools/verifyctl/
├── Cargo.toml
├── src/
├── tests/
├── benches/
└── README.md

target/verifyctl/
├── plans/
├── runs/
├── logs/
└── artifacts/
```

规则：

- `.agent/verification/**` 是可评审的策略 SSOT。
- `policy.lock.json` 是规范化派生物，必须提交并验证 digest。
- `target/verifyctl/**` 是临时输出，不得提交。
- CI workflow 只能传递上下文并调用 CLI，不得重写 Check、风险或通过规则。

## 2. 输入契约

Planner 的完整输入为：

```text
PlanInput = {
  repository_snapshot,
  subject,
  base,
  normalized_change_set,
  goal_contract,
  verification_policy,
  architecture_graph,
  tool_manifest,
  requested_profile,
  execution_context
}
```

所有输入均使用内容摘要绑定。环境时间、随机数、主机名和目录绝对路径不得参与语义规划。

### 2.1 Subject

```yaml
subject:
  vcs: git
  repository: infra.rs
  commit_sha: 9f6d...c81
  tree_sha: 41a2...e10
  source_digest: sha256:2f38...a05
```

- PR：Subject 为精确 PR head SHA。
- Merge Queue：Subject 为精确 `merge_group` synthetic SHA。
- Release：Subject 同时绑定 Commit、Tree 和构建输入摘要。
- 工作区有未提交变化时，仅允许 `developer-local`，并以 canonical patch digest 绑定。

### 2.2 Base

Base 必须是已解析的 Commit SHA，禁止把可移动分支名直接写入 Plan。Planner 可接受 `origin/main` 作为用户输入，但必须在规划前解析并固化 SHA。

### 2.3 Goal Contract

只接受 `goalctl compile` 产生且 Schema 合法的 Contract；至少读取：

- `goal_id`
- `contract_digest`
- `risk.level`
- `risk.escalation_triggers`
- `acceptance_criteria[]`
- `invariants[]`
- `constraints[]`
- `required_evidence[]`

## 3. Verification Policy Schema

```yaml
apiVersion: agent.xhyperium.dev/v1
kind: VerificationPolicy

metadata:
  id: verification-policy-main
  version: 1.0.0

spec:
  default_profile: fast
  default_on_unknown: full-workspace
  trust_order:
    - untrusted
    - developer-local
    - ci-trusted
    - release-trusted

  budgets:
    local:
      wall_time: 90s
      cpu: 4
      memory: 8Gi
    fast:
      wall_time: 300s
      cpu: 16
      memory: 32Gi
    depth:
      wall_time: 60m
      cpu: 32
      memory: 64Gi
    release:
      wall_time: 90m
      cpu: 32
      memory: 64Gi

  escalation:
    full_workspace_paths:
      - Cargo.toml
      - Cargo.lock
      - rust-toolchain.toml
      - .cargo/**
      - .agent/**
      # .architecture/** — monorepo 历史路径；infra.rs 不维护、不移植 archgate（OOS）
      - .architecture/**
    semantic_triggers:
      - build-script
      - proc-macro
      - code-generator
      - gate-engine
      - evidence-engine
      - authentication
      - authorization
      - migration
    on_graph_incomplete: full-workspace
    on_policy_conflict: fail

  cache:
    enabled: true
    require_artifact_digest: true
    namespace_by_trust: true
    max_age: 24h

  execution:
    network_default: deny
    repository_write: deny
    secret_default: deny
    retry_default: 0
    kill_grace_period: 5s
```

### 3.1 Policy 合并

组织策略、仓库策略、Goal 风险和 Profile 按以下原则合并：

1. 更严格 Trust 要求优先。
2. 更大的 Check 集合优先。
3. 更短超时不得覆盖 Check 声明的安全最低时长；发生冲突则失败。
4. Deny 覆盖 Allow。
5. Goal 风险升级器只能扩张验证闭包。
6. 未知字段在 major version 相同情况下失败，禁止忽略。

## 4. Check Schema

```yaml
apiVersion: agent.xhyperium.dev/v1
kind: VerificationCheck

metadata:
  id: rust.workspace.test
  version: 1.2.0
  owners: [platform]

spec:
  description: 对选择后的 workspace package 执行测试
  runner: process
  command:
    executable: cargo
    args: [test, --locked, --workspace, --all-targets]
  inputs:
    selectors: [rust-package]
    files: [Cargo.toml, Cargo.lock, rust-toolchain.toml]
    environment_allowlist: [CARGO_HOME, RUSTUP_HOME, RUST_BACKTRACE]
  outputs:
    result: junit
    artifacts: [target/nextest/**]
  coverage:
    capabilities: [unit-test, integration-test]
    goal_refs: []
    invariant_refs: []
  execution:
    network: deny
    repository_write: deny
    timeout: 240s
    retry:
      count: 0
      on: []
  trust:
    minimum_origin: ci-trusted
    independent: false
  dependencies:
    requires: [rust.workspace.build]
```

字段规则：

- `metadata.id` 全仓唯一且不可复用。
- `command.executable` 必须从 Tool Manifest 解析到固定版本。
- Shell 字符串默认禁止；必须使用 executable/args 数组。
- 所有环境变量均白名单传递，未声明变量被清空。
- `network`、`repository_write`、`secrets` 必须显式声明。
- `coverage.goal_refs` 和 `invariant_refs` 必须能解析到当前 Contract。
- `retry.count > 0` 必须声明可重试错误类别，并保留每次尝试。
- `independent: true` 只能由受保护 Policy 定义，运行者不可覆盖。

## 5. Profile 语义

| Profile | 用途 | 最低 Trust | 目标预算 | 允许缩小范围 |
|---|---|---:|---:|---|
| `local` | 编码反馈 | `developer-local` | 90s | 是；未知则提示全量或失败 |
| `fast` | PR/merge queue | `ci-trusted` | 300s | 是；未知自动全量 |
| `depth` | 独立反证 | `ci-trusted` | 60m | 仅按明确策略 |
| `release` | 发布候选 | `release-trusted` | 90m | 默认否 |

Profile 只选择预算、强度和 Trust，不能重新定义 Check 的事实语义。

## 6. 变更集规范化

执行：

```text
git diff --find-renames --find-copies --name-status -z <base>..<subject>
```

规范化条目：

```json
{
  "status": "renamed",
  "old_path": "crates/a/src/api.rs",
  "new_path": "crates/b/src/api.rs",
  "old_blob": "sha256:...",
  "new_blob": "sha256:...",
  "mode_changed": false
}
```

要求：

- rename/copy 同时进入旧、新路径影响分析。
- 删除仍保留旧路径所属组件和消费者。
- submodule 指针、可执行位、symlink 目标变化必须记录。
- 路径必须为 UTF-8 正规化仓库相对路径；禁止 `..` 和绝对路径。
- Git 报错、浅克隆缺失 Base 或 LFS 材料缺失时不得生成缩小 Plan。

## 7. 影响分析算法

### 7.1 图模型

统一影响图包含：

- `File → Rust Target`
- `Target → Package`
- `Package → Dependency`
- `Package → Reverse Dependency`
- `Feature → Target/Dependency`
- `Build Script/Proc Macro/Generator → Consumer`
- `Schema/Protocol → Producer/Consumer`
- `Architecture Component → Allowed/Forbidden Edge`
- `Check → Capability → AC/INV`

### 7.2 规划步骤

```text
1. 验证所有输入 Schema 与 digest。
2. 规范化 Change Set。
3. 将文件映射到组件、package、target、feature 和控制面类别。
4. 应用风险与全量升级触发器。
5. 计算有界反向传递闭包。
6. 加入集成消费者、协议、Feature 和 Target 矩阵。
7. 解析 Goal AC/INV 的 Check 覆盖要求。
8. 合并 Profile 基线 Check。
9. 构建并验证 DAG；检测环和悬空依赖。
10. 计算 Build Key、Cache Key 和资源声明。
11. 按稳定键排序并规范化序列化。
12. 计算 Plan digest。
```

### 7.3 保守升级

以下情况必须选择 `full-workspace`，并在 Plan 中记录原因码：

- 根依赖、锁文件、编译器、Cargo 配置变化。
- 依赖图或架构图解析失败/版本不兼容。
- 文件不能映射到已知组件。
- 过程宏、构建脚本、链接参数、生成器或公共 Schema 变化。
- 控制面、安全、权限、迁移、发布相关变化。
- Goal 风险为 R3 且策略未明确允许缩小。

禁止使用 LLM 自由文本判断替代该算法。

## 8. Coverage Closure

Planner 为每个 Required AC/INV 建立覆盖项：

```json
{
  "requirement_id": "VCTL-INV-009",
  "requirement_digest": "sha256:...",
  "required_trust": "ci-trusted",
  "checks": ["verifyctl.cache.trust-isolation"],
  "closure": "covered"
}
```

闭合规则：

- `checks` 为空：规划失败。
- Check 被 Profile 排除：规划失败，除非要求明确 `not_applicable` 且有 Policy rule。
- 只有所有 Required Check 为稳定 Pass 且 Trust 足够，运行态覆盖才为 satisfied。
- Manual Control 必须有 owner、时效、审批证据类型和不能自动化的原因。
- 一个 Check 可覆盖多个要求，但必须声明具体映射，禁止 `covers: all`。

## 9. Plan Schema

```json
{
  "apiVersion": "agent.xhyperium.dev/v1",
  "kind": "VerificationPlan",
  "metadata": {
    "plan_id": "vplan_01J...",
    "plan_digest": "sha256:...",
    "planner_version": "verifyctl/1.0.0"
  },
  "binding": {
    "subject_digest": "sha256:...",
    "base_commit": "...",
    "goal_id": "GOAL-2026-VERIFYCTL-001",
    "goal_contract_digest": "sha256:...",
    "policy_digest": "sha256:...",
    "architecture_digest": "sha256:...",
    "tool_manifest_digest": "sha256:..."
  },
  "profile": "fast",
  "risk": {"effective": "R3", "reasons": ["control-plane-change"]},
  "impact": {
    "mode": "full-workspace",
    "components": ["tools/verifyctl"],
    "packages": ["verifyctl"],
    "reasons": ["verification-policy-change"]
  },
  "nodes": [],
  "coverage": [],
  "budget": {"wall_time_ms": 300000},
  "created_from": {"input_digest": "sha256:..."}
}
```

Plan 不写入“当前时间”；`plan_id` 必须由输入摘要导出或作为非语义 envelope 字段。参与 digest 的 payload 必须完全确定。

## 10. DAG 与编译去重

节点类型：

- `prepare`
- `build`
- `check`
- `aggregate`

稳定排序键：`stage → build_key → check_id → shard_id`。

Build Key 至少包含：

```text
H(
  subject_tree,
  toolchain,
  compiler_flags,
  target_triple,
  profile,
  feature_set,
  dependency_lock,
  build_script_inputs,
  environment_allowlist_values,
  generator_digests
)
```

若两个 Check 的任一材料不同，禁止强行合并 Build Key。相同 Key 只保留一个 Build Node，消费者通过只读 artifact reference 使用。

## 11. Cache Key 与 Trust 隔离

```text
CacheKey = H(
  build_or_check_key,
  subject_digest,
  tool_manifest_digest,
  policy_semantics_digest,
  fixture_digest,
  sandbox_profile_digest,
  trust_namespace
)
```

规则：

- 命中前验证 metadata、payload digest 和实际文件 digest。
- `developer-local`、`ci-trusted`、`release-trusted` 使用不同命名空间。
- 只能同级复用，或由 Policy 明确允许高 Trust 向低 Trust 复用；禁止反向。
- 缓存损坏视为 Miss 并产生诊断，不视为 Check failure。
- 安全检查、时间敏感检查和外部撤销检查默认不可缓存。
- Cache metadata 中不得保存 secret、token、用户主目录或绝对工作路径。

## 12. 执行器

### 12.1 沙箱基线

- Linux 容器或等价隔离；固定镜像 digest。
- 非 root 用户、只读根文件系统、最小 capabilities。
- 仓库只读挂载；构建输出使用独立临时卷。
- 默认无网络、无 Secret、无云元数据服务访问。
- CPU、内存、进程、文件、磁盘和 wall time 限额。
- 清空非白名单环境变量。
- 日志实时转义控制字符并施加大小上限。

### 12.2 外部服务检查

允许的顺序：

1. 内存 Fixture 或录制且脱敏的契约 Fixture。
2. 临时容器化依赖，绑定镜像 digest。
3. 专用测试环境，使用短期最小权限身份。

禁止默认访问生产服务。任何网络 Allowlist 必须由 Check 声明且受 Policy 审核。

### 12.3 超时、取消与重试

状态模型：

```text
planned → queued → running → passed|failed|timed_out|cancelled|infra_error
```

- 超时先发 TERM，经过 `kill_grace_period` 后 KILL 整个进程组。
- 用户取消、上游失败取消和预算取消使用不同 reason code。
- 默认不重试测试失败。
- 仅可重试显式基础设施错误；每次尝试都进入 Run Result。
- 任一尝试失败而后通过，最终稳定性标记为 `flaky_observed`。

### 12.4 仓库写保护

执行前后分别计算 tracked-files tree digest。若变化：

- 当前 Check 状态为 `failed`。
- 输出 `VCTL-E-REPOSITORY-MUTATION`。
- 保存路径清单，不自动恢复或提交。

格式化检查必须使用 check/diff 模式，不能直接改写。

## 13. Run Result Schema

```json
{
  "apiVersion": "agent.xhyperium.dev/v1",
  "kind": "VerificationRun",
  "metadata": {
    "run_id": "vrun_01J...",
    "runner_version": "verifyctl/1.0.0",
    "schema_version": "1"
  },
  "binding": {
    "plan_digest": "sha256:...",
    "subject_digest": "sha256:...",
    "goal_contract_digest": "sha256:...",
    "policy_digest": "sha256:..."
  },
  "origin": {
    "trust_claim": "ci-trusted",
    "independent_claim": false,
    "provider": "github-actions",
    "workflow_ref": "...",
    "run_attempt": 1
  },
  "started_at": "2026-07-20T18:00:00Z",
  "finished_at": "2026-07-20T18:04:10Z",
  "status": "failed",
  "checks": [],
  "coverage": [],
  "artifacts": [],
  "diagnostics": []
}
```

重要：`origin.trust_claim` 只是自述。最终 Trust 由 `evidence` 根据运行身份、环境证明和 Policy 派生。

### 13.1 Check Result

每项至少包含：

- Check ID/version/definition digest。
- 规范化 executable 与 args；Secret 参数只保留占位符。
- Tool digest、sandbox image digest、Fixture digest。
- 开始/结束时间、单调时钟 duration。
- 状态、退出码、signal、reason code。
- 每次 attempt 的独立记录。
- stdout/stderr digest 与截断标记。
- JUnit、coverage、SBOM、扫描报告等 artifact reference。
- 输入材料和输出制品 digest。

## 14. Independent Verifier

独立性由受保护身份和执行关系证明，不接受命令行布尔值：

- Builder principal 与 Verifier principal 必须不同。
- Independent Job 的 workflow、environment、runner group 受保护。
- Verifier 输入只能是已绑定 Subject 和 Goal Contract。
- Verifier 不读取 Builder 的未签名“通过结论”；可读取不可变构建制品，但必须复核 digest。
- R3 默认执行反例、属性、变异、边界、权限和失败路径检查。
- `independent_claim` 由 runner adapter 注入，CLI 用户参数无法设为真。

## 15. 权限分离 CI

Fast CI 分为两个信任域：

1. **Unprivileged Verification Job**
   - checkout 并执行 PR Subject 代码。
   - `permissions: contents: read`，无 OIDC、无发布 Secret。
   - 上传原始 Run Result 和 artifact。

2. **Privileged Evidence/Gate Job**
   - 不 checkout、不执行 PR Subject 代码。
   - 下载固定 artifact ID，验证 digest。
   - 可按需获取 OIDC，用于签名 Evidence。
   - 运行来自受保护基线的 `evidence` 与 `gate` 二进制/Policy。

PR、`merge_group`、release 各自重新绑定精确 Subject，禁止仅凭 workflow 名称复用。

## 16. CLI

```text
verifyctl plan \
  --repository . \
  --base <commit> \
  --subject <commit> \
  --binding .agent/bindings/<goal>.json \
  --profile fast \
  --output target/verifyctl/plans/plan.json

verifyctl run --plan <plan.json> --output <run.json>
verifyctl verify --base <commit> --subject <commit> --binding <file> --profile fast
verifyctl explain --plan <plan.json> [--check <id>|--path <path>|--requirement <id>]
verifyctl coverage --plan <plan.json>
verifyctl replay --run <run.json> --materials <directory>
verifyctl compare --left <plan-or-run> --right <plan-or-run>
verifyctl list checks|profiles|capabilities
verifyctl doctor --repository .
```

全局参数：

- `--format human|json`
- `--no-color`
- `--log-level error|warn|info|debug`
- `--repository <path>`
- `--handoff <json>`：供编排器传递已经解析的严格上下文。

禁止提供 `--skip-required`、`--trust-me`、`--mark-pass` 或用户可设置的 `--independent`。

## 17. Rust 架构

```text
tools/verifyctl/src/
├── main.rs
├── cli.rs
├── app.rs
├── schema/
├── binding/
├── change_set/
├── graph/
│   ├── cargo.rs
│   ├── architecture.rs
│   └── protocol.rs
├── impact/
├── coverage/
├── policy/
├── planner/
├── dag/
├── build_key/
├── cache/
├── executor/
│   ├── sandbox.rs
│   ├── process.rs
│   └── limits.rs
├── result/
├── replay/
└── diagnostics/
```

核心接口：

```rust
pub trait ImpactResolver {
    fn resolve(&self, input: &ImpactInput) -> Result<ImpactClosure, ImpactError>;
}

pub trait Planner {
    fn plan(&self, input: &PlanInput) -> Result<VerificationPlan, PlanError>;
}

pub trait Executor {
    fn execute(
        &self,
        plan: &VerificationPlan,
        cancel: &CancellationToken,
    ) -> Result<VerificationRun, ExecutionError>;
}

pub trait CacheStore {
    fn lookup(&self, key: &CacheKey, minimum_trust: Trust) -> Result<CacheLookup, CacheError>;
    fn store(&self, entry: VerifiedCacheEntry) -> Result<(), CacheError>;
}
```

领域类型不得用自由字符串代替 `SubjectDigest`、`PlanDigest`、`CheckId`、`Trust`、`Risk` 和 `DurationBudget`。

## 18. 确定性与序列化

- 使用 JSON Canonicalization Scheme 或项目批准的等价规范。
- Map 在序列化前按 Unicode code point 排序。
- Duration 统一为整数毫秒，时间戳统一 UTC RFC 3339。
- Planner 语义 payload 不包含 wall clock、随机 UUID、线程调度顺序。
- 路径统一 `/` 分隔，移除工作目录前缀。
- 浮点数禁止进入摘要 payload。
- DAG 并发执行不影响最终 Check 排序。

## 19. 错误与退出码

| 退出码 | 类别 | 语义 |
|---:|---|---|
| 0 | Success | 规划成功，或所有 Required Check 稳定通过 |
| 2 | Invalid Input | Schema、参数、digest 或绑定错误 |
| 3 | Planning Blocked | 覆盖缺口、未知影响不能升级、策略冲突、DAG 环 |
| 4 | Verification Failed | 一个或多个 Required Check 失败 |
| 5 | Infrastructure Error | Runner、缓存、磁盘、容器等基础设施故障 |
| 6 | Timeout/Cancelled | 运行超时或被取消 |
| 7 | Internal Error | 不变量破坏或未分类内部错误 |

机器错误格式：

```json
{
  "code": "VCTL-E-COVERAGE-GAP",
  "message": "required invariant has no check",
  "details": {"requirement_id": "INV-..."},
  "retryable": false
}
```

错误消息不得包含 Secret、Token、完整环境变量或用户目录。

## 20. 可观测性

指标：

- `verifyctl_plan_duration_seconds`
- `verifyctl_run_duration_seconds{profile,status}`
- `verifyctl_check_duration_seconds{check_id,status}`
- `verifyctl_impact_mode_total{mode,reason}`
- `verifyctl_cache_lookup_total{result,trust}`
- `verifyctl_duplicate_builds_total`
- `verifyctl_flaky_observed_total{check_id}`
- `verifyctl_coverage_gap_total`

Trace 只记录 digest 和稳定 ID，不记录源码、Secret 或完整命令输出。指标标签不得包含 commit SHA、路径或用户输入，以防高基数。

## 21. 性能与容量

- 10,000 文件、500 crate 的仓库，冷规划 P95 `≤ 5s`。
- 热规划 P95 `≤ 1s`。
- Plan 内存峰值 `≤ 512MiB`。
- 单 Check 日志默认上限 50MiB，超过后截断并保存摘要。
- Artifact 总量由 Profile 限制，超限状态为 `infra_error`，不得丢弃后声称成功。
- 规划器支持至少 5,000 DAG 节点；超过 Policy 上限则显式失败。

## 22. 安全要求

- 所有仓库路径先 canonicalize 并验证仍在 repository root 下。
- 不通过 shell 拼接来自 Goal、文件名或 Policy 的输入。
- 防止 ANSI/终端注入、日志伪造和路径穿越。
- Artifact 解包限制条目数、总大小、压缩比、symlink 和目标路径。
- Runner 镜像、工具链、外部 action 均固定不可变 digest。
- 任何执行 PR 代码的 Job 都不得持有 OIDC 或生产 Secret。
- 临时身份必须最小权限、短期、绑定 workflow 和 Subject。
- 对 Policy/Check 变更执行旧版本 `verifyctl` 的全量与影子评估。

## 23. 兼容与迁移

- Schema 使用 `apiVersion` 和 `kind`。
- Minor 版本只允许新增可选字段；Major 版本可破坏兼容。
- Reader 至少支持当前和前一个 Major；Writer 只写当前版本。
- Policy 升级采用：旧引擎验证新 Policy → 双写/双读 → 影子比较 → 切换 → 兼容窗口结束。
- 新引擎或新 Policy 不能使用自己生成的结果为自己解除门禁。

## 24. 测试策略

### 24.1 单元与属性测试

- Change Set 解析：rename/delete/copy/symlink/submodule/非 UTF-8 错误。
- 图闭包：环、多 target、可选依赖、Feature unification、build dependency。
- 单调性属性：风险升高后 Check 集合为超集。
- 确定性属性：Map 顺序、线程数、目录位置变化不改变 Plan digest。
- Cache 属性：任一 Key 材料变化必 Miss；低 Trust 永不满足高 Trust。

### 24.2 Golden 与变异测试

- 维护至少 30 个真实变更 Golden Plan。
- 修改影响规则必须显式批准 Golden diff。
- 对“全量升级”“Required Check”“Trust 比较”条件做 mutation testing；存活变异为发布阻断。

### 24.3 对抗测试

- 恶意文件名、路径穿越、参数注入、ANSI 注入。
- Zip bomb、symlink artifact、伪造 JUnit 和摘要冲突。
- PR 修改 Check 定义试图降低验证。
- 缓存污染、跨 Trust 命名空间复制、过期 Fixture。
- 进程 fork 后超时逃逸、后台进程和磁盘耗尽。

### 24.4 端到端测试

- PR head → Fast Run Result。
- merge queue synthetic SHA → 新 Plan 和新 Run。
- R3 变更 → Full Workspace + Independent Depth。
- Release Commit → Release Profile + release-trusted origin claim。
- 执行 Job 无权限，Evidence Job 不执行 Subject 代码。

## 25. 实施任务

| ID | 任务 | 交付物 | 前置 |
|---|---|---|---|
| `VCTL-T01` | Schema 与规范化 | 4 类 Schema、canonicalizer | 无 |
| `VCTL-T02` | Git Change Set | rename/delete 等完整解析器 | T01 |
| `VCTL-T03` | Cargo 图 | package/target/feature/反向依赖图 | T01 |
| `VCTL-T04` | 架构与协议图 | 跨组件消费者关系 | T03 |
| `VCTL-T05` | 风险与影响闭包 | 保守升级、原因码 | T02–T04 |
| `VCTL-T06` | Coverage Closure | AC/INV 到 Check 映射 | T01 |
| `VCTL-T07` | DAG Planner | 确定性 Plan、环检测 | T05–T06 |
| `VCTL-T08` | Build/Cache Key | 去重、Trust 隔离 | T07 |
| `VCTL-T09` | Sandbox Executor | 限权、限额、取消 | T07 |
| `VCTL-T10` | Result Aggregator | Run Schema、artifact 摘要 | T09 |
| `VCTL-T11` | Explain/Replay | 可解释与可重放 CLI | T07–T10 |
| `VCTL-T12` | CI 权限分离 | Fast/Depth/Release workflow | T10 |
| `VCTL-T13` | 安全与对抗测试 | 威胁模型、攻击用例 | T08–T12 |
| `VCTL-T14` | SLO 与运维 | Dashboard、告警、Runbook | T12–T13 |

## 26. Spec 验收条件

| ID | 可执行验收 |
|---|---|
| `VCTL-SPEC-AC-001` | 四类 Schema 通过正例、反例和向后兼容测试。 |
| `VCTL-SPEC-AC-002` | Planner 拒绝 digest 与实际内容不一致的任一输入。 |
| `VCTL-SPEC-AC-003` | 可移动 Base ref 在 Plan 中被解析为固定 Commit SHA。 |
| `VCTL-SPEC-AC-004` | 工作区脏状态只能产生不高于 `developer-local` 的 Run。 |
| `VCTL-SPEC-AC-005` | rename 的旧、新路径均进入影响闭包。 |
| `VCTL-SPEC-AC-006` | 删除 package 对所有反向依赖产生验证节点。 |
| `VCTL-SPEC-AC-007` | 根 Cargo/lock/toolchain/.cargo 变化选择 full-workspace。 |
| `VCTL-SPEC-AC-008` | build script、proc macro、generator 变化选择全部消费者。 |
| `VCTL-SPEC-AC-009` | 依赖图损坏或未知路径时 Planner 不产生缩小 Plan。 |
| `VCTL-SPEC-AC-010` | Goal 风险提高的属性测试证明 Check 集合单调扩张。 |
| `VCTL-SPEC-AC-011` | 每个 Required AC/INV 无映射时退出码为 3。 |
| `VCTL-SPEC-AC-012` | `not_applicable` 只能由匹配的 Policy rule 产生。 |
| `VCTL-SPEC-AC-013` | DAG 环、悬空依赖和重复 Check ID 均在执行前失败。 |
| `VCTL-SPEC-AC-014` | 同一 Build Key 只创建一个 Build Node。 |
| `VCTL-SPEC-AC-015` | 编译参数、Feature、Toolchain、Lock 任一变化生成新 Build Key。 |
| `VCTL-SPEC-AC-016` | Cache artifact 实际摘要不匹配时视为 Miss 并告警。 |
| `VCTL-SPEC-AC-017` | `developer-local` Cache 无法命中 `ci-trusted` namespace。 |
| `VCTL-SPEC-AC-018` | Planner 在不同工作目录和线程数下生成相同 digest。 |
| `VCTL-SPEC-AC-019` | Runner 默认无网络、无 Secret、仓库只读且非 root。 |
| `VCTL-SPEC-AC-020` | 恶意 Check 参数不能触发 shell 注入。 |
| `VCTL-SPEC-AC-021` | 超时会终止整个进程组并记录 `timed_out`。 |
| `VCTL-SPEC-AC-022` | 用户取消、预算取消和上游取消有不同 reason code。 |
| `VCTL-SPEC-AC-023` | 测试失败默认不重试；基础设施重试保留所有 attempt。 |
| `VCTL-SPEC-AC-024` | 失败后重试通过的结果标记 `flaky_observed`。 |
| `VCTL-SPEC-AC-025` | tracked file 被 Check 修改时返回仓库变异错误。 |
| `VCTL-SPEC-AC-026` | Run Result 的命令和环境字段完成 Secret 脱敏。 |
| `VCTL-SPEC-AC-027` | 日志超限会截断、标记并保留完整内容摘要。 |
| `VCTL-SPEC-AC-028` | Artifact 路径穿越、symlink 逃逸和压缩炸弹被拒绝。 |
| `VCTL-SPEC-AC-029` | Builder 无法通过 CLI 或环境变量设置 independent 为真。 |
| `VCTL-SPEC-AC-030` | Independent principal 与 Builder principal 相同时验证失败。 |
| `VCTL-SPEC-AC-031` | PR 执行 Job 的权限扫描证明无 OIDC 和发布 Secret。 |
| `VCTL-SPEC-AC-032` | Evidence/Gate Job 的测试证明不 checkout 或执行 PR Subject。 |
| `VCTL-SPEC-AC-033` | PR head Run 不满足 merge queue synthetic SHA 的精确绑定。 |
| `VCTL-SPEC-AC-034` | `explain` 能为每个选中/未选 Check 输出稳定原因码。 |
| `VCTL-SPEC-AC-035` | `replay` 检测任何材料缺失或摘要漂移并停止。 |
| `VCTL-SPEC-AC-036` | 10,000 文件、500 crate 冷规划 P95 不超过 5 秒。 |
| `VCTL-SPEC-AC-037` | Fast 基准 P95 不超过 300 秒，且重复编译数为 0。 |
| `VCTL-SPEC-AC-038` | 连续 100 次确定性测试无 Plan payload 差异。 |
| `VCTL-SPEC-AC-039` | 关键分支 mutation score 达到团队批准阈值，且无存活 Gate 绕过变异。 |
| `VCTL-SPEC-AC-040` | `evidence collect` 能直接验证并消费 Run Result Schema。 |

## 27. 发布、回滚与故障处理

### 27.1 发布

- 二进制、Schema、Policy Lock 和 Tool Manifest 分别生成 digest。
- 先在非阻断影子模式比较旧/新 Plan 至少 7 天。
- 所有缩小差异必须由架构和安全 Owner 审批。
- 使用受保护基线的旧版工具决定新版能否启用。

### 27.2 回滚

- 保留当前和上一个已批准二进制与 Policy。
- 回滚只能切回已知 digest，不从 PR 分支现场构建。
- 回滚不删除既有 Run Result；新 Run 使用新 Plan ID。

### 27.3 降级

- 缓存不可用：禁用缓存继续执行。
- 影响服务不可用：切换 full-workspace。
- Runner 容量不足：状态为 blocked/infra_error，不宣称 Pass。
- Evidence 服务不可用：保留原始 Run，Gate 继续默认拒绝。

## 28. Definition of Done

实现完成必须满足：

- 所有 `VCTL-SPEC-AC-001` 至 `VCTL-SPEC-AC-040` 具备自动化或批准的演练 Evidence。
- Goal 的全部 14 条不变量已映射到持续检查。
- Planner 核心无网络、无时钟、无随机数、无执行 Subject 代码路径。
- Executor 通过最小权限和沙箱逃逸安全评审。
- CI 权限分离、merge queue、Depth、Release 四条链路完成 E2E。
- 性能 SLO、容量上限、告警、Runbook、回滚和 Schema 迁移经生产演练。
- 输出能被 `evidence` 无转换消费；任何 Trust 仍由 `evidence` 派生。
