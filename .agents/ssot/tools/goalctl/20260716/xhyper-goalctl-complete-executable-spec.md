# SPEC-GOALCTL-002：goalctl 生产级完整可执行规范

```text
Document ID:         SPEC-GOALCTL-002
Document Type:       Normative Executable Specification
Status:              PROPOSED
Target Package:      xhyper-goalctl
Binary:              goalctl
Implementation Root: tools/goalctl
Spec Root:           .agents/ssot/tools/goalctl
Baseline Commit:     db5eaee02662fe19dcb3b61a3d7b1390076adb70
Baseline Version:    0.1.0
Target MVA Version:  0.1.1
Target Cutover:      1.0.0 + independent Approved CR
Normative Terms:     MUST / MUST NOT / SHOULD / MAY
```

> 本规范定义终态和分阶段实现合同。未被当前 version-capability matrix 标记为可用的能力，MUST 返回 UNSUPPORTED，不能以“部分实现”或默认成功代替。

---

# 1. 目的

`goalctl` MUST 把仓库中的 Goal Delivery 制品编译为：

- 可复现的 Authority Snapshot；
- 可验证的 Artifact Index；
- 基于事实的 Reconciliation Report；
- commit/tree-bound Task Pack；
- 可执行 Validation Plan；
- 受限 Capability Grant；
- Evidence / Verifier / Gate 输入。

`goalctl` MUST NOT：

- 成为新的治理 SSOT；
- 创建 `.config/goal`；
- 自动批准；
- 自写 G0–G11 PASS；
- 在 Phase 1 修改业务代码；
- 以 Legacy narrative、目录存在或 Writer 自述证明完成；
- 在未批准 Cutover 前替换现有 required check。

---

# 2. 规范优先级

冲突时按仓库 Authority Policy 处理。本规范的局部规则不得覆盖：

1. `CONSTITUTION.md`；
2. Approved CR；
3. Approved ADR；
4. `docs/architecture/spec.md`；
5. `docs/goal/schema/authority-policy.yaml`；
6. 对应 Schema Registry。

本规范与既有 `CLI-CONTRACT.md`、`RUNTIME-STATE.md`、`VERSION-CAPABILITY-MATRIX.md` 冲突时：

- 在本规范未获 Approved CR 前，以既有已批准合同为准；
- 批准本规范时 MUST 同步升级相关合同版本，不允许长期双重语义。

---

# 3. 架构原则

## 3.1 Hexagonal Boundary

核心域 MUST 不直接依赖：

- GitHub API；
- 具体模型 SDK；
- shell 文本格式；
- 全局环境变量；
- live cwd；
- wall clock；
- 生产 Secrets。

建议端口：

```rust
trait RepositoryView
trait GitObjectReader
trait SchemaRegistry
trait AuthorityPolicyProvider
trait ArtifactSource
trait FactObserver
trait ApprovalRegistry
trait EvidenceReader
trait Canonicalizer
trait Clock
trait Runner
trait SignerVerifier
```

## 3.2 单向依赖

```text
domain models
    ↑
pure compiler / reconciler / policy
    ↑
git/fs/schema/evidence adapters
    ↑
CLI / CI / GitHub adapters
```

`crates/**` 业务模块 MUST NOT 依赖 `xhyper-goalctl`。

## 3.3 视图模型

所有读取 MUST 通过显式 Repository View。

```rust
enum RepositoryView {
    Committed {
        commit: GitSha,
        tree_id: GitSha,
    },
    Live {
        head_commit: GitSha,
        dirty_digest: String,
        non_authoritative: true,
    },
}
```

规则：

- Enforcing、Evidence、Gate、Task Pack 只允许 `Committed`；
- `Live` 仅允许 `doctor`、开发诊断和显式 `--view live`；
- `Live` 输出 MUST 包含 `non_authoritative=true`；
- 禁止读取 live 内容后标记为 committed source。

---

# 4. CLI 合同

## 4.1 全局标志

| Flag | 要求 |
|---|---|
| `--json` | stdout 只能有一个 canonical JSON 文档 |
| `--state-dir <path>` | CI/tests MUST 显式传入 |
| `--repo-root <path>` | 默认自 cwd 向上发现 |
| `--source-commit <sha>` | 默认 HEAD 解析后的 40 位小写 SHA；所有 subject-bound 命令必须支持 |
| `--trust-level <level>` | 必须实现并进入输出 |
| `--view committed|live` | 默认 committed；live 不得生成 enforcing 结果 |
| `--schema-dir <path>` | 仅测试/离线验证 MAY 使用；生产默认固定 registry |
| `--no-cache` | 禁用可删除缓存 |
| `--verbosity <level>` | 不影响 JSON 语义 |
| `--help` | exit 0 |
| `--version` | exit 0 |

