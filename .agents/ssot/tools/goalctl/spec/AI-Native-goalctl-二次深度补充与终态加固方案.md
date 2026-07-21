# AI-Native / `goalctl` 二次深度补充与终态加固方案

```text
Document ID:       AUDIT-GOALCTL-002
Supersedes:        不替代 AUDIT-GOALCTL-001，作为其增量补充
Audit Target:      AI-Native 工作流与 xhyper-goalctl 终态架构
Audit Perspective: Bootstrap / Trust / TOCTOU / Organization / Lifecycle
Audit Date:        2026-07-15
Previous Score:    78 / 100
After Audit-001:   约 90 / 100（完成建议后）
Target:            97 / 100
Verdict:           仍有 18 类长期结构性风险需要补齐
```

---

# 一、执行摘要

上一版十轮审计已经补齐了大部分显性工程合同：

- Authority Policy；
- Schema Registry；
- Approval Record；
- canonical serialization；
- Fact Observation；
- Capability Policy；
- Resource Budget；
- Evidence 隐私与保留；
-沙箱、Lease、Fencing、取消和恢复；
- Shadow / Mirror / Cutover；
- Eval、SLO 和成本治理。

继续从“系统最终会如何失效”反推，仍发现 18 类遗漏：

```text
1. Bootstrap 信任根与自举悖论；
2. Authority / Task / Evidence 的 TOCTOU；
3. Fork、外部 PR 与不可信 Git 对象边界；
4. Repository Identity 与重命名/迁移；
5. Policy-as-Code 的单元测试、模型检查与差分测试；
6. Break-glass 紧急权限与事后追责；
7. 人类审批的法定人数、职责分离和失联处理；
8. 插件/Adapter ABI 与能力协商；
9. 跨平台路径、文件系统和进程语义；
10. Git LFS、submodule、sparse checkout、partial clone；
11. Merge Queue、重基和 Evidence 失效；
12. 分布式执行的远程证明与 Runner 信任；
13. 时钟、时区、单调时间和过期语义；
14. 可观测性高基数、采样和敏感信息泄漏；
15. 灾难恢复、备份恢复演练和证据可读性；
16. 系统退役、Schema 日落和 Legacy 删除；
17. 组织责任、Support Model、On-call 与事故响应；
18. 经济攻击、资源耗尽与自动化反向激励。
```

这些问题通常不会在第一版实现时暴露，却会在以下场景中成为结构性故障：

- 仓库迁移或改名；
- Fork PR；
- Merge Queue 重新生成提交；
- Runner 被攻破；
- Policy 自身出错；
-审批人失联；
-紧急修复；
-多平台执行；
-证据保留多年后无法验证；
-系统从实验工具晋升为强制 Gate。

---

# 二、补充一：Bootstrap 信任根与自举悖论

## 2.1 问题

`goalctl` 负责验证：

- Authority；
- Schema；
- Policy；
- Task；
- Evidence；
- Gate 输入。

但谁来验证 `goalctl` 自身、它使用的 Authority Policy，以及它的发布二进制？

形成自举悖论：

```text
goalctl 验证规则
规则决定 goalctl 是否可信
goalctl 又读取和解释这些规则
```

如果没有独立信任根，一次恶意或错误更新可以同时修改：

```text
goalctl implementation
+ policy
+ tests
+ expected evidence
```

并让系统自我宣称 PASS。

## 2.2 必须增加 Trust Root

建议定义三层：

```text
Root of Trust
├── Protected bootstrap policy digest
├── Trusted release public keys
└── Minimum offline verifier

Operational Trust
├── goalctl signed binary
├── schema bundle
├── policy bundle
└── adapter manifests

Runtime Trust
├── Task Pack
├── Capability Grant
└── Evidence
```

## 2.3 Bootstrap Verifier

增加一个极小、稳定、低变更频率的验证器：

```text
goalctl-bootstrap-verify
```

