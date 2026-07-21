# DECISION-PACK-001：goalctl 进入实现前的冻结裁决

```text
Document ID:     DECISION-PACK-001
Status:          APPROVED（Human：2026-07-16 会话「全部批准」）
Date:            2026-07-16
Approved:        2026-07-16
Module:          tools/goalctl（planned package: xhyper-goalctl）
Related CR:      docs/goal/change-requests/CR-20260716-goalctl-foundation.md
Approval ID:     APPR-GOALCTL-FOUNDATION-001
Related Audits:  AUDIT-GOALCTL-001, AUDIT-GOALCTL-002
Supersedes:      无；不替代 docs/goal 方法论 SSOT
```

## 0. 用途与使用规则

本包裁定 **进入 `tools/goalctl` 编码（PR-1）之前必须冻结的 10 项边界**。

| 规则   | 说明                                                                                  |
| ------ | ------------------------------------------------------------------------------------- |
| 性质   | **已批准** Decision Pack；与 CR-20260716 一并构成 goalctl 前置强边界                  |
| 输入   | 摘要 `spec/01–07`、`AUDIT-GOALCTL-001/002`、已批准 monorepo CR（禁止 `.config/goal`） |
| 输出   | Recommended Default **已全部采纳为 DECIDED**（无逐项修订）                            |
| 禁止   | 借本包创建 `.config/goal`；把 `goalctl` 升为新 SSOT；直接改 G0–G11 编号               |
| 下一步 | PR-0A Schema/Policy 落盘 → 实现 CR → 再 PR-1 Skeleton                                 |

**完成定义**：10 项均为 `DECIDED` 或 `DEFERRED_WITH_OWNER`，且每项有 Owner；无 `OPEN` 阻塞项。**本包已满足。**

**状态枚举**（本包内部）：

```text
OPEN                  未裁定，阻塞 PR-1
PROPOSED_DEFAULT      已写推荐默认，待 Human 确认
DECIDED               Human 已确认
DEFERRED_WITH_OWNER   可延后，但有 Owner + 最迟波次
```

**批准记录**：2026-07-16 Human 回复「全部批准」——D01–D10 的 Recommended Default **原样采纳**，无修订。

**实现 vs 形状**：

| ID                 | 合同形状 | 运行实现                        |
| ------------------ | -------- | ------------------------------- |
| D01, D07, D08      | DECIDED  | 可按波次延后（见各节）          |
| D02, D04, D05, D06 | DECIDED  | PR-0A / PR-1 前必须可引用       |
| D03, D09           | DECIDED  | 形状 PR-1 前；完整适配可分阶段  |
| D10                | DECIDED  | Cutover 另 CR；对照面即日起有效 |

---

## 1. 十项裁决一览

| ID  | 主题                       | 裁定摘要                                                             | 状态    | 阻塞                                                         |
| --- | -------------------------- | -------------------------------------------------------------------- | ------- | ------------------------------------------------------------ |
| D01 | Bootstrap trust root       | 独立 bootstrap verifier + 签名/digest；goalctl 不得自证可信          | DECIDED | 形状即刻；binary 可延 PR-0B/发布前                           |
| D02 | Authority policy source    | Git 内机器可读 policy；**禁止** rank 硬编码为 SSOT                   | DECIDED | PR-1 前                                                      |
| D03 | Repository stable identity | 优先 hosting numeric id + root commit；名称仅 alias                  | DECIDED | PR-1 前形状；适配可分阶段                                    |
| D04 | Approval quorum            | Markdown `Status: Approved` ≠ 批准事实；需 ApprovalRecord + 职责分离 | DECIDED | PR-0A 前                                                     |
| D05 | Runtime state directory    | **禁止** Cargo `target/`；默认 XDG state + `--state-dir`             | DECIDED | PR-1 前                                                      |
| D06 | Schema compatibility       | SemVer + Schema Registry；安全输入 strict；报告可 additive           | DECIDED | PR-0A 前                                                     |
| D07 | Untrusted Fork policy      | 默认无 Secrets/无写/无特权 runner；两阶段验证                        | DECIDED | 形状即刻；Harness 前须完整；Phase 1 只读实现可延             |
| D08 | Break-glass                | 自动过期 + 事后复审；禁止静默永久扩权                                | DECIDED | 形状即刻；执行路径 Phase 1 可延；Enforcing Gate 前须完整     |
| D09 | Git tree / merge subject   | Snapshot = commit + tree_id；Evidence 绑 subject 非 branch           | DECIDED | index 可先 commit；tree_id 于 PR-2；reconcile/compile 前完备 |
| D10 | Legacy sunset              | Shadow→Mirror→Cutover 有量化门槛 + Owner + 删除日                    | DECIDED | 对照面即刻；Cutover/Sunset 另 CR                             |