Trust level：

```text
TRUSTED_INTERNAL
TRUSTED_BOT
UNTRUSTED_FORK
UNTRUSTED_EXTERNAL_SOURCE
```

未知值 MUST exit USAGE。

## 4.2 Phase 1 命令

```text
goalctl version
goalctl doctor
goalctl index [--module <name>]
goalctl resolve [--module <name>]
goalctl artifact inspect <path> [--mode strict|mixed|legacy]
goalctl artifact index [--module <name>] [--mode strict|mixed|legacy]
goalctl reconcile [--module <name>]
goalctl compile (--module <name> | --task-file <path>)
```

## 4.3 后续命令

仅在 capability matrix 开放后：

```text
goalctl evidence verify
goalctl harness run
goalctl verify
goalctl gate evaluate
goalctl shadow compare
goalctl replay
```

未开放时 MUST：

```text
exit 10
GC-UNSUPPORTED-COMMAND
ok=false
```

## 4.4 stdout / stderr

`--json`：

- stdout：唯一 JSON；
- stderr：人类 diagnostics/progress；
- 禁止 ANSI；
- 序列化失败 MUST exit INTERNAL；
- 不得只打印 stderr 而无 JSON failure envelope，除非进程无法初始化。

---

# 5. Exit Codes

| Code | Name | 语义 |
|---:|---|---|
| 0 | OK | 成功且无 error diagnostic |
| 1 | USAGE | 参数或请求非法 |
| 2 | POLICY | 策略拒绝 |
| 3 | NOT_PROVEN | 证据不足 |
| 4 | CONFLICT | 同级冲突或 blocked |
| 5 | IO | Git/文件/外部读取失败 |
| 6 | SCHEMA | 输入或输出 schema 失败 |
| 7 | INTERNAL | 不变量破坏 |
| 8 | TRUST | 签名、runner、identity、bootstrap 信任失败 |
| 9 | STALE | subject/policy/evidence 已变化 |
| 10 | UNSUPPORTED | 当前版本无能力 |
| 11 | BUDGET | 资源预算或超时 |
| 12 | CANCELLED | 明确取消 |

新增 8/9/11/12 属于 CLI 合同 major/minor 变更，落地前 MUST 更新合同和 schema。

---

# 6. Diagnostic 规范

所有代码 MUST 以 `GC-` 开头，不得复用 G0–G11。

最低新增集合：

```text
GC-REPOSITORY-TREE-CHANGED
GC-AUTHORITY-BLOB-CHANGED
GC-WORKTREE-EXTERNALLY-MODIFIED
GC-SUBMODULE-STATE-CHANGED
GC-LFS-OBJECT-MISSING
GC-APPROVAL-INVALID
GC-APPROVAL-EXPIRED
GC-APPROVAL-REVOKED
GC-APPROVAL-SCOPE-MISMATCH
GC-EVIDENCE-STALE
GC-EVIDENCE-SUBJECT-MISMATCH
GC-EVIDENCE-PRODUCER-UNTRUSTED
GC-SCHEMA-IMPLEMENTATION-DRIFT
GC-CANONICALIZATION-MISMATCH
GC-TRUST-LEVEL-DENIED
GC-BOOTSTRAP-VERIFY-FAILED
GC-RESOURCE-BUDGET
GC-EXECUTION-CANCELLED
```

Diagnostic MUST 包含：

```json
{
  "code": "GC-...",
  "severity": "info|warning|error",
  "message": "...",
  "path": null,
  "artifact_id": null,
  "subject": null,
  "remediation": null,
  "details": {}
}
```

---

# 7. Repository Identity

## 7.1 模型

```json
{
  "identity_version": 1,
  "repository_id": "repo:github:1297557216",
  "confidence": "FULL",
  "hosting": {
    "provider": "github",
    "provider_repository_id": 1297557216,
    "canonical_name": "xhyperium/infra.rs"
  },
  "root_commit": "<sha>",
  "aliases": ["xhyperium/infra.rs"]
}
```

## 7.2 规则

- enforcing mode MUST 为 FULL；
- DEGRADED 仅允许本地 read-only/advisory；
- Fork MUST 具有不同 repository_id；
- rename/transfer MUST 产生 `RepositoryIdentityMigration`；
- state/evidence/cache/lease MUST 以 repository_id 隔离；
- numeric id 可由可信 CI input、GitHub adapter 或已批准 identity manifest 提供；
- 未验证的环境变量 MUST NOT 直接升级到 FULL。

---

# 8. Repository Snapshot