职责仅限：

- 验证 `goalctl` release signature；
-验证 binary digest；
-验证 schema/policy bundle digest；
-验证兼容版本；
-拒绝未知签名者；
-输出 bootstrap verdict。

它不解析完整 Goal 工作流，减少攻击面。

## 2.4 双人修改原则

以下内容不能由同一个 PR 同时无额外审查地修改：

```text
goalctl verifier code
+ bootstrap policy
+ release signing keys
```

至少需要 Security / Governance 双重审批。

## 2.5 新不变量

```text
INV-BOOT-001 goalctl 不得独立证明自身可信
INV-BOOT-002 Policy bundle 必须被独立签名或固定 digest
INV-BOOT-003 Bootstrap verifier 的变更必须走最高风险审批
INV-BOOT-004 Release binary 与源 commit、SBOM、provenance 必须绑定
```

---

# 三、补充二：TOCTOU——检查时与使用时不一致

## 3.1 问题

系统可能执行：

```text
Resolve Authority at commit A
Compile Task Pack
工作区/分支移动到 commit B
Writer 开始执行
```

即使 Task Pack 记录 `source_commit=A`，如果实际读写的是可变工作目录，仍可能发生：

```text
Time Of Check != Time Of Use
```

## 3.2 必须使用不可变执行视图

执行必须绑定：

```text
Git tree ID
而不仅是 branch name 或 HEAD string
```

建议：

- 从固定 commit 创建 detached worktree；
-所有输入文件验证 blob SHA；
-开始执行前重新核对 tree ID；
-每个命令前可选验证关键文件 digest；
-任务结束时验证 base tree 未被外部改写。

## 3.3 Task Pack 应增加

```json
{
  "repository_snapshot": {
    "commit": "...",
    "tree_id": "...",
    "submodule_state_digest": "...",
    "lfs_manifest_digest": "...",
    "sparse_checkout_digest": null
  }
}
```

## 3.4 高风险任务的文件读一致性

对于受保护规则：

```text
Authority Snapshot 中的 blob
= Prompt Compiler 读取的 blob
= Gate 使用的 blob
```

不能分别从工作树重新读取。

## 3.5 新退出条件

```text
REPOSITORY_TREE_CHANGED
AUTHORITY_BLOB_CHANGED
SUBMODULE_STATE_CHANGED
LFS_OBJECT_MISSING
WORKTREE_EXTERNALLY_MODIFIED
```

---

# 四、补充三：Fork、外部 PR 与不可信 Git 对象

## 4.1 问题

在 Fork PR 中，攻击者可以提交：

- 恶意 `Cargo.toml`；
-恶意 build script；
-恶意 test；
-诱导 Agent 的文档；
-符号链接；
-超大文件；
-压缩炸弹；
-Git LFS pointer；
-更改 `.cargo/config.toml`。

如果 CI 或 Agent 直接执行 PR 内容，Secrets 和 Runner 都可能暴露。

## 4.2 信任等级

定义：

```text
TRUSTED_INTERNAL
TRUSTED_BOT
UNTRUSTED_FORK
UNTRUSTED_EXTERNAL_SOURCE
```

Task Pack 和 CI 必须携带 trust level。

## 4.3 Fork PR 策略

`UNTRUSTED_FORK` 默认：

```text
Secrets = none
Network = deny
GitHub write = deny
Self-hosted privileged runner = deny
Build scripts = restricted
Proc macros = restricted/observed
Artifacts = quarantined
```

不得让不可信 PR 使用生产型 self-hosted runner。

## 4.4 两阶段验证

```text
Stage 1 Untrusted Static Analysis
→ Human / policy approval
→ Stage 2 Trusted Build
```

Stage 2 只能在内容 digest 不变时运行。

## 4.5 新检查

- `.cargo/config*` 变化；
- `build.rs` 变化；
- proc-macro dependency 变化；
- GitHub Action 变化；
- executable bit 变化；
- symlink 变化；
- LFS pointer 变化；
- submodule URL 变化。