---

## 2. D01 — Bootstrap trust root

### Context

`goalctl` 将解释 Authority、Schema、Task 与 Evidence。若实现、policy 与期望证据可在同一 PR 中同时修改，系统可自我宣称 PASS（自举悖论，AUDIT-002 §2）。

### Options

1. **无独立信任根**：仅靠 code review（最弱）。
2. **Recommended**：最小 `goalctl-bootstrap-verify`（或等价离线步骤）校验 binary/schema/policy digest 与签名；goalctl 本体不独立证明自身可信。
3. **完整供应链**：SLSA/provenance + 密钥轮换 + SBOM（终态，非 Phase 1 必达）。

### Recommended Default

采用 **选项 2 的合同形状**，Phase 1 **不实现** bootstrap binary，但：

```text
INV-BOOT-001  goalctl 不得独立证明自身可信
INV-BOOT-002  Policy/Schema bundle 必须可被独立 digest 固定
INV-BOOT-003  bootstrap verifier / 签名密钥变更走最高风险审批
INV-BOOT-004  禁止同一 PR 无额外审查同时改：verifier 实现 + bootstrap policy + 签名密钥
```

Phase 1 最低落地：文档与 schema 预留 `bootstrap_policy_digest` / `schema_bundle_digest` 字段；实现挂在 **PR-0B / 发布前**。

### Consequences

- 早期二进制可为 unsigned local build，但 **不得** 作为 enforcing Gate 的唯一信任根。
- 与 crates.io/release fail-closed 现状一致：无 provenance 不假装生产可信。

### Rollback

撤销 bootstrap 字段扩展；不得改为“goalctl 自签 PASS”。

---

## 3. D02 — Authority policy source

### Context

若在 `tools/goalctl/src/authority/rank.rs` 硬编码 `CONSTITUTION=0, ADR=20…`，会形成：

```text
docs/goal/00-authority-map.md  vs  Rust 常量
```

第二套 SSOT，直接违背「goalctl 不是新 SSOT」。

### Options

1. 硬编码 rank（拒绝）。
2. **Recommended**：机器可读 Authority Policy（建议路径候选见下），代码只解析/验证。
3. 从 `00-authority-map.md` 唯一 Control Block 编译（过渡）。

### Recommended Default

```text
Authority Rank Policy = Git 中的治理事实
                     ≠ Rust 常量 / 环境变量默认表
```

**路径候选**（Human 批准后只选一条 SSOT，禁止双写）：

| 优先级 | 路径                                                        | 说明                             |
| ------ | ----------------------------------------------------------- | -------------------------------- |
| A      | `docs/goal/schema/authority-policy.yaml`（或 `.json`）      | 与现有 `docs/goal/schema/*` 并列 |
| B      | `docs/goal/00-authority-map.md` 内唯一 fenced Control Block | 零新文件；parser 更脆            |
| C      | `.agents/ssot/tools/goalctl/schemas/authority-policy.*`     | 模块局部；不利于全仓复用         |

**本包推荐 A**。未落盘前，`goalctl resolve` **不得** 假装有完整 rank 语义；可输出 `AUTHORITY_POLICY_MISSING`。

### Scope 必填字段（合同形状）

```json
{
  "authority_id": "ADR-016",
  "rank": 20,
  "scope": {
    "modules": ["bootstrap"],
    "paths": ["crates/infra/bootstrap/**"]
  },
  "effective_from": "commit-or-version",
  "effective_until": null,
  "supersedes": []
}
```