## 8.1 模型

```json
{
  "commit": "<40-lower-hex>",
  "tree_id": "<40-lower-hex>",
  "submodule_state_digest": null,
  "lfs_manifest_digest": null,
  "sparse_checkout_digest": null,
  "worktree_mode": "COMMITTED"
}
```

## 8.2 不变量

- `tree_id` MUST 等于 `git rev-parse <commit>^{tree}`；
- request 不能自报任意 tree；
- commit A + tree B MUST fail STALE/SCHEMA；
- `artifact/reconcile/compile/resolve/index` MUST 使用相同 snapshot；
- branch 移动不改变已生成 snapshot；
- merge queue/rebase 产生新 commit 后旧 Evidence MUST STALE；
- submodule/LFS 存在时必须绑定其状态，不能静默忽略。

---

# 9. Canonicalization

## 9.1 规范

优先采用 RFC 8785 JSON Canonicalization Scheme；若暂不采用，必须定义 `XHYPER-CANONICAL-JSON-1`，至少规定：

- UTF-8；
- object key Unicode code point 排序；
- 数字格式；
- string escaping；
- Unicode normalization；
- null/boolean；
- 无 insignificant whitespace；
- 数组顺序语义；
- unordered collection 的预排序规则；
- 禁止 NaN/Infinity；
- 禁止绝对本机路径；
- 禁止 wall clock/random 进入 deterministic object。

## 9.2 测试

- Rust/Python/Go golden vectors；
- 1000+ property cases；
- digest differential test；
- schema output replay；
- version 字段必须进入 digest domain separator。

---

# 10. Schema Registry

## 10.1 根

```text
.agents/ssot/tools/goalctl/schemas/
docs/goal/schema/
```

禁止第三棵 schema 根。

## 10.2 最低 Schema

```text
cli-output.schema.json
repository-identity.schema.json
repository-snapshot.schema.json
authority-snapshot.schema.json
approval-record.schema.json
artifact-envelope.schema.json
artifact-index.schema.json
fact-observation.schema.json
repository-index.schema.json
reconciliation-report.schema.json
task-pack.schema.json
validation-plan.schema.json
capability-grant.schema.json
evidence-manifest.schema.json
verifier-report.schema.json
gate-decision.schema.json
shadow-diff.schema.json
failure-corpus-entry.schema.json
```

## 10.3 兼容性

- Major：breaking；
- Minor：additive；
- Patch：clarification/fix；
- 安全输入 unknown fields fail-closed；
- 报告类可使用 `extensions`；
- Reader 遇更高 major MUST 拒绝；
- 每个 schema 必须有 owner、fixtures、negative fixtures、migration test；
- Rust 类型与 schema MUST 自动比较；
- 不允许只手写 validator 而无 schema differential test。

---

# 11. Authority Resolution

## 11.1 输入

- committed Authority Policy；
- committed authority candidate files；
- ApprovalRecord；
- repository snapshot；
- module/path/capability scope。

## 11.2 输出

`AuthoritySnapshot` MUST 包含：

```json
{
  "schema_version": "1.1.0",
  "repository_snapshot": {},
  "policy_id": "...",
  "policy_version": "...",
  "policy_digest": "sha256:...",
  "schema_bundle_digest": "sha256:...",
  "entries": [],
  "conflicts": [],
  "snapshot_digest": "sha256:..."
}
```

每个 entry：

- path；
- artifact/authority id；
- kind；
- rank；
- status；
- blob SHA；
- SHA-256；
- scope；
- effective range；
- supersedes；
- approval refs；
- approval validation status。

## 11.3 规则

- rank 只能来自 policy；
- 同 rank、同 subject、不同约束冲突 MUST BLOCKED；
- 不同 scope 不应误冲突；
- Approved narrative 无有效 ApprovalRecord 不得晋升；
- revoked/expired/superseded authority 不得继续授权；
- policy 与 candidate 都必须来自同一 commit；
- snapshot MUST 可重放。

---

# 12. Artifact

## 12.1 Parse Mode

### Strict

- 必须有唯一 Control Block；
- Schema 失败立即失败；
- unknown field 按 schema；
- path/type/module/subject 必须一致；
- duplicate artifact_id 失败。

### Mixed

- 优先 Control Block；
- 无 Control Block 可生成 `legacy=true` 的迁移记录；
- MUST warning；
- 不能生成正式 PASS。

### Legacy

- 只读迁移；
- 不产生 structured approval、VERIFIED、RELEASED；
- 禁止 execution lifecycle 词作为 document lifecycle。

## 12.2 Committed Read