---

# 五、补充四：Repository Identity 与迁移

## 5.1 问题

Evidence、State Store、Chain ID 和缓存不能只绑定：

```text
owner/repo
```

因为仓库可能：

- 改名；
-转移组织；
-Fork；
-镜像；
-本地 clone 路径变化；
-历史重写。

## 5.2 Repository Identity

建议建立稳定 ID：

```json
{
  "repository_id": "repo:<generated-stable-id>",
  "hosting": {
    "provider": "github",
    "provider_repository_id": 123456,
    "canonical_name": "xhyperium/infra.rs"
  },
  "root_commit": "...",
  "identity_version": 1
}
```

优先使用 GitHub 数字 repository ID，而不是仅用名称。

## 5.3 仓库迁移协议

需要：

```text
RepositoryIdentityMigration
```

记录：

- old identity；
-new identity；
-迁移批准；
-历史 Evidence 是否继续有效；
-State Store 如何搬迁；
-Chain ID 是否重建；
-旧名称 alias。

## 5.4 Fork 隔离

Fork 必须拥有不同 repository identity，不能共享：

- writer lease；
-state store；
-Evidence chain；
-cache verdict；
-approval record。

---

# 六、补充五：Policy-as-Code 必须可证明

## 6.1 问题

Authority Policy、Scope Policy、Approval Policy、Capability Policy 一旦成为强制 Gate，本身就是生产代码。

仅做普通 unit test 不足以防止：

- allow/deny 冲突；
-规则死区；
-默认放行；
-优先级反转；
-通配符过宽；
-策略循环；
-不可达审批路径。

## 6.2 Policy Test Suite

每个 Policy 必须有：

```text
Positive cases
Negative cases
Boundary cases
Mutation tests
Differential tests
Coverage report
```

## 6.3 差分验证

新旧 policy engine 同时输入同一 corpus：

```text
policy-v1(input) vs policy-v2(input)
```

所有差异必须被批准。

## 6.4 模型检查

关键安全性质可使用属性测试或状态空间检查：

```text
未经审批永远不能写 protected asset
无 network capability 永远不能启动联网 adapter
旧 fencing token 永远不能提交状态
Writer 永远不能批准自身结果
```

## 6.5 默认拒绝

任何 Policy 解析失败、版本未知或冲突：

```text
DENY / BLOCKED
```

不能回退到 permissive 默认值。

---

# 七、补充六：Break-glass 紧急机制

## 7.1 问题

完全刚性的 Gate 在重大安全事件、供应链中断或生产事故中可能阻止必要修复。

如果没有正式 Break-glass，人们会：

-临时关闭 Gate；
-直接改 main；
-绕过 Evidence；
-共享高权限 token；
-留下不可审计的暗门。

## 7.2 Break-glass 不是普通 override

必须定义：

```text
EmergencyChangeRecord
```

字段：

- incident ID；
- reason；
- scope；
- exact capabilities；
- approvers；
- start / expiry；
- affected assets；
- mandatory post-review；
- rollback deadline。

## 7.3 限制

```text
短时有效
最小范围
至少双人批准
自动过期
所有操作完整审计
禁止修改审计系统本身
```

## 7.4 事后要求

在 24～72 小时内：

-独立 Review；
-补齐正常 Evidence；
-形成 Regression；
-撤销临时权限；
-确认无残留凭据；
-G11 复盘。

## 7.5 禁止事项

Break-glass 不能用于：

-避免修复 flaky test；
-赶发布日期；
-减少 Review 时间；
-绕过成本预算；
-自动批准自身。

---

# 八、补充七：审批法定人数与职责分离

## 8.1 问题

当前只写“Platform / Governance / Architecture 批准”，但未定义：

-需要几人；
-是否可以是同一人；
-谁有替补；
-审批多久有效；
-审批人失联怎么办；
-利益冲突如何处理。

