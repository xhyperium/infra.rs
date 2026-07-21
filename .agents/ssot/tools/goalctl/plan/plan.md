# Plan — GOAL-GOALCTL-002 / SPEC-GOALCTL-002 Phase 1.1 完整执行计划

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-GOALCTL-002-phase1.1-v1` |
| Source Goal | [`20260716/xhyper-goalctl-complete-executable-goal.md`](../20260716/xhyper-goalctl-complete-executable-goal.md) · `GOAL-GOALCTL-002` |
| Source Spec | [`20260716/xhyper-goalctl-complete-executable-spec.md`](../20260716/xhyper-goalctl-complete-executable-spec.md) · `SPEC-GOALCTL-002` |
| Package | `xhyper-goalctl` @ `tools/goalctl` · **0.1.0 → 0.1.1** |
| Gap Matrix | [`gap-matrix.md`](./gap-matrix.md) |
| Tasks | [`tasks.md`](./tasks.md) |
| Residual | [`residual-open.md`](./residual-open.md) |
| Work Todo | [`.agents/ssot/goalctl/todo.md`](../../../goalctl/todo.md) |
| CURRENT-STATE | [`CURRENT-STATE.md`](./CURRENT-STATE.md) |
| 10x Verdict | [`goalctl-plan-10x-verdict.md`](./goalctl-plan-10x-verdict.md) |
| Strategy | **诚实台账 → Truth Hardening P0 → 合同/文档对齐 → 负例证据 → 十轮验收 → @liukongqiang5 APPROVE** |
| Campaign status | **DONE (Phase 1.1 agent-safe)** · MVA Phase 1.1 only · **≠** GOAL ACHIEVED · **≠** Cutover |
| Forbidden | `.config/goal` · Writer 自批 · 目录存在→VERIFIED · 伪造 APPROVE · 假 ACHIEVED |

---

## 0. 深度分析结论

### 0.1 goalctl 是什么

`goalctl` 的长期定位是 **Goal Delivery OS 的编译与验证内核**，不是又一个 CLI：

```text
Authority → Artifact → Fact → Reconcile → TaskPack → (Harness) → Evidence → Gate
```

Phase 1.0（0.1.0）证明了 **命令面存在**；**不**证明 snapshot 诚实、reconcile 无假阳性、approval 有效。

### 0.2 本战役北极星（MVA）

```text
所有 read-only 输出与同一 commit/tree 一致，
且 reconcile/compile 不产生可证明的假阳性。
```

版本建议：`0.1.1`；仍为 read-only；**不**宣称 Harness/Agent/Cutover。

### 0.3 P0 硬缺口（不得静默）

| ID | 风险 |
|----|------|
| GAP-001 | dirty artifact 盖 HEAD → 污染 snapshot |
| GAP-002 | 目录存在 → VERIFIED/OK → 假阳性放行 |
| GAP-003 | commit A + tree B 伪 TaskPack |
| GAP-005 | 任意 approval 字符串绕过 protected |
| GAP-007/008 | 合同漂移：trust-level / source-commit |

明细见 [`gap-matrix.md`](./gap-matrix.md) GAP-001…017。

### 0.4 与 OBJECTIVE「全部 DONE」的裁定

OBJECTIVE 要求「目标全部完成 DONE」与 Goal §6.3 终态 ACHIEVED **冲突**。

**本计划裁定**：

1. **全部 agent-safe 任务** → `DONE` + Evidence；
2. **Phase 2–6 / Cutover / Agent / Identity FULL / 跨语言 golden** → 显式 `HUMAN_ONLY` 或 `DEFERRED`；
3. **禁止**把 ACHIEVED/Cutover 标为 DONE；
4. todo 零 bare OPEN agent-safe 行 = 战役完成，**≠** GOAL-GOALCTL-002 ACHIEVED。

---

## 1. 执行策略

```text
1. 证据优先：PASS 绑定 cargo test / CLI 输出 / SCRATCH 或 evidence 路径
2. 外科手术：只改 tools/goalctl + 本 plan/todo + 对齐文档
3. 单 writer 路径分片：docs/plan | core view | reconcile | compile/approval | align | 10x
4. residual 纪律：DONE / HUMAN_ONLY / DEFERRED / POLICY only
5. 禁止：假 VERIFIED、伪造 APPROVE、目录存在即完成
6. 十轮：fail_rounds=0
7. 分支：fix/goalctl-phase11-truth-hardening（非 main）
```

### 1.1 Agent team 路径分片

| Wave | Owner 角色 | 路径 | 产出 |
|------|-----------|------|------|
| W0 | plan-writer | `.agents/ssot/tools/goalctl/plan/**` · `.agents/ssot/goalctl/todo.md` | 台账 |
| W1 | core | `src/repo.rs` · `repository_view` · `main.rs` · trust/source | view + flags |
| W2 | artifact | `src/artifact.rs` + tests | GAP-001 |
| W3 | reconcile | `src/reconcile.rs` + tests | GAP-002 |
| W4 | compile | `src/compile.rs` + approval + tests | GAP-003/005 |
| W5 | align | README · CURRENT-STATE · VERSION matrix · CHANGELOG | GAP-014 |
| W6 | verify | 10x · PR · liukongqiang5 APPROVE | 收口 |

同文件禁止并行写；W1 先于 W2–W4 合并接口。

---

## 2. 实现要点（agent-safe）

### 2.1 RepositoryView

```rust
enum RepositoryView {
  Committed { commit: GitSha, tree_id: GitSha },
  Live { head_commit: GitSha, dirty_digest: String, non_authoritative: true },
}
```

- Enforcing / artifact / reconcile / compile 默认 Committed
- Live 仅 doctor 或显式诊断；输出 `non_authoritative=true`

### 2.2 artifact（GAP-001）

- 读路径：`git show {commit}:{path}`（committed）
- 支持 `--source-commit`
- 禁止 live 内容盖 committed source_commit

### 2.3 reconcile（GAP-002）

- 删除：`evidence/`/`tests/` 目录存在 → `VERIFIED`
- 删除：README/AGENTS 存在 → Operations `OK`
- 无 Fact → 维度 `NOT_PROVEN`
- 测试面存在 → 最多 `PRESENT` + StructuredArtifact（非 VerifiedEvidence）

### 2.4 compile（GAP-003/005）

- `request.source_commit` 必须等于 resolved subject commit
- `tree_id` 必须等于 `git rev-parse {commit}^{tree}`
- protected 路径：每个 `approval_ref` 必须解析出 **ACTIVE** ApprovalRecord（`approval_id` + `subject_digest`）；任意字符串失败 `GC-APPROVAL-INVALID` / `GC-COMPILE-PROTECTED-NO-APPROVAL`

### 2.5 CLI 合同（GAP-007/008）

- 全局 `--trust-level`（未知 → USAGE）
- 全局 `--source-commit` 传入 artifact/reconcile/compile/index/resolve

### 2.6 版本

- bump `0.1.1` + CHANGELOG `[Unreleased]` → 发布段（若战役完成）
- VERSION-CAPABILITY-MATRIX 增加 0.1.1 Truth Hardening 行；删除「tools/goalctl 尚未存在」类假话

---

## 3. 验证计划（战役门禁）

```bash
test ! -d .config/goal
cargo test -p xhyper-goalctl
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
cargo fmt -p xhyper-goalctl -- --check
just goal-check
# 负例探针（dirty / wrong tree / bogus approval）见 tasks.md Evidence 约定
```

十轮：plan 完整性 · todo 无 OPEN agent-safe · 测试绿 · clippy · goal-check · 无 .config/goal · 对齐文档 · GAP 映射 · residual 诚实 · CLI 负例

---

## 4. 批准

- `export LIUKONGQIANG5_APPROVE_TOKEN=…`
- PR 上 `@liukongqiang5` APPROVE；readback 写入 SCRATCH / evidence
- token/API 不可用 → `approve-env-limit.txt`；**不**伪造

---

## 5. Task checklist（战役进度）

- [x] Deep-read 20260716 Goal+Spec + tools/goalctl；gap-matrix GAP-001…017 + AC 映射
- [x] Author plan package + `.agents/ssot/goalctl/todo.md`
- [x] Implement Phase 1.1 P0/P1 agent-safe fixes + tests
- [x] Run tests/clippy/goal-check；SCRATCH logs
- [x] Alignment docs + todo 终态
- [x] 10 check rounds；fail_rounds=0
- [x] PR + @liukongqiang5 APPROVE readback（或诚实 env-limit）

## Deviations

- （执行中追加；每条一句：改了什么 + 为什么）

- 在独立 worktree 执行：主 workspace 分支被并发重置。
- CLI smoke task-file 假 source_commit 改为 HEAD，避免 GAP-003 遮蔽其它负例。
- Skeptic: prior 10x used `clippy ... -- -D warnings -q` (invalid); re-ran fail-closed 10x without fabricating PASS.