- `artifact inspect/index` MUST 接受 `--source-commit`；
- 默认从 Git object database 读取；
- live 文件仅 `--view live`；
- symlink、submodule、LFS pointer 必须显式处理；
- module filter 必须基于精确路径段，不得 substring。

## 12.3 Artifact Envelope

最低字段：

```json
{
  "schema_version": "1.1.0",
  "artifact_type": "goal|spec|design|plan|tasks|...",
  "artifact_id": "...",
  "module": "goalctl",
  "status": "DRAFT|PROPOSED|APPROVED|SUPERSEDED|RETIRED",
  "subject_digest": "sha256:...",
  "source_commit": "<sha>",
  "effective_commit": "<sha|null>",
  "owner": "...",
  "approval_refs": [],
  "evidence_ids": [],
  "gate_ids": [],
  "supersedes": null,
  "extensions": {}
}
```

---

# 13. Fact Observation

Reconcile MUST NOT 直接扫描目录并推断完成状态。必须先生成 `FactObservation`。

## 13.1 Fact 类型

```text
GitTreeFact
CargoWorkspaceFact
TestRunFact
LintRunFact
EvidenceFact
GateVerdictFact
ReleaseTagFact
RegistryFact
DeploymentFact
RuntimeHealthFact
IncidentFact
ApprovalFact
```

## 13.2 Fact 字段

```json
{
  "fact_id": "...",
  "fact_type": "TestRunFact",
  "subject": {
    "repository_id": "...",
    "commit": "...",
    "tree_id": "..."
  },
  "producer": {
    "id": "...",
    "version": "...",
    "trust_level": "..."
  },
  "observed_at": "...",
  "valid_until": null,
  "payload_digest": "sha256:...",
  "evidence_refs": [],
  "value": {}
}
```

## 13.3 规则

- `tests/` 目录存在仅能产生 `TestSurfacePresent`，不能产生 `TestRunPassed`；
- `evidence/` 目录存在不能产生 VerifiedEvidence；
- README/AGENTS 存在不能产生 Operations OK；
- CHANGELOG 存在不能产生 Released；
- 事实必须绑定 subject；
- 事实必须有 producer trust；
- 过期事实降级或 STALE；
- 无事实必须 NOT_PROVEN。

---

# 14. Reconciliation

## 14.1 五维

```text
Specification
Implementation
Verification
Release
Operations
```

## 14.2 强度

强度顺序必须来自 Authority Policy 或独立 machine policy，不得只在 Rust enum 中成为 SSOT。

建议：

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

## 14.3 算法

对每个 dimension：

1. 过滤 subject 不匹配、过期、无效 producer；
2. 按 scope 过滤；
3. 取最高 claim strength；
4. 同强度不同 value → BLOCKED + conflict；
5. 无 claim → NOT_PROVEN；
6. Legacy 不得使 Verification=VERIFIED 或 Release=RELEASED；
7. 生成 explain trace。

## 14.4 输出

```json
{
  "schema_version": "1.1.0",
  "repository_snapshot": {},
  "module": "goalctl",
  "dimensions": {},
  "claims_used": [],
  "claims_rejected": [],
  "contradictions": [],
  "verdict": "CONSISTENT|NOT_PROVEN|BLOCKED|CONTRADICTION",
  "report_digest": "sha256:..."
}
```

## 14.5 Exit

- CONTRADICTION/BLOCKED → 4；
- NOT_PROVEN/UNKNOWN → 3；
- 只有所有必需维度满足 policy 才能 0。

---

# 15. ApprovalRecord

## 15.1 模型

```json
{
  "schema_version": "1.0.0",
  "approval_id": "APPR-...",
  "subject_id": "...",
  "subject_digest": "sha256:...",
  "repository_snapshot": {},
  "scope": {
    "modules": [],
    "paths": [],
    "capabilities": [],
    "actions": []
  },
  "approvers": [
    {
      "identity": "...",
      "role": "Governance|Security|Owner"
    }
  ],
  "quorum_policy_id": "...",
  "approved_at": "...",
  "expires_at": null,
  "status": "ACTIVE|SUSPENDED|REVOKED|EXPIRED|SUPERSEDED",
  "revocation_ref": null,
  "signature": null
}
```

## 15.2 Validator

Protected asset 放行必须验证：

- ref 可解析；
- schema valid；
- subject_id/digest 匹配；
- repository snapshot 匹配；
- path/capability/action scope 覆盖；
- status ACTIVE；
- 未过期；
- quorum 满足；
- approver 与 Writer 职责分离；
- 高风险需 Governance + Security；
- revocation/supersedes 生效。

仅 `approval_refs.len() > 0` MUST NOT 视为有效批准。

---