无 scope 的全局规则不得静默覆盖 unrelated subject。

### Consequences

- PR-0A 必须交付 policy 文件或明确的 Control Block 规范 + 负例测试计划。
- rank 变更走 CR，不走 silent code change。

---

## 4. D03 — Repository stable identity

### Context

Evidence、lease、chain id 不能只绑 `owner/repo` 字符串（改名/转移/Fork 会炸）。

### Recommended Default

```json
{
  "repository_id": "repo:<stable>",
  "hosting": {
    "provider": "github",
    "provider_repository_id": null,
    "canonical_name": "xhyperium/infra.rs"
  },
  "root_commit": "<first-parent-root-or-documented-anchor>",
  "identity_version": 1
}
```

| 规则    | 说明                                                                                                                                           |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| 稳定 ID | 优先 GitHub numeric repository id；本地/离线可用 documented root commit + remote URL digest 的 **降级 identity**，并标记 `confidence=DEGRADED` |
| 名称    | 仅 alias；变更触发 `RepositoryIdentityMigration` 记录，不静默换 id                                                                             |
| Fork    | **必须** 不同 identity；禁止共享 lease / evidence chain / approval                                                                             |

Phase 1：`doctor`/`index` 输出 identity 块；numeric id 可 `null` + 警告，但不得写入「已发布 Evidence 链头」除非 confidence=FULL。

与现有 monorepo self-test 的 repository identity 检查 **对齐语义，不重复造第二套字符串匹配 SSOT**；具体算法在 PR-0A 写清并单测。

---

## 5. D04 — Approval quorum

### Context

仅改 Markdown 为 `Status: Approved` 即可晋升 Authority，是 Critical 漏洞。

### Recommended Default

| 概念         | 规则                                                                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 叙述状态     | `PROPOSED` / `APPROVED` 等是 **声明**，非事实                                                                                            |
| 批准事实     | `ApprovalRecord`：subject_id、subject_digest、approver(s)、scope、approved_commit、status、revocation                                    |
| Phase 1 最低 | 高风险晋升（Authority Policy、Schema major、Protected Asset、Cutover）要求 **可引用的 ApprovalRecord** 或已批准 CR 链接 + subject digest |
| 职责分离     | Writer ≠ 唯一 Approver（高风险）；Constitution / bootstrap / signing keys 需 Security+Governance 双轨（形状先定，工具后做）              |
| 过期         | 可选 `expires_at`；过期后 Approval 不得继续授权新的 enforcing 行为                                                                       |

```json
{
  "approval_id": "APPR-GOALCTL-…",
  "subject_id": "SPEC-GOALCTL-001",
  "subject_digest": "sha256:…",
  "scope": ["IMPLEMENT_PHASE_1_READ_ONLY"],
  "approvers": [],
  "approved_commit": null,
  "status": "PROPOSED",
  "revoked_by": null
}
```

**Phase 1 只读 MVA** 可对「本地 doctor/index」放宽到无 ApprovalRecord；一旦输出进入 PR 检查或 Gate 输入，必须 fail-closed。

---

## 6. D05 — Runtime state directory

### Context

摘要写 `target/goalctl/**`；本仓强制：

```toml
# .cargo/config.toml
[build]
target-dir = "../.cargo/target"
```

运行态写入 Cargo target 会污染缓存、跨 worktree 互踩，并违反「禁止写死 `./target/`」的工程纪律。

### Recommended Default

| 用途                                  | 路径                                                                    |
| ------------------------------------- | ----------------------------------------------------------------------- |
| 可删除运行态（lease、cache、scratch） | `${XDG_STATE_HOME:-$HOME/.local/state}/xhyper-goalctl/<repo-identity>/` |
| 覆盖                                  | `--state-dir <path>`（测试与 CI 必用）                                  |
| 构建产物                              | 仅 Cargo 外置 target；**goalctl 业务状态不得放入**                      |
| 可提交制品                            | 仍只在 `.agents/ssot/**`、`docs/goal/**`、`evidence/**` 等既有规则下    |

禁止：

```text
./target/goalctl/**
../.cargo/target/goalctl/**   # 除非未来显式 CR 批准「构建旁车」且与 rust-cache 策略一致
.config/goal/**
```