## 8.2 Approval Policy

示例：

| 风险 | Quorum | 独立性 |
|---|---:|---|
| R0 | 1 | Writer 可提交，Reviewer 批准 |
| R1 | 1 | Reviewer ≠ Writer |
| R2 | 2 | Owner + independent Reviewer |
| R3 | 2/3 | Platform + Governance，至少一人非实现者 |
| R4 | 3 | Security + Domain Owner + Human Operator |

## 8.3 审批过期

Approval 应绑定：

```text
subject digest
source commit
scope
expiration
```

内容改变后自动失效。

## 8.4 失联与替补

必须定义：

- delegate；
-escalation path；
-timeout；
-不可降低 quorum 的规则；
-紧急时转 Break-glass，而不是静默少人批准。

---

# 九、补充八：Adapter / Plugin ABI 与能力协商

## 9.1 问题

Agent、GitHub、Evidence、Fact Observer、Sandbox 都是 Adapter。

如果没有明确协议，会出现：

- Adapter 版本不兼容；
-某 Adapter 不支持必要安全能力；
-错误能力被静默忽略；
-厂商字段污染核心模型。

## 9.2 Adapter Manifest

```json
{
  "adapter_id": "codex-cli",
  "adapter_version": "1.2.0",
  "protocol_version": "1.0.0",
  "capabilities": [
    "filesystem_write_scope",
    "stream_events",
    "cancel",
    "token_usage"
  ],
  "security_properties": {
    "network_isolation": false,
    "secret_isolation": false,
    "process_tree_cancel": true
  }
}
```

## 9.3 Capability Negotiation

执行前：

```text
Task required capabilities
⊆ Adapter supported and approved capabilities
```

不满足时必须拒绝，不能降级执行。

## 9.4 Adapter Certification

生产使用的 Adapter 需要：

- conformance suite；
-security review；
-version allowlist；
-binary digest；
-known limitations；
-deprecation date。

---

# 十、补充九：跨平台语义

## 10.1 问题

方案主要按 Linux 语义设计，但 Rust 工具可能运行在：

- Debian/Linux；
-macOS；
-Windows；
-container；
-不同大小写文件系统。

路径和进程行为不同：

- Windows drive / UNC；
-大小写不敏感；
-symlink 权限；
-executable bit；
-进程组终止；
-file locking；
-atomic rename；
-fsync 语义。

## 10.2 支持矩阵

明确：

```text
Tier 1: Linux x86_64
Tier 2: macOS
Tier 3: Windows
Unsupported: ...
```

Phase 1 可以仅承诺 Linux，但必须 fail clearly，而不是在其他平台产生错误安全假设。

## 10.3 Path Semantics Version

Task Pack 已建议包含：

```text
path_semantics_version
```

还应增加：

```text
filesystem_case_sensitivity
symlink_policy
unicode_normalization
platform
```

## 10.4 锁与原子写

跨平台实现必须通过 conformance test，而不是假设 `create_new` + rename 在所有平台等价。

---

# 十一、补充十：Git LFS、Submodule 与部分克隆

## 11.1 问题

`git show HEAD:path` 不一定返回完整业务内容：

- LFS 返回 pointer；
-submodule 返回 gitlink；
-partial clone 可能缺 blob；
-sparse checkout 工作树缺文件；
-filter 可能延迟下载。

## 11.2 Snapshot 必须声明完整性

```json
{
  "git_materialization": {
    "partial_clone": false,
    "sparse_checkout": false,
    "submodules_initialized": true,
    "lfs_objects_complete": true
  }
}
```

## 11.3 Submodule Policy

默认建议：

```text
Submodule content = untrusted external dependency
```

记录：

- URL；
-commit；
-签名或 allowlist；
-是否递归；
-是否允许 Agent 修改。

## 11.4 LFS Policy