# 16. Task Compilation

## 16.1 真实编译链

```text
Goal
→ Spec
→ Design（可选但高风险必需）
→ Plan
→ Task DAG
→ Acceptance Criteria
→ Validation Plan
→ Scope
→ Capability Grant
→ Task Pack
```

默认 `compile --module` MUST：

1. 从 Artifact Index 找到该模块的有效 Artifact；
2. 根据 Authority Snapshot 选择有效版本；
3. 验证 trace 完整；
4. 验证 supersedes 与 status；
5. 验证所有 P0/P1 AC 有执行或 verifier；
6. 验证 scope；
7. 验证 approval；
8. 输出 digest-bound Task Pack。

禁止生成虚构 `GOAL-{module}`、`SPEC-{module}`、`PLAN-{module}` 作为已存在事实。

## 16.2 Task Pack

必须包含：

```text
schema_version
task_pack_id
repository_identity
repository_snapshot
authority_snapshot_digest
artifact_index_digest
reconciliation_report_digest
approval_refs
trace
objective
non_goals
acceptance_criteria
validation_plan
scope
capability_grant
resource_budget
side_effect_class
stop_conditions
human_approval
task_pack_digest
```

## 16.3 Scope

统一 PathSpec Grammar：

```text
exact path
directory/**
**
```

Phase 1 不建议支持复杂 `* ? [] {}`，除非采用成熟 glob crate 且固定版本语义。

规则：

- 仓库相对；
- 无 `..`；
- 无绝对路径；
- 无 URI/Windows drive escape；
- normalize 后求交；
- symlink target 不得越界；
- allowed ∩ prohibited 非空失败；
- protected path 无有效 ApprovalRecord 失败。

## 16.4 AC Coverage

每个 P0/P1 AC：

- 必须有 `verification_ref`；
- verification 必须映射到 Validation Plan 或 Independent Verifier；
- 空命令、`true`、`echo pass` 等不可作为高优先级证明；
- validation command 必须有 cwd、timeout、capability、expected output；
- coverage matrix 必须 100%。

---

# 17. Capability Grant

默认 deny：

```json
{
  "filesystem_read": [],
  "filesystem_write": [],
  "network": [],
  "secrets": [],
  "process": [],
  "git_commit": false,
  "git_push": false,
  "github_write": false,
  "production": false,
  "privileged": false
}
```

Trust policy：

| Trust | Secrets | Network | GitHub write | Privileged runner |
|---|---|---|---|---|
| TRUSTED_INTERNAL | policy | policy | policy | policy |
| TRUSTED_BOT | scoped | scoped | draft-only MAY | deny default |
| UNTRUSTED_FORK | none | deny | deny | deny |
| UNTRUSTED_EXTERNAL_SOURCE | none | deny | deny | deny |

Capability Grant MUST 被 Harness 强制执行，不能只写入 JSON。

---

# 18. Harness

## 18.1 Execution

- detached worktree @ commit；
- execution 前复核 tree；
- 单 Writer Lease；
- fencing token；
- process group；
- timeout；
- cancellation；
- disk/cpu/memory/tool-call budget；
- network namespace；
- secrets broker；
- output size limit；
- redaction；
- cleanup/recovery。

## 18.2 Side Effect

```text
PURE
IDEMPOTENT
AT_MOST_ONCE
IRREVERSIBLE
```

- IRREVERSIBLE 必须人工批准；
- Phase 1/2 默认只允许 PURE/IDEMPOTENT；
- Git push、merge、deploy 不在早期能力。

## 18.3 Run Record

必须记录：

- TaskPack digest；
- binary/schema/policy digest；
- runner identity；
- environment digest；
- command；
- start/end monotonic time；
- exit；
- stdout/stderr digest；
- artifacts；
- cancellation；
- resource usage。

---

# 19. Evidence

## 19.1 双轨

### Review Bundle

便于 PR review：

- Goal/Spec/Task trace；
- diff summary；
- AC coverage；
- validation result；
- risk；
- open issue；
- rollback。

### Audit Chain

append-only：

- subject；
- producer；
- digest；
- prior record；
- signature；
- retention；
- redaction；
- external anchor。

## 19.2 新鲜度

Evidence 必须匹配：

```text
repository_id
commit
tree_id
authority_snapshot_digest
schema_bundle_digest
task_pack_digest
environment/runner policy
```

任一变化默认 STALE。

## 19.3 复用

Evidence cache key 必须包含上述 digest。禁止按 branch、PR 号或命令字符串单独复用。

---

# 20. Independent Verifier

Verifier MUST：