`doctor` 必须报告 resolved state-dir 与「未创建 .config/goal」。

---

## 7. D06 — Schema compatibility

### Context

多处 `schema_version` 但无 Registry → 实现即漂移。

### Recommended Default

**Registry 根（推荐）**：

```text
.agents/ssot/tools/goalctl/schemas/     # goalctl 私有 schema（Phase 1）
docs/goal/schema/                       # 全仓 Goal 方法论 schema（已有）；Authority Policy 优先落这里
```

不新建第三棵 schema 树。跨域复用优先 `docs/goal/schema/`；goalctl 运行输出（TaskPack、ReconciliationReport）放模块 `schemas/`。

**兼容性**：

```text
Major: breaking（Reader 遇更高 major → 拒绝）
Minor: additive
Patch: 澄清 / 缺陷

Reader v1.x:
- 必须读 1.0～当前 1.x
- 拒绝更高 major
```

**unknown fields**：

| 类别                                         | 行为                              |
| -------------------------------------------- | --------------------------------- |
| Authority / Security / Approval / Capability | `deny_unknown_fields` fail-closed |
| Operational reports（index/reconcile 报告）  | 版本门控下可保留 extensions       |

**Canonical bytes**（确定性）：UTF-8、稳定 key 序（建议 JSON 用明确 canonical 规则，不仅 pretty-print）、仓库相对路径、无墙钟/随机/绝对本机路径。

最小 Phase 1 schema 清单（PR-0A）：

```text
authority-snapshot.schema.json
artifact-envelope.schema.json
repository-index.schema.json
reconciliation-report.schema.json   # 可延到 PR-2A
task-pack.schema.json               # 可延到 PR-4
```

PR-1 至少：`repository-index` + CLI JSON 合同。

---

## 8. D07 — Untrusted Fork policy

### Recommended Default

Trust levels：

```text
TRUSTED_INTERNAL
TRUSTED_BOT
UNTRUSTED_FORK
UNTRUSTED_EXTERNAL_SOURCE
```

`UNTRUSTED_FORK` 默认：

```text
Secrets = none
Network = deny
GitHub write = deny
Privileged self-hosted runner = deny
Build scripts / proc-macros = restricted or observed
Artifacts = quarantined
```

两阶段：静态分析 → Human/policy 批准 → 内容 digest 不变后的可信构建。

**Phase 1（只读）**：可不实现 runner 隔离，但 CLI/文档必须：

- 接受 `--trust-level`（默认对非本 remote 为 UNTRUSTED 或 DEGRADED）；
- 禁止在 UNTRUSTED 下写出「可发布」Evidence 语义。

---

## 9. D08 — Break-glass

### Recommended Default

Break-glass ≠ 普通 override。

| 要求     | 说明                                                                     |
| -------- | ------------------------------------------------------------------------ |
| 记录     | `EmergencyChangeRecord`：who、why、scope、start、**auto_expire**、ticket |
| 自动过期 | 到期恢复 deny；禁止无过期的永久 token                                    |
| 事后     | 强制 retrospective + 可选 revoke 相关 approval                           |
| 禁止     | 用 break-glass 改 Constitution 静默合入；关闭 audit；删除 Evidence       |

Phase 1 只读 MVA：**可不实现执行路径**，但必须在 Release/非目标中写明「无 break-glass 后门命令」。

---

## 10. D09 — Git tree / merge subject

### Recommended Default

| 概念                 | 定义                                                             |
| -------------------- | ---------------------------------------------------------------- |
| Repository Snapshot  | `commit` + `tree_id` +（可选）submodule/LFS digest               |
| 执行视图             | detached worktree @ 固定 commit；开始前重核 tree_id              |
| Evidence subject     | 明确 commit/tree 或 merge result subject；**禁止**只绑 branch 名 |
| Merge Queue / rebase | 新 commit → 旧 Evidence 标 STALE；继承规则显式，默认不自动 PASS  |

Phase 1：`index`/`resolve` 至少绑定 `source_commit`；`tree_id` 在 PR-2 进入 Snapshot。缺 tree 时不得声称「不可变执行视图已保证」。

