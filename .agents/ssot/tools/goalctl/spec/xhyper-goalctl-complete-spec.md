# AI-Native SPEC：终态规范

```text
Status:     PROPOSED 摘要（≠ SPEC-GOALCTL-001 Approved 全文）
Foundation: DECISION-PACK-001 + CR-20260716 Approved（2026-07-16）
PR-0A:      Schema/Policy/CLI/state 形状已落盘（2026-07-16）
Related:    decisions/DECISION-PACK-001.md
            docs/goal/change-requests/CR-20260716-goalctl-foundation.md
            contracts/ · schemas/
            docs/goal/schema/authority-policy.yaml
Index:      README.md
```

## 定位

goalctl 是 AI-Native Goal Workflow Compiler，不是新的 SSOT，不是新的 Goal Gate。

**进入 PR-1 前**：Decision Pack 已 DECIDED；PR-0A 形状已具备；尚须 **实现 CR**。本文件仅描述终态形状，不单独授权实现。

## 输入

- Constitution
- Goal / Spec / Design / Plan / Tasks / Matrix / Gate
- Approved CR/ADR + ApprovalRecord（高风险）
- `docs/goal/schema/authority-policy.yaml`
- Repository Snapshot（commit；tree_id 于 PR-2+）

## 核心模块

### Authority

- Policy SSOT：`docs/goal/schema/authority-policy.yaml`（**禁止** Rust 硬编码 rank）
- Authority Snapshot / Approval Record：见 `schemas/`

### Artifact

Artifact Envelope（`schemas/artifact-envelope.schema.json`）、Strict/Mixed/Legacy Parser、Artifact Index

### Reconciliation

五维：Specification / Implementation / Verification / Release / Operations
（`schemas/reconciliation-report.schema.json` 骨架）

### Compiler

Task Pack / Prompt Pack / Validation Plan（`schemas/task-pack.schema.json` 骨架）

### Harness / Evidence / Agent

远期；Phase 1 不实现。Evidence 复用 `xhyper-evidence`，不平行发明链。

## 运行合同（PR-0A）

| 主题 | 权威 |
|------|------|
| CLI / exit / GC-* | [contracts/CLI-CONTRACT.md](../contracts/CLI-CONTRACT.md) |
| state-dir | [contracts/RUNTIME-STATE.md](../contracts/RUNTIME-STATE.md) |
| 版本能力 | [contracts/VERSION-CAPABILITY-MATRIX.md](../contracts/VERSION-CAPABILITY-MATRIX.md) |
| JSON 输出 | [schemas/](../schemas/) |

**state-dir 默认**：`${XDG_STATE_HOME:-$HOME/.local/state}/xhyper-goalctl/<repo-id>/`
**禁止**：`./target/**`、`.cargo/target/**`、历史 `../.cargo/target/**`（业务 state）、`.config/goal/**`

## 核心不变量

- 不创建 `.config/goal`
- 不产生第二 SSOT（含硬编码 Authority rank）
- Evidence 必须绑定 Commit（+ tree 于执行期）
- Legacy 不得独立证明 PASS
- Writer 不得自审；不得改 Task Pack
- Protected Asset 必须审批
- 相同输入产生相同输出（canonical JSON）
- goalctl 输出 Diagnostic（GC-*），不自写 G0–G11 PASS

## 实施路线

```text
PR-0 Governance ✅（摘要 + Decision + CR）
PR-0A Schema/Policy/CLI/state ✅ 形状
→ 实现 CR（未开）
→ PR-1 Skeleton doctor/index
→ PR-2 Authority/Artifact
→ PR-2A Fact Model
→ PR-3 Reconciliation
→ PR-4 Compiler（Phase 1 = 0.1.0）
→ PR-5… Evidence / Harness / Agent / Verifier / Shadow / Cutover / Sunset
```

## 验收标准

- Phase 1：见 VERSION-CAPABILITY-MATRIX + Decision Pack AC
- 终态：G0–G11 全链路、Evidence 完整、Matrix 完整、无 P0 Orphan、独立 Review、Rollback 可验证、Self-improving 闭环（均需后续 CR）