- pointer digest；
-object digest；
-object availability；
-size limit；
-content scanning；
-Evidence 中记录实际 object digest。

## 11.5 缺对象时

返回：

```text
SNAPSHOT_INCOMPLETE
```

不能继续生成“完整 Authority Snapshot”。

---

# 十二、补充十一：Merge Queue、Rebase 与 Evidence 失效

## 12.1 问题

PR 测试的 commit 可能是：

- PR head；
-merge commit；
-merge queue synthetic commit；
-rebased commit；
-main 合并后的新 commit。

Evidence 必须明确 subject 类型。

## 12.2 Subject Model

```json
{
  "subject_type": "HEAD|MERGE_CANDIDATE|MERGED_COMMIT|RELEASE_TAG",
  "commit": "...",
  "parents": [],
  "base_ref": "main",
  "base_commit": "..."
}
```

## 12.3 Evidence 继承规则

默认：

```text
Commit A 的 Evidence 不自动继承到 Commit B
```

可以做安全复用的前提：

- tree ID 相同；
-相关路径 blob 相同；
-环境相同；
-validation plan 相同；
-policy 允许。

## 12.4 Merge 后验证

合并后至少重新运行：

- scope；
-critical tests；
-release manifest；
-provenance；
-Gate subject check。

---

# 十三、补充十二：远程 Runner 与执行证明

## 13.1 问题

即使 Evidence hash 正确，Runner 也可能撒谎：

-没有真正执行命令；
-篡改输出；
-使用不同代码；
-隐藏网络访问；
-伪造环境。

## 13.2 Runner Identity

记录：

```text
runner_id
runner_image_digest
host attestation
orchestrator identity
tool versions
sandbox policy digest
```

## 13.3 信任层级

```text
LOCAL_UNATTESTED
CI_MANAGED
SELF_HOSTED_TRUSTED
HARDWARE_ATTESTED
```

Gate 根据风险要求最低信任等级。

## 13.4 高风险 Evidence

R3/R4 任务应要求：

- ephemeral runner；
-immutable image；
-signed provenance；
-可能的 TPM/云 attestation；
-外部日志锚定。

不是所有任务都需要硬件证明，但合同必须可表达。

---

# 十四、补充十三：时间语义

## 14.1 问题

Approval expiration、Lease、Evidence、观察窗口都依赖时间。

墙上时间可能：

-回拨；
-漂移；
-时区混乱；
-Runner 时钟不可信。

## 14.2 时间类型

区分：

```text
WallClockTimestamp
MonotonicDuration
LogicalSequence
TrustedTimestamp
```

Lease timeout 应使用单调时间；跨进程恢复需要墙上时间 + fencing，而不能只依赖 wall clock。

## 14.3 时间来源

Evidence 应记录：

- event_time；
-recorded_at；
-time source；
-clock quality；
-是否可信时间戳服务锚定。

## 14.4 日期字段

所有机器字段使用 RFC3339 UTC：

```text
2026-07-15T08:30:00Z
```

人类文档可显示本地时区，但不得作为机器比较输入。

---

# 十五、补充十四：可观测性治理

## 15.1 高基数风险

指标标签不能包含：

- commit SHA 全量；
-run ID；
-task ID；
-file path；
-user；
-Prompt ID。

否则 Prometheus 高基数失控。

## 15.2 推荐模型

Metrics 使用低基数：

```text
command
result
risk_class
adapter_type
phase
```

详细 ID 放日志或 Trace。

## 15.3 采样

Agent Trace 可能巨大，应定义：

-成功采样率；
-失败全量；
-高风险全量；
-正文脱敏；
-最大 Trace 大小。

## 15.4 Observability Failure

监控失败不能让低风险只读命令完全不可用，但在正式执行阶段：

```text
Audit failure = fail closed
Metrics failure = degraded warning
Trace failure = policy-defined
```

三者不能混为一谈。

---