- 与 Writer 使用不同执行身份或至少独立上下文；
- 读取 Goal/Spec/TaskPack/Diff/Evidence；
- 主动寻找反例；
- 不修改实现；
- 不接受 Writer summary 作为事实；
- 输出结构化 VerifierReport；
- 对 P0 失败 fail-closed。

Verifier 可包含模型，但最终确定性事实必须来自 tests/evidence/policy。

---

# 21. Gate Adapter

- 只适配既有 G0–G11；
- 不创建新编号；
- 不直接写 Gate 文档为 PASS；
- 输出 `GateDecisionCandidate`；
- final write 由现有 Gate owner/approved workflow 完成；
- PASS_WITH_RISK 仅限 policy 明确允许；
- 高风险仍需人类批准。

---

# 22. Bootstrap Trust

## 22.1 最小 verifier

`goalctl-bootstrap-verify` 只验证：

- goalctl binary digest/signature；
- source commit/provenance；
- schema bundle digest；
- policy bundle digest；
- trusted key；
- compatibility。

## 22.2 规则

- goalctl 不能独立证明自身；
- enforcing binary 必须签名；
- verifier、bootstrap policy、signing key 不得在同 PR 无额外审查同时修改；
- release 必须有 SBOM/provenance；
- 本地 unsigned build 只能 advisory。

---

# 23. Runtime State

默认：

```text
${XDG_STATE_HOME:-$HOME/.local/state}/xhyper-goalctl/<repository-id>/
```

结构：

```text
cache/
leases/
scratch/
logs/
runs/
quarantine/
replay/
```

禁止：

- `./target/**`
- `../.cargo/target/**`
- `.config/goal/**`
- 仓库内隐藏控制面

CI/tests MUST 显式 `--state-dir`.

所有 state 文件：

- owner-only permission；
- atomic write；
- lock/fencing；
- versioned；
- 可删除缓存与不可删除 audit 明确分离。

---

# 24. 安全要求

## 24.1 输入

- path traversal；
- symlink escape；
- malformed JSON/YAML；
- duplicate keys；
- oversized files；
- UTF-8/Unicode confusable；
- zip bomb；
- LFS pointer；
- submodule URL；
- build.rs/proc-macro；
- workflow change；
- executable bit change；
- prompt injection in docs。

均必须有明确策略。

## 24.2 Untrusted Fork

两阶段：

```text
Stage 1: no-secrets static analysis
→ content digest approval
→ Stage 2: trusted build on identical digest
```

## 24.3 Secrets

- 不进入 Task Pack；
- 只引用 secret capability；
- broker 动态注入；
- 输出 redaction；
- 失败日志也必须脱敏。

---

# 25. 性能与 SLO

| 项目 | 目标 |
|---|---:|
| doctor p95 | < 1 s |
| index p95 | < 5 s |
| resolve p95 | < 5 s |
| artifact index p95 | < 10 s |
| reconcile p95 | < 10 s |
| compile p95 | < 5 s |
| deterministic replay | 100% |
| cache corruption acceptance | 0 |
| P0 false negative | 0 |
| schema conformance | 100% |

性能优化 MUST NOT 牺牲 subject binding。

---

# 26. 测试策略

## 26.1 单元测试

- status exact match；
- module exact segment；
- path normalization；
- glob grammar；
- rank/scope；
- approval；
- claim strength；
- AC coverage；
- digest。

## 26.2 Property/Fuzz

- arbitrary path；
- Unicode；
- JSON/YAML；
- Control Block；
- duplicate fields；
- tree/commit；
- glob intersection；
- canonicalization；
- Artifact migration。

## 26.3 Golden

- canonical JSON；
- Schema fixtures；
- Authority Snapshot；
- Artifact Index；
- Reconciliation Report；
- Task Pack；
- diagnostics；
- cross-language digest。

## 26.4 Differential

- Rust model vs JSON Schema；
- committed view vs `git show`；
- legacy tool vs goalctl Shadow；
- path matcher vs reference implementation；
- canonicalizer across languages。

## 26.5 Mutation

- policy allow→deny；
- rank swap；
- approval removal；
- evidence subject change；
- tree change；
- diagnostic suppression；
- default allow injection。

Mutation surviving rate目标为 0（P0 policy）。

## 26.6 E2E

Fixtures MUST 覆盖：

- clean HEAD；
- dirty non-Cargo file；
- dirty Cargo manifest；
- non-HEAD commit；
- detached worktree；
- renamed repo；
- Fork；
- symlink escape；
- submodule；
- LFS；
- stale evidence；
- revoked approval；
- merge queue new commit；
- cancelled run；
- budget exceeded。

---

# 27. 当前缺陷的强制验收条件