退出码/诊断预留：

```text
REPOSITORY_TREE_CHANGED
AUTHORITY_BLOB_CHANGED
WORKTREE_EXTERNALLY_MODIFIED
```

---

## 11. D10 — Legacy sunset

### 对照面（当前事实）

| 组件                | 角色                                         |
| ------------------- | -------------------------------------------- |
| `docs/goal/tools/*` | 既有 Python/shell Goal 工具链                |
| `just goal-check`   | monorepo drift + soft gate；**不是** goalctl |
| `tools/goalctl`     | **不存在**（规划中）                         |
| `.config/goal`      | **禁止创建**                                 |

### Recommended Default

```text
Shadow（advisory 并行）
  → Mirror（差异可解释 + 回滚演练）
  → Cutover（required check 需独立 CR）
  → Sunset（删除日 + Owner）
```

| 门槛           | 建议（可在 Cutover CR 量化）                             |
| -------------- | -------------------------------------------------------- |
| Shadow→Mirror  | 真实模块样本 N≥3；差异 100% 有解释；无 P0 false negative |
| Mirror→Cutover | 回滚演练通过；Owner 审批；CI 接线单独 CR                 |
| Sunset         | 无消费者；只读历史 verifier 保留；文档与 CI 无引用       |

**Owner（提案）**：Platform（工具）+ Governance（规则）双签 Cutover。

在 Cutover CR 批准前：`goalctl` **不得** 替换 `lint-goal.sh` / `goal-check` / self-test 的 required 语义。

---

## 12. 与 Phase / PR 波次的门闩

```text
本 Decision Pack Human Approval     ✅ 2026-07-16
        ↓
PR-0  补齐治理制品（完整 SPEC/CR 链接/非目标三分）
PR-0A Schema + Authority Policy + Approval 形状 + CLI 合同 + state-dir
PR-0B Bootstrap / Identity 深度 / Break-glass 形状（可与 0A 合并，不可跳过裁定）
        ↓
PR-1  doctor / index          ← 实现 CR 批准后；无 OPEN 阻塞项
PR-2  resolve / artifact
…
Phase 1 止于 compile；不进 Agent / Native Gate / required CI
```

### Version-capability（已裁定）

| 里程碑 | 命令                | 版本标签建议              |
| ------ | ------------------- | ------------------------- |
| PR-1   | doctor, index       | 0.1.0-dev                 |
| PR-2   | + resolve, artifact | 0.1.0-dev                 |
| PR-3   | + reconcile         | 0.1.0-rc.1                |
| PR-4   | + compile           | 0.1.0（Phase 1 Complete） |

禁止每个 PR 都宣称「0.1.0 已实现」。

---

## 13. 非目标三分（已冻结）

| 分类                                         | 示例                                                                         |
| -------------------------------------------- | ---------------------------------------------------------------------------- |
| **Permanent Non-goals**                      | 新 SSOT；`.config/goal`；自动批准 Constitution；生产交易执行；Writer 自批    |
| **Deferred**                                 | Agent Adapter；GitHub Draft PR；Native G0–G11；required CI cutover           |
| **Explicitly Forbidden（无独立 CR 永不可）** | 自动 merge 高风险 PR；关闭 Evidence 绑定 commit；用 Legacy 叙述证明 RELEASED |

---

## 14. 验收（本包自身）

Human Approval 后应满足：

- [x] 10 项无 `OPEN`（全部 DECIDED）
- [x] 关联 CR 状态变为 `Approved`
- [x] `test ! -d .config/goal`（批准时工作区检查）
- [x] 未新增 `tools/goalctl` 实现代码
- [x] `docs/goal/CHANGELOG.md` 已记录本 CR 链接

---

## 15. 变更记录

| 日期       | 变更                                                                |
| ---------- | ------------------------------------------------------------------- |
| 2026-07-16 | 初稿 PROPOSED；基于 AUDIT-001/002 与 monorepo no-control-plane 约束 |
| 2026-07-16 | Human「全部批准」→ Status APPROVED；D01–D10 → DECIDED；无逐项修订   |