# 十六、补充十五：灾难恢复与长期可验证性

## 16.1 问题

多年后需要验证旧 Release 时，可能缺少：

-旧 goalctl binary；
-旧 Schema；
-旧 toolchain；
-旧 container；
-签名公钥；
-原始 artifacts；
-外部 anchor。

## 16.2 Verification Capsule

每个正式 Release 保存：

```text
Verifier binary
Schema bundle
Policy bundle
Public keys
Golden vectors
Environment manifest
Evidence bundle
Artifact locator
```

形成：

```text
verification-capsule/<release-id>
```

## 16.3 恢复演练

至少周期性验证：

-从备份恢复 Evidence Store；
-旧 verifier 可读取；
-签名仍可验证；
-artifact locator 可访问；
-外部 anchor 可查询；
-Chain head 对得上。

## 16.4 加密算法迁移

SHA-256 当前足够，但长期系统应定义 algorithm agility：

```text
algorithm_id
digest_version
migration/checkpoint strategy
```

不能把 digest 长度写死到所有上层模型。

---

# 十七、补充十六：退役与 Legacy 删除

## 17.1 问题

Shadow/Mirror 往往容易开始，难以结束。

如果不定义删除条件，会长期保留：

-旧 parser；
-旧 scripts；
-双写；
-双 Gate；
-兼容分支；
-过时 Schema。

## 17.2 Sunset Manifest

```json
{
  "component": "legacy-goal-validator",
  "deprecation_start": "...",
  "last_supported_version": "...",
  "removal_criteria": [],
  "removal_date": "...",
  "owner": "...",
  "rollback_window_end": "..."
}
```

## 17.3 删除门槛

- 所有消费者迁移；
-无旧格式新增；
-历史数据有只读 verifier；
-兼容窗口结束；
-文档和 CI 无引用；
-回滚包保留。

## 17.4 Schema 日落

Reader 可以长期保留旧 Schema 验证能力，但 Writer 必须在明确日期后禁止生成旧版本。

---

# 十八、补充十七：组织责任与运行支持

## 18.1 问题

工具从 advisory 变为 enforcing 后，会成为开发关键路径。

必须有人负责：

-失败排障；
-误阻断；
-安全事件；
-升级；
-兼容；
-性能；
-恢复。

## 18.2 RACI

至少定义：

| 能力 | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| Authority Policy | Governance | Governance Owner | Architecture | Teams |
| Schema | Platform | Platform Owner | Security | Teams |
| Runtime | Tooling | Platform Owner | Infra | Teams |
| Security | Security | Security Owner | Platform | Maintainers |
| Release | Release | Release Owner | Platform | Teams |
| Incident | On-call | Platform Owner | Security | Stakeholders |

## 18.3 Support SLO

例如：

```text
Critical false block response < 1h
High severity bug triage < 4h
Schema migration notice >= 30d
Breaking CLI change notice >= 1 release cycle
```

## 18.4 Incident Playbook

必须覆盖：

- Gate 全面误阻断；
-Gate 错误放行；
-Evidence corruption；
-signing key compromise；
-runner compromise；
-policy bad release；
-schema bad migration。

---

# 十九、补充十八：经济攻击与反向激励

## 19.1 问题

自动化系统可能被滥用：

-创建大量无价值 Task；
-触发昂贵模型；
-无限 retry；
-通过拆小任务规避预算；
-生成大量 PR 占用 Reviewer；
-优化“通过 Gate”而非真实价值。

## 19.2 预算层级

```text
Per Run
Per Task
Per Goal
Per Repository
Per Team
Per Day / Month
```

## 19.3 Admission Control

任务启动前评估：

```text
expected value
risk
estimated cost
review capacity
dependency readiness
```

低价值、高成本任务应排队或拒绝。

## 19.4 Anti-gaming 指标

不要只奖励：

-代码行数；
-PR 数量；
-Task 完成数；
-Gate Pass 数。