## AC-P0-SNAPSHOT

- artifact/reconcile/compile 对 `--source-commit A` 只读 A；
- dirty worktree 不改变输出；
- commit/tree mismatch 非零；
- 输出携带真实 tree。

## AC-P0-RECONCILE

- 只有 Evidence/Fact 可以产生 VERIFIED；
- 只有 Release/Registry Fact 可以产生 RELEASED；
- README/AGENTS 不能产生 Operations OK；
- 无事实 → NOT_PROVEN。

## AC-P0-COMPILE

- 不虚构 Goal/Spec/Plan ID；
- trace artifact 必须存在且 digest 匹配；
- P0/P1 100% coverage；
- invalid approval 失败；
- scope/capability/budget 完整。

## AC-P1-CONTRACT

- CLI flags 与合同一致；
- help snapshot 与 schema 同步；
- 所有命令支持一致 source/view/trust；
- JSON output 100% schema valid。

## AC-P1-DETERMINISM

- 同 snapshot 100 次运行字节一致；
- Linux/macOS/Windows fixture digest 一致；
- Python/Go/Rust canonical vectors 一致。

---

# 28. 实施 PR 波次

## PR-A：事实与文档收敛

允许路径：

```text
.agents/ssot/tools/goalctl/**
tools/goalctl/README.md
tools/goalctl/CHANGELOG.md
docs/goal/change-requests/**
```

任务：

- 落盘 GOAL-GOALCTL-002 / SPEC-GOALCTL-002；
- 创建 CURRENT-STATE；
- 修正过时状态；
- 升级 CLI/VERSION 合同草案；
- 不改实现。

验收：

```bash
test ! -d .config/goal
just goal-check
git diff --check
```

## PR-B：RepositoryView 与 Snapshot

文件建议：

```text
tools/goalctl/src/repository_view.rs
tools/goalctl/src/snapshot.rs
tools/goalctl/src/repo.rs
tools/goalctl/src/main.rs
tools/goalctl/src/lib.rs
```

任务：

- 统一 committed/live view；
- 所有命令接入 `--source-commit`；
- commit/tree 验证；
- dirty/live 标记；
- non-HEAD replay。

## PR-C：Artifact committed read + PathSpec

文件建议：

```text
tools/goalctl/src/artifact.rs
tools/goalctl/src/pathspec.rs
```

任务：

- Git object 读取；
- 精确 module filter；
- 统一 path grammar；
- symlink/submodule/LFS 负例。

## PR-D：Fact Observer + Reconcile

文件建议：

```text
tools/goalctl/src/fact.rs
tools/goalctl/src/observers/**
tools/goalctl/src/reconcile.rs
```

任务：

- 删除目录存在假阳性；
- FactSet；
- freshness；
- rejected claims；
- explain trace。

## PR-E：Approval Registry

文件建议：

```text
tools/goalctl/src/approval.rs
tools/goalctl/src/policy.rs
```

任务：

- ApprovalRecord；
- digest/scope/quorum/expiry/revocation；
- protected path 真实验证。

## PR-F：真实 Task Compiler

文件建议：

```text
tools/goalctl/src/compile.rs
tools/goalctl/src/trace.rs
tools/goalctl/src/validation_plan.rs
tools/goalctl/src/capability.rs
```

任务：

- Artifact trace 编译；
- coverage matrix；
- CapabilityGrant；
- budget；
- 不虚构 ID。

## PR-G：Schema/Canonical Conformance

任务：

- Schema validator/codegen；
- RFC 8785 或 Xhyper canonical v1；
- cross-language golden；
- output validation。

## PR-H：Identity/Trust

任务：

- FULL identity；
- trust-level；
- Fork policy；
- migration。

## PR-I：Evidence/Harness

另 CR 后实施。

## PR-J：Verifier/Shadow

另 CR 后实施。

## PR-K：Mirror/Cutover

独立 Cutover CR，禁止与实现 PR 混合。

---

# 29. 每个实现 PR 的统一验证命令

```bash
test ! -d .config/goal

cargo fmt --all -- --check
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
cargo test -p xhyper-goalctl

cargo run -p xhyper-goalctl -- doctor \
  --state-dir /tmp/xhyper-goalctl-test --json

cargo run -p xhyper-goalctl -- index \
  --source-commit "$(git rev-parse HEAD)" --json

cargo run -p xhyper-goalctl -- resolve \
  --module goalctl \
  --source-commit "$(git rev-parse HEAD)" --json

cargo run -p xhyper-goalctl -- artifact index \
  --module goalctl \
  --mode mixed \
  --source-commit "$(git rev-parse HEAD)" --json

cargo run -p xhyper-goalctl -- reconcile \
  --module goalctl \
  --source-commit "$(git rev-parse HEAD)" --json

cargo run -p xhyper-goalctl -- compile \
  --module goalctl \
  --source-commit "$(git rev-parse HEAD)" --json

cargo xtl lint-deps
cargo xtl naming-check --mode strict
just goal-check
git diff --check
```