应衡量：

```text
accepted verified value
escaped defect
rollback
human review load
duplicate work
business/engineering outcome
```

## 19.5 Reviewer 容量是硬约束

Agent 吞吐不能超过人工验证能力，否则只会制造 PR 队列。

Orchestrator 应有：

```text
review_capacity_budget
max_open_agent_prs
max_parallel_high_risk_tasks
```

---

# 二十、还需要新增的文档与 Schema

## P0：应立即增加

```text
docs/goal/policies/bootstrap-trust-policy.*
docs/goal/policies/approval-policy.*
docs/goal/policies/repository-identity-policy.*
docs/goal/policies/untrusted-contribution-policy.*
docs/goal/policies/break-glass-policy.*
docs/goal/policies/policy-testing-standard.md

.agents/ssot/tools/goalctl/schemas/
├── repository-identity.schema.json
├── approval-record.schema.json
├── adapter-manifest.schema.json
├── emergency-change-record.schema.json
├── repository-snapshot.schema.json
└── verification-capsule.schema.json
```

## P1：Harness 前增加

```text
docs/goal/policies/runner-trust-policy.*
docs/goal/policies/time-semantics-policy.*
docs/goal/policies/observability-policy.*
docs/goal/policies/evidence-retention-policy.*
docs/goal/policies/disaster-recovery-policy.*
```

## P2：Cutover 前增加

```text
docs/goal/policies/deprecation-sunset-policy.*
docs/goal/policies/support-incident-policy.*
docs/goal/policies/automation-economics-policy.*
```

具体位置必须服从最终 Authority Map；以上为职责建议，不应未经 CR 直接创建新的 SSOT 根。

---

# 二十一、修订后的整体 PR 路线

```text
PR-0   Governance Artifacts
PR-0A  Schema / Authority / Approval Foundation
PR-0B  Bootstrap Trust / Repository Identity / Break-glass

PR-1   Skeleton / Doctor / Index
PR-2   Authority / Artifact
PR-2A  Fact Model / Time / Validity
PR-2B  Git Materialization / Fork Trust / Merge Subject

PR-3   Reconciliation
PR-4   Task Compiler / Capability / Resource Budget
PR-4A  Adapter Protocol / Conformance

PR-5   Evidence Bundle / Audit Chain
PR-5A  Privacy / Retention / Anchor / Verification Capsule

PR-6   State Store / Lease / Fencing / Cancel
PR-6A  Sandbox / Runner Trust / Remote Attestation
PR-6B  Disaster Recovery Drill

PR-7   Agent Adapters
PR-7A  Prompt Injection / Supply-chain Hardening

PR-8   Independent Verifier
PR-8A  Policy Differential / Adversarial Eval

PR-9   Shadow / Mirror
PR-9A  Support / Incident / Economics

PR-10  Cutover
PR-11  Legacy Sunset
```

这并不意味着必须产生 20 个大型 PR。相邻小合同可以合并，但不得跳过相应的裁定点。

---

# 二十二、修订后的终态系统

```text
Offline / Bootstrap Trust
        ↓
Signed Policy + Schema Bundle
        ↓
Repository Identity + Immutable Snapshot
        ↓
Authority + Approval Resolution
        ↓
Artifact / Fact Model
        ↓
Reconciliation
        ↓
Task Pack
  ├── Scope
  ├── Capability
  ├── Resource Budget
  ├── Side-effect Class
  └── Approval
        ↓
Trusted Runner + Sandbox + Fencing
        ↓
Writer
        ↓
Review Bundle + Audit Chain + External Anchor
        ↓
Independent Verifier
        ↓
Policy-tested Gate Adapter
        ↓
Merge Candidate / Release Provenance
        ↓
Post-merge Observation
        ↓
Eval / Cost / Incident / Sunset
```

---

# 二十三、不可再妥协的终态不变量

```text
INV-T01 规则解释器不能独立证明自身可信
INV-T02 检查的 Git tree 必须等于执行的 Git tree
INV-T03 Fork 内容默认不可信且无 Secret
INV-T04 Repository 名称变化不能破坏身份和 Evidence
INV-T05 Policy 变更必须通过差分测试
INV-T06 Break-glass 必须自动过期且事后复审
INV-T07 高风险审批必须满足 quorum 和职责分离
INV-T08 Adapter 能力不足时必须拒绝执行
INV-T09 不支持的平台必须显式失败
INV-T10 LFS/Submodule/Partial clone 不完整时不得声称完整 Snapshot
INV-T11 Evidence 必须绑定明确的 merge/release subject
INV-T12 高风险 Runner 必须达到最低信任等级
INV-T13 Lease 过期不能让旧 Writer 恢复写入
INV-T14 Metrics 不得造成高基数或泄露敏感信息
INV-T15 正式 Release 必须携带长期可验证 Capsule
INV-T16 Legacy 必须有 Sunset 日期和删除 owner
INV-T17 Enforcing Gate 必须有 On-call 和 Incident Playbook
INV-T18 自动化吞吐不能超过 Review 容量与预算
```

---

# 二十四、最小可行补充行动

在继续 PR-1 之前，最低限度新增一个 **Decision Pack**，只裁定以下十项：

```text
1. Bootstrap trust root；
2. Authority policy source；
3. Repository stable identity；
4. Approval quorum；
5. Runtime state directory；
6. Schema compatibility；
7. Untrusted Fork policy；
8. Break-glass policy；
9. Git tree / merge subject semantics；
10. Legacy sunset owner and deadline。
```

不需要立刻实现全部高级能力，但必须先决定其边界，否则早期 Schema 和核心类型会反复破坏性变更。

---

# 二十五、1 天、7 天、30 天补充计划

## 1 天

- 创建 Decision Pack；
-冻结 RepositoryIdentity、ApprovalRecord、RepositorySnapshot；
-裁定 runtime state root；
-裁定 Authority Policy 不硬编码；
-定义 Fork 默认拒绝能力；
-定义 Break-glass 的最小规则。

## 7 天

- 增加核心 Schema；
-建立 Policy fixture 和差分测试框架；
-建立 bootstrap verifier 设计；
-建立 merge subject / tree ID 测试；
-建立 LFS/submodule incomplete fixture；
-建立 Adapter Manifest；
-建立 Quorum Policy。

## 30 天

- 完成 PR-0A / PR-0B；
-完成 policy conformance corpus；
-完成 Fork PR 威胁演练；
-完成 bad-policy rollback drill；
-完成 key compromise tabletop exercise；
-完成 verification capsule 原型；
-完成 support/incident RACI；
-重新评审 PR-1 数据模型后再编码。

---

# 二十六、最终评分与裁定

| 维度 | Audit-001 补齐后 | 本次补齐后目标 |
|---|---:|---:|
| 核心架构 | 95 | 98 |
| Authority / Schema | 95 | 98 |
| Bootstrap Trust | 55 | 96 |
| Git / TOCTOU | 68 | 97 |
| 外部贡献安全 | 60 | 96 |
| Approval / Break-glass | 65 | 96 |
| Adapter / Portability | 70 | 95 |
| Evidence 长期性 | 85 | 97 |
| 组织与 Incident | 60 | 95 |
| 经济与容量治理 | 55 | 94 |
| **综合终态** | **约 90** | **97** |

最终裁定：

> 上一版解决了“如何正确构建 goalctl”；本次补充解决的是“goalctl 在成为关键基础设施后，如何不因自举、信任、Git 竞态、组织失效和长期维护而崩溃”。

推荐不要继续无限扩展普通功能。下一步应先落地 Decision Pack 与 PR-0A/PR-0B，把信任根、身份、审批、Git Subject 和退役边界固化，再开始大规模实现。