当命令尚未实现对应 flag 时，PR-B 必须先更新测试，再更新实现；不得长期保留文档与实现不一致。

---

# 30. CI Jobs

```text
goalctl-contract
goalctl-unit
goalctl-negative
goalctl-property
goalctl-fuzz-smoke
goalctl-schema
goalctl-canonical-crosslang
goalctl-snapshot-replay
goalctl-policy-mutation
goalctl-shadow
```

Required 顺序：

- Phase 1.1：contract/unit/negative/schema；
- Phase 1.2：+ property/snapshot/canonical；
- Shadow：非 required；
- Cutover 后：经 CR 指定的 stable aggregate check。

禁止大量内部 job 成为 branch protection SSOT；应使用稳定聚合 check。

---

# 31. Evidence 要求

每个 PR 必须提供：

- baseline commit；
- changed files；
- Goal/Spec/AC trace；
- positive test；
- negative test；
- schema validation；
- deterministic replay；
- security impact；
- rollback；
- unresolved risk；
- independent review。

P0 修复没有负例证明时不得关闭。

---

# 32. Rollback

- 每个 PR 独立可 revert；
- Schema major migration 必须双读/迁移；
- Cutover 前 legacy required 仍保留；
- Cutover rollback 必须在 15 分钟内恢复 legacy aggregate；
- Audit/Evidence 不因 rollback 删除；
- 回滚不能创建 `.config/goal`；
- 回滚不能放宽 approval/evidence/subject binding。

---

# 33. Shadow → Mirror → Cutover

## Shadow

- advisory；
- 不影响 merge；
- 样本 ≥ 3 模块、≥ 30 PR；
- 记录差异。

## Mirror

- 两条链路都运行；
- 每个差异有分类、owner、deadline；
- P0 false negative = 0；
- rollback drill 通过。

## Cutover

必须：

- 独立 Approved CR；
- Governance + Platform；高风险加 Security；
- branch protection 变更记录；
- stable check name；
- rollback runbook；
- on-call；
- SLO；
- Sunset date。

## Sunset

- consumer scan 为 0；
- docs/CI 无引用；
- 历史 verifier 可保留；
- owner 与删除日期明确。

---

# 34. Self-improving

每次失败生成 `FailureCorpusEntry`：

```json
{
  "failure_id": "...",
  "category": "SNAPSHOT_TOCTOU",
  "subject": {},
  "input_digests": {},
  "observed": {},
  "expected": {},
  "root_cause": "...",
  "fix_refs": [],
  "regression_refs": [],
  "eval_refs": [],
  "status": "OPEN|FIXED|REPLAYED"
}
```

关闭条件：

- root cause；
- regression；
- historical replay；
- owner；
- evidence；
- 无相同类别未解释 failure。

---

# 35. Definition of Done

SPEC-GOALCTL-002 完成必须满足：

- [ ] CR Approved；
- [ ] CLI contract 版本同步；
- [ ] version-capability matrix 同步；
- [ ] RepositoryView 完成；
- [ ] 所有命令 committed-subject；
- [ ] Reconcile 使用 FactSet；
- [ ] ApprovalRecord 完整验证；
- [ ] Compile 真实 trace；
- [ ] Schema/实现自动一致；
- [ ] canonical cross-language；
- [ ] identity/trust 完整；
- [ ] Evidence/Harness/Verifier 完整；
- [ ] Shadow/Mirror 指标达标；
- [ ] rollback drill；
- [ ] Cutover CR；
- [ ] Support/On-call；
- [ ] Failure Corpus/Replay；
- [ ] Legacy Sunset。

---

# 36. 当前优先执行顺序

```text
1. PR-A 文档与合同事实收敛
2. PR-B RepositoryView / commit-tree truth
3. PR-C Artifact / PathSpec
4. PR-D Fact Observer / Reconcile
5. PR-E Approval
6. PR-F True Compiler
7. PR-G Schema / Canonical
8. PR-H Identity / Trust
9. 独立 CR：Evidence / Harness
10. 独立 CR：Verifier / Shadow
11. 独立 CR：Cutover
12. 最后：Agent Writer
```

最终工程原则：

> **任何自动化能力必须建立在更强的事实绑定、验证独立性和回滚能力之上；不能用更高吞吐量交换更低可信度。**
